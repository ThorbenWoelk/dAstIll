use std::collections::HashSet;
use std::time::Duration;

use tokio::time::sleep;

use crate::db;
use crate::handlers::content;
use crate::models::{AiStatus, ContentStatus, Video};
use crate::state::AppState;

const QUEUE_SCAN_LIMIT: usize = 4;
const QUEUE_POLL_INTERVAL: Duration = Duration::from_secs(5);
const CHANNEL_REFRESH_INTERVAL: Duration = Duration::from_secs(30 * 60);
const CHANNEL_GAP_SCAN_INTERVAL: Duration = Duration::from_secs(10 * 60);
const CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL: usize = 8;
const SUMMARY_EVAL_SCAN_LIMIT: usize = 4;
const SUMMARY_EVAL_POLL_INTERVAL: Duration = Duration::from_secs(7);
const MAX_DISTILLATION_RETRIES: u8 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum QueueTask {
    Transcript,
    Summary,
    Skip,
}

fn next_queue_task(video: &Video) -> QueueTask {
    if video.retry_count >= MAX_DISTILLATION_RETRIES {
        return QueueTask::Skip;
    }

    match video.transcript_status {
        ContentStatus::Pending | ContentStatus::Loading | ContentStatus::Failed => {
            QueueTask::Transcript
        }
        ContentStatus::Ready => match video.summary_status {
            ContentStatus::Pending | ContentStatus::Loading | ContentStatus::Failed => {
                QueueTask::Summary
            }
            ContentStatus::Ready => QueueTask::Skip,
        },
    }
}

fn should_queue_summary_auto_regeneration(quality_score: u8, auto_regen_attempts: u8) -> bool {
    quality_score < content::MIN_SUMMARY_QUALITY_SCORE_FOR_ACCEPTANCE
        && auto_regen_attempts < content::MAX_SUMMARY_AUTO_REGEN_ATTEMPTS
}

fn should_run_summary_evaluation(evaluator_status: AiStatus, evaluator_model: &str) -> bool {
    match evaluator_status {
        AiStatus::Cloud => true,
        AiStatus::LocalOnly => !crate::services::is_cloud_model(evaluator_model),
        AiStatus::Offline => false,
    }
}

pub fn spawn_queue_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!(
            poll_interval_secs = QUEUE_POLL_INTERVAL.as_secs(),
            "queue worker started"
        );

        loop {
            let queue = {
                let conn = state.db.lock().await;
                db::list_videos_for_queue_processing(
                    &conn,
                    QUEUE_SCAN_LIMIT,
                    MAX_DISTILLATION_RETRIES,
                )
                .await
                .map_err(|err| err.to_string())
            };

            let queue = match queue {
                Ok(videos) => videos,
                Err(err) => {
                    tracing::error!(error = %err, "queue worker failed to load queue");
                    sleep(QUEUE_POLL_INTERVAL).await;
                    continue;
                }
            };

            for video in queue {
                let task = next_queue_task(&video);

                // Fast-path skip if transcript rate limits apply to avoid log spam
                if task == QueueTask::Transcript && state.transcript_cooldown.is_active() {
                    continue;
                }

                tracing::info!(video_id = %video.id, "queue worker processing video");
                let result = match task {
                    QueueTask::Transcript => {
                        tracing::info!(video_id = %video.id, "queue worker ensuring transcript");
                        content::ensure_transcript(&state, &video.id)
                            .await
                            .map(|_| ())
                    }
                    QueueTask::Summary => {
                        tracing::info!(video_id = %video.id, "queue worker ensuring summary");
                        content::ensure_summary(&state, &video.id).await.map(|_| ())
                    }
                    QueueTask::Skip => {
                        tracing::debug!(video_id = %video.id, "queue worker skipping video");
                        Ok(())
                    }
                };

                if let Err((status, message)) = result {
                    // Only log as warning/increment retry if it's not a quota/rate limit error we know about
                    if status == axum::http::StatusCode::TOO_MANY_REQUESTS {
                        tracing::debug!(
                            video_id = %video.id,
                            "queue worker paused for video due to rate limits"
                        );
                    } else {
                        tracing::warn!(
                            video_id = %video.id,
                            http_status = %status,
                            error = %message,
                            "queue worker failed to process video"
                        );
                        let conn = state.db.lock().await;
                        let _ = db::increment_video_retry_count(&conn, &video.id).await;
                    }
                } else {
                    let conn = state.db.lock().await;
                    let _ = db::reset_video_retry_count(&conn, &video.id).await;
                }
            }

            sleep(QUEUE_POLL_INTERVAL).await;
        }
    });
}

