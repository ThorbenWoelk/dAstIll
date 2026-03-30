use firestore::*;

use crate::models::{ContentStatus, Video};

use super::{Store, StoreError};

pub(super) const COLLECTION: &str = "dastill_videos";


impl From<firestore::errors::FirestoreError> for StoreError {
    fn from(err: firestore::errors::FirestoreError) -> Self {
        StoreError::Other(format!("Firestore error: {err}"))
    }
}

/// Upsert a video, preserving processing state fields when the document already exists.
pub async fn fs_insert_video(
    store: &Store,
    video: &Video,
) -> Result<super::VideoInsertOutcome, StoreError> {
    let existing: Option<Video> = store
        .firestore
        .fluent()
        .select()
        .by_id_in(COLLECTION)
        .obj()
        .one(&video.id)
        .await?;

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
        (video.clone(), super::VideoInsertOutcome::Inserted)
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
    let mut video: Option<Video> = store
        .firestore
        .fluent()
        .select()
        .by_id_in(COLLECTION)
        .obj()
        .one(id)
        .await?;

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
            .obj()
            .batch(chunk_ids)
            .await?;
        while let Some((_id, maybe_video)) = stream.next().await {
            let Some(mut video): Option<Video> = maybe_video else { continue };
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


pub async fn fs_update_video_acknowledged(
    store: &Store,
    video_id: &str,
    acknowledged: bool,
) -> Result<(), StoreError> {
    store
        .firestore
        .fluent()
        .update()
        .fields(paths!(Video::{acknowledged}))
        .in_col(COLLECTION)
        .document_id(video_id)
        .object(&Video {
            acknowledged,
            ..default_video_for_partial_update(video_id)
        })
        .execute::<Video>()
        .await?;
    Ok(())
}

pub async fn fs_update_video_transcript_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    store
        .firestore
        .fluent()
        .update()
        .fields(paths!(Video::{transcript_status}))
        .in_col(COLLECTION)
        .document_id(video_id)
        .object(&Video {
            transcript_status: status,
            ..default_video_for_partial_update(video_id)
        })
        .execute::<Video>()
        .await?;
    Ok(())
}

pub async fn fs_update_video_summary_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    store
        .firestore
        .fluent()
        .update()
        .fields(paths!(Video::{summary_status}))
        .in_col(COLLECTION)
        .document_id(video_id)
        .object(&Video {
            summary_status: status,
            ..default_video_for_partial_update(video_id)
        })
        .execute::<Video>()
        .await?;
    Ok(())
}

pub async fn fs_increment_video_retry_count(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    // Firestore doesn't support server-side increment via the fluent API,
    // so we read-then-write (acceptable for queue processing which is serialized).
    let video: Option<Video> = store
        .firestore
        .fluent()
        .select()
        .by_id_in(COLLECTION)
        .obj()
        .one(video_id)
        .await?;

    if let Some(video) = video {
        let new_count = video.retry_count.saturating_add(1);
        store
            .firestore
            .fluent()
            .update()
            .fields(paths!(Video::{retry_count}))
            .in_col(COLLECTION)
            .document_id(video_id)
            .object(&Video {
                retry_count: new_count,
                ..default_video_for_partial_update(video_id)
            })
            .execute::<Video>()
            .await?;
    }
    Ok(())
}

pub async fn fs_reset_video_retry_count(store: &Store, video_id: &str) -> Result<(), StoreError> {
    store
        .firestore
        .fluent()
        .update()
        .fields(paths!(Video::{retry_count}))
        .in_col(COLLECTION)
        .document_id(video_id)
        .object(&Video {
            retry_count: 0,
            ..default_video_for_partial_update(video_id)
        })
        .execute::<Video>()
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
        let batch: Vec<Video> = store
            .firestore
            .fluent()
            .select()
            .from(COLLECTION)
            .filter(|q| q.field(path!(Video::transcript_status)).eq(status))
            .obj()
            .query()
            .await?;
        pending_transcripts.extend(batch);
    }

    // Query each non-ready summary status where transcript is ready.
    let mut pending_summaries = Vec::new();
    for status in [
        ContentStatus::Pending,
        ContentStatus::Loading,
        ContentStatus::Failed,
    ] {
        let batch: Vec<Video> = store
            .firestore
            .fluent()
            .select()
            .from(COLLECTION)
            .filter(|q| {
                q.field(path!(Video::transcript_status))
                    .eq(ContentStatus::Ready)
            })
            .filter(|q| q.field(path!(Video::summary_status)).eq(status))
            .obj()
            .query()
            .await?;
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
        let batch: Vec<Video> = store
            .firestore
            .fluent()
            .select()
            .from(COLLECTION)
            .filter(|q| q.field(path!(Video::transcript_status)).eq(status))
            .obj()
            .query()
            .await?;
        pending_transcripts.extend(batch);
    }

    let mut pending_summaries = Vec::new();
    for status in [
        ContentStatus::Pending,
        ContentStatus::Loading,
        ContentStatus::Failed,
    ] {
        let batch: Vec<Video> = store
            .firestore
            .fluent()
            .select()
            .from(COLLECTION)
            .filter(|q| {
                q.field(path!(Video::transcript_status))
                    .eq(ContentStatus::Ready)
            })
            .filter(|q| q.field(path!(Video::summary_status)).eq(status))
            .obj()
            .query()
            .await?;
        pending_summaries.extend(batch);
    }

    let mut healed = 0usize;
    for mut video in pending_transcripts.into_iter().chain(pending_summaries) {
        if !super::videos::apply_heal_queue_video_fields(&mut video, max_retries) {
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
