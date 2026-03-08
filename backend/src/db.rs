use std::sync::Arc;

use libsql::{Connection, Value, params};
use tokio::sync::Mutex;

use crate::models::{
    Channel, ContentStatus, Summary, SummaryEvaluationJob, Transcript, Video, VideoInfo,
};

pub type DbPool = Arc<Mutex<Connection>>;

#[derive(Debug, Clone)]
pub struct ChannelSnapshotData {
    pub channel: Channel,
    pub derived_earliest_ready_date: Option<chrono::DateTime<chrono::Utc>>,
    pub videos: Vec<Video>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceBootstrapData {
    pub channels: Vec<Channel>,
    pub selected_channel_id: Option<String>,
    pub snapshot: Option<ChannelSnapshotData>,
}

pub async fn init_db(db: libsql::Database) -> Result<DbPool, libsql::Error> {
    let conn = db.connect()?;
    run_migrations(&conn).await?;
    Ok(Arc::new(Mutex::new(conn)))
}

pub async fn init_db_memory() -> Result<DbPool, libsql::Error> {
    let db = libsql::Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    run_migrations(&conn).await?;
    Ok(Arc::new(Mutex::new(conn)))
}

async fn run_migrations(conn: &Connection) -> Result<(), libsql::Error> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS channels (
            id TEXT PRIMARY KEY,
            handle TEXT,
            name TEXT NOT NULL,
            thumbnail_url TEXT,
            added_at TEXT NOT NULL,
            earliest_sync_date TEXT
        );

        CREATE TABLE IF NOT EXISTS videos (
            id TEXT PRIMARY KEY,
            channel_id TEXT NOT NULL,
            title TEXT NOT NULL,
            thumbnail_url TEXT,
            published_at TEXT NOT NULL,
            is_short INTEGER NOT NULL DEFAULT 0,
            transcript_status TEXT DEFAULT 'pending',
            summary_status TEXT DEFAULT 'pending',
            acknowledged INTEGER NOT NULL DEFAULT 0,
            retry_count INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(channel_id) REFERENCES channels(id)
        );

        CREATE TABLE IF NOT EXISTS transcripts (
            video_id TEXT PRIMARY KEY,
            raw_text TEXT,
            formatted_markdown TEXT,
            FOREIGN KEY(video_id) REFERENCES videos(id)
        );

        CREATE TABLE IF NOT EXISTS summaries (
            video_id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            model_used TEXT,
            quality_score INTEGER,
            auto_regen_attempts INTEGER NOT NULL DEFAULT 0,
            quality_note TEXT,
            FOREIGN KEY(video_id) REFERENCES videos(id)
        );

        CREATE TABLE IF NOT EXISTS video_info (
            video_id TEXT PRIMARY KEY,
            watch_url TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            thumbnail_url TEXT,
            channel_name TEXT,
            channel_id TEXT,
            published_at TEXT,
            duration_iso8601 TEXT,
            duration_seconds INTEGER,
            view_count INTEGER,
            fetched_at TEXT NOT NULL,
            FOREIGN KEY(video_id) REFERENCES videos(id)
        );

        CREATE INDEX IF NOT EXISTS idx_videos_channel ON videos(channel_id);
        CREATE INDEX IF NOT EXISTS idx_videos_published ON videos(published_at DESC);
        CREATE INDEX IF NOT EXISTS idx_videos_channel_published ON videos(channel_id, published_at DESC);
        CREATE INDEX IF NOT EXISTS idx_videos_channel_short_published ON videos(channel_id, is_short, published_at DESC);
        CREATE INDEX IF NOT EXISTS idx_videos_channel_ack_published ON videos(channel_id, acknowledged, published_at DESC);
        CREATE INDEX IF NOT EXISTS idx_video_info_fetched_at ON video_info(fetched_at DESC);
        "#,
    )
    .await?;
    ensure_videos_is_short_column(conn).await?;
    ensure_videos_acknowledged_column(conn).await?;
    ensure_videos_retry_count_column(conn).await?;
    ensure_summary_quality_columns(conn).await?;
    ensure_channels_earliest_sync_date_column(conn).await?;
    ensure_channels_earliest_sync_date_user_set_column(conn).await?;
    Ok(())
}

async fn ensure_channels_earliest_sync_date_user_set_column(
    conn: &Connection,
) -> Result<(), libsql::Error> {
    let mut rows = conn.query("PRAGMA table_info(channels)", ()).await?;
    let mut has_col = false;
    while let Some(row) = rows.next().await? {
        let name: String = row.get(1)?;
        if name == "earliest_sync_date_user_set" {
            has_col = true;
            break;
        }
    }

    if !has_col {
        conn.execute(
            "ALTER TABLE channels ADD COLUMN earliest_sync_date_user_set INTEGER NOT NULL DEFAULT 0",
            (),
        )
        .await?;
    }

    Ok(())
}

async fn ensure_channels_earliest_sync_date_column(conn: &Connection) -> Result<(), libsql::Error> {
    let mut rows = conn.query("PRAGMA table_info(channels)", ()).await?;
    let mut has_col = false;
    while let Some(row) = rows.next().await? {
        let name: String = row.get(1)?;
        if name == "earliest_sync_date" {
            has_col = true;
            break;
        }
    }

    if !has_col {
        conn.execute(
            "ALTER TABLE channels ADD COLUMN earliest_sync_date TEXT",
            (),
        )
        .await?;
    }

    Ok(())
}

async fn ensure_videos_is_short_column(conn: &Connection) -> Result<(), libsql::Error> {
    let mut rows = conn.query("PRAGMA table_info(videos)", ()).await?;
    let mut has_is_short = false;
    while let Some(row) = rows.next().await? {
        let name: String = row.get(1)?;
        if name == "is_short" {
            has_is_short = true;
            break;
        }
    }

    if !has_is_short {
        conn.execute(
            "ALTER TABLE videos ADD COLUMN is_short INTEGER NOT NULL DEFAULT 0",
            (),
        )
        .await?;
    }

    Ok(())
}

async fn ensure_videos_acknowledged_column(conn: &Connection) -> Result<(), libsql::Error> {
    let mut rows = conn.query("PRAGMA table_info(videos)", ()).await?;
    let mut has_col = false;
    while let Some(row) = rows.next().await? {
        let name: String = row.get(1)?;
        if name == "acknowledged" {
            has_col = true;
            break;
        }
    }

    if !has_col {
        conn.execute(
            "ALTER TABLE videos ADD COLUMN acknowledged INTEGER NOT NULL DEFAULT 0",
            (),
        )
        .await?;
    }

    Ok(())
}

