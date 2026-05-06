use libsql::{Value, params};

use crate::models::{ContentStatus, Video};

use super::{Store, StoreError};

impl From<libsql::Error> for StoreError {
    fn from(err: libsql::Error) -> Self {
        StoreError::Other(format!("libSQL error: {err}"))
    }
}

fn content_status_to_str(status: ContentStatus) -> &'static str {
    match status {
        ContentStatus::Pending => "pending",
        ContentStatus::Loading => "loading",
        ContentStatus::Ready => "ready",
        ContentStatus::Failed => "failed",
    }
}

fn content_status_from_str(s: &str) -> ContentStatus {
    match s {
        "loading" => ContentStatus::Loading,
        "ready" => ContentStatus::Ready,
        "failed" => ContentStatus::Failed,
        _ => ContentStatus::Pending,
    }
}

fn parse_published_at(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

fn row_to_video(row: &libsql::Row) -> Result<Video, StoreError> {
    let id: String = row.get(0)?;
    let channel_id: String = row.get(1)?;
    let title: String = row.get(2)?;
    let thumbnail_url: Option<String> = match row.get_value(3)? {
        Value::Null => None,
        Value::Text(s) => Some(s),
        _ => None,
    };
    let published_at_str: String = row.get(4)?;
    let is_short: i64 = row.get(5)?;
    let transcript_status_str: String = row.get(6)?;
    let summary_status_str: String = row.get(7)?;
    let retry_count: i64 = row.get(8)?;
    let quality_score: Option<u8> = match row.get_value(9)? {
        Value::Null => None,
        Value::Integer(v) => Some(v.clamp(0, 255) as u8),
        _ => None,
    };

    Ok(Video {
        id,
        channel_id,
        title,
        thumbnail_url,
        published_at: parse_published_at(&published_at_str),
        is_short: is_short != 0,
        transcript_status: content_status_from_str(&transcript_status_str),
        summary_status: content_status_from_str(&summary_status_str),
        acknowledged: false,
        retry_count: retry_count.clamp(0, 255) as u8,
        quality_score,
    })
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

const SELECT_ALL_COLUMNS: &str = "id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, retry_count, quality_score";

/// Upsert a video, preserving processing state fields when the row already exists.
pub async fn sql_insert_video(
    store: &Store,
    video: &Video,
) -> Result<super::VideoInsertOutcome, StoreError> {
    // Check if existing
    let mut rows = store
        .sql
        .query(
            &format!("SELECT {SELECT_ALL_COLUMNS} FROM videos WHERE id = ?1"),
            params![video.id.clone()],
        )
        .await?;

    let existing = if let Some(row) = rows.next().await? {
        Some(row_to_video(&row)?)
    } else {
        None
    };

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
            acknowledged: false,
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
        .sql
        .execute(
            r#"INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, retry_count, quality_score)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
               ON CONFLICT(id) DO UPDATE SET
                 channel_id = excluded.channel_id,
                 title = excluded.title,
                 thumbnail_url = excluded.thumbnail_url,
                 published_at = excluded.published_at,
                 is_short = excluded.is_short,
                 transcript_status = excluded.transcript_status,
                 summary_status = excluded.summary_status,
                 retry_count = excluded.retry_count,
                 quality_score = excluded.quality_score"#,
            params![
                merged.id.clone(),
                merged.channel_id.clone(),
                merged.title.clone(),
                merged.thumbnail_url.clone(),
                merged.published_at.to_rfc3339(),
                merged.is_short as i64,
                content_status_to_str(merged.transcript_status),
                content_status_to_str(merged.summary_status),
                merged.retry_count as i64,
                merged.quality_score.map(|v| v as i64),
            ],
        )
        .await?;

    match outcome {
        super::VideoInsertOutcome::Inserted => {
            tracing::info!(video_id = %video.id, title = %video.title, "inserted new video (libsql)");
        }
        super::VideoInsertOutcome::Existing => {
            tracing::debug!(video_id = %video.id, title = %video.title, "found existing video (libsql)");
        }
    }

    Ok(outcome)
}

pub async fn sql_bulk_insert_videos(
    store: &Store,
    videos: Vec<Video>,
) -> Result<usize, StoreError> {
    if videos.is_empty() {
        return Ok(0);
    }

    // Batch-fetch existing rows
    let ids: Vec<String> = videos.iter().map(|v| v.id.clone()).collect();
    let mut existing_map = std::collections::HashMap::new();
    for chunk in ids.chunks(30) {
        let placeholders: String = chunk
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("SELECT {SELECT_ALL_COLUMNS} FROM videos WHERE id IN ({placeholders})");
        let values: Vec<Value> = chunk.iter().map(|id| Value::Text(id.clone())).collect();
        let mut rows = store
            .sql
            .query(&sql, libsql::params_from_iter(values))
            .await?;
        while let Some(row) = rows.next().await? {
            if let Ok(video) = row_to_video(&row) {
                existing_map.insert(video.id.clone(), video);
            }
        }
    }

    let mut inserted = 0usize;
    for video in &videos {
        let (merged, outcome) = if let Some(existing) = existing_map.get(&video.id) {
            let merged = Video {
                id: video.id.clone(),
                channel_id: video.channel_id.clone(),
                title: video.title.clone(),
                thumbnail_url: video.thumbnail_url.clone(),
                published_at: video.published_at,
                is_short: video.is_short,
                transcript_status: existing.transcript_status,
                summary_status: existing.summary_status,
                acknowledged: false,
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
            .sql
            .execute(
                r#"INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, retry_count, quality_score)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                   ON CONFLICT(id) DO UPDATE SET
                     channel_id = excluded.channel_id,
                     title = excluded.title,
                     thumbnail_url = excluded.thumbnail_url,
                     published_at = excluded.published_at,
                     is_short = excluded.is_short,
                     transcript_status = excluded.transcript_status,
                     summary_status = excluded.summary_status,
                     retry_count = excluded.retry_count,
                     quality_score = excluded.quality_score"#,
                params![
                    merged.id.clone(),
                    merged.channel_id.clone(),
                    merged.title.clone(),
                    merged.thumbnail_url.clone(),
                    merged.published_at.to_rfc3339(),
                    merged.is_short as i64,
                    content_status_to_str(merged.transcript_status),
                    content_status_to_str(merged.summary_status),
                    merged.retry_count as i64,
                    merged.quality_score.map(|v| v as i64),
                ],
            )
            .await?;

        if outcome == super::VideoInsertOutcome::Inserted {
            tracing::info!(video_id = %video.id, title = %video.title, "inserted new video (libsql bulk)");
            inserted += 1;
        }
    }
    Ok(inserted)
}

pub async fn sql_get_video(
    store: &Store,
    id: &str,
    include_summary: bool,
) -> Result<Option<Video>, StoreError> {
    let mut rows = store
        .sql
        .query(
            &format!("SELECT {SELECT_ALL_COLUMNS} FROM videos WHERE id = ?1"),
            params![id],
        )
        .await?;

    let mut video = match rows.next().await? {
        Some(row) => Some(row_to_video(&row)?),
        None => None,
    };

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

pub async fn sql_get_videos(
    store: &Store,
    ids: &[impl AsRef<str>],
    include_summary: bool,
) -> Result<std::collections::HashMap<String, Video>, StoreError> {
    if ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let mut results = std::collections::HashMap::new();

    for chunk in ids.chunks(30) {
        let placeholders: String = chunk
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("SELECT {SELECT_ALL_COLUMNS} FROM videos WHERE id IN ({placeholders})");
        let values: Vec<Value> = chunk
            .iter()
            .map(|id| Value::Text(id.as_ref().to_string()))
            .collect();
        let mut rows = store
            .sql
            .query(&sql, libsql::params_from_iter(values))
            .await?;
        while let Some(row) = rows.next().await? {
            let mut video = row_to_video(&row)?;
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

pub async fn sql_load_all_videos(store: &Store) -> Result<Vec<Video>, StoreError> {
    let mut rows = store
        .sql
        .query(&format!("SELECT {SELECT_ALL_COLUMNS} FROM videos"), ())
        .await?;

    let mut videos = Vec::new();
    while let Some(row) = rows.next().await? {
        match row_to_video(&row) {
            Ok(video) => videos.push(video),
            Err(err) => {
                tracing::warn!(error = %err, "skipping malformed video row in load_all_videos");
            }
        }
    }
    Ok(videos)
}

pub async fn sql_count_videos(store: &Store) -> Result<usize, StoreError> {
    let mut rows = store.sql.query("SELECT COUNT(*) FROM videos", ()).await?;
    let Some(row) = rows.next().await? else {
        return Ok(0);
    };
    let count: i64 = row.get(0)?;
    Ok(count.max(0) as usize)
}

pub async fn sql_list_channel_videos_window(
    store: &Store,
    channel_id: &str,
    limit: usize,
    offset: usize,
    descending: bool,
) -> Result<Vec<Video>, StoreError> {
    let order = if descending { "DESC" } else { "ASC" };
    let sql = format!(
        "SELECT {SELECT_ALL_COLUMNS} FROM videos WHERE channel_id = ?1 ORDER BY published_at {order} LIMIT ?2 OFFSET ?3"
    );
    let mut rows = store
        .sql
        .query(&sql, params![channel_id, limit as i64, offset as i64])
        .await?;

    let mut videos = Vec::new();
    while let Some(row) = rows.next().await? {
        match row_to_video(&row) {
            Ok(video) => videos.push(video),
            Err(err) => {
                tracing::warn!(error = %err, "skipping malformed video row in list_channel_videos_window");
            }
        }
    }
    Ok(videos)
}

pub async fn sql_update_video_transcript_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    store
        .sql
        .execute(
            "UPDATE videos SET transcript_status = ?1 WHERE id = ?2",
            params![content_status_to_str(status), video_id],
        )
        .await?;
    Ok(())
}

pub async fn sql_update_video_summary_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    store
        .sql
        .execute(
            "UPDATE videos SET summary_status = ?1 WHERE id = ?2",
            params![content_status_to_str(status), video_id],
        )
        .await?;
    Ok(())
}

/// Atomic increment — no read-then-write needed with SQL.
pub async fn sql_increment_video_retry_count(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    store
        .sql
        .execute(
            "UPDATE videos SET retry_count = MIN(retry_count + 1, 255) WHERE id = ?1",
            params![video_id],
        )
        .await?;
    Ok(())
}

pub async fn sql_reset_video_retry_count(store: &Store, video_id: &str) -> Result<(), StoreError> {
    store
        .sql
        .execute(
            "UPDATE videos SET retry_count = 0 WHERE id = ?1",
            params![video_id],
        )
        .await?;
    Ok(())
}

pub async fn sql_heal_queue_videos(
    store: &Store,
    max_retries: u8,
) -> Result<Vec<String>, StoreError> {
    // Fetch all non-ready videos in a single query.
    let mut rows = store
        .sql
        .query(
            &format!(
                "SELECT {SELECT_ALL_COLUMNS} FROM videos WHERE transcript_status != 'ready' OR (transcript_status = 'ready' AND summary_status != 'ready')"
            ),
            (),
        )
        .await?;

    let mut candidates = Vec::new();
    while let Some(row) = rows.next().await? {
        if let Ok(video) = row_to_video(&row) {
            candidates.push(video);
        }
    }

    let mut healed_video_ids = Vec::new();
    for mut video in candidates {
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
            .sql
            .execute(
                "UPDATE videos SET transcript_status = ?1, summary_status = ?2, retry_count = ?3 WHERE id = ?4",
                params![
                    content_status_to_str(video.transcript_status),
                    content_status_to_str(video.summary_status),
                    video.retry_count as i64,
                    video.id.clone(),
                ],
            )
            .await?;
        healed_video_ids.push(video.id.clone());
    }
    Ok(healed_video_ids)
}

pub async fn sql_delete_videos(store: &Store, video_ids: &[String]) -> Result<(), StoreError> {
    if video_ids.is_empty() {
        return Ok(());
    }

    for chunk in video_ids.chunks(30) {
        let placeholders = chunk
            .iter()
            .enumerate()
            .map(|(index, _)| format!("?{}", index + 1))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("DELETE FROM videos WHERE id IN ({placeholders})");
        let values: Vec<Value> = chunk
            .iter()
            .map(|video_id| Value::Text(video_id.clone()))
            .collect();
        store
            .sql
            .execute(&sql, libsql::params_from_iter(values))
            .await?;
    }

    Ok(())
}

pub async fn sql_update_video_quality_score(
    store: &Store,
    video_id: &str,
    quality_score: Option<u8>,
) -> Result<(), StoreError> {
    store
        .sql
        .execute(
            "UPDATE videos SET quality_score = ?1 WHERE id = ?2",
            params![quality_score.map(|v| v as i64), video_id],
        )
        .await?;
    Ok(())
}

#[cfg(test)]
#[path = "sql_videos_tests.rs"]
mod sql_videos_tests;
