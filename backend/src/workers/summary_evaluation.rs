use crate::{
    db::{self, Store, StoreError, SummaryEvaluationFailureKind, SummaryEvaluationFailureOutcome},
    handlers::content,
    models::{AiStatus, ContentStatus, Summary, SummaryEvaluationJob, VocabularyReplacement},
    services::summary_evaluator::{
        SummaryEvaluation, SummaryEvaluatorError, UnscorableCause, transcript_for_evaluation,
    },
    state::AppState,
};
use tracing::Instrument;

use super::{
    PollBackoffState, SUMMARY_EVAL_IDLE_POLL_INTERVAL, SUMMARY_EVAL_IDLE_POLL_MAX_INTERVAL,
    SUMMARY_EVAL_POLL_BACKOFF, SUMMARY_EVAL_POLL_INTERVAL, SUMMARY_EVAL_SCAN_LIMIT,
    sleep_with_backoff,
};

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

/// Whether the summary from before an automatic regeneration should come back.
/// A scored summary beats an unscored one; otherwise only a strictly higher score wins.
pub(super) fn previous_summary_scored_better(
    previous_score: Option<u8>,
    new_score: Option<u8>,
) -> bool {
    match (previous_score, new_score) {
        (Some(previous), Some(new)) => previous > new,
        (Some(_), None) => true,
        (None, _) => false,
    }
}

/// Whether the summary the evaluator just looked at should be generated again.
pub(super) fn summary_needs_regeneration(
    summary: &Summary,
    unscorable_cause: Option<UnscorableCause>,
) -> bool {
    if db::is_manual_summary(summary) {
        return false;
    }
    match summary.quality_score {
        Some(score) => score < content::MIN_SUMMARY_QUALITY_SCORE_FOR_ACCEPTANCE,
        None => unscorable_cause == Some(UnscorableCause::Summary),
    }
}

