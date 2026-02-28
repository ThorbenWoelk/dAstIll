use rusqlite::{Connection, Result, params};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::models::{
    Channel, ContentStatus, Summary, SummaryEvaluationJob, Transcript, Video, VideoInfo,
};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init_db(path: &Path) -> Result<DbPool> {
    let conn = Connection::open(path)?;
    run_migrations(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

pub fn init_db_memory() -> Result<DbPool> {
    let conn = Connection::open_in_memory()?;
    run_migrations(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS channels (
            id TEXT PRIMARY KEY,
            handle TEXT,
            name TEXT NOT NULL,
            thumbnail_url TEXT,
            added_at TEXT NOT NULL
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
        CREATE INDEX IF NOT EXISTS idx_video_info_fetched_at ON video_info(fetched_at DESC);
        "#,
    )?;
    ensure_videos_is_short_column(conn)?;
    ensure_videos_acknowledged_column(conn)?;
    ensure_summary_quality_columns(conn)?;
    Ok(())
}

fn ensure_videos_is_short_column(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(videos)")?;
    let mut rows = stmt.query([])?;
    let mut has_is_short = false;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "is_short" {
            has_is_short = true;
            break;
        }
    }

    if !has_is_short {
        conn.execute(
            "ALTER TABLE videos ADD COLUMN is_short INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    Ok(())
}

fn ensure_videos_acknowledged_column(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(videos)")?;
    let mut rows = stmt.query([])?;
    let mut has_col = false;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "acknowledged" {
            has_col = true;
            break;
        }
    }

    if !has_col {
        conn.execute(
            "ALTER TABLE videos ADD COLUMN acknowledged INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    Ok(())
}

fn ensure_summary_quality_columns(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(summaries)")?;
    let mut rows = stmt.query([])?;
    let mut has_quality_score = false;
    let mut has_auto_regen_attempts = false;
    let mut has_quality_note = false;
    while let Some(row) = rows.next()? {
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
        conn.execute("ALTER TABLE summaries ADD COLUMN quality_score INTEGER", [])?;
    }
    if !has_auto_regen_attempts {
        conn.execute(
            "ALTER TABLE summaries ADD COLUMN auto_regen_attempts INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !has_quality_note {
        conn.execute("ALTER TABLE summaries ADD COLUMN quality_note TEXT", [])?;
    }

    Ok(())
}

pub fn insert_channel(conn: &Connection, channel: &Channel) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO channels (id, handle, name, thumbnail_url, added_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            channel.id,
            channel.handle,
            channel.name,
            channel.thumbnail_url,
            channel.added_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub fn get_channel(conn: &Connection, id: &str) -> Result<Option<Channel>> {
    let mut stmt = conn
        .prepare("SELECT id, handle, name, thumbnail_url, added_at FROM channels WHERE id = ?1")?;
    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row_to_channel(row)?))
    } else {
        Ok(None)
    }
}

pub fn list_channels(conn: &Connection) -> Result<Vec<Channel>> {
    let mut stmt = conn.prepare(
        "SELECT id, handle, name, thumbnail_url, added_at FROM channels ORDER BY added_at DESC",
    )?;
    let rows = stmt.query_map([], row_to_channel)?;
    rows.collect()
}

pub fn delete_channel(conn: &Connection, id: &str) -> Result<bool> {
    // Delete associated data first
    conn.execute(
        "DELETE FROM summaries WHERE video_id IN (SELECT id FROM videos WHERE channel_id = ?1)",
        params![id],
    )?;
    conn.execute(
        "DELETE FROM transcripts WHERE video_id IN (SELECT id FROM videos WHERE channel_id = ?1)",
        params![id],
    )?;
    conn.execute("DELETE FROM videos WHERE channel_id = ?1", params![id])?;
    let changes = conn.execute("DELETE FROM channels WHERE id = ?1", params![id])?;
    Ok(changes > 0)
}

fn row_to_channel(row: &rusqlite::Row) -> Result<Channel> {
    let added_at: String = row.get(4)?;
    Ok(Channel {
        id: row.get(0)?,
        handle: row.get(1)?,
        name: row.get(2)?,
        thumbnail_url: row.get(3)?,
        added_at: chrono::DateTime::parse_from_rfc3339(&added_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
    })
}

pub fn insert_video(conn: &Connection, video: &Video) -> Result<()> {
    conn.execute(
        "INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, acknowledged)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
             channel_id = excluded.channel_id,
             title = excluded.title,
             thumbnail_url = excluded.thumbnail_url,
             published_at = excluded.published_at,
             is_short = excluded.is_short",
        params![
            video.id,
            video.channel_id,
            video.title,
            video.thumbnail_url,
            video.published_at.to_rfc3339(),
            video.is_short,
            video.transcript_status.as_str(),
            video.summary_status.as_str(),
            video.acknowledged,
        ],
    )?;
    Ok(())
}

pub fn get_video(conn: &Connection, id: &str) -> Result<Option<Video>> {
    let mut stmt = conn.prepare("SELECT id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, acknowledged FROM videos WHERE id = ?1")?;
    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row_to_video(row)?))
    } else {
        Ok(None)
    }
}

pub fn list_videos_by_channel(
    conn: &Connection,
    channel_id: &str,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
) -> Result<Vec<Video>> {
    let short_filter: Option<i64> = is_short.map(|value| if value { 1 } else { 0 });
    let ack_filter: Option<i64> = acknowledged.map(|value| if value { 1 } else { 0 });
    let mut stmt = conn.prepare(
        "SELECT id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, acknowledged
         FROM videos
         WHERE channel_id = ?1
           AND (?4 IS NULL OR is_short = ?4)
           AND (?5 IS NULL OR acknowledged = ?5)
         ORDER BY published_at DESC
         LIMIT ?2 OFFSET ?3"
    )?;
    let rows = stmt.query_map(
        params![
            channel_id,
            limit as i64,
            offset as i64,
            short_filter,
            ack_filter
        ],
        row_to_video,
    )?;
    rows.collect()
}

pub fn count_videos_by_channel(conn: &Connection, channel_id: &str) -> Result<usize> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM videos WHERE channel_id = ?1",
        params![channel_id],
        |row| row.get(0),
    )?;
    Ok(count.max(0) as usize)
}

