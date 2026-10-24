use super::{Store, StoreError};
use crate::models::Transcript;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const REQUESTS: &str = "youtube-transcript-requests/";
static REQUEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct YoutubeTranscriptRequest {
    pub id: String,
    pub video_id: String,
    pub requested_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub(crate) enum YoutubeTranscriptResult {
    Ready(Transcript),
    Deferred {
        retry_at: DateTime<Utc>,
        message: String,
    },
    Failed {
        status: u16,
        message: String,
    },
}

fn request_key(video_id: &str) -> String {
    format!("{REQUESTS}{video_id}.json")
}

fn result_key(request: &YoutubeTranscriptRequest) -> String {
    format!(
        "youtube-transcript-results/{}/{}.json",
        request.video_id, request.id
    )
}

fn retry_key(request: &YoutubeTranscriptRequest) -> String {
    format!("youtube-transcript-retries/{}.json", request.id)
}

pub(crate) async fn request_youtube_transcript(
    store: &Store,
    video_id: &str,
) -> Result<YoutubeTranscriptRequest, StoreError> {
    let _guard = REQUEST_LOCK.lock().await;
    let key = request_key(video_id);
    if let Some(request) = store.get_json(&key).await? {
        return Ok(request);
    }
    let mut bytes = [0u8; 16];
    rustls::crypto::ring::default_provider()
        .secure_random
        .fill(&mut bytes)
        .map_err(|_| StoreError::Other("could not create transcript request id".into()))?;
    let request = YoutubeTranscriptRequest {
        id: bytes.iter().map(|byte| format!("{byte:02x}")).collect(),
        video_id: video_id.into(),
        requested_at: Utc::now(),
    };
    store.put_json(&key, &request).await?;
    Ok(request)
}

pub(crate) async fn youtube_transcript_result(
    store: &Store,
    request: &YoutubeTranscriptRequest,
) -> Result<Option<YoutubeTranscriptResult>, StoreError> {
    if let Some(result) = store.get_json(&result_key(request)).await? {
        return Ok(Some(result));
    }
    store.get_json(&retry_key(request)).await
}

pub(crate) async fn save_youtube_transcript_result(
    store: &Store,
    request: &YoutubeTranscriptRequest,
    result: &YoutubeTranscriptResult,
) -> Result<(), StoreError> {
    let current: Option<YoutubeTranscriptRequest> =
        store.get_json(&request_key(&request.video_id)).await?;
    if current.is_some_and(|current| current.id == request.id) {
        if matches!(result, YoutubeTranscriptResult::Deferred { .. }) {
            store.put_json(&retry_key(request), result).await?;
        } else {
            store.put_json(&result_key(request), result).await?;
            store.delete_key(&retry_key(request)).await?;
        }
    }
    Ok(())
}

pub(crate) async fn pending_youtube_transcript_requests(
    store: &Store,
) -> Result<Vec<YoutubeTranscriptRequest>, StoreError> {
    let mut requests: Vec<YoutubeTranscriptRequest> = store.load_all(REQUESTS).await?;
    requests.sort_by_key(|request| request.requested_at);
    let mut pending = Vec::new();
    for request in requests {
        let ready_to_fetch = match youtube_transcript_result(store, &request).await? {
            None => true,
            Some(YoutubeTranscriptResult::Deferred { retry_at, .. }) => retry_at <= Utc::now(),
            Some(_) => false,
        };
        if ready_to_fetch {
            pending.push(request);
        }
    }
    Ok(pending)
}

pub(crate) async fn cancel_youtube_transcript_request(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    let _guard = REQUEST_LOCK.lock().await;
    let key = request_key(video_id);
    let request: Option<YoutubeTranscriptRequest> = store.get_json(&key).await?;
    store.delete_key(&key).await?;
    if let Some(request) = request {
        store.delete_key(&result_key(&request)).await?;
        store.delete_key(&retry_key(&request)).await?;
    }
    Ok(())
}

pub(crate) async fn requested_youtube_video_ids(
    store: &Store,
) -> Result<std::collections::HashSet<String>, StoreError> {
    Ok(store
        .list_keys(REQUESTS)
        .await?
        .into_iter()
        .filter_map(|key| {
            key.strip_prefix(REQUESTS)?
                .strip_suffix(".json")
                .map(str::to_owned)
        })
        .collect())
}

pub(crate) async fn completed_youtube_video_ids(
    store: &Store,
) -> Result<std::collections::HashSet<String>, StoreError> {
    Ok(store
        .list_keys("youtube-transcript-results/")
        .await?
        .into_iter()
        .filter_map(|key| {
            key.strip_prefix("youtube-transcript-results/")?
                .split_once('/')
                .map(|(id, _)| id.to_owned())
        })
        .collect())
}

#[cfg(test)]
#[path = "transcript_jobs_tests.rs"]
mod tests;
