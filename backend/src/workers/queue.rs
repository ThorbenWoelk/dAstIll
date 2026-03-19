use crate::{
    db,
    handlers::content,
    models::{ContentStatus, Video},
    state::AppState,
};
use tracing::Instrument;

use super::{
    MAX_DISTILLATION_RETRIES, PollBackoffState, QUEUE_IDLE_POLL_INTERVAL,
    QUEUE_IDLE_POLL_MAX_INTERVAL, QUEUE_POLL_BACKOFF, QUEUE_POLL_INTERVAL, QUEUE_SCAN_LIMIT,
    QueueTask, sleep_with_backoff,
};

pub(super) fn next_queue_task(video: &Video) -> QueueTask {
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

pub fn spawn_queue_worker(state: AppState) {
    let span = logfire::span!(
        "worker.queue",
        active_poll_interval_secs = QUEUE_POLL_INTERVAL.as_secs(),
        idle_poll_start_secs = QUEUE_IDLE_POLL_INTERVAL.as_secs(),
        idle_poll_max_secs = QUEUE_IDLE_POLL_MAX_INTERVAL.as_secs(),
        scan_limit = QUEUE_SCAN_LIMIT,
    );

    tokio::spawn(
        async move {
            tracing::info!(
                active_poll_interval_secs = QUEUE_POLL_INTERVAL.as_secs(),
                idle_poll_start_secs = QUEUE_IDLE_POLL_INTERVAL.as_secs(),
                idle_poll_max_secs = QUEUE_IDLE_POLL_MAX_INTERVAL.as_secs(),
                "queue worker started"
            );
            let mut backoff_state = PollBackoffState::default();

            loop {
                let queue = {
                    let conn = state.db.connect();
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
                        sleep_with_backoff(QUEUE_POLL_BACKOFF, &mut backoff_state, false).await;
                        continue;
                    }
                };
                let had_activity = !queue.is_empty();

                for video in queue {
                    let task = next_queue_task(&video);

                    // Fast-path skip if transcript rate limits apply to avoid log spam
                    if task == QueueTask::Transcript && state.transcript_cooldown.is_active() {
                        continue;
                    }

                    let task_name = match task {
                        QueueTask::Transcript => "transcript",
                        QueueTask::Summary => "summary",
                        QueueTask::Skip => "skip",
                    };
                    let video_span = logfire::span!(
                        "worker.queue.process",
                        video.id = video.id.clone(),
                        task = task_name,
                        retry_count = video.retry_count,
                    );

                    async {
                        tracing::info!(video_id = %video.id, task = task_name, "queue worker processing video");
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
                                let conn = state.db.connect();
                                let _ = db::increment_video_retry_count(&conn, &video.id).await;
                            }
                        } else {
                            let conn = state.db.connect();
                            let _ = db::reset_video_retry_count(&conn, &video.id).await;
                        }
                    }
                    .instrument(video_span)
                    .await;
                }

                sleep_with_backoff(QUEUE_POLL_BACKOFF, &mut backoff_state, had_activity).await;
            }
        }
        .instrument(span),
    );
}