pub fn list_videos_for_queue_processing(conn: &Connection, limit: usize) -> Result<Vec<Video>> {
    let mut stmt = conn.prepare(
        "SELECT id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, acknowledged
         FROM videos
         WHERE transcript_status IN ('pending', 'loading')
            OR (transcript_status = 'ready' AND summary_status IN ('pending', 'loading'))
         ORDER BY published_at DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], row_to_video)?;
    rows.collect()
}

pub fn update_video_transcript_status(
    conn: &Connection,
    video_id: &str,
    status: ContentStatus,
) -> Result<()> {
    conn.execute(
        "UPDATE videos SET transcript_status = ?1 WHERE id = ?2",
        params![status.as_str(), video_id],
    )?;
    Ok(())
}

pub fn update_video_summary_status(
    conn: &Connection,
    video_id: &str,
    status: ContentStatus,
) -> Result<()> {
    conn.execute(
        "UPDATE videos SET summary_status = ?1 WHERE id = ?2",
        params![status.as_str(), video_id],
    )?;
    Ok(())
}

pub fn update_video_acknowledged(
    conn: &Connection,
    video_id: &str,
    acknowledged: bool,
) -> Result<()> {
    conn.execute(
        "UPDATE videos SET acknowledged = ?1 WHERE id = ?2",
        params![acknowledged, video_id],
    )?;
    Ok(())
}

fn row_to_video(row: &rusqlite::Row) -> Result<Video> {
    let published_at: String = row.get(4)?;
    let transcript_status: String = row.get(6)?;
    let summary_status: String = row.get(7)?;
    Ok(Video {
        id: row.get(0)?,
        channel_id: row.get(1)?,
        title: row.get(2)?,
        thumbnail_url: row.get(3)?,
        published_at: chrono::DateTime::parse_from_rfc3339(&published_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
        is_short: row.get(5)?,
        transcript_status: ContentStatus::from_db_value(&transcript_status),
        summary_status: ContentStatus::from_db_value(&summary_status),
        acknowledged: row.get(8).unwrap_or(false),
    })
}

pub fn upsert_transcript(conn: &Connection, transcript: &Transcript) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO transcripts (video_id, raw_text, formatted_markdown) VALUES (?1, ?2, ?3)",
        params![transcript.video_id, transcript.raw_text, transcript.formatted_markdown],
    )?;
    Ok(())
}