async fn ensure_videos_retry_count_column(conn: &Connection) -> Result<(), libsql::Error> {
    let mut rows = conn.query("PRAGMA table_info(videos)", ()).await?;
    let mut has_col = false;
    while let Some(row) = rows.next().await? {
        let name: String = row.get(1)?;
        if name == "retry_count" {
            has_col = true;
            break;
        }
    }

    if !has_col {
        conn.execute(
            "ALTER TABLE videos ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0",
            (),
        )
        .await?;
    }

    Ok(())
}

async fn ensure_summary_quality_columns(conn: &Connection) -> Result<(), libsql::Error> {
    let mut rows = conn.query("PRAGMA table_info(summaries)", ()).await?;
    let mut has_quality_score = false;
    let mut has_auto_regen_attempts = false;
    let mut has_quality_note = false;
    while let Some(row) = rows.next().await? {
        let name: String = row.get(1)?;
        if name == "quality_score" {
            has_quality_score = true;
        } else if name == "auto_regen_attempts" {
            has_auto_regen_attempts = true;
        } else if name == "quality_note" {
            has_quality_note = true;
        }
    }

    if !has_quality_score {
        conn.execute("ALTER TABLE summaries ADD COLUMN quality_score INTEGER", ())
            .await?;
    }
    if !has_auto_regen_attempts {
        conn.execute(
            "ALTER TABLE summaries ADD COLUMN auto_regen_attempts INTEGER NOT NULL DEFAULT 0",
            (),
        )
        .await?;
    }
    if !has_quality_note {
        conn.execute("ALTER TABLE summaries ADD COLUMN quality_note TEXT", ())
            .await?;
    }

    Ok(())
}

pub async fn insert_channel(conn: &Connection, channel: &Channel) -> Result<(), libsql::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO channels (id, handle, name, thumbnail_url, added_at, earliest_sync_date, earliest_sync_date_user_set) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            channel.id.as_str(),
            channel.handle.as_deref(),
            channel.name.as_str(),
            channel.thumbnail_url.as_deref(),
            channel.added_at.to_rfc3339(),
            channel.earliest_sync_date.map(|dt| dt.to_rfc3339()),
            if channel.earliest_sync_date_user_set { 1i64 } else { 0i64 },
        ],
    )
    .await?;
    Ok(())
}

pub async fn get_channel(conn: &Connection, id: &str) -> Result<Option<Channel>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT id, handle, name, thumbnail_url, added_at, earliest_sync_date, earliest_sync_date_user_set FROM channels WHERE id = ?1",
            params![id],
        )
        .await?;

    if let Some(row) = rows.next().await? {
        Ok(Some(row_to_channel(&row)?))
    } else {
        Ok(None)
    }
}

pub async fn list_channels(conn: &Connection) -> Result<Vec<Channel>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT id, handle, name, thumbnail_url, added_at, earliest_sync_date, COALESCE(earliest_sync_date_user_set, 0) FROM channels ORDER BY added_at DESC",
            (),
        )
        .await?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await? {
        results.push(row_to_channel(&row)?);
    }
    Ok(results)
}

pub async fn get_oldest_ready_video_published_at(
    conn: &Connection,
    channel_id: &str,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT MIN(published_at) FROM videos
             WHERE channel_id = ?1 AND transcript_status = 'ready' AND summary_status = 'ready'",
            params![channel_id],
        )
        .await?;

    if let Some(row) = rows.next().await? {
        let raw: Option<String> = row.get(0)?;
        if let Some(s) = raw {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
                return Ok(Some(dt.with_timezone(&chrono::Utc)));
            }
        }
    }
    Ok(None)
}

pub async fn delete_channel(conn: &Connection, id: &str) -> Result<bool, libsql::Error> {
    conn.execute(
        "DELETE FROM summaries WHERE video_id IN (SELECT id FROM videos WHERE channel_id = ?1)",
        params![id],
    )
    .await?;
    conn.execute(
        "DELETE FROM transcripts WHERE video_id IN (SELECT id FROM videos WHERE channel_id = ?1)",
        params![id],
    )
    .await?;
    conn.execute("DELETE FROM videos WHERE channel_id = ?1", params![id])
        .await?;
    let changes = conn
        .execute("DELETE FROM channels WHERE id = ?1", params![id])
        .await?;
    Ok(changes > 0)
}