/// Refresh all channels by fetching their RSS feeds and inserting new videos.
async fn refresh_all_channels(state: &AppState) {
    let channels = {
        let conn = state.db.lock().await;
        db::list_channels(&conn)
            .await
            .map_err(|err| err.to_string())
    };

    let channels = match channels {
        Ok(list) => list,
        Err(err) => {
            tracing::error!(error = %err, "refresh worker failed to list channels");
            return;
        }
    };

    if channels.is_empty() {
        return;
    }

    tracing::info!(channel_count = channels.len(), "refreshing all channels");

    for (i, channel) in channels.iter().enumerate() {
        if i > 0 {
            sleep(Duration::from_secs(1)).await;
        }
        match state.youtube.fetch_videos(&channel.id).await {
            Ok(videos) => {
                let conn = state.db.lock().await;
                let n = db::bulk_insert_videos(&conn, videos).await.unwrap_or(0);
                if n > 0 {
                    tracing::info!(
                        channel_id = %channel.id,
                        new_videos = n,
                        "refresh worker found new videos"
                    );
                }
            }
            Err(err) => {
                tracing::warn!(
                    channel_id = %channel.id,
                    error = %err,
                    "refresh worker failed to fetch videos"
                );
            }
        }
    }
}

pub fn spawn_refresh_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!(
            interval_secs = CHANNEL_REFRESH_INTERVAL.as_secs(),
            "channel refresh worker started"
        );

        // Run an initial refresh at startup so new videos appear immediately.
        refresh_all_channels(&state).await;

        loop {
            sleep(CHANNEL_REFRESH_INTERVAL).await;
            refresh_all_channels(&state).await;
        }
    });
}

async fn fill_channel_gaps(
    state: &AppState,
    channel_id: &str,
    limit: usize,
    until: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<usize, String> {
    let known_video_ids = {
        let conn = state.db.lock().await;
        db::list_video_ids_by_channel(&conn, channel_id)
            .await
            .map_err(|err| err.to_string())?
            .into_iter()
            .collect::<HashSet<_>>()
    };

    let (videos, _exhausted) = state
        .youtube
        .fetch_videos_backfill_missing(channel_id, &known_video_ids, limit, until)
        .await
        .map_err(|err| err.to_string())?;

    let conn = state.db.lock().await;
    let inserted = db::bulk_insert_videos(&conn, videos)
        .await
        .map_err(|err| err.to_string())?;
    Ok(inserted)
}

async fn scan_all_channels_for_gaps(state: &AppState) {
    if state.youtube_quota_cooldown.is_active() {
        tracing::debug!("skipping gap scan worker - youtube quota cooldown active");
        return;
    }

    let channels = {
        let conn = state.db.lock().await;
        db::list_channels(&conn)
            .await
            .map_err(|err| err.to_string())
    };

    let channels = match channels {
        Ok(list) => list,
        Err(err) => {
            tracing::error!(error = %err, "gap scan worker failed to list channels");
            return;
        }
    };

    if channels.is_empty() {
        return;
    }

    tracing::info!(
        channel_count = channels.len(),
        per_channel_limit = CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL,
        "gap scan worker scanning channels"
    );

    for (i, channel) in channels.into_iter().enumerate() {
        if i > 0 {
            sleep(Duration::from_secs(1)).await;
        }
        match fill_channel_gaps(
            state,
            &channel.id,
            CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL,
            channel.earliest_sync_date,
        )
        .await
        {
            Ok(inserted) if inserted > 0 => {
                tracing::info!(
                    channel_id = %channel.id,
                    inserted,
                    "gap scan worker inserted missing videos"
                );
            }
            Ok(_) => {}
            Err(err) => {
                tracing::warn!(
                    channel_id = %channel.id,
                    error = %err,
                    "gap scan worker failed for channel"
                );
            }
        }
    }
}

pub fn spawn_gap_scan_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!(
            interval_secs = CHANNEL_GAP_SCAN_INTERVAL.as_secs(),
            per_channel_limit = CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL,
            "channel gap scan worker started"
        );

        scan_all_channels_for_gaps(&state).await;

        loop {
            sleep(CHANNEL_GAP_SCAN_INTERVAL).await;
            scan_all_channels_for_gaps(&state).await;
        }
    });
}