/// Only an answer the model gave but that could not be used counts toward the
/// unscorable limit. Request failures (timeouts, HTTP errors) are retried, but
/// still move the summary behind others in the scan.
pub(super) fn evaluation_failure_kind(
    err: &SummaryEvaluatorError,
) -> Option<SummaryEvaluationFailureKind> {
    match err {
        SummaryEvaluatorError::NotAvailable => None,
        SummaryEvaluatorError::ParseFailed(_) => Some(SummaryEvaluationFailureKind::UnusableAnswer),
        SummaryEvaluatorError::EvaluationFailed(_) | SummaryEvaluatorError::RequestFailed(_) => {
            Some(SummaryEvaluationFailureKind::RequestFailed)
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct StoredEvaluation {
    /// `false` when the summary changed while the evaluator ran; nothing was stored.
    pub stored: bool,
    /// The better-scored summary that was put back, if any.
    pub restored: Option<Summary>,
    /// Whether the video was moved to `Pending` for another automatic regeneration.
    pub queued_regeneration: bool,
}

/// Stores an evaluation and decides what happens next:
/// - keeps the better-scored summary when an automatic regeneration scored lower
/// - queues another regeneration for low scores or a malformed summary, within
///   `MAX_SUMMARY_AUTO_REGEN_ATTEMPTS`
/// - never queues hand-written summaries
pub(super) async fn store_summary_evaluation(
    store: &Store,
    job: &SummaryEvaluationJob,
    evaluation: &SummaryEvaluation,
) -> Result<StoredEvaluation, StoreError> {
    let result = &evaluation.result;
    let stored = db::update_summary_quality(
        store,
        &job.video_id,
        &job.summary_content,
        result.quality_score,
        result.quality_note.as_deref(),
        result.quality_model_used.as_deref(),
        Some(&result.summary_tags),
    )
    .await?;
    if !stored {
        return Ok(StoredEvaluation::default());
    }
    db::clear_summary_evaluation_failures(store, &job.video_id).await?;

    let Some(mut current) = db::get_summary(store, &job.video_id).await? else {
        return Ok(StoredEvaluation {
            stored: true,
            ..StoredEvaluation::default()
        });
    };

    let mut restored = None;
    if let Some(previous) = db::take_summary_before_regeneration(store, &job.video_id).await?
        && previous.content != current.content
        && !db::is_manual_summary(&current)
        && previous_summary_scored_better(previous.quality_score, current.quality_score)
    {
        db::upsert_summary(store, &previous).await?;
        current = previous.clone();
        restored = Some(previous);
    }

    let mut queued_regeneration = false;
    let unscorable_cause = if restored.is_some() {
        None
    } else {
        evaluation.unscorable_cause
    };
    if summary_needs_regeneration(&current, unscorable_cause) {
        let attempts = db::get_summary_auto_regen_attempts(store, &job.video_id).await?;
        if attempts < content::MAX_SUMMARY_AUTO_REGEN_ATTEMPTS {
            if current.quality_score.is_some() {
                db::save_summary_before_regeneration(store, &current).await?;
            }
            db::update_video_summary_status(store, &job.video_id, ContentStatus::Pending).await?;
            queued_regeneration = true;
        }
    }

    Ok(StoredEvaluation {
        stored: true,
        restored,
        queued_regeneration,
    })
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

async fn evaluate_summary_job(
    state: &AppState,
    job: &SummaryEvaluationJob,
    vocabulary_replacements: &[VocabularyReplacement],
) {
    tracing::info!(video_id = %job.video_id, "summary evaluation worker processing video");
    let transcript = transcript_for_evaluation(&job.transcript_text, vocabulary_replacements);
    let evaluation = state
        .summary_evaluator
        .evaluate_with_cause(&transcript, &job.summary_content, &job.video_title)
        .await;

    match evaluation {
        Ok(evaluation) => {
            let conn = state.db.connect();
            let outcome = match store_summary_evaluation(&conn, job, &evaluation).await {
                Ok(outcome) => outcome,
                Err(err) => {
                    tracing::warn!(
                        video_id = %job.video_id,
                        error = %err,
                        "failed to store summary evaluation"
                    );
                    return;
                }
            };
            if !outcome.stored {
                tracing::info!(
                    video_id = %job.video_id,
                    "summary changed during evaluation - result discarded"
                );
                return;
            }
            if let Some(restored) = &outcome.restored {
                if let Err((_, err)) = content::restore_summary(state, restored).await {
                    tracing::warn!(
                        video_id = %job.video_id,
                        error = %err,
                        "failed to refresh search after restoring previous summary"
                    );
                }
                tracing::info!(
                    video_id = %job.video_id,
                    new_score = ?evaluation.result.quality_score,
                    kept_score = ?restored.quality_score,
                    "regenerated summary scored lower - kept the previous summary"
                );
            }
            evict_video_scope_cache(state, &job.video_id).await;

            tracing::info!(
                video_id = %job.video_id,
                score = ?evaluation.result.quality_score,
                unscorable_cause = ?evaluation.unscorable_cause,
                model = evaluation.result.quality_model_used.as_deref().unwrap_or("-"),
                "summary evaluation completed"
            );
            if outcome.queued_regeneration {
                tracing::info!(
                    video_id = %job.video_id,
                    threshold = content::MIN_SUMMARY_QUALITY_SCORE_FOR_ACCEPTANCE,
                    max_attempts = content::MAX_SUMMARY_AUTO_REGEN_ATTEMPTS,
                    "queued summary for automatic regeneration"
                );
            }
        }
        Err(err) => {
            let Some(kind) = evaluation_failure_kind(&err) else {
                tracing::debug!(
                    video_id = %job.video_id,
                    "summary evaluation deferred - evaluator not available"
                );
                return;
            };
            let conn = state.db.connect();
            let outcome = db::record_summary_evaluation_failure(
                &conn,
                &job.video_id,
                &job.summary_content,
                &err.to_string(),
                kind,
                content::MAX_SUMMARY_EVALUATION_FAILURES,
            )
            .await;
            match outcome {
                Ok(SummaryEvaluationFailureOutcome::GaveUp { unusable_answers }) => {
                    evict_video_scope_cache(state, &job.video_id).await;
                    tracing::warn!(
                        video_id = %job.video_id,
                        error = %err,
                        unusable_answers,
                        "summary evaluation failed too often - marked unscorable"
                    );
                }
                Ok(outcome) => tracing::warn!(
                    video_id = %job.video_id,
                    error = %err,
                    kind = ?kind,
                    outcome = ?outcome,
                    "summary evaluation failed - will retry"
                ),
                Err(store_err) => tracing::warn!(
                    video_id = %job.video_id,
                    error = %err,
                    store_error = %store_err,
                    "summary evaluation failed and the failure could not be recorded"
                ),
            }
        }
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
                if state.user_activity.is_idle() {
                    tracing::debug!("summary evaluation worker skipped - no active user");
                    sleep_with_backoff(SUMMARY_EVAL_POLL_BACKOFF, &mut backoff_state, false).await;
                    continue;
                }

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

                // The summarizer reads a vocabulary-normalized transcript; the evaluator must too.
                let vocabulary_replacements = match db::get_preferences(&state.db).await {
                    Ok(preferences) => preferences.vocabulary_replacements,
                    Err(err) => {
                        tracing::warn!(
                            error = %err,
                            "summary evaluation worker could not load vocabulary replacements"
                        );
                        Vec::new()
                    }
                };

                for job in queue {
                    let video_span = logfire::span!(
                        "worker.eval.process",
                        video.id = job.video_id.clone(),
                        transcript_chars = job.transcript_text.chars().count(),
                        summary_chars = job.summary_content.chars().count(),
                        model = state.summary_evaluator.model().to_string(),
                    );

                    evaluate_summary_job(&state, &job, &vocabulary_replacements)
                        .instrument(video_span)
                        .await;
                }

                sleep_with_backoff(SUMMARY_EVAL_POLL_BACKOFF, &mut backoff_state, true).await;
            }
        }
        .instrument(span),
    );
}