fn row_to_channel(row: &libsql::Row) -> Result<Channel, libsql::Error> {
    let added_at: String = row.get(4)?;
    let earliest_sync_date_raw: Option<String> = row.get(5)?;
    let user_set: i64 = row.get(6).unwrap_or(0);
    Ok(Channel {
        id: row.get(0)?,
        handle: row.get(1)?,
        name: row.get(2)?,
        thumbnail_url: row.get(3)?,
        added_at: chrono::DateTime::parse_from_rfc3339(&added_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| libsql::Error::ToSqlConversionFailure(Box::new(e)))?,
        earliest_sync_date: earliest_sync_date_raw.and_then(|raw| {
            chrono::DateTime::parse_from_rfc3339(&raw)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok()
        }),
        earliest_sync_date_user_set: user_set != 0,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoInsertOutcome {
    Inserted,
    Existing,
}

pub async fn insert_video(
    conn: &Connection,
    video: &Video,
) -> Result<VideoInsertOutcome, libsql::Error> {
    let already_exists = conn
        .query(
            "SELECT 1 FROM videos WHERE id = ?1 LIMIT 1",
            params![video.id.as_str()],
        )
        .await?
        .next()
        .await?
        .is_some();

    conn.execute(
        "INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, acknowledged, retry_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
             channel_id = excluded.channel_id,
             title = excluded.title,
             thumbnail_url = excluded.thumbnail_url,
             published_at = excluded.published_at,
             is_short = excluded.is_short",
        params![
            video.id.as_str(),
            video.channel_id.as_str(),
            video.title.as_str(),
            video.thumbnail_url.as_deref(),
            video.published_at.to_rfc3339(),
            video.is_short as i64,
            video.transcript_status.as_str(),
            video.summary_status.as_str(),
            video.acknowledged as i64,
            video.retry_count as i64,
        ],
    )
    .await?;

    if already_exists {
        tracing::debug!(
            channel_id = %video.channel_id,
            video_id = %video.id,
            title = %video.title,
            published_at = %video.published_at.to_rfc3339(),
            "found existing video"
        );
        Ok(VideoInsertOutcome::Existing)
    } else {
        tracing::info!(
            channel_id = %video.channel_id,
            video_id = %video.id,
            title = %video.title,
            published_at = %video.published_at.to_rfc3339(),
            "inserted new video"
        );
        Ok(VideoInsertOutcome::Inserted)
    }
}

pub async fn get_video(conn: &Connection, id: &str) -> Result<Option<Video>, libsql::Error> {
    let mut rows = conn.query("SELECT id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, acknowledged, retry_count FROM videos WHERE id = ?1", params![id]).await?;

    if let Some(row) = rows.next().await? {
        Ok(Some(row_to_video(&row)?))
    } else {
        Ok(None)
    }
}

pub async fn list_videos_by_channel(
    conn: &Connection,
    channel_id: &str,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_only: bool,
) -> Result<Vec<Video>, libsql::Error> {
    let mut sql = String::from(
        "SELECT id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, acknowledged, retry_count
         FROM videos
         WHERE channel_id = ?1",
    );
    let mut query_params = vec![
        Value::from(channel_id.to_string()),
        Value::from(limit as i64),
        Value::from(offset as i64),
    ];
    let mut next_param_index = 4;

    if let Some(short_filter) = is_short {
        sql.push_str(&format!(" AND is_short = ?{next_param_index}"));
        query_params.push(Value::from(if short_filter { 1i64 } else { 0i64 }));
        next_param_index += 1;
    }

    if let Some(ack_filter) = acknowledged {
        sql.push_str(&format!(" AND acknowledged = ?{next_param_index}"));
        query_params.push(Value::from(if ack_filter { 1i64 } else { 0i64 }));
    }

    if queue_only {
        sql.push_str(" AND (transcript_status != 'ready' OR summary_status != 'ready')");
    }

    sql.push_str(" ORDER BY published_at DESC LIMIT ?2 OFFSET ?3");

    let mut rows = conn.query(&sql, query_params).await?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await? {
        results.push(row_to_video(&row)?);
    }
    Ok(results)
}

pub async fn list_video_ids_by_channel(
    conn: &Connection,
    channel_id: &str,
) -> Result<Vec<String>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT id FROM videos WHERE channel_id = ?1 ORDER BY published_at DESC",
            params![channel_id],
        )
        .await?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await? {
        results.push(row.get(0)?);
    }
    Ok(results)
}

async fn build_channel_snapshot_data(
    conn: &Connection,
    channel: Channel,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_only: bool,
) -> Result<ChannelSnapshotData, libsql::Error> {
    let derived_earliest_ready_date =
        get_oldest_ready_video_published_at(conn, &channel.id).await?;
    let videos = list_videos_by_channel(
        conn,
        &channel.id,
        limit,
        offset,
        is_short,
        acknowledged,
        queue_only,
    )
    .await?;

    Ok(ChannelSnapshotData {
        channel,
        derived_earliest_ready_date,
        videos,
    })
}

pub async fn load_channel_snapshot_data(
    conn: &Connection,
    channel_id: &str,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_only: bool,
) -> Result<Option<ChannelSnapshotData>, libsql::Error> {
    let channel = get_channel(conn, channel_id).await?;
    match channel {
        Some(channel) => Ok(Some(
            build_channel_snapshot_data(
                conn,
                channel,
                limit,
                offset,
                is_short,
                acknowledged,
                queue_only,
            )
            .await?,
        )),
        None => Ok(None),
    }
}

pub async fn load_workspace_bootstrap_data(
    conn: &Connection,
    preferred_channel_id: Option<&str>,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_only: bool,
) -> Result<WorkspaceBootstrapData, libsql::Error> {
    let channels = list_channels(conn).await?;
    let selected_channel = preferred_channel_id
        .and_then(|channel_id| channels.iter().find(|channel| channel.id == channel_id))
        .cloned()
        .or_else(|| channels.first().cloned());
    let selected_channel_id = selected_channel.as_ref().map(|channel| channel.id.clone());
    let snapshot = match selected_channel {
        Some(channel) => Some(
            build_channel_snapshot_data(
                conn,
                channel,
                limit,
                offset,
                is_short,
                acknowledged,
                queue_only,
            )
            .await?,
        ),
        None => None,
    };

    Ok(WorkspaceBootstrapData {
        channels,
        selected_channel_id,
        snapshot,
    })
}

pub async fn list_videos_for_queue_processing(
    conn: &Connection,
    limit: usize,
    max_retries: u8,
) -> Result<Vec<Video>, libsql::Error> {
    let mut rows = conn.query(
        "SELECT id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, acknowledged, retry_count
         FROM videos
         WHERE (transcript_status IN ('pending', 'loading', 'failed')
            OR (transcript_status = 'ready' AND summary_status IN ('pending', 'loading', 'failed')))
           AND retry_count < ?2
         ORDER BY published_at DESC
         LIMIT ?1",
        params![limit as i64, max_retries as i64],
    ).await?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await? {
        results.push(row_to_video(&row)?);
    }
    Ok(results)
}

pub async fn update_video_transcript_status(
    conn: &Connection,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), libsql::Error> {
    conn.execute(
        "UPDATE videos SET transcript_status = ?1 WHERE id = ?2",
        params![status.as_str(), video_id],
    )
    .await?;
    Ok(())
}

pub async fn update_video_summary_status(
    conn: &Connection,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), libsql::Error> {
    conn.execute(
        "UPDATE videos SET summary_status = ?1 WHERE id = ?2",
        params![status.as_str(), video_id],
    )
    .await?;
    Ok(())
}

pub async fn update_video_acknowledged(
    conn: &Connection,
    video_id: &str,
    acknowledged: bool,
) -> Result<(), libsql::Error> {
    conn.execute(
        "UPDATE videos SET acknowledged = ?1 WHERE id = ?2",
        params![acknowledged as i64, video_id],
    )
    .await?;
    Ok(())
}

pub async fn increment_video_retry_count(
    conn: &Connection,
    video_id: &str,
) -> Result<(), libsql::Error> {
    conn.execute(
        "UPDATE videos SET retry_count = retry_count + 1 WHERE id = ?1",
        params![video_id],
    )
    .await?;
    Ok(())
}

pub async fn reset_video_retry_count(
    conn: &Connection,
    video_id: &str,
) -> Result<(), libsql::Error> {
    conn.execute(
        "UPDATE videos SET retry_count = 0 WHERE id = ?1",
        params![video_id],
    )
    .await?;
    Ok(())
}

fn row_to_video(row: &libsql::Row) -> Result<Video, libsql::Error> {
    let published_at: String = row.get(4)?;
    let transcript_status: String = row.get(6)?;
    let summary_status: String = row.get(7)?;
    let is_short_val: i64 = row.get(5)?;
    let acknowledged_val: i64 = row.get::<i64>(8).unwrap_or(0);
    let retry_count: i64 = row.get::<i64>(9).unwrap_or(0);
    Ok(Video {
        id: row.get(0)?,
        channel_id: row.get(1)?,
        title: row.get(2)?,
        thumbnail_url: row.get(3)?,
        published_at: chrono::DateTime::parse_from_rfc3339(&published_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| libsql::Error::ToSqlConversionFailure(Box::new(e)))?,
        is_short: is_short_val != 0,
        transcript_status: ContentStatus::from_db_value(&transcript_status),
        summary_status: ContentStatus::from_db_value(&summary_status),
        acknowledged: acknowledged_val != 0,
        retry_count: retry_count.clamp(0, 255) as u8,
    })
}

pub async fn upsert_transcript(
    conn: &Connection,
    transcript: &Transcript,
) -> Result<(), libsql::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO transcripts (video_id, raw_text, formatted_markdown) VALUES (?1, ?2, ?3)",
        params![
            transcript.video_id.as_str(),
            transcript.raw_text.as_deref(),
            transcript.formatted_markdown.as_deref()
        ],
    )
    .await?;
    Ok(())
}

