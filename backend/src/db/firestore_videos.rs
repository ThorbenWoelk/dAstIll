use firestore::*;
use serde::{Deserialize, Serialize};

use crate::models::{ContentStatus, Video};

use super::{Store, StoreError};

pub(super) const COLLECTION: &str = "dastill_videos";

impl From<firestore::errors::FirestoreError> for StoreError {
    fn from(err: firestore::errors::FirestoreError) -> Self {
        StoreError::Other(format!("Firestore error: {err}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredVideoRecord {
    #[serde(default)]
    id: Option<String>,
    channel_id: String,
    title: String,
    thumbnail_url: Option<String>,
    published_at: chrono::DateTime<chrono::Utc>,
    is_short: bool,
    transcript_status: ContentStatus,
    summary_status: ContentStatus,
    acknowledged: bool,
    #[serde(default)]
    retry_count: u8,
    quality_score: Option<u8>,
}

fn document_id_from_path(document: &FirestoreDocument) -> Result<&str, StoreError> {
    document
        .name
        .rsplit('/')
        .next()
        .filter(|id| !id.is_empty())
        .ok_or_else(|| {
            StoreError::Other(format!(
                "Firestore video document missing document ID in path `{}`",
                document.name
            ))
        })
}

fn deserialize_video_document(document: &FirestoreDocument) -> Result<Video, StoreError> {
    let record: StoredVideoRecord = firestore_document_to_serializable(document)?;
    let id = record
        .id
        .as_deref()
        .filter(|id| !id.trim().is_empty())
        .unwrap_or(document_id_from_path(document)?)
        .to_string();

    Ok(Video {
        id,
        channel_id: record.channel_id,
        title: record.title,
        thumbnail_url: record.thumbnail_url,
        published_at: record.published_at,
        is_short: record.is_short,
        transcript_status: record.transcript_status,
        summary_status: record.summary_status,
        acknowledged: record.acknowledged,
        retry_count: record.retry_count,
        quality_score: record.quality_score,
    })
}

fn log_malformed_video_document(document: &FirestoreDocument, error: &StoreError, operation: &str) {
    tracing::warn!(
        operation,
        document_path = %document.name,
        error = %error,
        "skipping malformed Firestore video document"
    );
}

fn deserialize_video_document_lossy(
    document: &FirestoreDocument,
    operation: &str,
) -> Option<Video> {
    match deserialize_video_document(document) {
        Ok(video) => Some(video),
        Err(error) => {
            log_malformed_video_document(document, &error, operation);
            None
        }
    }
}

fn deserialize_video_documents(documents: Vec<FirestoreDocument>, operation: &str) -> Vec<Video> {
    documents
        .iter()
        .filter_map(|document| deserialize_video_document_lossy(document, operation))
        .collect()
}

fn partial_update_document(
    store: &Store,
    video_id: &str,
    video: &Video,
) -> Result<FirestoreDocument, StoreError> {
    let document_path = format!(
        "{}/{COLLECTION}/{video_id}",
        store.firestore.get_documents_path()
    );
    Ok(firestore_document_from_serializable(document_path, video)?)
}

async fn delete_video_document(store: &Store, video_id: &str) -> Result<(), StoreError> {
    store
        .firestore
        .fluent()
        .delete()
        .from(COLLECTION)
        .document_id(video_id)
        .execute()
        .await?;
    Ok(())
}

fn transcript_storage_key(video_id: &str) -> String {
    format!("transcripts/{video_id}.json")
}

fn summary_storage_key(video_id: &str) -> String {
    format!("summaries/{video_id}.json")
}

fn reconcile_video_statuses_from_storage(
    video: &Video,
    transcript_exists: bool,
    summary_exists: bool,
) -> Video {
    let mut reconciled = video.clone();
    if transcript_exists {
        reconciled.transcript_status = ContentStatus::Ready;
    }
    if summary_exists {
        reconciled.summary_status = ContentStatus::Ready;
    }
    reconciled
}

async fn hydrate_inserted_video_from_storage(
    store: &Store,
    video: &Video,
) -> Result<Video, StoreError> {
    let transcript_exists = store.key_exists(&transcript_storage_key(&video.id)).await?;
    let summary_exists = store.key_exists(&summary_storage_key(&video.id)).await?;
    Ok(reconcile_video_statuses_from_storage(
        video,
        transcript_exists,
        summary_exists,
    ))
}

/// Upsert a video, preserving processing state fields when the document already exists.
pub async fn fs_insert_video(
    store: &Store,
    video: &Video,
) -> Result<super::VideoInsertOutcome, StoreError> {
    let existing = store
        .firestore
        .fluent()
        .select()
        .by_id_in(COLLECTION)
        .one(&video.id)
        .await?
        .as_ref()
        .and_then(|document| deserialize_video_document_lossy(document, "insert_video_existing"));

    let (merged, outcome) = if let Some(existing) = existing {
        let merged = Video {
            id: video.id.clone(),
            channel_id: video.channel_id.clone(),
            title: video.title.clone(),
            thumbnail_url: video.thumbnail_url.clone(),
            published_at: video.published_at,
            is_short: video.is_short,
            transcript_status: existing.transcript_status,
            summary_status: existing.summary_status,
            acknowledged: existing.acknowledged,
            retry_count: existing.retry_count,
            quality_score: existing.quality_score,
        };
        (merged, super::VideoInsertOutcome::Existing)
    } else {
        (
            hydrate_inserted_video_from_storage(store, video).await?,
            super::VideoInsertOutcome::Inserted,
        )
    };

    store
        .firestore
        .fluent()
        .update()
        .in_col(COLLECTION)
        .document_id(&merged.id)
        .object(&merged)
        .execute::<Video>()
        .await?;

    match outcome {
        super::VideoInsertOutcome::Inserted => {
            tracing::info!(video_id = %video.id, title = %video.title, "inserted new video (firestore)");
        }
        super::VideoInsertOutcome::Existing => {
            tracing::debug!(video_id = %video.id, title = %video.title, "found existing video (firestore)");
        }
    }

    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use firestore::firestore_document_from_serializable;

    use super::{COLLECTION, deserialize_video_document, reconcile_video_statuses_from_storage};
    use crate::models::{ContentStatus, Video};

    fn build_video() -> Video {
        Video {
            id: "video-1".to_string(),
            channel_id: "channel-1".to_string(),
            title: "Example".to_string(),
            thumbnail_url: None,
            published_at: chrono::Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Pending,
            summary_status: ContentStatus::Pending,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        }
    }

    #[test]
    fn inserted_video_becomes_ready_when_storage_artifacts_exist() {
        let video = build_video();
        let reconciled = reconcile_video_statuses_from_storage(&video, true, true);
        assert_eq!(reconciled.transcript_status, ContentStatus::Ready);
        assert_eq!(reconciled.summary_status, ContentStatus::Ready);
    }

    #[test]
    fn inserted_video_preserves_missing_summary_when_only_transcript_exists() {
        let video = build_video();
        let reconciled = reconcile_video_statuses_from_storage(&video, true, false);
        assert_eq!(reconciled.transcript_status, ContentStatus::Ready);
        assert_eq!(reconciled.summary_status, ContentStatus::Pending);
    }

    #[test]
    fn storage_reconcile_preserves_existing_ready_statuses() {
        let mut video = build_video();
        video.transcript_status = ContentStatus::Ready;
        let reconciled = reconcile_video_statuses_from_storage(&video, false, false);
        assert_eq!(reconciled.transcript_status, ContentStatus::Ready);
        assert_eq!(reconciled.summary_status, ContentStatus::Pending);
    }

    #[test]
    fn legacy_firestore_video_without_id_uses_document_id() {
        #[derive(serde::Serialize)]
        struct LegacyVideoRecord {
            channel_id: String,
            title: String,
            thumbnail_url: Option<String>,
            published_at: chrono::DateTime<chrono::Utc>,
            is_short: bool,
            transcript_status: ContentStatus,
            summary_status: ContentStatus,
            acknowledged: bool,
            retry_count: u8,
            quality_score: Option<u8>,
        }

        let record = LegacyVideoRecord {
            channel_id: "channel-1".to_string(),
            title: "Legacy".to_string(),
            thumbnail_url: None,
            published_at: chrono::Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Pending,
            acknowledged: true,
            retry_count: 2,
            quality_score: Some(7),
        };
        let document = firestore_document_from_serializable(
            format!(
                "projects/test-project/databases/(default)/documents/{COLLECTION}/legacy-video"
            ),
            &record,
        )
        .expect("legacy firestore document");

        let video = deserialize_video_document(&document).expect("legacy video should deserialize");

        assert_eq!(video.id, "legacy-video");
        assert_eq!(video.channel_id, "channel-1");
        assert_eq!(video.title, "Legacy");
        assert_eq!(video.retry_count, 2);
        assert_eq!(video.quality_score, Some(7));
    }

    #[test]
    fn malformed_firestore_video_without_channel_id_is_skipped_in_bulk_reads() {
        #[derive(serde::Serialize)]
        struct MalformedVideoRecord {
            title: String,
            thumbnail_url: Option<String>,
            published_at: chrono::DateTime<chrono::Utc>,
            is_short: bool,
            transcript_status: ContentStatus,
            summary_status: ContentStatus,
            acknowledged: bool,
            retry_count: u8,
            quality_score: Option<u8>,
        }

        let record = MalformedVideoRecord {
            title: "Broken".to_string(),
            thumbnail_url: None,
            published_at: chrono::Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Pending,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        };
        let document = firestore_document_from_serializable(
            format!(
                "projects/test-project/databases/(default)/documents/{COLLECTION}/broken-video"
            ),
            &record,
        )
        .expect("malformed firestore document");

        let videos = super::deserialize_video_documents(vec![document], "test_bulk_read");

        assert!(videos.is_empty());
    }
}

pub async fn fs_bulk_insert_videos(store: &Store, videos: Vec<Video>) -> Result<usize, StoreError> {
    if videos.is_empty() {
        return Ok(0);
    }

    let mut inserted = 0usize;
    for video in &videos {
        let outcome = fs_insert_video(store, video).await?;
        if outcome == super::VideoInsertOutcome::Inserted {
            inserted += 1;
        }
    }
    Ok(inserted)
}

pub async fn fs_get_video(
    store: &Store,
    id: &str,
    include_summary: bool,
) -> Result<Option<Video>, StoreError> {
    let mut video = store
        .firestore
        .fluent()
        .select()
        .by_id_in(COLLECTION)
        .one(id)
        .await?
        .as_ref()
        .and_then(|document| deserialize_video_document_lossy(document, "get_video"));

    if include_summary {
        if let Some(ref mut v) = video {
            if let Some(summary) = store
                .get_json::<crate::models::Summary>(&format!("summaries/{id}.json"))
                .await?
            {
                v.quality_score = summary.quality_score;
            }
        }
    }

    Ok(video)
}

pub async fn fs_get_videos(
    store: &Store,
    ids: &[impl AsRef<str>],
    include_summary: bool,
) -> Result<std::collections::HashMap<String, Video>, StoreError> {
    if ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let mut results = std::collections::HashMap::new();

    // by_id_in supports up to 30 document IDs per call and returns a stream of (id, Option<doc>).
    for chunk in ids.chunks(30) {
        let chunk_ids: Vec<&str> = chunk.iter().map(|s| s.as_ref()).collect();
        use tokio_stream::StreamExt;
        let mut stream = store
            .firestore
            .fluent()
            .select()
            .by_id_in(COLLECTION)
            .batch(chunk_ids)
            .await?;
        while let Some((_id, maybe_document)) = stream.next().await {
            let Some(document) = maybe_document else {
                continue;
            };
            let Some(mut video) = deserialize_video_document_lossy(&document, "get_videos_batch")
            else {
                continue;
            };
            if include_summary {
                if let Some(summary) = store
                    .get_json::<crate::models::Summary>(&format!("summaries/{}.json", video.id))
                    .await?
                {
                    video.quality_score = summary.quality_score;
                }
            }
            results.insert(video.id.clone(), video);
        }
    }

    Ok(results)
}

pub async fn fs_load_all_videos(store: &Store) -> Result<Vec<Video>, StoreError> {
    let documents = store
        .firestore
        .fluent()
        .select()
        .from(COLLECTION)
        .query()
        .await?;
    Ok(deserialize_video_documents(documents, "load_all_videos"))
}

pub async fn prune_malformed_video_documents(store: &Store) -> Result<usize, StoreError> {
    let documents = store
        .firestore
        .fluent()
        .select()
        .from(COLLECTION)
        .query()
        .await?;

    let mut pruned = 0usize;
    for document in documents {
        let Err(error) = deserialize_video_document(&document) else {
            continue;
        };
        let Ok(video_id) = document_id_from_path(&document).map(str::to_string) else {
            tracing::error!(
                document_path = %document.name,
                error = %error,
                "encountered malformed Firestore video document with unreadable path"
            );
            continue;
        };

        tracing::warn!(
            video_id = %video_id,
            document_path = %document.name,
            error = %error,
            "deleting malformed Firestore video document so canonical ingest can repopulate it"
        );
        delete_video_document(store, &video_id).await?;
        pruned += 1;
    }

    Ok(pruned)
}

pub async fn fs_update_video_acknowledged(
    store: &Store,
    video_id: &str,
    acknowledged: bool,
) -> Result<(), StoreError> {
    let document = partial_update_document(
        store,
        video_id,
        &Video {
            acknowledged,
            ..default_video_for_partial_update(video_id)
        },
    )?;
    store
        .firestore
        .fluent()
        .update()
        .fields(paths!(Video::{acknowledged}))
        .in_col(COLLECTION)
        .document(document)
        .execute()
        .await?;
    Ok(())
}

pub async fn fs_update_video_transcript_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    let document = partial_update_document(
        store,
        video_id,
        &Video {
            transcript_status: status,
            ..default_video_for_partial_update(video_id)
        },
    )?;
    store
        .firestore
        .fluent()
        .update()
        .fields(paths!(Video::{transcript_status}))
        .in_col(COLLECTION)
        .document(document)
        .execute()
        .await?;
    Ok(())
}

pub async fn fs_update_video_summary_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    let document = partial_update_document(
        store,
        video_id,
        &Video {
            summary_status: status,
            ..default_video_for_partial_update(video_id)
        },
    )?;
    store
        .firestore
        .fluent()
        .update()
        .fields(paths!(Video::{summary_status}))
        .in_col(COLLECTION)
        .document(document)
        .execute()
        .await?;
    Ok(())
}

pub async fn fs_increment_video_retry_count(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    // Firestore doesn't support server-side increment via the fluent API,
    // so we read-then-write (acceptable for queue processing which is serialized).
    let video = store
        .firestore
        .fluent()
        .select()
        .by_id_in(COLLECTION)
        .one(video_id)
        .await?
        .as_ref()
        .and_then(|document| deserialize_video_document_lossy(document, "increment_retry_count"));

    if let Some(video) = video {
        let new_count = video.retry_count.saturating_add(1);
        let document = partial_update_document(
            store,
            video_id,
            &Video {
                retry_count: new_count,
                ..default_video_for_partial_update(video_id)
            },
        )?;
        store
            .firestore
            .fluent()
            .update()
            .fields(paths!(Video::{retry_count}))
            .in_col(COLLECTION)
            .document(document)
            .execute()
            .await?;
    }
    Ok(())
}

pub async fn fs_reset_video_retry_count(store: &Store, video_id: &str) -> Result<(), StoreError> {
    let document = partial_update_document(
        store,
        video_id,
        &Video {
            retry_count: 0,
            ..default_video_for_partial_update(video_id)
        },
    )?;
    store
        .firestore
        .fluent()
        .update()
        .fields(paths!(Video::{retry_count}))
        .in_col(COLLECTION)
        .document(document)
        .execute()
        .await?;
    Ok(())
}

pub async fn fs_list_videos_for_queue_processing(
    store: &Store,
    limit: usize,
    max_retries: u8,
) -> Result<Vec<Video>, StoreError> {
    // Query each non-ready transcript status with equality filters (avoids neq range indexes).
    let mut pending_transcripts = Vec::new();
    for status in [
        ContentStatus::Pending,
        ContentStatus::Loading,
        ContentStatus::Failed,
    ] {
        let batch = deserialize_video_documents(
            store
                .firestore
                .fluent()
                .select()
                .from(COLLECTION)
                .filter(|q| q.field(path!(Video::transcript_status)).eq(status))
                .query()
                .await?,
            "queue_processing_transcripts",
        );
        pending_transcripts.extend(batch);
    }

    // Query each non-ready summary status where transcript is ready.
    let mut pending_summaries = Vec::new();
    for status in [
        ContentStatus::Pending,
        ContentStatus::Loading,
        ContentStatus::Failed,
    ] {
        let batch = deserialize_video_documents(
            store
                .firestore
                .fluent()
                .select()
                .from(COLLECTION)
                .filter(|q| {
                    q.field(path!(Video::transcript_status))
                        .eq(ContentStatus::Ready)
                })
                .filter(|q| q.field(path!(Video::summary_status)).eq(status))
                .query()
                .await?,
            "queue_processing_summaries",
        );
        pending_summaries.extend(batch);
    }

    let mut combined: Vec<Video> = pending_transcripts
        .into_iter()
        .chain(pending_summaries)
        .filter(|v| v.retry_count < max_retries)
        .collect();

    combined.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    combined.truncate(limit);
    Ok(combined)
}

pub async fn fs_heal_queue_videos(store: &Store, max_retries: u8) -> Result<usize, StoreError> {
    // Query each non-ready transcript status with equality filters.
    let mut pending_transcripts = Vec::new();
    for status in [
        ContentStatus::Pending,
        ContentStatus::Loading,
        ContentStatus::Failed,
    ] {
        let batch = deserialize_video_documents(
            store
                .firestore
                .fluent()
                .select()
                .from(COLLECTION)
                .filter(|q| q.field(path!(Video::transcript_status)).eq(status))
                .query()
                .await?,
            "heal_queue_transcripts",
        );
        pending_transcripts.extend(batch);
    }

    let mut pending_summaries = Vec::new();
    for status in [
        ContentStatus::Pending,
        ContentStatus::Loading,
        ContentStatus::Failed,
    ] {
        let batch = deserialize_video_documents(
            store
                .firestore
                .fluent()
                .select()
                .from(COLLECTION)
                .filter(|q| {
                    q.field(path!(Video::transcript_status))
                        .eq(ContentStatus::Ready)
                })
                .filter(|q| q.field(path!(Video::summary_status)).eq(status))
                .query()
                .await?,
            "heal_queue_summaries",
        );
        pending_summaries.extend(batch);
    }

    let mut healed = 0usize;
    for mut video in pending_transcripts.into_iter().chain(pending_summaries) {
        let reconciled = hydrate_inserted_video_from_storage(store, &video).await?;
        let mut changed = reconciled.transcript_status != video.transcript_status
            || reconciled.summary_status != video.summary_status;
        video = reconciled;

        if super::videos::apply_heal_queue_video_fields(&mut video, max_retries) {
            changed = true;
        }

        if !changed {
            continue;
        }
        store
            .firestore
            .fluent()
            .update()
            .in_col(COLLECTION)
            .document_id(&video.id)
            .object(&video)
            .execute::<Video>()
            .await?;
        healed += 1;
    }
    Ok(healed)
}

/// Placeholder Video used for partial field updates where only specific fields
/// are written (controlled by the `fields()` mask).
fn default_video_for_partial_update(id: &str) -> Video {
    Video {
        id: id.to_string(),
        channel_id: String::new(),
        title: String::new(),
        thumbnail_url: None,
        published_at: chrono::Utc::now(),
        is_short: false,
        transcript_status: ContentStatus::Pending,
        summary_status: ContentStatus::Pending,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    }
}