pub fn update_transcript_content(conn: &Connection, video_id: &str, content: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO transcripts (video_id, raw_text, formatted_markdown) VALUES (?1, ?2, ?3)",
        params![video_id, content, content],
    )?;
    Ok(())
}

pub fn get_transcript(conn: &Connection, video_id: &str) -> Result<Option<Transcript>> {
    let mut stmt = conn.prepare(
        "SELECT video_id, raw_text, formatted_markdown FROM transcripts WHERE video_id = ?1",
    )?;
    let mut rows = stmt.query(params![video_id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Transcript {
            video_id: row.get(0)?,
            raw_text: row.get(1)?,
            formatted_markdown: row.get(2)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn upsert_summary(conn: &Connection, summary: &Summary) -> Result<()> {
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
            summary.video_id,
            summary.content,
            summary.model_used,
            summary.quality_score.map(i64::from),
            summary.quality_note,
        ],
    )?;
    Ok(())
}

pub fn update_summary_content(
    conn: &Connection,
    video_id: &str,
    content: &str,
    model_used: Option<&str>,
) -> Result<()> {
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
    )?;
    Ok(())
}

pub fn update_summary_quality(
    conn: &Connection,
    video_id: &str,
    quality_score: Option<u8>,
    quality_note: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE summaries SET quality_score = ?1, quality_note = ?2 WHERE video_id = ?3",
        params![quality_score.map(i64::from), quality_note, video_id],
    )?;
    Ok(())
}

pub fn get_summary_auto_regen_attempts(conn: &Connection, video_id: &str) -> Result<u8> {
    match conn.query_row(
        "SELECT COALESCE(auto_regen_attempts, 0) FROM summaries WHERE video_id = ?1",
        params![video_id],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(value) => Ok(value.clamp(0, i64::from(u8::MAX)) as u8),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
        Err(err) => Err(err),
    }
}

pub fn increment_summary_auto_regen_attempts(conn: &Connection, video_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE summaries
         SET auto_regen_attempts = COALESCE(auto_regen_attempts, 0) + 1
         WHERE video_id = ?1",
        params![video_id],
    )?;
    Ok(())
}