pub async fn update_transcript_content(
    conn: &Connection,
    video_id: &str,
    content: &str,
) -> Result<(), libsql::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO transcripts (video_id, raw_text, formatted_markdown) VALUES (?1, ?2, ?3)",
        params![video_id, content, content],
    )
    .await?;
    Ok(())
}

pub async fn get_transcript(
    conn: &Connection,
    video_id: &str,
) -> Result<Option<Transcript>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT video_id, raw_text, formatted_markdown FROM transcripts WHERE video_id = ?1",
            params![video_id],
        )
        .await?;

    if let Some(row) = rows.next().await? {
        Ok(Some(Transcript {
            video_id: row.get(0)?,
            raw_text: row.get(1)?,
            formatted_markdown: row.get(2)?,
        }))
    } else {
        Ok(None)
    }
}

pub async fn upsert_summary(conn: &Connection, summary: &Summary) -> Result<(), libsql::Error> {
    conn.execute(
        "INSERT INTO summaries (video_id, content, model_used, quality_score, quality_note, auto_regen_attempts)
         VALUES (
             ?1,
             ?2,
             ?3,
             ?4,
             ?5,
             COALESCE((SELECT auto_regen_attempts FROM summaries WHERE video_id = ?1), 0)
         )
         ON CONFLICT(video_id) DO UPDATE SET
             content = excluded.content,
             model_used = excluded.model_used,
             quality_score = excluded.quality_score,
             quality_note = excluded.quality_note,
             auto_regen_attempts = excluded.auto_regen_attempts",
        params![
            summary.video_id.as_str(),
            summary.content.as_str(),
            summary.model_used.as_deref(),
            summary.quality_score.map(i64::from),
            summary.quality_note.as_deref(),
        ],
    )
    .await?;
    Ok(())
}

pub async fn update_summary_content(
    conn: &Connection,
    video_id: &str,
    content: &str,
    model_used: Option<&str>,
) -> Result<(), libsql::Error> {
    conn.execute(
        "INSERT INTO summaries (video_id, content, model_used, quality_score, quality_note, auto_regen_attempts)
         VALUES (?1, ?2, ?3, NULL, NULL, 0)
         ON CONFLICT(video_id) DO UPDATE SET
             content = excluded.content,
             model_used = excluded.model_used,
             quality_score = NULL,
             quality_note = NULL,
             auto_regen_attempts = 0",
        params![video_id, content, model_used],
    )
    .await?;
    Ok(())
}

pub async fn update_summary_quality(
    conn: &Connection,
    video_id: &str,
    quality_score: Option<u8>,
    quality_note: Option<&str>,
) -> Result<(), libsql::Error> {
    conn.execute(
        "UPDATE summaries SET quality_score = ?1, quality_note = ?2 WHERE video_id = ?3",
        params![quality_score.map(i64::from), quality_note, video_id],
    )
    .await?;
    Ok(())
}

pub async fn get_summary_auto_regen_attempts(
    conn: &Connection,
    video_id: &str,
) -> Result<u8, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT COALESCE(auto_regen_attempts, 0) FROM summaries WHERE video_id = ?1",
            params![video_id],
        )
        .await?;
    if let Some(row) = rows.next().await? {
        let value: i64 = row.get(0)?;
        Ok(value.clamp(0, i64::from(u8::MAX)) as u8)
    } else {
        Ok(0)
    }
}

pub async fn increment_summary_auto_regen_attempts(
    conn: &Connection,
    video_id: &str,
) -> Result<(), libsql::Error> {
    conn.execute(
        "UPDATE summaries
         SET auto_regen_attempts = COALESCE(auto_regen_attempts, 0) + 1
         WHERE video_id = ?1",
        params![video_id],
    )
    .await?;
    Ok(())
}

pub async fn delete_summary(conn: &Connection, video_id: &str) -> Result<bool, libsql::Error> {
    let changes = conn
        .execute(
            "DELETE FROM summaries WHERE video_id = ?1",
            params![video_id],
        )
        .await?;
    Ok(changes > 0)
}

