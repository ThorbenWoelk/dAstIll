use crate::{
    db,
    handlers::content,
    models::{AiStatus, ContentStatus},
    state::AppState,
};
use tracing::Instrument;

use super::{
    PollBackoffState, SUMMARY_EVAL_IDLE_POLL_INTERVAL, SUMMARY_EVAL_IDLE_POLL_MAX_INTERVAL,
    SUMMARY_EVAL_POLL_BACKOFF, SUMMARY_EVAL_POLL_INTERVAL, SUMMARY_EVAL_SCAN_LIMIT,
    sleep_with_backoff,
};

pub(super) fn should_queue_summary_auto_regeneration(
    quality_score: u8,
    auto_regen_attempts: u8,
) -> bool {
    quality_score < content::MIN_SUMMARY_QUALITY_SCORE_FOR_ACCEPTANCE
        && auto_regen_attempts < content::MAX_SUMMARY_AUTO_REGEN_ATTEMPTS
}

pub(super) fn should_run_summary_evaluation(
    evaluator_status: AiStatus,
    evaluator_model: &str,
) -> bool {
    match evaluator_status {
        AiStatus::Cloud => true,
        AiStatus::LocalOnly => !crate::services::is_cloud_model(evaluator_model),
        AiStatus::Offline => false,
    }
}

async fn evict_video_scope_cache(state: &AppState, video_id: &str) {
    let conn = state.db.connect();
    let Ok(Some(video)) = db::get_video(&conn, video_id, false).await else {
        return;
    };

    let is_subscribed = db::get_channel(&conn, &video.channel_id)
        .await
        .ok()
        .flatten()
        .is_some();

    if is_subscribed {
        state.read_cache.evict_channel(&video.channel_id).await;
    } else {
        state
            .read_cache
            .evict_channel(crate::models::OTHERS_CHANNEL_ID)
            .await;
    }
}

pub fn spawn_summary_evaluation_worker(state: AppState) {
    let span = logfire::span!(
        "worker.eval",
        active_poll_interval_secs = SUMMARY_EVAL_POLL_INTERVAL.as_secs(),
        idle_poll_start_secs = SUMMARY_EVAL_IDLE_POLL_INTERVAL.as_secs(),
        idle_poll_max_secs = SUMMARY_EVAL_IDLE_POLL_MAX_INTERVAL.as_secs(),
        model = state.summary_evaluator.model().to_string(),
    );

    tokio::spawn(
        async move {
            tracing::info!(
                active_poll_interval_secs = SUMMARY_EVAL_POLL_INTERVAL.as_secs(),
                idle_poll_start_secs = SUMMARY_EVAL_IDLE_POLL_INTERVAL.as_secs(),
                idle_poll_max_secs = SUMMARY_EVAL_IDLE_POLL_MAX_INTERVAL.as_secs(),
                model = %state.summary_evaluator.model(),
                "summary evaluation worker started"
            );
            let mut backoff_state = PollBackoffState::default();

            loop {
                let queue = {
                    let conn = state.db.connect();
                    db::list_summaries_pending_quality_eval(&conn, SUMMARY_EVAL_SCAN_LIMIT)
                        .await
                        .map_err(|err| err.to_string())
                };

                let queue = match queue {
                    Ok(rows) => rows,
                    Err(err) => {
                        tracing::error!(error = %err, "summary evaluation worker failed to load queue");
                        sleep_with_backoff(SUMMARY_EVAL_POLL_BACKOFF, &mut backoff_state, false).await;
                        continue;
                    }
                };

                if queue.is_empty() {
                    sleep_with_backoff(SUMMARY_EVAL_POLL_BACKOFF, &mut backoff_state, false).await;
                    continue;
                }

                let evaluator_available = state.summary_evaluator.is_available().await;
                let evaluator_status = state
                    .summary_evaluator
                    .indicator_status(state.cloud_cooldown.is_active(), evaluator_available);

                if !should_run_summary_evaluation(evaluator_status, state.summary_evaluator.model()) {
                    tracing::debug!(
                        evaluator_status = ?evaluator_status,
                        "summary evaluation paused - evaluator unavailable or preserving local capacity"
                    );
                    sleep_with_backoff(SUMMARY_EVAL_POLL_BACKOFF, &mut backoff_state, false).await;
                    continue;
                }

                for job in queue {
                    let video_span = logfire::span!(
                        "worker.eval.process",
                        video.id = job.video_id.clone(),
                        transcript_chars = job.transcript_text.chars().count(),
                        summary_chars = job.summary_content.chars().count(),
                        model = state.summary_evaluator.model().to_string(),
                    );

                    async {
                        tracing::info!(video_id = %job.video_id, "summary evaluation worker processing video");
                        let evaluation = state
                            .summary_evaluator
                            .evaluate(&job.transcript_text, &job.summary_content, &job.video_title)
                            .await;

                        match evaluation {
                            Ok(result) => {
                                let conn = state.db.connect();
                                let _ = db::update_summary_quality(
                                    &conn,
                                    &job.video_id,
                                    Some(result.quality_score),
                                    result.quality_note.as_deref(),
                                    result.quality_model_used.as_deref(),
                                )
                                .await;
                                evict_video_scope_cache(&state, &job.video_id).await;

                                tracing::info!(
                                    video_id = %job.video_id,
                                    score = result.quality_score,
                                    model = result.quality_model_used.as_deref().unwrap_or("-"),
                                    "summary evaluation completed"
                                );

                                if let Ok(auto_regen_attempts) =
                                    db::get_summary_auto_regen_attempts(&conn, &job.video_id).await
                                {
                                    if should_queue_summary_auto_regeneration(
                                        result.quality_score,
                                        auto_regen_attempts,
                                    ) {
                                        if let Err(err) = db::update_video_summary_status(
                                            &conn,
                                            &job.video_id,
                                            ContentStatus::Pending,
                                        )
                                        .await
                                        {
                                            tracing::warn!(
                                                video_id = %job.video_id,
                                                error = %err,
                                                "failed to queue low-quality summary regeneration"
                                            );
                                        } else {
                                            evict_video_scope_cache(&state, &job.video_id).await;
                                            tracing::info!(
                                                video_id = %job.video_id,
                                                score = result.quality_score,
                                                attempts = auto_regen_attempts,
                                                threshold = content::MIN_SUMMARY_QUALITY_SCORE_FOR_ACCEPTANCE,
                                                max_attempts = content::MAX_SUMMARY_AUTO_REGEN_ATTEMPTS,
                                                "queued summary for automatic regeneration"
                                            );
                                        }
                                    }
                                }
                            }
                            Err(ref err)
                                if matches!(
                                    err,
                                    crate::services::summary_evaluator::SummaryEvaluatorError::NotAvailable
                                ) =>
                            {
                                tracing::debug!(
                                    video_id = %job.video_id,
                                    "summary evaluation deferred - evaluator not available"
                                );
                                // Leave quality_score/quality_note NULL so the job is retried later
                            }
                            Err(err) => {
                                tracing::warn!(
                                    video_id = %job.video_id,
                                    error = %err,
                                    "summary evaluation failed"
                                );
                                // Permanent failure - mark with note but no score so it can be retried
                            }
                        }
                    }
                    .instrument(video_span)
                    .await;
                }

                sleep_with_backoff(SUMMARY_EVAL_POLL_BACKOFF, &mut backoff_state, true).await;
            }
        }
        .instrument(span),
    );
}