pub fn get_summary(conn: &Connection, video_id: &str) -> Result<Option<Summary>> {
    let mut stmt = conn.prepare(
        "SELECT video_id, content, model_used, quality_score, quality_note
         FROM summaries
         WHERE video_id = ?1",
    )?;
    let mut rows = stmt.query(params![video_id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Summary {
            video_id: row.get(0)?,
            content: row.get(1)?,
            model_used: row.get(2)?,
            quality_score: row
                .get::<_, Option<i64>>(3)?
                .map(|score| score.clamp(0, 10) as u8),
            quality_note: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_summaries_pending_quality_eval(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<SummaryEvaluationJob>> {
    let mut stmt = conn.prepare(
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
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| {
        let raw_text: Option<String> = row.get(2)?;
        let formatted_markdown: Option<String> = row.get(3)?;
        let transcript_text = raw_text
            .or(formatted_markdown)
            .unwrap_or_default()
            .trim()
            .to_string();

        Ok(SummaryEvaluationJob {
            video_id: row.get(0)?,
            video_title: row.get(1)?,
            transcript_text,
            summary_content: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn upsert_video_info(conn: &Connection, info: &VideoInfo) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO video_info
         (video_id, watch_url, title, description, thumbnail_url, channel_name, channel_id, published_at, duration_iso8601, duration_seconds, view_count, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            info.video_id,
            info.watch_url,
            info.title,
            info.description,
            info.thumbnail_url,
            info.channel_name,
            info.channel_id,
            info.published_at.map(|dt| dt.to_rfc3339()),
            info.duration_iso8601,
            info.duration_seconds.map(|v| v as i64),
            info.view_count.map(|v| v as i64),
            chrono::Utc::now().to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub fn get_video_info(conn: &Connection, video_id: &str) -> Result<Option<VideoInfo>> {
    let mut stmt = conn.prepare(
        "SELECT video_id, watch_url, title, description, thumbnail_url, channel_name, channel_id, published_at, duration_iso8601, duration_seconds, view_count
         FROM video_info
         WHERE video_id = ?1",
    )?;
    let mut rows = stmt.query(params![video_id])?;
    if let Some(row) = rows.next()? {
        let published_at_raw: Option<String> = row.get(7)?;
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
            duration_seconds: row.get::<_, Option<i64>>(9)?.map(|value| value as u64),
            view_count: row.get::<_, Option<i64>>(10)?.map(|value| value as u64),
        }))
    } else {
        Ok(None)
    }
}

pub fn list_video_ids_missing_info(conn: &Connection, limit: usize) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT v.id
         FROM videos v
         LEFT JOIN video_info vi ON vi.video_id = v.id
         WHERE vi.video_id IS NULL
         ORDER BY v.published_at DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| row.get(0))?;
    rows.collect()
}

pub fn list_video_ids_for_info_refresh(conn: &Connection, limit: usize) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT id
         FROM videos
         ORDER BY published_at DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| row.get(0))?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_channel_crud() {
        let pool = init_db_memory().unwrap();
        let conn = pool.lock().unwrap();

        let channel = Channel {
            id: "UC123".to_string(),
            handle: Some("@test".to_string()),
            name: "Test Channel".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
        };

        insert_channel(&conn, &channel).unwrap();

        let fetched = get_channel(&conn, "UC123").unwrap().unwrap();
        assert_eq!(fetched.name, "Test Channel");

        let channels = list_channels(&conn).unwrap();
        assert_eq!(channels.len(), 1);

        assert!(delete_channel(&conn, "UC123").unwrap());
        assert!(get_channel(&conn, "UC123").unwrap().is_none());
    }

    #[test]
    fn test_video_with_transcript_and_summary() {
        let pool = init_db_memory().unwrap();
        let conn = pool.lock().unwrap();

        let channel = Channel {
            id: "UC123".to_string(),
            handle: None,
            name: "Test".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
        };
        insert_channel(&conn, &channel).unwrap();

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
        };
        insert_video(&conn, &video).unwrap();

        let transcript = Transcript {
            video_id: "vid1".to_string(),
            raw_text: Some("Hello world".to_string()),
            formatted_markdown: Some("# Hello\n\nWorld".to_string()),
        };
        upsert_transcript(&conn, &transcript).unwrap();
        update_video_transcript_status(&conn, "vid1", ContentStatus::Ready).unwrap();

        let fetched = get_transcript(&conn, "vid1").unwrap().unwrap();
        assert_eq!(fetched.raw_text, Some("Hello world".to_string()));

        let video = get_video(&conn, "vid1").unwrap().unwrap();
        assert_eq!(video.transcript_status, ContentStatus::Ready);
    }

    #[test]
    fn test_update_transcript_content() {
        let pool = init_db_memory().unwrap();
        let conn = pool.lock().unwrap();

        let channel = Channel {
            id: "UC999".to_string(),
            handle: None,
            name: "Test".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
        };
        insert_channel(&conn, &channel).unwrap();

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
        };
        insert_video(&conn, &video).unwrap();

        update_transcript_content(&conn, "vid2", "## Edited").unwrap();
        update_video_transcript_status(&conn, "vid2", ContentStatus::Ready).unwrap();

        let transcript = get_transcript(&conn, "vid2").unwrap().unwrap();
        assert_eq!(transcript.formatted_markdown, Some("## Edited".to_string()));
        assert_eq!(transcript.raw_text, Some("## Edited".to_string()));
    }

    #[test]
    fn test_update_summary_content() {
        let pool = init_db_memory().unwrap();
        let conn = pool.lock().unwrap();

        let channel = Channel {
            id: "UC777".to_string(),
            handle: None,
            name: "Test".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
        };
        insert_channel(&conn, &channel).unwrap();

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
        };
        insert_video(&conn, &video).unwrap();

        update_summary_content(&conn, "vid3", "Summary text", Some("manual")).unwrap();
        update_video_summary_status(&conn, "vid3", ContentStatus::Ready).unwrap();

        let summary = get_summary(&conn, "vid3").unwrap().unwrap();
        assert_eq!(summary.content, "Summary text");
        assert_eq!(summary.model_used, Some("manual".to_string()));
    }

    #[test]
    fn test_summary_quality_fields_roundtrip() {
        let pool = init_db_memory().unwrap();
        let conn = pool.lock().unwrap();

        let channel = Channel {
            id: "UC_EVAL".to_string(),
            handle: None,
            name: "Eval".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
        };
        insert_channel(&conn, &channel).unwrap();

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
        };
        insert_video(&conn, &video).unwrap();

        conn.execute(
            "INSERT INTO summaries (video_id, content, model_used, quality_score, quality_note)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["vid_eval", "Summary body", "manual", 8i64, "Missed nuance"],
        )
        .unwrap();

        let summary = get_summary(&conn, "vid_eval").unwrap().unwrap();
        assert_eq!(summary.quality_score, Some(8));
        assert_eq!(summary.quality_note, Some("Missed nuance".to_string()));
    }

    #[test]
    fn test_update_summary_content_resets_quality_fields() {
        let pool = init_db_memory().unwrap();
        let conn = pool.lock().unwrap();

        let channel = Channel {
            id: "UC_EVAL2".to_string(),
            handle: None,
            name: "Eval 2".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
        };
        insert_channel(&conn, &channel).unwrap();

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
        };
        insert_video(&conn, &video).unwrap();

        conn.execute(
            "INSERT INTO summaries (video_id, content, model_used, quality_score, quality_note)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["vid_eval2", "Before", "manual", 9i64, "Minor mismatch"],
        )
        .unwrap();

        update_summary_content(&conn, "vid_eval2", "After edit", Some("manual")).unwrap();
        let summary = get_summary(&conn, "vid_eval2").unwrap().unwrap();
        assert_eq!(summary.content, "After edit");
        assert_eq!(summary.quality_score, None);
        assert_eq!(summary.quality_note, None);
    }

    #[test]
    fn test_list_summaries_pending_quality_eval_and_store_result() {
        let pool = init_db_memory().unwrap();
        let conn = pool.lock().unwrap();

        let channel = Channel {
            id: "UC_EVAL3".to_string(),
            handle: None,
            name: "Eval 3".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
        };
        insert_channel(&conn, &channel).unwrap();

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
        };
        insert_video(&conn, &video).unwrap();

        upsert_transcript(
            &conn,
            &Transcript {
                video_id: "vid_eval3".to_string(),
                raw_text: Some("Transcript body".to_string()),
                formatted_markdown: None,
            },
        )
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
        .unwrap();

        let pending = list_summaries_pending_quality_eval(&conn, 10).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].video_id, "vid_eval3");
        assert_eq!(pending[0].transcript_text, "Transcript body");
        assert_eq!(pending[0].summary_content, "Summary body");

        update_summary_quality(&conn, "vid_eval3", Some(7), Some("Missed one claim")).unwrap();
        let updated = get_summary(&conn, "vid_eval3").unwrap().unwrap();
        assert_eq!(updated.quality_score, Some(7));
        assert_eq!(updated.quality_note, Some("Missed one claim".to_string()));

        let pending_after = list_summaries_pending_quality_eval(&conn, 10).unwrap();
        assert!(pending_after.is_empty());
    }

    #[test]
    fn test_summary_auto_regen_attempts_increment_and_manual_reset() {
        let pool = init_db_memory().unwrap();
        let conn = pool.lock().unwrap();

        let channel = Channel {
            id: "UC_REGEN".to_string(),
            handle: None,
            name: "Regen".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
        };
        insert_channel(&conn, &channel).unwrap();

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
        };
        insert_video(&conn, &video).unwrap();

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
        .unwrap();

        let attempts = get_summary_auto_regen_attempts(&conn, "vid_regen").unwrap();
        assert_eq!(attempts, 0);

        increment_summary_auto_regen_attempts(&conn, "vid_regen").unwrap();
        increment_summary_auto_regen_attempts(&conn, "vid_regen").unwrap();
        let attempts_after_increment = get_summary_auto_regen_attempts(&conn, "vid_regen").unwrap();
        assert_eq!(attempts_after_increment, 2);

        update_summary_content(&conn, "vid_regen", "Manual edit", Some("manual")).unwrap();
        let attempts_after_manual_edit =
            get_summary_auto_regen_attempts(&conn, "vid_regen").unwrap();
        assert_eq!(attempts_after_manual_edit, 0);
    }

    #[test]
    fn test_list_videos_for_queue_processing_filters_failed_and_ready() {
        let pool = init_db_memory().unwrap();
        let conn = pool.lock().unwrap();

        let channel = Channel {
            id: "UCQ".to_string(),
            handle: None,
            name: "Queue".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
        };
        insert_channel(&conn, &channel).unwrap();

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
            },
        ];

        for video in videos {
            insert_video(&conn, &video).unwrap();
        }

        let queue = list_videos_for_queue_processing(&conn, 10).unwrap();
        let ids = queue.into_iter().map(|video| video.id).collect::<Vec<_>>();

        assert!(ids.contains(&"v_pending".to_string()));
        assert!(ids.contains(&"v_loading".to_string()));
        assert!(ids.contains(&"v_summary_pending".to_string()));
        assert!(!ids.contains(&"v_done".to_string()));
        assert!(!ids.contains(&"v_failed".to_string()));
    }

    #[test]
    fn test_insert_video_preserves_existing_content_statuses_on_refresh() {
        let pool = init_db_memory().unwrap();
        let conn = pool.lock().unwrap();

        let channel = Channel {
            id: "UC_REFRESH".to_string(),
            handle: None,
            name: "Refresh".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
        };
        insert_channel(&conn, &channel).unwrap();

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
        };
        insert_video(&conn, &existing).unwrap();

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
        };
        insert_video(&conn, &refreshed).unwrap();

        let saved = get_video(&conn, "vid_refresh").unwrap().unwrap();
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

    #[test]
    fn test_list_videos_by_channel_can_filter_video_type() {
        let pool = init_db_memory().unwrap();
        let conn = pool.lock().unwrap();

        let channel = Channel {
            id: "UCF".to_string(),
            handle: None,
            name: "Filter".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
        };
        insert_channel(&conn, &channel).unwrap();

        conn.execute(
            "INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, transcript_status, summary_status, is_short)
             VALUES (?1, ?2, ?3, NULL, ?4, 'pending', 'pending', 1)",
            params!["short_vid", "UCF", "Short", Utc::now().to_rfc3339()],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, transcript_status, summary_status, is_short)
             VALUES (?1, ?2, ?3, NULL, ?4, 'pending', 'pending', 0)",
            params!["long_vid", "UCF", "Long", Utc::now().to_rfc3339()],
        )
        .unwrap();

        let all_videos = list_videos_by_channel(&conn, "UCF", 10, 0, None, None).unwrap();
        assert_eq!(all_videos.len(), 2);

        let long_only = list_videos_by_channel(&conn, "UCF", 10, 0, Some(false), None).unwrap();
        assert_eq!(long_only.len(), 1);
        assert_eq!(long_only[0].id, "long_vid");

        let short_only = list_videos_by_channel(&conn, "UCF", 10, 0, Some(true), None).unwrap();
        assert_eq!(short_only.len(), 1);
        assert_eq!(short_only[0].id, "short_vid");
    }

    #[test]
    fn test_video_info_roundtrip_and_backfill_queries() {
        let pool = init_db_memory().unwrap();
        let conn = pool.lock().unwrap();

        let channel = Channel {
            id: "UC_INFO".to_string(),
            handle: None,
            name: "Info".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
        };
        insert_channel(&conn, &channel).unwrap();

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
        };
        insert_video(&conn, &newer).unwrap();
        insert_video(&conn, &older).unwrap();

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
        upsert_video_info(&conn, &info).unwrap();

        let saved = get_video_info(&conn, "vid_info_new").unwrap().unwrap();
        assert_eq!(saved.title, "Full Title");
        assert_eq!(saved.duration_seconds, Some(190));
        assert_eq!(saved.view_count, Some(1234));

        let missing = list_video_ids_missing_info(&conn, 10).unwrap();
        assert_eq!(missing, vec!["vid_info_old".to_string()]);

        let refresh_all = list_video_ids_for_info_refresh(&conn, 10).unwrap();
        assert_eq!(
            refresh_all,
            vec!["vid_info_new".to_string(), "vid_info_old".to_string()]
        );
    }
}