pub async fn get_summary(
    conn: &Connection,
    video_id: &str,
) -> Result<Option<Summary>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT video_id, content, model_used, quality_score, quality_note
             FROM summaries
             WHERE video_id = ?1",
            params![video_id],
        )
        .await?;

    if let Some(row) = rows.next().await? {
        let quality_score: Option<i64> = row.get(3)?;
        Ok(Some(Summary {
            video_id: row.get(0)?,
            content: row.get(1)?,
            model_used: row.get(2)?,
            quality_score: quality_score.map(|score| score.clamp(0, 10) as u8),
            quality_note: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}

pub async fn list_summaries_pending_quality_eval(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<SummaryEvaluationJob>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT s.video_id, v.title, t.raw_text, t.formatted_markdown, s.content
         FROM summaries s
         JOIN videos v ON v.id = s.video_id
         LEFT JOIN transcripts t ON t.video_id = s.video_id
         WHERE v.transcript_status = 'ready'
           AND v.summary_status = 'ready'
           AND s.quality_score IS NULL
           AND s.quality_note IS NULL
           AND TRIM(s.content) <> ''
           AND TRIM(COALESCE(t.raw_text, t.formatted_markdown, '')) <> ''
         ORDER BY v.published_at DESC
         LIMIT ?1",
            params![limit as i64],
        )
        .await?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await? {
        let raw_text: Option<String> = row.get(2)?;
        let formatted_markdown: Option<String> = row.get(3)?;
        let transcript_text = raw_text
            .or(formatted_markdown)
            .unwrap_or_default()
            .trim()
            .to_string();

        results.push(SummaryEvaluationJob {
            video_id: row.get(0)?,
            video_title: row.get(1)?,
            transcript_text,
            summary_content: row.get(4)?,
        });
    }
    Ok(results)
}

pub async fn upsert_video_info(conn: &Connection, info: &VideoInfo) -> Result<(), libsql::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO video_info
         (video_id, watch_url, title, description, thumbnail_url, channel_name, channel_id, published_at, duration_iso8601, duration_seconds, view_count, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            info.video_id.as_str(),
            info.watch_url.as_str(),
            info.title.as_str(),
            info.description.as_deref(),
            info.thumbnail_url.as_deref(),
            info.channel_name.as_deref(),
            info.channel_id.as_deref(),
            info.published_at.map(|dt| dt.to_rfc3339()),
            info.duration_iso8601.as_deref(),
            info.duration_seconds.map(|v| v as i64),
            info.view_count.map(|v| v as i64),
            chrono::Utc::now().to_rfc3339(),
        ],
    )
    .await?;
    Ok(())
}