pub fn spawn_summary_evaluation_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!(
            poll_interval_secs = SUMMARY_EVAL_POLL_INTERVAL.as_secs(),
            model = %state.summary_evaluator.model(),
            "summary evaluation worker started"
        );

        loop {
            let queue = {
                let conn = state.db.lock().await;
                db::list_summaries_pending_quality_eval(&conn, SUMMARY_EVAL_SCAN_LIMIT)
                    .await
                    .map_err(|err| err.to_string())
            };

            let queue = match queue {
                Ok(rows) => rows,
                Err(err) => {
                    tracing::error!(error = %err, "summary evaluation worker failed to load queue");
                    sleep(SUMMARY_EVAL_POLL_INTERVAL).await;
                    continue;
                }
            };

            if queue.is_empty() {
                sleep(SUMMARY_EVAL_POLL_INTERVAL).await;
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
                // Back off longer when evaluator is offline to avoid log spam
                sleep(Duration::from_secs(60)).await;
                continue;
            }

            for job in queue {
                tracing::info!(video_id = %job.video_id, "summary evaluation worker processing video");
                let evaluation = state
                    .summary_evaluator
                    .evaluate(&job.transcript_text, &job.summary_content, &job.video_title)
                    .await;

                match evaluation {
                    Ok(result) => {
                        let conn = state.db.lock().await;
                        let _ = db::update_summary_quality(
                            &conn,
                            &job.video_id,
                            Some(result.quality_score),
                            result.quality_note.as_deref(),
                        )
                        .await;

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

            sleep(SUMMARY_EVAL_POLL_INTERVAL).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{
        QueueTask, next_queue_task, should_queue_summary_auto_regeneration,
        should_run_summary_evaluation,
    };
    use crate::models::{AiStatus, ContentStatus, Video};

    fn video_with_statuses(
        transcript_status: ContentStatus,
        summary_status: ContentStatus,
    ) -> Video {
        Video {
            id: "video".to_string(),
            channel_id: "channel".to_string(),
            title: "Title".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status,
            summary_status,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        }
    }

    #[test]
    fn next_queue_task_prioritizes_transcript_when_not_ready() {
        let video = video_with_statuses(ContentStatus::Pending, ContentStatus::Ready);
        assert_eq!(next_queue_task(&video), QueueTask::Transcript);

        let loading_video = video_with_statuses(ContentStatus::Loading, ContentStatus::Pending);
        assert_eq!(next_queue_task(&loading_video), QueueTask::Transcript);
    }

    #[test]
    fn next_queue_task_summarizes_only_after_transcript_ready() {
        let video = video_with_statuses(ContentStatus::Ready, ContentStatus::Pending);
        assert_eq!(next_queue_task(&video), QueueTask::Summary);

        let loading_summary = video_with_statuses(ContentStatus::Ready, ContentStatus::Loading);
        assert_eq!(next_queue_task(&loading_summary), QueueTask::Summary);
    }

    #[test]
    fn next_queue_task_retries_failed_rows() {
        let failed_transcript = video_with_statuses(ContentStatus::Failed, ContentStatus::Pending);
        assert_eq!(next_queue_task(&failed_transcript), QueueTask::Transcript);

        let failed_summary = video_with_statuses(ContentStatus::Ready, ContentStatus::Failed);
        assert_eq!(next_queue_task(&failed_summary), QueueTask::Summary);
    }

    #[test]
    fn next_queue_task_skips_complete_rows() {
        let done = video_with_statuses(ContentStatus::Ready, ContentStatus::Ready);
        assert_eq!(next_queue_task(&done), QueueTask::Skip);
    }

    #[test]
    fn should_queue_summary_auto_regeneration_only_for_low_scores_with_remaining_attempts() {
        assert!(should_queue_summary_auto_regeneration(6, 0));
        assert!(should_queue_summary_auto_regeneration(0, 1));
        assert!(!should_queue_summary_auto_regeneration(7, 0));
        assert!(!should_queue_summary_auto_regeneration(9, 0));
        assert!(!should_queue_summary_auto_regeneration(6, 2));
    }

    #[test]
    fn summary_evaluation_runs_only_when_it_wont_consume_local_fallback_capacity() {
        assert!(should_run_summary_evaluation(
            AiStatus::Cloud,
            "qwen3.5:397b-cloud"
        ));
        assert!(!should_run_summary_evaluation(
            AiStatus::LocalOnly,
            "qwen3.5:397b-cloud"
        ));
        assert!(should_run_summary_evaluation(
            AiStatus::LocalOnly,
            "qwen3:8b"
        ));
        assert!(!should_run_summary_evaluation(
            AiStatus::Offline,
            "qwen3.5:397b-cloud"
        ));
    }
}
