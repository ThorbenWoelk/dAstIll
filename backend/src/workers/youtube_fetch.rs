use std::time::Duration;

use crate::handlers::content::generation::download_youtube_transcript;
use crate::{db, state::AppState};

const FETCH_POLL_INTERVAL: Duration = Duration::from_secs(60);
const RETRY_DELAY_MINUTES: i64 = 15;

pub(super) fn spawn_youtube_fetch_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!("local YouTube fetch worker started; browser activity is not required");
        loop {
            if let Err(error) = fetch_requested_transcripts(&state).await {
                tracing::warn!(%error, "local YouTube fetch round failed");
            }
            tokio::time::sleep(FETCH_POLL_INTERVAL).await;
        }
    });
}

async fn fetch_requested_transcripts(state: &AppState) -> Result<(), db::StoreError> {
    for request in db::pending_youtube_transcript_requests(&state.db)
        .await?
        .into_iter()
        .take(4)
    {
        // Results carry request identities, so cancellation/reset makes late downloads harmless.
        let result = match download_youtube_transcript(state, &request.video_id).await {
            Ok(transcript) => db::YoutubeTranscriptResult::Ready(transcript),
            Err((status, message)) if status.as_u16() == 429 || status.as_u16() == 503 => {
                db::YoutubeTranscriptResult::Deferred {
                    retry_at: chrono::Utc::now() + chrono::Duration::minutes(RETRY_DELAY_MINUTES),
                    message,
                }
            }
            Err((status, message)) => db::YoutubeTranscriptResult::Failed {
                status: status.as_u16(),
                message,
            },
        };
        tracing::info!(video_id = %request.video_id, ready = matches!(result, db::YoutubeTranscriptResult::Ready(_)), "local YouTube fetch completed");
        db::save_youtube_transcript_result(&state.db, &request, &result).await?;
    }
    Ok(())
}