pub async fn get_video_info(
    conn: &Connection,
    video_id: &str,
) -> Result<Option<VideoInfo>, libsql::Error> {
    let mut rows = conn.query(
        "SELECT video_id, watch_url, title, description, thumbnail_url, channel_name, channel_id, published_at, duration_iso8601, duration_seconds, view_count
         FROM video_info
         WHERE video_id = ?1",
        params![video_id],
    ).await?;
    if let Some(row) = rows.next().await? {
        let published_at_raw: Option<String> = row.get(7)?;
        let duration_seconds: Option<i64> = row.get(9)?;
        let view_count: Option<i64> = row.get(10)?;
        Ok(Some(VideoInfo {
            video_id: row.get(0)?,
            watch_url: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            thumbnail_url: row.get(4)?,
            channel_name: row.get(5)?,
            channel_id: row.get(6)?,
            published_at: published_at_raw.and_then(|value| {
                chrono::DateTime::parse_from_rfc3339(&value)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            duration_iso8601: row.get(8)?,
            duration_seconds: duration_seconds.map(|value| value as u64),
            view_count: view_count.map(|value| value as u64),
        }))
    } else {
        Ok(None)
    }
}

pub async fn list_video_ids_missing_info(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<String>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT v.id
         FROM videos v
         LEFT JOIN video_info vi ON vi.video_id = v.id
         WHERE vi.video_id IS NULL
         ORDER BY v.published_at DESC
         LIMIT ?1",
            params![limit as i64],
        )
        .await?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await? {
        results.push(row.get(0)?);
    }
    Ok(results)
}

pub async fn list_video_ids_for_info_refresh(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<String>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT id
             FROM videos
             ORDER BY published_at DESC
             LIMIT ?1",
            params![limit as i64],
        )
        .await?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await? {
        results.push(row.get(0)?);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_channel_crud() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC123".to_string(),
            handle: Some("@test".to_string()),
            name: "Test Channel".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };

        insert_channel(&conn, &channel).await.unwrap();

        let fetched = get_channel(&conn, "UC123").await.unwrap().unwrap();
        assert_eq!(fetched.name, "Test Channel");

        let channels = list_channels(&conn).await.unwrap();
        assert_eq!(channels.len(), 1);

        assert!(delete_channel(&conn, "UC123").await.unwrap());
        assert!(get_channel(&conn, "UC123").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_video_with_transcript_and_summary() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC123".to_string(),
            handle: None,
            name: "Test".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let video = Video {
            id: "vid1".to_string(),
            channel_id: "UC123".to_string(),
            title: "Test Video".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Pending,
            summary_status: ContentStatus::Pending,
            acknowledged: false,
            retry_count: 0,
        };
        insert_video(&conn, &video).await.unwrap();

        let transcript = Transcript {
            video_id: "vid1".to_string(),
            raw_text: Some("Hello world".to_string()),
            formatted_markdown: Some("# Hello\n\nWorld".to_string()),
        };
        upsert_transcript(&conn, &transcript).await.unwrap();
        update_video_transcript_status(&conn, "vid1", ContentStatus::Ready)
            .await
            .unwrap();

        let fetched = get_transcript(&conn, "vid1").await.unwrap().unwrap();
        assert_eq!(fetched.raw_text, Some("Hello world".to_string()));

        let video = get_video(&conn, "vid1").await.unwrap().unwrap();
        assert_eq!(video.transcript_status, ContentStatus::Ready);
    }

    #[tokio::test]
    async fn test_update_transcript_content() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC999".to_string(),
            handle: None,
            name: "Test".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let video = Video {
            id: "vid2".to_string(),
            channel_id: "UC999".to_string(),
            title: "Test Video".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Pending,
            summary_status: ContentStatus::Pending,
            acknowledged: false,
            retry_count: 0,
        };
        insert_video(&conn, &video).await.unwrap();

        update_transcript_content(&conn, "vid2", "## Edited")
            .await
            .unwrap();
        update_video_transcript_status(&conn, "vid2", ContentStatus::Ready)
            .await
            .unwrap();

        let transcript = get_transcript(&conn, "vid2").await.unwrap().unwrap();
        assert_eq!(transcript.formatted_markdown, Some("## Edited".to_string()));
        assert_eq!(transcript.raw_text, Some("## Edited".to_string()));
    }

    #[tokio::test]
    async fn test_update_summary_content() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC777".to_string(),
            handle: None,
            name: "Test".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let video = Video {
            id: "vid3".to_string(),
            channel_id: "UC777".to_string(),
            title: "Test Video".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Pending,
            summary_status: ContentStatus::Pending,
            acknowledged: false,
            retry_count: 0,
        };
        insert_video(&conn, &video).await.unwrap();

        update_summary_content(&conn, "vid3", "Summary text", Some("manual"))
            .await
            .unwrap();
        update_video_summary_status(&conn, "vid3", ContentStatus::Ready)
            .await
            .unwrap();

        let summary = get_summary(&conn, "vid3").await.unwrap().unwrap();
        assert_eq!(summary.content, "Summary text");
        assert_eq!(summary.model_used, Some("manual".to_string()));
    }

    #[tokio::test]
    async fn test_summary_quality_fields_roundtrip() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC_EVAL".to_string(),
            handle: None,
            name: "Eval".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let video = Video {
            id: "vid_eval".to_string(),
            channel_id: "UC_EVAL".to_string(),
            title: "Eval Video".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
        };
        insert_video(&conn, &video).await.unwrap();

        conn.execute(
            "INSERT INTO summaries (video_id, content, model_used, quality_score, quality_note)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["vid_eval", "Summary body", "manual", 8i64, "Missed nuance"],
        )
        .await
        .unwrap();

        let summary = get_summary(&conn, "vid_eval").await.unwrap().unwrap();
        assert_eq!(summary.quality_score, Some(8));
        assert_eq!(summary.quality_note, Some("Missed nuance".to_string()));
    }

    #[tokio::test]
    async fn test_update_summary_content_resets_quality_fields() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC_EVAL2".to_string(),
            handle: None,
            name: "Eval 2".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let video = Video {
            id: "vid_eval2".to_string(),
            channel_id: "UC_EVAL2".to_string(),
            title: "Eval Video 2".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
        };
        insert_video(&conn, &video).await.unwrap();

        conn.execute(
            "INSERT INTO summaries (video_id, content, model_used, quality_score, quality_note)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["vid_eval2", "Before", "manual", 9i64, "Minor mismatch"],
        )
        .await
        .unwrap();

        update_summary_content(&conn, "vid_eval2", "After edit", Some("manual"))
            .await
            .unwrap();
        let summary = get_summary(&conn, "vid_eval2").await.unwrap().unwrap();
        assert_eq!(summary.content, "After edit");
        assert_eq!(summary.quality_score, None);
        assert_eq!(summary.quality_note, None);
    }

    #[tokio::test]
    async fn test_list_summaries_pending_quality_eval_and_store_result() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC_EVAL3".to_string(),
            handle: None,
            name: "Eval 3".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let video = Video {
            id: "vid_eval3".to_string(),
            channel_id: "UC_EVAL3".to_string(),
            title: "Eval Video 3".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
        };
        insert_video(&conn, &video).await.unwrap();

        upsert_transcript(
            &conn,
            &Transcript {
                video_id: "vid_eval3".to_string(),
                raw_text: Some("Transcript body".to_string()),
                formatted_markdown: None,
            },
        )
        .await
        .unwrap();
        upsert_summary(
            &conn,
            &Summary {
                video_id: "vid_eval3".to_string(),
                content: "Summary body".to_string(),
                model_used: Some("manual".to_string()),
                quality_score: None,
                quality_note: None,
            },
        )
        .await
        .unwrap();

        let pending = list_summaries_pending_quality_eval(&conn, 10)
            .await
            .unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].video_id, "vid_eval3");
        assert_eq!(pending[0].transcript_text, "Transcript body");
        assert_eq!(pending[0].summary_content, "Summary body");

        update_summary_quality(&conn, "vid_eval3", Some(7), Some("Missed one claim"))
            .await
            .unwrap();
        let updated = get_summary(&conn, "vid_eval3").await.unwrap().unwrap();
        assert_eq!(updated.quality_score, Some(7));
        assert_eq!(updated.quality_note, Some("Missed one claim".to_string()));

        let pending_after = list_summaries_pending_quality_eval(&conn, 10)
            .await
            .unwrap();
        assert!(pending_after.is_empty());
    }

    #[tokio::test]
    async fn test_summary_auto_regen_attempts_increment_and_manual_reset() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC_REGEN".to_string(),
            handle: None,
            name: "Regen".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let video = Video {
            id: "vid_regen".to_string(),
            channel_id: "UC_REGEN".to_string(),
            title: "Regen Video".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
        };
        insert_video(&conn, &video).await.unwrap();

        upsert_summary(
            &conn,
            &Summary {
                video_id: "vid_regen".to_string(),
                content: "Initial summary".to_string(),
                model_used: Some("model".to_string()),
                quality_score: None,
                quality_note: None,
            },
        )
        .await
        .unwrap();

        let attempts = get_summary_auto_regen_attempts(&conn, "vid_regen")
            .await
            .unwrap();
        assert_eq!(attempts, 0);

        increment_summary_auto_regen_attempts(&conn, "vid_regen")
            .await
            .unwrap();
        increment_summary_auto_regen_attempts(&conn, "vid_regen")
            .await
            .unwrap();
        let attempts_after_increment = get_summary_auto_regen_attempts(&conn, "vid_regen")
            .await
            .unwrap();
        assert_eq!(attempts_after_increment, 2);

        update_summary_content(&conn, "vid_regen", "Manual edit", Some("manual"))
            .await
            .unwrap();
        let attempts_after_manual_edit = get_summary_auto_regen_attempts(&conn, "vid_regen")
            .await
            .unwrap();
        assert_eq!(attempts_after_manual_edit, 0);
    }

    #[tokio::test]
    async fn test_list_videos_for_queue_processing_filters_failed_and_ready() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UCQ".to_string(),
            handle: None,
            name: "Queue".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let videos = [
            Video {
                id: "v_pending".to_string(),
                channel_id: "UCQ".to_string(),
                title: "Pending transcript".to_string(),
                thumbnail_url: None,
                published_at: Utc::now(),
                is_short: false,
                transcript_status: ContentStatus::Pending,
                summary_status: ContentStatus::Pending,
                acknowledged: false,
                retry_count: 0,
            },
            Video {
                id: "v_loading".to_string(),
                channel_id: "UCQ".to_string(),
                title: "Loading transcript".to_string(),
                thumbnail_url: None,
                published_at: Utc::now(),
                is_short: false,
                transcript_status: ContentStatus::Loading,
                summary_status: ContentStatus::Pending,
                acknowledged: false,
                retry_count: 0,
            },
            Video {
                id: "v_summary_pending".to_string(),
                channel_id: "UCQ".to_string(),
                title: "Summary pending".to_string(),
                thumbnail_url: None,
                published_at: Utc::now(),
                is_short: false,
                transcript_status: ContentStatus::Ready,
                summary_status: ContentStatus::Pending,
                acknowledged: false,
                retry_count: 0,
            },
            Video {
                id: "v_done".to_string(),
                channel_id: "UCQ".to_string(),
                title: "Done".to_string(),
                thumbnail_url: None,
                published_at: Utc::now(),
                is_short: false,
                transcript_status: ContentStatus::Ready,
                summary_status: ContentStatus::Ready,
                acknowledged: false,
                retry_count: 0,
            },
            Video {
                id: "v_failed".to_string(),
                channel_id: "UCQ".to_string(),
                title: "Failed".to_string(),
                thumbnail_url: None,
                published_at: Utc::now(),
                is_short: false,
                transcript_status: ContentStatus::Failed,
                summary_status: ContentStatus::Failed,
                acknowledged: false,
                retry_count: 0,
            },
        ];

        for video in videos {
            insert_video(&conn, &video).await.unwrap();
        }

        let queue = list_videos_for_queue_processing(&conn, 10, 3)
            .await
            .unwrap();
        let ids = queue.into_iter().map(|video| video.id).collect::<Vec<_>>();

        assert!(ids.contains(&"v_pending".to_string()));
        assert!(ids.contains(&"v_loading".to_string()));
        assert!(ids.contains(&"v_summary_pending".to_string()));
        assert!(!ids.contains(&"v_done".to_string()));
        assert!(ids.contains(&"v_failed".to_string()));
    }

    #[tokio::test]
    async fn test_insert_video_preserves_existing_content_statuses_on_refresh() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC_REFRESH".to_string(),
            handle: None,
            name: "Refresh".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let existing = Video {
            id: "vid_refresh".to_string(),
            channel_id: "UC_REFRESH".to_string(),
            title: "Old title".to_string(),
            thumbnail_url: Some("https://example.com/old.jpg".to_string()),
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: true,
            retry_count: 0,
        };
        let first_insert = insert_video(&conn, &existing).await.unwrap();
        assert_eq!(first_insert, VideoInsertOutcome::Inserted);

        let refreshed = Video {
            id: "vid_refresh".to_string(),
            channel_id: "UC_REFRESH".to_string(),
            title: "New title".to_string(),
            thumbnail_url: Some("https://example.com/new.jpg".to_string()),
            published_at: Utc::now(),
            is_short: true,
            transcript_status: ContentStatus::Pending,
            summary_status: ContentStatus::Pending,
            acknowledged: false,
            retry_count: 0,
        };
        let refresh_insert = insert_video(&conn, &refreshed).await.unwrap();
        assert_eq!(refresh_insert, VideoInsertOutcome::Existing);

        let saved = get_video(&conn, "vid_refresh").await.unwrap().unwrap();
        assert_eq!(saved.title, "New title");
        assert_eq!(
            saved.thumbnail_url,
            Some("https://example.com/new.jpg".to_string())
        );
        assert!(saved.is_short);
        assert_eq!(saved.transcript_status, ContentStatus::Ready);
        assert_eq!(saved.summary_status, ContentStatus::Ready);
        assert!(saved.acknowledged);
    }

    #[tokio::test]
    async fn test_list_videos_by_channel_can_filter_video_type() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UCF".to_string(),
            handle: None,
            name: "Filter".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        conn.execute(
            "INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, transcript_status, summary_status, is_short)
             VALUES (?1, ?2, ?3, NULL, ?4, 'pending', 'pending', 1)",
            params!["short_vid", "UCF", "Short", Utc::now().to_rfc3339()],
        )
        .await
        .unwrap();

        conn.execute(
            "INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, transcript_status, summary_status, is_short)
             VALUES (?1, ?2, ?3, NULL, ?4, 'pending', 'pending', 0)",
            params!["long_vid", "UCF", "Long", Utc::now().to_rfc3339()],
        )
        .await
        .unwrap();

        let all_videos = list_videos_by_channel(&conn, "UCF", 10, 0, None, None, false)
            .await
            .unwrap();
        assert_eq!(all_videos.len(), 2);

        let long_only = list_videos_by_channel(&conn, "UCF", 10, 0, Some(false), None, false)
            .await
            .unwrap();
        assert_eq!(long_only.len(), 1);
        assert_eq!(long_only[0].id, "long_vid");

        let short_only = list_videos_by_channel(&conn, "UCF", 10, 0, Some(true), None, false)
            .await
            .unwrap();
        assert_eq!(short_only.len(), 1);
        assert_eq!(short_only[0].id, "short_vid");
    }

    #[tokio::test]
    async fn test_list_videos_by_channel_queue_only_filter() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UCQ".to_string(),
            handle: None,
            name: "Queue".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        conn.execute(
            "INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, transcript_status, summary_status, is_short)
             VALUES (?1, ?2, ?3, NULL, ?4, 'ready', 'ready', 0)",
            params!["ready_vid", "UCQ", "Ready", Utc::now().to_rfc3339()],
        )
        .await
        .unwrap();

        conn.execute(
            "INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, transcript_status, summary_status, is_short)
             VALUES (?1, ?2, ?3, NULL, ?4, 'pending', 'pending', 0)",
            params!["queued_vid", "UCQ", "Queued", Utc::now().to_rfc3339()],
        )
        .await
        .unwrap();

        let all = list_videos_by_channel(&conn, "UCQ", 10, 0, None, None, false)
            .await
            .unwrap();
        assert_eq!(all.len(), 2);

        let queued_only = list_videos_by_channel(&conn, "UCQ", 10, 0, None, None, true)
            .await
            .unwrap();
        assert_eq!(queued_only.len(), 1);
        assert_eq!(queued_only[0].id, "queued_vid");
    }

    #[tokio::test]
    async fn test_video_info_roundtrip_and_backfill_queries() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC_INFO".to_string(),
            handle: None,
            name: "Info".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let newer = Video {
            id: "vid_info_new".to_string(),
            channel_id: "UC_INFO".to_string(),
            title: "New".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
        };
        let older = Video {
            id: "vid_info_old".to_string(),
            channel_id: "UC_INFO".to_string(),
            title: "Old".to_string(),
            thumbnail_url: None,
            published_at: Utc::now() - chrono::Duration::days(1),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
        };
        insert_video(&conn, &newer).await.unwrap();
        insert_video(&conn, &older).await.unwrap();

        let info = VideoInfo {
            video_id: "vid_info_new".to_string(),
            watch_url: "https://www.youtube.com/watch?v=vid_info_new".to_string(),
            title: "Full Title".to_string(),
            description: Some("Detailed description".to_string()),
            thumbnail_url: Some("https://img.example.com/new.jpg".to_string()),
            channel_name: Some("Info Channel".to_string()),
            channel_id: Some("UC_INFO".to_string()),
            published_at: Some(Utc::now()),
            duration_iso8601: Some("PT3M10S".to_string()),
            duration_seconds: Some(190),
            view_count: Some(1234),
        };
        upsert_video_info(&conn, &info).await.unwrap();

        let saved = get_video_info(&conn, "vid_info_new")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved.title, "Full Title");
        assert_eq!(saved.duration_seconds, Some(190));
        assert_eq!(saved.view_count, Some(1234));

        let missing = list_video_ids_missing_info(&conn, 10).await.unwrap();
        assert_eq!(missing, vec!["vid_info_old".to_string()]);

        let refresh_all = list_video_ids_for_info_refresh(&conn, 10).await.unwrap();
        assert_eq!(
            refresh_all,
            vec!["vid_info_new".to_string(), "vid_info_old".to_string()]
        );
    }

    #[tokio::test]
    async fn test_list_video_ids_by_channel_returns_all_ids_for_channel() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel_a = Channel {
            id: "UC_GAP_A".to_string(),
            handle: None,
            name: "Gap A".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        let channel_b = Channel {
            id: "UC_GAP_B".to_string(),
            handle: None,
            name: "Gap B".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel_a).await.unwrap();
        insert_channel(&conn, &channel_b).await.unwrap();

        for id in ["a_vid_1", "a_vid_2", "a_vid_3"] {
            insert_video(
                &conn,
                &Video {
                    id: id.to_string(),
                    channel_id: "UC_GAP_A".to_string(),
                    title: id.to_string(),
                    thumbnail_url: None,
                    published_at: Utc::now(),
                    is_short: false,
                    transcript_status: ContentStatus::Pending,
                    summary_status: ContentStatus::Pending,
                    acknowledged: false,
                    retry_count: 0,
                },
            )
            .await
            .unwrap();
        }

        insert_video(
            &conn,
            &Video {
                id: "b_vid_1".to_string(),
                channel_id: "UC_GAP_B".to_string(),
                title: "b_vid_1".to_string(),
                thumbnail_url: None,
                published_at: Utc::now(),
                is_short: false,
                transcript_status: ContentStatus::Pending,
                summary_status: ContentStatus::Pending,
                acknowledged: false,
                retry_count: 0,
            },
        )
        .await
        .unwrap();

        let mut ids = list_video_ids_by_channel(&conn, "UC_GAP_A").await.unwrap();
        ids.sort();
        assert_eq!(
            ids,
            vec![
                "a_vid_1".to_string(),
                "a_vid_2".to_string(),
                "a_vid_3".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn test_load_channel_snapshot_data_returns_videos_and_sync_depth() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;
        let earliest_sync_date = Utc::now() - chrono::Duration::days(14);
        let ready_published_at = Utc::now() - chrono::Duration::days(10);
        let pending_published_at = Utc::now() - chrono::Duration::days(2);

        let channel = Channel {
            id: "UC_SNAPSHOT".to_string(),
            handle: None,
            name: "Snapshot".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: Some(earliest_sync_date),
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        insert_video(
            &conn,
            &Video {
                id: "ready_vid".to_string(),
                channel_id: channel.id.clone(),
                title: "Ready".to_string(),
                thumbnail_url: None,
                published_at: ready_published_at,
                is_short: false,
                transcript_status: ContentStatus::Ready,
                summary_status: ContentStatus::Ready,
                acknowledged: false,
                retry_count: 0,
            },
        )
        .await
        .unwrap();

        insert_video(
            &conn,
            &Video {
                id: "pending_vid".to_string(),
                channel_id: channel.id.clone(),
                title: "Pending".to_string(),
                thumbnail_url: None,
                published_at: pending_published_at,
                is_short: true,
                transcript_status: ContentStatus::Pending,
                summary_status: ContentStatus::Pending,
                acknowledged: true,
                retry_count: 0,
            },
        )
        .await
        .unwrap();

        let snapshot = load_channel_snapshot_data(&conn, &channel.id, 10, 0, None, None, false)
            .await
            .unwrap()
            .expect("snapshot should exist");

        assert_eq!(snapshot.channel.id, channel.id);
        assert_eq!(snapshot.videos.len(), 2);
        assert_eq!(snapshot.videos[0].id, "pending_vid");
        assert_eq!(
            snapshot
                .derived_earliest_ready_date
                .expect("derived ready date should exist"),
            ready_published_at
        );
    }

    #[tokio::test]
    async fn test_load_workspace_bootstrap_data_prefers_selected_channel() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let first = Channel {
            id: "UC_BOOT_1".to_string(),
            handle: None,
            name: "First".to_string(),
            thumbnail_url: None,
            added_at: Utc::now() - chrono::Duration::days(1),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        let second = Channel {
            id: "UC_BOOT_2".to_string(),
            handle: None,
            name: "Second".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &first).await.unwrap();
        insert_channel(&conn, &second).await.unwrap();

        insert_video(
            &conn,
            &Video {
                id: "boot_vid".to_string(),
                channel_id: first.id.clone(),
                title: "Boot".to_string(),
                thumbnail_url: None,
                published_at: Utc::now(),
                is_short: false,
                transcript_status: ContentStatus::Ready,
                summary_status: ContentStatus::Ready,
                acknowledged: false,
                retry_count: 0,
            },
        )
        .await
        .unwrap();

        let bootstrap =
            load_workspace_bootstrap_data(&conn, Some(&first.id), 10, 0, None, None, false)
                .await
                .unwrap();

        assert_eq!(bootstrap.channels.len(), 2);
        assert_eq!(
            bootstrap.selected_channel_id.as_deref(),
            Some(first.id.as_str())
        );
        let snapshot = bootstrap.snapshot.expect("selected snapshot should exist");
        assert_eq!(snapshot.channel.id, first.id);
        assert_eq!(snapshot.videos.len(), 1);
        assert_eq!(snapshot.videos[0].id, "boot_vid");
    }
}
