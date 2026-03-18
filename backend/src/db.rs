use std::sync::Arc;

use libsql::{Connection, Database, Value, params};

use crate::models::{
    Channel, ContentStatus, Highlight, HighlightChannelGroup, HighlightSource, HighlightVideoGroup,
    Summary, SummaryEvaluationJob, Transcript, TranscriptRenderMode, Video, VideoInfo,
};
use crate::services::search::{SearchCandidate, SearchIndexChunk, SearchSourceKind};

const SEARCH_SOURCES_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS search_sources (
    id INTEGER PRIMARY KEY,
    video_id TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    source_generation INTEGER NOT NULL DEFAULT 0,
    embedding_model TEXT,
    index_status TEXT NOT NULL DEFAULT 'pending',
    last_indexed_at TEXT,
    last_error TEXT,
    UNIQUE(video_id, source_kind),
    FOREIGN KEY(video_id) REFERENCES videos(id)
)
"#;

const SEARCH_CHUNKS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS search_chunks (
    id INTEGER PRIMARY KEY,
    search_source_id INTEGER NOT NULL,
    source_generation INTEGER NOT NULL,
    chunk_index INTEGER NOT NULL,
    section_title TEXT,
    chunk_text TEXT NOT NULL,
    token_count INTEGER NOT NULL,
    embedding F32_BLOB(512),
    UNIQUE(search_source_id, source_generation, chunk_index),
    FOREIGN KEY(search_source_id) REFERENCES search_sources(id)
)
"#;

const SEARCH_CHUNKS_FTS_SQL: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS search_chunks_fts USING fts5(
    section_title,
    chunk_text,
    content = 'search_chunks',
    content_rowid = 'id',
    tokenize = 'unicode61'
)
"#;

const SEARCH_CHUNKS_FTS_INSERT_TRIGGER_SQL: &str = r#"
CREATE TRIGGER IF NOT EXISTS search_chunks_ai AFTER INSERT ON search_chunks BEGIN
    INSERT INTO search_chunks_fts(rowid, section_title, chunk_text)
    VALUES (new.id, new.section_title, new.chunk_text);
END;
"#;

const SEARCH_CHUNKS_FTS_DELETE_TRIGGER_SQL: &str = r#"
CREATE TRIGGER IF NOT EXISTS search_chunks_ad AFTER DELETE ON search_chunks BEGIN
    INSERT INTO search_chunks_fts(search_chunks_fts, rowid, section_title, chunk_text)
    VALUES ('delete', old.id, old.section_title, old.chunk_text);
END;
"#;

const SEARCH_CHUNKS_FTS_UPDATE_TRIGGER_SQL: &str = r#"
CREATE TRIGGER IF NOT EXISTS search_chunks_au AFTER UPDATE ON search_chunks BEGIN
    INSERT INTO search_chunks_fts(search_chunks_fts, rowid, section_title, chunk_text)
    VALUES ('delete', old.id, old.section_title, old.chunk_text);
    INSERT INTO search_chunks_fts(rowid, section_title, chunk_text)
    VALUES (new.id, new.section_title, new.chunk_text);
END;
"#;

#[derive(Clone)]
pub struct DbPool {
    database: Arc<Database>,
    #[cfg(test)]
    _temp_path: Option<Arc<tempfile::TempPath>>,
}

impl DbPool {
    pub fn connect(&self) -> Connection {
        self.database
            .connect()
            .expect("failed to open libsql connection from pool")
    }

    #[cfg(test)]
    pub async fn lock(&self) -> Connection {
        self.connect()
    }
}

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

#[derive(Debug, Clone)]
pub struct SearchSourceState {
    pub id: i64,
    pub source_generation: i64,
    pub video_id: String,
    pub source_kind: SearchSourceKind,
    pub content_hash: String,
    pub embedding_model: Option<String>,
    pub index_status: String,
    pub last_indexed_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchMaterial {
    pub video_id: String,
    pub channel_name: String,
    pub video_title: String,
    pub source_kind: SearchSourceKind,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct SearchProgressMaterial {
    pub video_id: String,
    pub source_kind: SearchSourceKind,
    pub content: String,
    pub index_status: Option<String>,
    pub embedding_model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchSourceCounts {
    pub pending: usize,
    pub indexing: usize,
    pub ready: usize,
    pub failed: usize,
    pub total_sources: usize,
}

async fn load_search_materials_from_rows(
    mut rows: libsql::Rows,
    source_kind: SearchSourceKind,
) -> Result<Vec<SearchMaterial>, libsql::Error> {
    let mut materials = Vec::new();
    while let Some(row) = rows.next().await? {
        materials.push(SearchMaterial {
            video_id: row.get(0)?,
            channel_name: row.get(1)?,
            video_title: row.get(2)?,
            source_kind,
            content: row.get(3)?,
        });
    }
    Ok(materials)
}

async fn load_search_progress_materials_from_rows(
    mut rows: libsql::Rows,
    source_kind: SearchSourceKind,
) -> Result<Vec<SearchProgressMaterial>, libsql::Error> {
    let mut materials = Vec::new();
    while let Some(row) = rows.next().await? {
        materials.push(SearchProgressMaterial {
            video_id: row.get(0)?,
            source_kind,
            content: row.get(1)?,
            index_status: row.get(2)?,
            embedding_model: row.get(3)?,
        });
    }
    Ok(materials)
}

pub async fn init_db(db: libsql::Database) -> Result<DbPool, libsql::Error> {
    let conn = db.connect()?;
    run_migrations(&conn).await?;
    Ok(DbPool {
        database: Arc::new(db),
        #[cfg(test)]
        _temp_path: None,
    })
}

#[cfg(test)]
pub async fn init_db_memory() -> Result<DbPool, libsql::Error> {
    let temp_path = tempfile::NamedTempFile::new()
        .map_err(|err| libsql::Error::ToSqlConversionFailure(Box::new(err)))?
        .into_temp_path();
    let db = libsql::Builder::new_local(temp_path.as_ref() as &std::path::Path)
        .build()
        .await?;
    let conn = db.connect()?;
    run_migrations(&conn).await?;
    Ok(DbPool {
        database: Arc::new(db),
        _temp_path: Some(Arc::new(temp_path)),
    })
}

async fn run_migrations(conn: &Connection) -> Result<(), libsql::Error> {
    ensure_table(
        conn,
        "channels",
        r#"
        CREATE TABLE IF NOT EXISTS channels (
            id TEXT PRIMARY KEY,
            handle TEXT,
            name TEXT NOT NULL,
            thumbnail_url TEXT,
            added_at TEXT NOT NULL,
            earliest_sync_date TEXT
        )
        "#,
    )
    .await?;
    ensure_table(
        conn,
        "videos",
        r#"
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
        )
        "#,
    )
    .await?;
    ensure_table(
        conn,
        "transcripts",
        r#"
        CREATE TABLE IF NOT EXISTS transcripts (
            video_id TEXT PRIMARY KEY,
            raw_text TEXT,
            formatted_markdown TEXT,
            render_mode TEXT NOT NULL DEFAULT 'plain_text',
            FOREIGN KEY(video_id) REFERENCES videos(id)
        )
        "#,
    )
    .await?;
    ensure_table(
        conn,
        "summaries",
        r#"
        CREATE TABLE IF NOT EXISTS summaries (
            video_id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            model_used TEXT,
            quality_score INTEGER,
            auto_regen_attempts INTEGER NOT NULL DEFAULT 0,
            quality_note TEXT,
            quality_model_used TEXT,
            FOREIGN KEY(video_id) REFERENCES videos(id)
        )
        "#,
    )
    .await?;
    ensure_table(
        conn,
        "video_info",
        r#"
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
        )
        "#,
    )
    .await?;
    ensure_table(
        conn,
        "highlights",
        r#"
        CREATE TABLE IF NOT EXISTS highlights (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            video_id TEXT NOT NULL,
            source TEXT NOT NULL,
            text TEXT NOT NULL,
            normalized_text TEXT NOT NULL,
            prefix_context TEXT NOT NULL DEFAULT '',
            suffix_context TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(video_id) REFERENCES videos(id)
        )
        "#,
    )
    .await?;
    ensure_index(
        conn,
        "idx_videos_channel",
        "CREATE INDEX IF NOT EXISTS idx_videos_channel ON videos(channel_id)",
    )
    .await?;
    ensure_index(
        conn,
        "idx_videos_published",
        "CREATE INDEX IF NOT EXISTS idx_videos_published ON videos(published_at DESC)",
    )
    .await?;
    ensure_index(
        conn,
        "idx_videos_channel_published",
        "CREATE INDEX IF NOT EXISTS idx_videos_channel_published ON videos(channel_id, published_at DESC)",
    )
    .await?;
    ensure_index(
        conn,
        "idx_videos_channel_short_published",
        "CREATE INDEX IF NOT EXISTS idx_videos_channel_short_published ON videos(channel_id, is_short, published_at DESC)",
    )
    .await?;
    ensure_index(
        conn,
        "idx_videos_channel_ack_published",
        "CREATE INDEX IF NOT EXISTS idx_videos_channel_ack_published ON videos(channel_id, acknowledged, published_at DESC)",
    )
    .await?;
    ensure_index(
        conn,
        "idx_video_info_fetched_at",
        "CREATE INDEX IF NOT EXISTS idx_video_info_fetched_at ON video_info(fetched_at DESC)",
    )
    .await?;
    ensure_index(
        conn,
        "idx_highlights_video_created",
        "CREATE INDEX IF NOT EXISTS idx_highlights_video_created ON highlights(video_id, created_at DESC, id DESC)",
    )
    .await?;
    ensure_index(
        conn,
        "idx_highlights_created",
        "CREATE INDEX IF NOT EXISTS idx_highlights_created ON highlights(created_at DESC, id DESC)",
    )
    .await?;
    ensure_index(
        conn,
        "idx_highlights_video_match",
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_highlights_video_match ON highlights(video_id, source, normalized_text, prefix_context, suffix_context)",
    )
    .await?;
    // Note: vector index (idx_search_chunks_embedding) is NOT created during migration.
    // It causes 30s+ per INSERT on remote Turso. Create it via ensure_vector_index()
    // after bulk indexing is complete.
    ensure_column(
        conn,
        "videos",
        "is_short",
        "is_short INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_column(
        conn,
        "videos",
        "acknowledged",
        "acknowledged INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_column(
        conn,
        "videos",
        "retry_count",
        "retry_count INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_column(conn, "summaries", "quality_score", "quality_score INTEGER").await?;
    ensure_column(
        conn,
        "summaries",
        "auto_regen_attempts",
        "auto_regen_attempts INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_column(conn, "summaries", "quality_note", "quality_note TEXT").await?;
    ensure_column(
        conn,
        "summaries",
        "quality_model_used",
        "quality_model_used TEXT",
    )
    .await?;
    ensure_column(
        conn,
        "channels",
        "earliest_sync_date",
        "earliest_sync_date TEXT",
    )
    .await?;
    ensure_column(
        conn,
        "channels",
        "earliest_sync_date_user_set",
        "earliest_sync_date_user_set INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_column(
        conn,
        "transcripts",
        "render_mode",
        "render_mode TEXT NOT NULL DEFAULT 'plain_text'",
    )
    .await?;
    ensure_column(
        conn,
        "highlights",
        "source",
        "source TEXT NOT NULL DEFAULT 'transcript'",
    )
    .await?;
    ensure_column(conn, "highlights", "text", "text TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(
        conn,
        "highlights",
        "normalized_text",
        "normalized_text TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_column(
        conn,
        "highlights",
        "prefix_context",
        "prefix_context TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_column(
        conn,
        "highlights",
        "suffix_context",
        "suffix_context TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_column(
        conn,
        "highlights",
        "created_at",
        "created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP",
    )
    .await?;
    ensure_search_projection_schema(conn).await?;
    ensure_index(
        conn,
        "idx_videos_channel_ack_short_published",
        "CREATE INDEX IF NOT EXISTS idx_videos_channel_ack_short_published ON videos(channel_id, acknowledged, is_short, published_at DESC)",
    )
    .await?;
    ensure_index(
        conn,
        "idx_videos_channel_transcript_queue_published",
        "CREATE INDEX IF NOT EXISTS idx_videos_channel_transcript_queue_published ON videos(channel_id, published_at DESC) WHERE transcript_status != 'ready'",
    )
    .await?;
    ensure_index(
        conn,
        "idx_videos_channel_summary_queue_published",
        "CREATE INDEX IF NOT EXISTS idx_videos_channel_summary_queue_published ON videos(channel_id, published_at DESC) WHERE transcript_status = 'ready' AND summary_status != 'ready'",
    )
    .await?;
    ensure_index(
        conn,
        "idx_videos_channel_ready_published",
        "CREATE INDEX IF NOT EXISTS idx_videos_channel_ready_published ON videos(channel_id, published_at) WHERE transcript_status = 'ready' AND summary_status = 'ready'",
    )
    .await?;
    Ok(())
}

async fn schema_object_exists(
    conn: &Connection,
    object_type: &str,
    object_name: &str,
) -> Result<bool, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT 1 FROM sqlite_master WHERE type = ?1 AND name = ?2 LIMIT 1",
            params![object_type, object_name],
        )
        .await?;
    Ok(rows.next().await?.is_some())
}

async fn ensure_table(
    conn: &Connection,
    table_name: &str,
    create_sql: &str,
) -> Result<(), libsql::Error> {
    if schema_object_exists(conn, "table", table_name).await? {
        return Ok(());
    }
    conn.execute(create_sql, ()).await?;
    Ok(())
}

async fn ensure_index(
    conn: &Connection,
    index_name: &str,
    create_sql: &str,
) -> Result<(), libsql::Error> {
    if schema_object_exists(conn, "index", index_name).await? {
        return Ok(());
    }
    conn.execute(create_sql, ()).await?;
    Ok(())
}

async fn ensure_trigger(
    conn: &Connection,
    trigger_name: &str,
    create_sql: &str,
) -> Result<(), libsql::Error> {
    if schema_object_exists(conn, "trigger", trigger_name).await? {
        return Ok(());
    }
    conn.execute(create_sql, ()).await?;
    Ok(())
}

async fn ensure_search_projection_schema(conn: &Connection) -> Result<(), libsql::Error> {
    let sources_sql = get_table_sql(conn, "search_sources").await?;
    let chunks_sql = get_table_sql(conn, "search_chunks").await?;
    let fts_sql = get_table_sql(conn, "search_chunks_fts").await?;
    let needs_reset = !search_sources_schema_is_current(sources_sql.as_deref())
        || !search_chunks_schema_is_current(chunks_sql.as_deref())
        || !search_chunks_fts_schema_is_current(fts_sql.as_deref());

    if needs_reset {
        conn.execute("DROP TRIGGER IF EXISTS search_chunks_ai", ())
            .await?;
        conn.execute("DROP TRIGGER IF EXISTS search_chunks_ad", ())
            .await?;
        conn.execute("DROP TRIGGER IF EXISTS search_chunks_au", ())
            .await?;
        conn.execute("DROP TABLE IF EXISTS search_chunks_fts", ())
            .await?;
        conn.execute("DROP TABLE IF EXISTS search_chunks", ())
            .await?;
        conn.execute("DROP TABLE IF EXISTS search_sources", ())
            .await?;
    }

    ensure_table(conn, "search_sources", SEARCH_SOURCES_SQL).await?;
    ensure_table(conn, "search_chunks", SEARCH_CHUNKS_SQL).await?;
    ensure_table(conn, "search_chunks_fts", SEARCH_CHUNKS_FTS_SQL).await?;
    ensure_index(
        conn,
        "idx_search_sources_status",
        "CREATE INDEX IF NOT EXISTS idx_search_sources_status
         ON search_sources(index_status, source_kind, id)",
    )
    .await?;
    ensure_index(
        conn,
        "idx_search_chunks_source_generation",
        "CREATE INDEX IF NOT EXISTS idx_search_chunks_source_generation
         ON search_chunks(search_source_id, source_generation, chunk_index)",
    )
    .await?;
    ensure_trigger(
        conn,
        "search_chunks_ai",
        SEARCH_CHUNKS_FTS_INSERT_TRIGGER_SQL,
    )
    .await?;
    ensure_trigger(
        conn,
        "search_chunks_ad",
        SEARCH_CHUNKS_FTS_DELETE_TRIGGER_SQL,
    )
    .await?;
    ensure_trigger(
        conn,
        "search_chunks_au",
        SEARCH_CHUNKS_FTS_UPDATE_TRIGGER_SQL,
    )
    .await?;

    Ok(())
}

fn search_sources_schema_is_current(sql: Option<&str>) -> bool {
    let Some(sql) = sql else {
        return false;
    };
    let normalized = sql.split_whitespace().collect::<String>();
    normalized.contains("idINTEGERPRIMARYKEY")
        && normalized.contains("video_idTEXTNOTNULL")
        && normalized.contains("source_kindTEXTNOTNULL")
        && normalized.contains("content_hashTEXTNOTNULL")
        && normalized.contains("source_generationINTEGERNOTNULLDEFAULT0")
        && normalized.contains("UNIQUE(video_id,source_kind)")
}

fn search_chunks_schema_is_current(sql: Option<&str>) -> bool {
    let Some(sql) = sql else {
        return false;
    };
    let normalized = sql.split_whitespace().collect::<String>();
    normalized.contains("idINTEGERPRIMARYKEY")
        && normalized.contains("search_source_idINTEGERNOTNULL")
        && normalized.contains("source_generationINTEGERNOTNULL")
        && normalized.contains("chunk_textTEXTNOTNULL")
        && normalized.contains("token_countINTEGERNOTNULL")
        && normalized.contains("UNIQUE(search_source_id,source_generation,chunk_index)")
        && !normalized.contains("video_idTEXTNOTNULL")
        && !normalized.contains("source_kindTEXTNOTNULL")
}

fn search_chunks_fts_schema_is_current(sql: Option<&str>) -> bool {
    let Some(sql) = sql else {
        return false;
    };
    let normalized = sql.split_whitespace().collect::<String>();
    normalized.contains("section_title")
        && normalized.contains("chunk_text")
        && normalized.contains("content='search_chunks'")
        && normalized.contains("content_rowid='id'")
        && !normalized.contains("video_idUNINDEXED")
        && !normalized.contains("chunk_idUNINDEXED")
}

async fn get_table_sql(
    conn: &Connection,
    table_name: &str,
) -> Result<Option<String>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
            params![table_name],
        )
        .await?;
    if let Some(row) = rows.next().await? {
        return row.get(0);
    }
    Ok(None)
}

async fn ensure_column(
    conn: &Connection,
    table: &str,
    column: &str,
    col_definition: &str,
) -> Result<(), libsql::Error> {
    let mut rows = conn
        .query(&format!("PRAGMA table_info({table})"), ())
        .await?;
    while let Some(row) = rows.next().await? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(());
        }
    }
    conn.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {col_definition}"),
        (),
    )
    .await?;
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
        "DELETE FROM search_chunks
         WHERE search_source_id IN (
            SELECT id FROM search_sources WHERE video_id IN (
                SELECT id FROM videos WHERE channel_id = ?1
            )
         )",
        params![id],
    )
    .await?;
    conn.execute(
        "DELETE FROM search_sources WHERE video_id IN (SELECT id FROM videos WHERE channel_id = ?1)",
        params![id],
    )
    .await?;
    conn.execute(
        "DELETE FROM highlights WHERE video_id IN (SELECT id FROM videos WHERE channel_id = ?1)",
        params![id],
    )
    .await?;
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

fn parse_rfc3339_to_utc(value: &str) -> Result<chrono::DateTime<chrono::Utc>, libsql::Error> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| libsql::Error::ToSqlConversionFailure(Box::new(e)))
}

fn parse_sqlite_datetime_to_utc(
    value: &str,
) -> Result<chrono::DateTime<chrono::Utc>, libsql::Error> {
    parse_rfc3339_to_utc(value).or_else(|_| {
        chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
            .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc))
            .map_err(|e| libsql::Error::ToSqlConversionFailure(Box::new(e)))
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

pub async fn bulk_insert_videos(
    conn: &Connection,
    videos: Vec<Video>,
) -> Result<usize, libsql::Error> {
    let mut inserted = 0;
    for video in &videos {
        if matches!(
            insert_video(conn, video).await?,
            VideoInsertOutcome::Inserted
        ) {
            inserted += 1;
        }
    }
    Ok(inserted)
}

pub async fn get_video(conn: &Connection, id: &str) -> Result<Option<Video>, libsql::Error> {
    let mut rows = conn.query("SELECT v.id, v.channel_id, v.title, v.thumbnail_url, v.published_at, v.is_short, v.transcript_status, v.summary_status, v.acknowledged, v.retry_count, s.quality_score FROM videos v LEFT JOIN summaries s ON s.video_id = v.id WHERE v.id = ?1", params![id]).await?;

    if let Some(row) = rows.next().await? {
        Ok(Some(row_to_video(&row)?))
    } else {
        Ok(None)
    }
}

/// Queue tab filter for splitting the queue view into separate concerns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueFilter {
    /// Legacy: transcript OR summary not ready.
    AnyIncomplete,
    /// Transcript not ready (pending, loading, failed).
    TranscriptsOnly,
    /// Transcript ready, but summary not ready.
    SummariesOnly,
    /// Both transcript and summary ready, but evaluation pending (quality_score IS NULL).
    EvaluationsOnly,
}

pub async fn list_videos_by_channel(
    conn: &Connection,
    channel_id: &str,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
) -> Result<Vec<Video>, libsql::Error> {
    let mut sql = String::from(
        "SELECT v.id, v.channel_id, v.title, v.thumbnail_url, v.published_at, v.is_short, v.transcript_status, v.summary_status, v.acknowledged, v.retry_count, s.quality_score
         FROM videos v
         LEFT JOIN summaries s ON s.video_id = v.id
         WHERE v.channel_id = ?1",
    );
    let mut query_params = vec![
        Value::from(channel_id.to_string()),
        Value::from(limit as i64),
        Value::from(offset as i64),
    ];
    let mut next_param_index = 4;

    if let Some(short_filter) = is_short {
        sql.push_str(&format!(" AND v.is_short = ?{next_param_index}"));
        query_params.push(Value::from(if short_filter { 1i64 } else { 0i64 }));
        next_param_index += 1;
    }

    if let Some(ack_filter) = acknowledged {
        sql.push_str(&format!(" AND v.acknowledged = ?{next_param_index}"));
        query_params.push(Value::from(if ack_filter { 1i64 } else { 0i64 }));
    }

    match queue_filter {
        Some(QueueFilter::AnyIncomplete) => {
            sql.push_str(" AND (v.transcript_status != 'ready' OR v.summary_status != 'ready')");
        }
        Some(QueueFilter::TranscriptsOnly) => {
            sql.push_str(" AND v.transcript_status != 'ready'");
        }
        Some(QueueFilter::SummariesOnly) => {
            sql.push_str(" AND v.transcript_status = 'ready' AND v.summary_status != 'ready'");
        }
        Some(QueueFilter::EvaluationsOnly) => {
            sql.push_str(
                " AND v.transcript_status = 'ready' AND v.summary_status = 'ready' AND s.quality_score IS NULL",
            );
        }
        None => {}
    }

    sql.push_str(" ORDER BY v.published_at DESC LIMIT ?2 OFFSET ?3");

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
    queue_filter: Option<QueueFilter>,
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
        queue_filter,
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
    queue_filter: Option<QueueFilter>,
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
                queue_filter,
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
    queue_filter: Option<QueueFilter>,
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
                queue_filter,
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
        "SELECT v.id, v.channel_id, v.title, v.thumbnail_url, v.published_at, v.is_short, v.transcript_status, v.summary_status, v.acknowledged, v.retry_count, s.quality_score
         FROM videos v
         LEFT JOIN summaries s ON s.video_id = v.id
         WHERE (v.transcript_status IN ('pending', 'loading', 'failed')
            OR (v.transcript_status = 'ready' AND v.summary_status IN ('pending', 'loading', 'failed')))
           AND v.retry_count < ?2
         ORDER BY v.published_at DESC
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
    let quality_score: Option<i64> = row.get::<Option<i64>>(10).unwrap_or(None);
    Ok(Video {
        id: row.get(0)?,
        channel_id: row.get(1)?,
        title: row.get(2)?,
        thumbnail_url: row.get(3)?,
        published_at: parse_rfc3339_to_utc(&published_at)?,
        is_short: is_short_val != 0,
        transcript_status: ContentStatus::from_db_value(&transcript_status),
        summary_status: ContentStatus::from_db_value(&summary_status),
        acknowledged: acknowledged_val != 0,
        retry_count: retry_count.clamp(0, 255) as u8,
        quality_score: quality_score.map(|s| s.clamp(0, 10) as u8),
    })
}

pub async fn upsert_transcript(
    conn: &Connection,
    transcript: &Transcript,
) -> Result<(), libsql::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO transcripts (video_id, raw_text, formatted_markdown, render_mode) VALUES (?1, ?2, ?3, ?4)",
        params![
            transcript.video_id.as_str(),
            transcript.raw_text.as_deref(),
            transcript.formatted_markdown.as_deref(),
            transcript.render_mode.as_str(),
        ],
    )
    .await?;
    Ok(())
}

fn normalize_highlight_text(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn clamp_highlight_context(input: &str) -> String {
    const MAX_CONTEXT_CHARS: usize = 160;
    input.chars().take(MAX_CONTEXT_CHARS).collect()
}

fn row_to_highlight(row: &libsql::Row) -> Result<Highlight, libsql::Error> {
    let source: String = row.get(2)?;
    let created_at: String = row.get(6)?;
    Ok(Highlight {
        id: row.get(0)?,
        video_id: row.get(1)?,
        source: HighlightSource::from_db_value(&source),
        text: row.get(3)?,
        prefix_context: row.get(4)?,
        suffix_context: row.get(5)?,
        created_at: parse_sqlite_datetime_to_utc(&created_at)?,
    })
}

pub async fn create_highlight(
    conn: &Connection,
    video_id: &str,
    source: HighlightSource,
    text: &str,
    prefix_context: &str,
    suffix_context: &str,
) -> Result<Highlight, libsql::Error> {
    let normalized_text = normalize_highlight_text(text);
    let prefix_context = clamp_highlight_context(prefix_context);
    let suffix_context = clamp_highlight_context(suffix_context);

    conn.execute(
        "INSERT OR IGNORE INTO highlights (video_id, source, text, normalized_text, prefix_context, suffix_context, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            video_id,
            source.as_str(),
            text,
            normalized_text.as_str(),
            prefix_context.as_str(),
            suffix_context.as_str(),
            chrono::Utc::now().to_rfc3339(),
        ],
    )
    .await?;

    let mut rows = conn
        .query(
            "SELECT id, video_id, source, text, prefix_context, suffix_context, created_at
             FROM highlights
             WHERE video_id = ?1
               AND source = ?2
               AND normalized_text = ?3
               AND prefix_context = ?4
               AND suffix_context = ?5
             ORDER BY id DESC
             LIMIT 1",
            params![
                video_id,
                source.as_str(),
                normalized_text.as_str(),
                prefix_context.as_str(),
                suffix_context.as_str()
            ],
        )
        .await?;

    rows.next()
        .await?
        .map(|row| row_to_highlight(&row))
        .transpose()?
        .ok_or_else(|| panic!("highlight should exist immediately after insert"))
}

pub async fn list_video_highlights(
    conn: &Connection,
    video_id: &str,
) -> Result<Vec<Highlight>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT id, video_id, source, text, prefix_context, suffix_context, created_at
             FROM highlights
             WHERE video_id = ?1
             ORDER BY created_at DESC, id DESC",
            params![video_id],
        )
        .await?;

    let mut highlights = Vec::new();
    while let Some(row) = rows.next().await? {
        highlights.push(row_to_highlight(&row)?);
    }
    Ok(highlights)
}

pub async fn delete_highlight(conn: &Connection, highlight_id: i64) -> Result<bool, libsql::Error> {
    let changes = conn
        .execute(
            "DELETE FROM highlights WHERE id = ?1",
            params![highlight_id],
        )
        .await?;
    Ok(changes > 0)
}

pub async fn list_highlights_grouped(
    conn: &Connection,
) -> Result<Vec<HighlightChannelGroup>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT
                c.id,
                c.name,
                c.thumbnail_url,
                v.id,
                v.title,
                v.thumbnail_url,
                v.published_at,
                h.id,
                h.source,
                h.text,
                h.prefix_context,
                h.suffix_context,
                h.created_at
             FROM highlights h
             INNER JOIN videos v ON v.id = h.video_id
             INNER JOIN channels c ON c.id = v.channel_id
             ORDER BY v.published_at DESC, h.created_at DESC, h.id DESC",
            (),
        )
        .await?;

    let mut groups: Vec<HighlightChannelGroup> = Vec::new();
    while let Some(row) = rows.next().await? {
        let channel_id: String = row.get(0)?;
        let channel_name: String = row.get(1)?;
        let channel_thumbnail_url: Option<String> = row.get(2)?;
        let video_id: String = row.get(3)?;
        let video_title: String = row.get(4)?;
        let video_thumbnail_url: Option<String> = row.get(5)?;
        let video_published_at: String = row.get(6)?;
        let parsed_published_at = parse_rfc3339_to_utc(&video_published_at)?;
        let highlight_source: String = row.get(8)?;
        let highlight = Highlight {
            id: row.get(7)?,
            video_id: video_id.clone(),
            source: HighlightSource::from_db_value(&highlight_source),
            text: row.get(9)?,
            prefix_context: row.get(10)?,
            suffix_context: row.get(11)?,
            created_at: parse_sqlite_datetime_to_utc(&row.get::<String>(12)?)?,
        };

        let channel_index = groups
            .iter()
            .position(|group| group.channel_id == channel_id)
            .unwrap_or_else(|| {
                groups.push(HighlightChannelGroup {
                    channel_id: channel_id.clone(),
                    channel_name: channel_name.clone(),
                    channel_thumbnail_url: channel_thumbnail_url.clone(),
                    videos: Vec::new(),
                });
                groups.len() - 1
            });

        let video_index = groups[channel_index]
            .videos
            .iter()
            .position(|video| video.video_id == video_id)
            .unwrap_or_else(|| {
                groups[channel_index].videos.push(HighlightVideoGroup {
                    video_id: video_id.clone(),
                    title: video_title.clone(),
                    thumbnail_url: video_thumbnail_url.clone(),
                    published_at: parsed_published_at,
                    highlights: Vec::new(),
                });
                groups[channel_index].videos.len() - 1
            });

        groups[channel_index].videos[video_index]
            .highlights
            .push(highlight);
    }

    Ok(groups)
}

pub async fn mark_search_source_pending(
    conn: &Connection,
    video_id: &str,
    source_kind: SearchSourceKind,
    content_hash: &str,
) -> Result<(), libsql::Error> {
    conn.execute(
        "INSERT INTO search_sources (
            video_id,
            source_kind,
            content_hash,
            source_generation,
            embedding_model,
            index_status,
            last_indexed_at,
            last_error
         )
         VALUES (?1, ?2, ?3, 1, NULL, 'pending', NULL, NULL)
         ON CONFLICT(video_id, source_kind) DO UPDATE SET
             content_hash = excluded.content_hash,
             source_generation = search_sources.source_generation + 1,
             embedding_model = NULL,
             index_status = 'pending',
             last_indexed_at = NULL,
             last_error = NULL",
        params![video_id, source_kind.as_str(), content_hash],
    )
    .await?;
    Ok(())
}

pub async fn clear_search_source(
    conn: &Connection,
    video_id: &str,
    source_kind: SearchSourceKind,
) -> Result<(), libsql::Error> {
    delete_search_rows_for_source(conn, video_id, source_kind).await?;
    conn.execute(
        "DELETE FROM search_sources WHERE video_id = ?1 AND source_kind = ?2",
        params![video_id, source_kind.as_str()],
    )
    .await?;
    Ok(())
}

pub async fn get_search_source_state(
    conn: &Connection,
    video_id: &str,
    source_kind: SearchSourceKind,
) -> Result<Option<SearchSourceState>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT
                id,
                source_generation,
                video_id,
                source_kind,
                content_hash,
                embedding_model,
                index_status,
                last_indexed_at,
                last_error
             FROM search_sources
             WHERE video_id = ?1 AND source_kind = ?2",
            params![video_id, source_kind.as_str()],
        )
        .await?;

    rows.next()
        .await?
        .map(|row| {
            let source_kind_raw: String = row.get(3)?;
            Ok(SearchSourceState {
                id: row.get(0)?,
                source_generation: row.get(1)?,
                video_id: row.get(2)?,
                source_kind: SearchSourceKind::from_db_value(&source_kind_raw),
                content_hash: row.get(4)?,
                embedding_model: row.get(5)?,
                index_status: row.get(6)?,
                last_indexed_at: row.get(7)?,
                last_error: row.get(8)?,
            })
        })
        .transpose()
}

pub async fn list_pending_search_sources(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<SearchSourceState>, libsql::Error> {
    let mut results = Vec::new();

    let mut summary_rows = conn
        .query(
            "SELECT
                id,
                source_generation,
                video_id,
                source_kind,
                content_hash,
                embedding_model,
                index_status,
                last_indexed_at,
                last_error
             FROM search_sources
             WHERE index_status = 'pending'
               AND source_kind = 'summary'
             LIMIT ?1",
            params![limit as i64],
        )
        .await?;
    while let Some(row) = summary_rows.next().await? {
        let source_kind_raw: String = row.get(3)?;
        results.push(SearchSourceState {
            id: row.get(0)?,
            source_generation: row.get(1)?,
            video_id: row.get(2)?,
            source_kind: SearchSourceKind::from_db_value(&source_kind_raw),
            content_hash: row.get(4)?,
            embedding_model: row.get(5)?,
            index_status: row.get(6)?,
            last_indexed_at: row.get(7)?,
            last_error: row.get(8)?,
        });
    }

    if results.len() >= limit {
        results.truncate(limit);
        return Ok(results);
    }

    let remaining = limit - results.len();
    let mut transcript_rows = conn
        .query(
            "SELECT
                id,
                source_generation,
                video_id,
                source_kind,
                content_hash,
                embedding_model,
                index_status,
                last_indexed_at,
                last_error
             FROM search_sources
             WHERE index_status = 'pending'
               AND source_kind = 'transcript'
             LIMIT ?1",
            params![remaining as i64],
        )
        .await?;
    while let Some(row) = transcript_rows.next().await? {
        let source_kind_raw: String = row.get(3)?;
        results.push(SearchSourceState {
            id: row.get(0)?,
            source_generation: row.get(1)?,
            video_id: row.get(2)?,
            source_kind: SearchSourceKind::from_db_value(&source_kind_raw),
            content_hash: row.get(4)?,
            embedding_model: row.get(5)?,
            index_status: row.get(6)?,
            last_indexed_at: row.get(7)?,
            last_error: row.get(8)?,
        });
    }

    Ok(results)
}

pub async fn mark_search_source_indexing(
    conn: &Connection,
    video_id: &str,
    source_kind: SearchSourceKind,
    content_hash: &str,
) -> Result<bool, libsql::Error> {
    let changes = conn
        .execute(
            "UPDATE search_sources
             SET index_status = 'indexing', last_error = NULL
             WHERE video_id = ?1
               AND source_kind = ?2
               AND content_hash = ?3
               AND index_status = 'pending'",
            params![video_id, source_kind.as_str(), content_hash],
        )
        .await?;
    Ok(changes > 0)
}

pub async fn mark_search_source_failed(
    conn: &Connection,
    video_id: &str,
    source_kind: SearchSourceKind,
    content_hash: &str,
    error: &str,
) -> Result<(), libsql::Error> {
    conn.execute(
        "UPDATE search_sources
         SET index_status = 'failed', last_error = ?4
         WHERE video_id = ?1 AND source_kind = ?2 AND content_hash = ?3",
        params![video_id, source_kind.as_str(), content_hash, error],
    )
    .await?;
    Ok(())
}

pub async fn replace_search_chunks(
    conn: &Connection,
    video_id: &str,
    source_kind: SearchSourceKind,
    content_hash: &str,
    embedding_model: Option<&str>,
    chunks: &[SearchIndexChunk],
) -> Result<bool, libsql::Error> {
    let current = get_search_source_state(conn, video_id, source_kind).await?;
    let Some(current) = current else {
        return Ok(false);
    };

    if current.content_hash != content_hash || current.index_status != "indexing" {
        return Ok(false);
    }

    delete_search_rows_for_generation(conn, current.id, current.source_generation).await?;
    for chunk in chunks {
        if let Some(embedding_json) = chunk.embedding_json.as_deref() {
            conn.execute(
                "INSERT INTO search_chunks (
                    search_source_id, source_generation, chunk_index, section_title, chunk_text,
                    token_count, embedding
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, vector32(?7)
                 )",
                params![
                    current.id,
                    current.source_generation,
                    chunk.chunk_index as i64,
                    chunk.section_title.as_deref(),
                    chunk.chunk_text.as_str(),
                    chunk.token_count as i64,
                    embedding_json,
                ],
            )
            .await?;
        } else {
            conn.execute(
                "INSERT INTO search_chunks (
                    search_source_id, source_generation, chunk_index, section_title, chunk_text,
                    token_count, embedding
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, NULL
                 )",
                params![
                    current.id,
                    current.source_generation,
                    chunk.chunk_index as i64,
                    chunk.section_title.as_deref(),
                    chunk.chunk_text.as_str(),
                    chunk.token_count as i64,
                ],
            )
            .await?;
        }
    }

    conn.execute(
        "UPDATE search_sources
         SET embedding_model = ?4, index_status = 'ready', last_indexed_at = ?5, last_error = NULL
         WHERE id = ?1
           AND video_id = ?2
           AND source_kind = ?3
           AND content_hash = ?6
           AND source_generation = ?7",
        params![
            current.id,
            video_id,
            source_kind.as_str(),
            embedding_model,
            chrono::Utc::now().to_rfc3339(),
            content_hash,
            current.source_generation,
        ],
    )
    .await?;
    Ok(true)
}

pub async fn load_search_material(
    conn: &Connection,
    video_id: &str,
    source_kind: SearchSourceKind,
) -> Result<Option<SearchMaterial>, libsql::Error> {
    let (sql, params) = match source_kind {
        SearchSourceKind::Transcript => (
            "SELECT
                v.id,
                COALESCE(c.name, ''),
                v.title,
                TRIM(COALESCE(t.raw_text, t.formatted_markdown, ''))
             FROM videos v
             JOIN channels c ON c.id = v.channel_id
             JOIN transcripts t ON t.video_id = v.id
             WHERE v.id = ?1
               AND v.transcript_status = 'ready'",
            params![video_id],
        ),
        SearchSourceKind::Summary => (
            "SELECT
                v.id,
                COALESCE(c.name, ''),
                v.title,
                TRIM(COALESCE(s.content, ''))
             FROM videos v
             JOIN channels c ON c.id = v.channel_id
             JOIN summaries s ON s.video_id = v.id
             WHERE v.id = ?1
               AND v.summary_status = 'ready'",
            params![video_id],
        ),
    };

    let mut rows = conn.query(sql, params).await?;
    if let Some(row) = rows.next().await? {
        let content: String = row.get(3)?;
        if content.trim().is_empty() {
            return Ok(None);
        }
        return Ok(Some(SearchMaterial {
            video_id: row.get(0)?,
            channel_name: row.get(1)?,
            video_title: row.get(2)?,
            source_kind,
            content,
        }));
    }

    Ok(None)
}

pub async fn list_search_backfill_materials(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<SearchMaterial>, libsql::Error> {
    let mut materials = Vec::new();

    let summary_rows = conn
        .query(
            "SELECT
                v.id,
                COALESCE(c.name, ''),
                v.title,
                TRIM(COALESCE(s.content, ''))
             FROM videos v
             JOIN channels c ON c.id = v.channel_id
             JOIN summaries s ON s.video_id = v.id
             LEFT JOIN search_sources ss
               ON ss.video_id = v.id
              AND ss.source_kind = 'summary'
             WHERE v.summary_status = 'ready'
               AND TRIM(COALESCE(s.content, '')) <> ''
               AND ss.video_id IS NULL
             ORDER BY v.published_at DESC
             LIMIT ?1",
            params![limit as i64],
        )
        .await?;
    materials
        .extend(load_search_materials_from_rows(summary_rows, SearchSourceKind::Summary).await?);

    if materials.len() >= limit {
        materials.truncate(limit);
        return Ok(materials);
    }

    let remaining = limit - materials.len();
    let transcript_rows = conn
        .query(
            "SELECT
                v.id,
                COALESCE(c.name, ''),
                v.title,
                TRIM(COALESCE(t.raw_text, t.formatted_markdown, ''))
             FROM videos v
             JOIN channels c ON c.id = v.channel_id
             JOIN transcripts t ON t.video_id = v.id
             LEFT JOIN search_sources ss
               ON ss.video_id = v.id
              AND ss.source_kind = 'transcript'
             WHERE v.transcript_status = 'ready'
               AND TRIM(COALESCE(t.raw_text, t.formatted_markdown, '')) <> ''
               AND ss.video_id IS NULL
             ORDER BY v.published_at DESC
             LIMIT ?1",
            params![remaining as i64],
        )
        .await?;
    materials.extend(
        load_search_materials_from_rows(transcript_rows, SearchSourceKind::Transcript).await?,
    );

    Ok(materials)
}

pub async fn list_search_reconciliation_materials(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<SearchMaterial>, libsql::Error> {
    // Only reconcile sources that already have a search_sources row (ready/failed).
    // Backfill handles discovering brand-new content.
    let summary_rows = conn
        .query(
            "SELECT
                v.id,
                COALESCE(c.name, ''),
                v.title,
                TRIM(COALESCE(s.content, ''))
             FROM videos v
             JOIN channels c ON c.id = v.channel_id
             JOIN summaries s ON s.video_id = v.id
             JOIN search_sources ss
               ON ss.video_id = v.id AND ss.source_kind = 'summary'
             WHERE v.summary_status = 'ready'
               AND TRIM(COALESCE(s.content, '')) <> ''
               AND ss.index_status IN ('ready', 'failed')
             ORDER BY ss.last_indexed_at ASC
             LIMIT ?1",
            params![limit as i64],
        )
        .await?;
    let mut materials =
        load_search_materials_from_rows(summary_rows, SearchSourceKind::Summary).await?;

    if materials.len() >= limit {
        materials.truncate(limit);
        return Ok(materials);
    }

    let remaining = limit - materials.len();
    let transcript_rows = conn
        .query(
            "SELECT
                v.id,
                COALESCE(c.name, ''),
                v.title,
                TRIM(COALESCE(t.raw_text, t.formatted_markdown, ''))
             FROM videos v
             JOIN channels c ON c.id = v.channel_id
             JOIN transcripts t ON t.video_id = v.id
             JOIN search_sources ss
               ON ss.video_id = v.id AND ss.source_kind = 'transcript'
             WHERE v.transcript_status = 'ready'
               AND TRIM(COALESCE(t.raw_text, t.formatted_markdown, '')) <> ''
               AND ss.index_status IN ('ready', 'failed')
             ORDER BY ss.last_indexed_at ASC
             LIMIT ?1",
            params![remaining as i64],
        )
        .await?;
    materials.extend(
        load_search_materials_from_rows(transcript_rows, SearchSourceKind::Transcript).await?,
    );

    Ok(materials)
}

pub async fn list_search_progress_materials(
    conn: &Connection,
) -> Result<Vec<SearchProgressMaterial>, libsql::Error> {
    let summary_rows = conn
        .query(
            "SELECT
                v.id,
                TRIM(COALESCE(s.content, '')),
                ss.index_status,
                ss.embedding_model
             FROM videos v
             JOIN summaries s ON s.video_id = v.id
             LEFT JOIN search_sources ss
               ON ss.video_id = v.id
              AND ss.source_kind = 'summary'
             WHERE v.summary_status = 'ready'
               AND TRIM(COALESCE(s.content, '')) <> ''",
            (),
        )
        .await?;
    let mut materials =
        load_search_progress_materials_from_rows(summary_rows, SearchSourceKind::Summary).await?;

    let transcript_rows = conn
        .query(
            "SELECT
                v.id,
                TRIM(COALESCE(t.raw_text, t.formatted_markdown, '')),
                ss.index_status,
                ss.embedding_model
             FROM videos v
             JOIN transcripts t ON t.video_id = v.id
             LEFT JOIN search_sources ss
               ON ss.video_id = v.id
              AND ss.source_kind = 'transcript'
             WHERE v.transcript_status = 'ready'
               AND TRIM(COALESCE(t.raw_text, t.formatted_markdown, '')) <> ''",
            (),
        )
        .await?;
    materials.extend(
        load_search_progress_materials_from_rows(transcript_rows, SearchSourceKind::Transcript)
            .await?,
    );

    Ok(materials)
}

pub async fn search_vector_candidates(
    conn: &Connection,
    query_embedding: &str,
    embedding_model: &str,
    source_kind: Option<SearchSourceKind>,
    channel_id: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchCandidate>, libsql::Error> {
    let mut sql = String::from(
        "SELECT
            sc.id,
            ss.video_id,
            v.channel_id,
            COALESCE(c.name, ''),
            v.title,
            ss.source_kind,
            sc.section_title,
            sc.chunk_text,
            v.published_at
         FROM search_chunks sc
         JOIN vector_top_k('idx_search_chunks_embedding', vector32(?1), ?2) vt
           ON vt.id = sc.id
         JOIN search_sources ss
           ON ss.id = sc.search_source_id
          AND ss.source_generation = sc.source_generation
          AND ss.index_status = 'ready'
          AND ss.embedding_model = ?3
         JOIN videos v ON v.id = ss.video_id
         LEFT JOIN channels c ON c.id = v.channel_id
         WHERE 1 = 1",
    );
    let mut params_vec = vec![
        Value::from(query_embedding.to_string()),
        Value::from(limit as i64),
        Value::from(embedding_model.to_string()),
    ];
    let mut next_param_index = 4;

    if let Some(source_kind) = source_kind {
        sql.push_str(&format!(" AND ss.source_kind = ?{next_param_index}"));
        params_vec.push(Value::from(source_kind.as_str().to_string()));
        next_param_index += 1;
    }

    if let Some(channel_id) = channel_id {
        sql.push_str(&format!(" AND v.channel_id = ?{next_param_index}"));
        params_vec.push(Value::from(channel_id.to_string()));
    }

    // The vector index may not exist yet (deferred during bulk indexing).
    // Gracefully return empty results if the query fails.
    let rows_result = conn.query(&sql, params_vec).await;
    let mut rows = match rows_result {
        Ok(rows) => rows,
        Err(err) => {
            tracing::debug!(error = %err, "vector search unavailable (index may not exist yet)");
            return Ok(Vec::new());
        }
    };
    let mut matches = Vec::new();
    while let Some(row) = rows.next().await? {
        let chunk_id: i64 = row.get(0)?;
        let source_kind_raw: String = row.get(5)?;
        matches.push(SearchCandidate {
            chunk_id: chunk_id.to_string(),
            video_id: row.get(1)?,
            channel_id: row.get(2)?,
            channel_name: row.get(3)?,
            video_title: row.get(4)?,
            source_kind: SearchSourceKind::from_db_value(&source_kind_raw),
            section_title: row.get(6)?,
            chunk_text: row.get(7)?,
            published_at: row.get(8)?,
        });
    }
    Ok(matches)
}

pub async fn has_vector_index(conn: &Connection) -> Result<bool, libsql::Error> {
    if !schema_object_exists(conn, "table", "libsql_vector_meta_shadow").await? {
        return Ok(false);
    }

    let mut rows = conn
        .query(
            "SELECT 1 FROM libsql_vector_meta_shadow WHERE name = ?1 LIMIT 1",
            params!["idx_search_chunks_embedding"],
        )
        .await?;
    Ok(rows.next().await?.is_some())
}

/// Create the vector index for semantic search. Expensive on remote Turso -
/// should only be called once after bulk indexing is complete.
pub async fn ensure_vector_index(conn: &Connection) -> Result<(), libsql::Error> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_search_chunks_embedding ON search_chunks(libsql_vector_idx(embedding, 'metric=cosine'))",
        (),
    )
    .await?;
    Ok(())
}

pub async fn reset_search_projection(conn: &Connection) -> Result<(), libsql::Error> {
    conn.execute("DROP TRIGGER IF EXISTS search_chunks_ai", ())
        .await?;
    conn.execute("DROP TRIGGER IF EXISTS search_chunks_ad", ())
        .await?;
    conn.execute("DROP TRIGGER IF EXISTS search_chunks_au", ())
        .await?;
    conn.execute("DROP TABLE IF EXISTS search_chunks_fts", ())
        .await?;
    conn.execute("DROP TABLE IF EXISTS search_chunks", ())
        .await?;
    conn.execute("DROP TABLE IF EXISTS search_sources", ())
        .await?;
    ensure_search_projection_schema(conn).await?;
    Ok(())
}

pub async fn search_exact_candidates(
    conn: &Connection,
    query_embedding: &str,
    embedding_model: &str,
    candidate_ids: &[i64],
    limit: usize,
) -> Result<Vec<SearchCandidate>, libsql::Error> {
    if candidate_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut sql = String::from(
        "SELECT
            sc.id,
            ss.video_id,
            v.channel_id,
            COALESCE(c.name, ''),
            v.title,
            ss.source_kind,
            sc.section_title,
            sc.chunk_text,
            v.published_at
         FROM search_chunks sc
         JOIN search_sources ss
           ON ss.id = sc.search_source_id
          AND ss.source_generation = sc.source_generation
          AND ss.index_status = 'ready'
          AND ss.embedding_model = ?2
         JOIN videos v ON v.id = ss.video_id
         LEFT JOIN channels c ON c.id = v.channel_id
         WHERE sc.id IN (",
    );

    let mut params_vec = vec![
        Value::from(query_embedding.to_string()),
        Value::from(embedding_model.to_string()),
    ];

    for (index, candidate_id) in candidate_ids.iter().enumerate() {
        if index > 0 {
            sql.push_str(", ");
        }
        let param_index = index + 3;
        sql.push_str(&format!("?{param_index}"));
        params_vec.push(Value::from(*candidate_id));
    }

    let limit_param_index = candidate_ids.len() + 3;
    sql.push_str(") ORDER BY vector_distance_cos(sc.embedding, vector32(?1)) ASC");
    sql.push_str(&format!(" LIMIT ?{limit_param_index}"));
    params_vec.push(Value::from(limit as i64));

    let mut rows = conn.query(&sql, params_vec).await?;
    let mut matches = Vec::new();
    while let Some(row) = rows.next().await? {
        let chunk_id: i64 = row.get(0)?;
        let source_kind_raw: String = row.get(5)?;
        matches.push(SearchCandidate {
            chunk_id: chunk_id.to_string(),
            video_id: row.get(1)?,
            channel_id: row.get(2)?,
            channel_name: row.get(3)?,
            video_title: row.get(4)?,
            source_kind: SearchSourceKind::from_db_value(&source_kind_raw),
            section_title: row.get(6)?,
            chunk_text: row.get(7)?,
            published_at: row.get(8)?,
        });
    }
    Ok(matches)
}

pub async fn search_exact_global_candidates(
    conn: &Connection,
    query_embedding: &str,
    embedding_model: &str,
    source_kind: Option<SearchSourceKind>,
    channel_id: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchCandidate>, libsql::Error> {
    let mut sql = String::from(
        "SELECT
            sc.id,
            ss.video_id,
            v.channel_id,
            COALESCE(c.name, ''),
            v.title,
            ss.source_kind,
            sc.section_title,
            sc.chunk_text,
            v.published_at
         FROM search_chunks sc
         JOIN search_sources ss
           ON ss.id = sc.search_source_id
          AND ss.source_generation = sc.source_generation
          AND ss.index_status = 'ready'
          AND ss.embedding_model = ?2
         JOIN videos v ON v.id = ss.video_id
         LEFT JOIN channels c ON c.id = v.channel_id
         WHERE 1 = 1",
    );
    let mut params_vec = vec![
        Value::from(query_embedding.to_string()),
        Value::from(embedding_model.to_string()),
    ];
    let mut next_param_index = 3;

    if let Some(source_kind) = source_kind {
        sql.push_str(&format!(" AND ss.source_kind = ?{next_param_index}"));
        params_vec.push(Value::from(source_kind.as_str().to_string()));
        next_param_index += 1;
    }

    if let Some(channel_id) = channel_id {
        sql.push_str(&format!(" AND v.channel_id = ?{next_param_index}"));
        params_vec.push(Value::from(channel_id.to_string()));
        next_param_index += 1;
    }

    sql.push_str(" ORDER BY vector_distance_cos(sc.embedding, vector32(?1)) ASC");
    sql.push_str(&format!(" LIMIT ?{next_param_index}"));
    params_vec.push(Value::from(limit as i64));

    let mut rows = conn.query(&sql, params_vec).await?;
    let mut matches = Vec::new();
    while let Some(row) = rows.next().await? {
        let chunk_id: i64 = row.get(0)?;
        let source_kind_raw: String = row.get(5)?;
        matches.push(SearchCandidate {
            chunk_id: chunk_id.to_string(),
            video_id: row.get(1)?,
            channel_id: row.get(2)?,
            channel_name: row.get(3)?,
            video_title: row.get(4)?,
            source_kind: SearchSourceKind::from_db_value(&source_kind_raw),
            section_title: row.get(6)?,
            chunk_text: row.get(7)?,
            published_at: row.get(8)?,
        });
    }
    Ok(matches)
}

pub async fn search_fts_candidates(
    conn: &Connection,
    query: &str,
    embedding_model: Option<&str>,
    source_kind: Option<SearchSourceKind>,
    channel_id: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchCandidate>, libsql::Error> {
    let mut sql = String::from(
        "SELECT
            sc.id,
            ss.video_id,
            v.channel_id,
            COALESCE(c.name, ''),
            v.title,
            ss.source_kind,
            sc.section_title,
            sc.chunk_text,
            v.published_at
         FROM search_chunks_fts
         JOIN search_chunks sc ON sc.id = search_chunks_fts.rowid
         JOIN search_sources ss
           ON ss.id = sc.search_source_id
          AND ss.source_generation = sc.source_generation
          AND ss.index_status = 'ready'
         JOIN videos v ON v.id = ss.video_id
         LEFT JOIN channels c ON c.id = v.channel_id
         WHERE search_chunks_fts MATCH ?1",
    );
    let mut params_vec = vec![Value::from(query.to_string())];
    let mut next_param_index = 2;

    if let Some(embedding_model) = embedding_model {
        sql.push_str(&format!(" AND ss.embedding_model = ?{next_param_index}"));
        params_vec.push(Value::from(embedding_model.to_string()));
        next_param_index += 1;
    }

    if let Some(source_kind) = source_kind {
        sql.push_str(&format!(" AND ss.source_kind = ?{next_param_index}"));
        params_vec.push(Value::from(source_kind.as_str().to_string()));
        next_param_index += 1;
    }

    if let Some(channel_id) = channel_id {
        sql.push_str(&format!(" AND v.channel_id = ?{next_param_index}"));
        params_vec.push(Value::from(channel_id.to_string()));
        next_param_index += 1;
    }

    sql.push_str(&format!(
        " ORDER BY bm25(search_chunks_fts) ASC LIMIT ?{next_param_index}"
    ));
    params_vec.push(Value::from(limit as i64));
    let mut rows = conn.query(&sql, params_vec).await?;
    let mut matches = Vec::new();
    while let Some(row) = rows.next().await? {
        let chunk_id: i64 = row.get(0)?;
        let source_kind_raw: String = row.get(5)?;
        matches.push(SearchCandidate {
            chunk_id: chunk_id.to_string(),
            video_id: row.get(1)?,
            channel_id: row.get(2)?,
            channel_name: row.get(3)?,
            video_title: row.get(4)?,
            source_kind: SearchSourceKind::from_db_value(&source_kind_raw),
            section_title: row.get(6)?,
            chunk_text: row.get(7)?,
            published_at: row.get(8)?,
        });
    }
    Ok(matches)
}

pub async fn get_search_source_counts(
    conn: &Connection,
) -> Result<SearchSourceCounts, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT
                SUM(CASE WHEN index_status = 'pending' THEN 1 ELSE 0 END),
                SUM(CASE WHEN index_status = 'indexing' THEN 1 ELSE 0 END),
                SUM(CASE WHEN index_status = 'ready' THEN 1 ELSE 0 END),
                SUM(CASE WHEN index_status = 'failed' THEN 1 ELSE 0 END)
             FROM search_sources",
            (),
        )
        .await?;

    let (pending, indexing, ready, failed) = if let Some(row) = rows.next().await? {
        (
            row.get::<Option<i64>>(0)?.unwrap_or(0).max(0) as usize,
            row.get::<Option<i64>>(1)?.unwrap_or(0).max(0) as usize,
            row.get::<Option<i64>>(2)?.unwrap_or(0).max(0) as usize,
            row.get::<Option<i64>>(3)?.unwrap_or(0).max(0) as usize,
        )
    } else {
        (0, 0, 0, 0)
    };

    // Count indexable sources from canonical video readiness flags so startup status
    // does not need to scan large transcript/summary text tables on remote libSQL.
    let mut total_rows = conn
        .query(
            "SELECT
                SUM(CASE WHEN transcript_status = 'ready' THEN 1 ELSE 0 END)
                +
                SUM(CASE WHEN summary_status = 'ready' THEN 1 ELSE 0 END)
             FROM videos",
            (),
        )
        .await?;

    let total_sources = if let Some(row) = total_rows.next().await? {
        row.get::<Option<i64>>(0)?.unwrap_or(0).max(0) as usize
    } else {
        0
    };

    Ok(SearchSourceCounts {
        pending,
        indexing,
        ready,
        failed,
        total_sources,
    })
}

pub async fn prune_stale_search_rows(
    conn: &Connection,
    limit: usize,
) -> Result<usize, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT sc.id
             FROM search_chunks sc
             LEFT JOIN search_sources ss
               ON ss.id = sc.search_source_id
              AND ss.source_generation = sc.source_generation
              AND ss.index_status = 'ready'
             WHERE ss.id IS NULL
             ORDER BY sc.search_source_id ASC, sc.chunk_index ASC
             LIMIT ?1",
            params![limit as i64],
        )
        .await?;

    let mut chunk_ids = Vec::new();
    while let Some(row) = rows.next().await? {
        chunk_ids.push(row.get::<i64>(0)?);
    }

    for chunk_id in &chunk_ids {
        conn.execute(
            "DELETE FROM search_chunks WHERE id = ?1",
            params![*chunk_id],
        )
        .await?;
    }

    Ok(chunk_ids.len())
}

async fn delete_search_rows_for_source(
    conn: &Connection,
    video_id: &str,
    source_kind: SearchSourceKind,
) -> Result<(), libsql::Error> {
    conn.execute(
        "DELETE FROM search_chunks
         WHERE search_source_id IN (
            SELECT id FROM search_sources WHERE video_id = ?1 AND source_kind = ?2
         )",
        params![video_id, source_kind.as_str()],
    )
    .await?;
    Ok(())
}

async fn delete_search_rows_for_generation(
    conn: &Connection,
    search_source_id: i64,
    source_generation: i64,
) -> Result<(), libsql::Error> {
    conn.execute(
        "DELETE FROM search_chunks
         WHERE search_source_id = ?1 AND source_generation = ?2",
        params![search_source_id, source_generation],
    )
    .await?;
    Ok(())
}

async fn write_transcript_content(
    conn: &Connection,
    video_id: &str,
    content: &str,
    render_mode: TranscriptRenderMode,
) -> Result<Transcript, libsql::Error> {
    let existing = get_transcript(conn, video_id).await?;
    let transcript = transcript_with_render_mode(video_id, content, render_mode, existing);
    upsert_transcript(conn, &transcript).await?;
    Ok(transcript)
}

pub async fn save_manual_transcript(
    conn: &Connection,
    video_id: &str,
    content: &str,
    render_mode: TranscriptRenderMode,
) -> Result<Transcript, libsql::Error> {
    let transcript = write_transcript_content(conn, video_id, content, render_mode).await?;
    update_video_transcript_status(conn, video_id, ContentStatus::Ready).await?;
    Ok(transcript)
}

pub async fn get_transcript(
    conn: &Connection,
    video_id: &str,
) -> Result<Option<Transcript>, libsql::Error> {
    let mut rows = conn
        .query(
            "SELECT video_id, raw_text, formatted_markdown, COALESCE(render_mode, 'plain_text') FROM transcripts WHERE video_id = ?1",
            params![video_id],
        )
        .await?;

    if let Some(row) = rows.next().await? {
        let render_mode: String = row.get(3)?;
        Ok(Some(Transcript {
            video_id: row.get(0)?,
            raw_text: row.get(1)?,
            formatted_markdown: row.get(2)?,
            render_mode: TranscriptRenderMode::from_db_value(&render_mode),
        }))
    } else {
        Ok(None)
    }
}

fn transcript_with_render_mode(
    video_id: &str,
    content: &str,
    render_mode: TranscriptRenderMode,
    existing: Option<Transcript>,
) -> Transcript {
    match render_mode {
        TranscriptRenderMode::PlainText => Transcript {
            video_id: video_id.to_string(),
            raw_text: Some(content.to_string()),
            formatted_markdown: None,
            render_mode,
        },
        TranscriptRenderMode::Markdown => {
            let raw_text = existing
                .as_ref()
                .and_then(|transcript| transcript.raw_text.clone())
                .filter(|value| !value.trim().is_empty())
                .or_else(|| {
                    existing
                        .as_ref()
                        .and_then(|transcript| transcript.formatted_markdown.clone())
                        .filter(|value| !value.trim().is_empty())
                })
                .or_else(|| Some(content.to_string()));

            Transcript {
                video_id: video_id.to_string(),
                raw_text,
                formatted_markdown: Some(content.to_string()),
                render_mode,
            }
        }
    }
}

pub async fn upsert_summary(conn: &Connection, summary: &Summary) -> Result<(), libsql::Error> {
    conn.execute(
        "INSERT INTO summaries (video_id, content, model_used, quality_score, quality_note, quality_model_used, auto_regen_attempts)
         VALUES (
             ?1,
             ?2,
             ?3,
             ?4,
             ?5,
             ?6,
             COALESCE((SELECT auto_regen_attempts FROM summaries WHERE video_id = ?1), 0)
         )
         ON CONFLICT(video_id) DO UPDATE SET
             content = excluded.content,
             model_used = excluded.model_used,
             quality_score = excluded.quality_score,
             quality_note = excluded.quality_note,
             quality_model_used = excluded.quality_model_used,
             auto_regen_attempts = excluded.auto_regen_attempts",
        params![
            summary.video_id.as_str(),
            summary.content.as_str(),
            summary.model_used.as_deref(),
            summary.quality_score.map(i64::from),
            summary.quality_note.as_deref(),
            summary.quality_model_used.as_deref(),
        ],
    )
    .await?;
    Ok(())
}

async fn write_summary_content(
    conn: &Connection,
    video_id: &str,
    content: &str,
    model_used: Option<&str>,
) -> Result<Summary, libsql::Error> {
    conn.execute(
        "INSERT INTO summaries (video_id, content, model_used, quality_score, quality_note, quality_model_used, auto_regen_attempts)
         VALUES (?1, ?2, ?3, NULL, NULL, NULL, 0)
         ON CONFLICT(video_id) DO UPDATE SET
             content = excluded.content,
             model_used = excluded.model_used,
             quality_score = NULL,
             quality_note = NULL,
             quality_model_used = NULL,
             auto_regen_attempts = 0",
        params![video_id, content, model_used],
    )
    .await?;
    Ok(Summary {
        video_id: video_id.to_string(),
        content: content.to_string(),
        model_used: model_used.map(ToOwned::to_owned),
        quality_score: None,
        quality_note: None,
        quality_model_used: None,
    })
}

pub async fn save_manual_summary(
    conn: &Connection,
    video_id: &str,
    content: &str,
    model_used: Option<&str>,
) -> Result<Summary, libsql::Error> {
    let summary = write_summary_content(conn, video_id, content, model_used).await?;
    update_video_summary_status(conn, video_id, ContentStatus::Ready).await?;
    Ok(summary)
}

pub async fn update_summary_quality(
    conn: &Connection,
    video_id: &str,
    quality_score: Option<u8>,
    quality_note: Option<&str>,
    quality_model_used: Option<&str>,
) -> Result<(), libsql::Error> {
    conn.execute(
        "UPDATE summaries SET quality_score = ?1, quality_note = ?2, quality_model_used = ?3 WHERE video_id = ?4",
        params![
            quality_score.map(i64::from),
            quality_note,
            quality_model_used,
            video_id
        ],
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
    clear_search_source(conn, video_id, SearchSourceKind::Summary).await?;
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
            "SELECT video_id, content, model_used, quality_score, quality_note, quality_model_used
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
            quality_model_used: row.get(5)?,
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
    use crate::services::search::vector_to_json;
    use chrono::Utc;

    #[tokio::test]
    async fn test_db_pool_connect_shares_state_across_callers() {
        let pool = init_db_memory().await.unwrap();
        let conn_a = pool.connect();
        let conn_b = pool.connect();

        conn_a
            .execute(
                "CREATE TABLE pool_check (id INTEGER PRIMARY KEY, label TEXT)",
                (),
            )
            .await
            .unwrap();
        conn_b
            .execute(
                "INSERT INTO pool_check (id, label) VALUES (?1, ?2)",
                params![1i64, "shared"],
            )
            .await
            .unwrap();

        let mut rows = conn_a
            .query("SELECT label FROM pool_check WHERE id = ?1", params![1i64])
            .await
            .unwrap();
        let row = rows.next().await.unwrap().expect("row should exist");
        let label: String = row.get(0).unwrap();
        assert_eq!(label, "shared");
    }

    #[tokio::test]
    async fn test_db_pool_connect_keeps_connection_local_state_isolated() {
        let pool = init_db_memory().await.unwrap();
        let conn_a = pool.connect();
        let conn_b = pool.connect();

        conn_a
            .execute(
                "CREATE TEMP TABLE pool_connection_local_check (label TEXT)",
                (),
            )
            .await
            .unwrap();
        conn_a
            .execute(
                "INSERT INTO pool_connection_local_check (label) VALUES (?1)",
                params!["local"],
            )
            .await
            .unwrap();

        let err = conn_b
            .query("SELECT label FROM pool_connection_local_check", ())
            .await
            .expect_err("second handle should not see temp tables from another connection");

        assert!(
            err.to_string().contains("no such table"),
            "unexpected error: {err}"
        );
    }

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
            quality_score: None,
        };
        insert_video(&conn, &video).await.unwrap();

        let transcript = Transcript {
            video_id: "vid1".to_string(),
            raw_text: Some("Hello world".to_string()),
            formatted_markdown: Some("# Hello\n\nWorld".to_string()),
            render_mode: TranscriptRenderMode::PlainText,
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
    async fn test_write_transcript_content() {
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
            quality_score: None,
        };
        insert_video(&conn, &video).await.unwrap();

        write_transcript_content(&conn, "vid2", "## Edited", TranscriptRenderMode::PlainText)
            .await
            .unwrap();
        update_video_transcript_status(&conn, "vid2", ContentStatus::Ready)
            .await
            .unwrap();

        let transcript = get_transcript(&conn, "vid2").await.unwrap().unwrap();
        assert_eq!(transcript.formatted_markdown, None);
        assert_eq!(transcript.raw_text, Some("## Edited".to_string()));
        assert_eq!(transcript.render_mode, TranscriptRenderMode::PlainText);
    }

    #[tokio::test]
    async fn test_save_manual_transcript_marks_video_ready_and_returns_content() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC999_MANUAL".to_string(),
            handle: None,
            name: "Test".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let video = Video {
            id: "vid_manual_transcript".to_string(),
            channel_id: "UC999_MANUAL".to_string(),
            title: "Test Video".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Pending,
            summary_status: ContentStatus::Pending,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        };
        insert_video(&conn, &video).await.unwrap();

        let transcript = save_manual_transcript(
            &conn,
            "vid_manual_transcript",
            "## Edited",
            TranscriptRenderMode::PlainText,
        )
        .await
        .unwrap();

        assert_eq!(transcript.formatted_markdown, None);
        assert_eq!(transcript.raw_text, Some("## Edited".to_string()));
        assert_eq!(transcript.render_mode, TranscriptRenderMode::PlainText);
        let saved_video = get_video(&conn, "vid_manual_transcript")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved_video.transcript_status, ContentStatus::Ready);
    }

    #[tokio::test]
    async fn test_save_manual_transcript_preserves_existing_raw_text_for_markdown_mode() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC999_MANUAL_MD".to_string(),
            handle: None,
            name: "Test".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let video = Video {
            id: "vid_manual_transcript_md".to_string(),
            channel_id: "UC999_MANUAL_MD".to_string(),
            title: "Test Video".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Pending,
            summary_status: ContentStatus::Pending,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        };
        insert_video(&conn, &video).await.unwrap();

        upsert_transcript(
            &conn,
            &Transcript {
                video_id: "vid_manual_transcript_md".to_string(),
                raw_text: Some("Original raw text".to_string()),
                formatted_markdown: None,
                render_mode: TranscriptRenderMode::PlainText,
            },
        )
        .await
        .unwrap();

        let transcript = save_manual_transcript(
            &conn,
            "vid_manual_transcript_md",
            "## Edited markdown",
            TranscriptRenderMode::Markdown,
        )
        .await
        .unwrap();

        assert_eq!(transcript.raw_text, Some("Original raw text".to_string()));
        assert_eq!(
            transcript.formatted_markdown,
            Some("## Edited markdown".to_string())
        );
        assert_eq!(transcript.render_mode, TranscriptRenderMode::Markdown);
    }

    #[tokio::test]
    async fn test_write_summary_content() {
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
            quality_score: None,
        };
        insert_video(&conn, &video).await.unwrap();

        write_summary_content(&conn, "vid3", "Summary text", Some("manual"))
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
    async fn test_save_manual_summary_marks_video_ready_and_resets_quality() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC777_MANUAL".to_string(),
            handle: None,
            name: "Test".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let video = Video {
            id: "vid_manual_summary".to_string(),
            channel_id: "UC777_MANUAL".to_string(),
            title: "Test Video".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Pending,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        };
        insert_video(&conn, &video).await.unwrap();

        conn.execute(
            "INSERT INTO summaries (video_id, content, model_used, quality_score, quality_note, auto_regen_attempts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params!["vid_manual_summary", "Before", "manual", 9i64, "Minor mismatch", 2i64],
        )
        .await
        .unwrap();

        let summary =
            save_manual_summary(&conn, "vid_manual_summary", "After edit", Some("manual"))
                .await
                .unwrap();

        assert_eq!(summary.content, "After edit");
        assert_eq!(summary.model_used, Some("manual".to_string()));
        assert_eq!(summary.quality_score, None);
        assert_eq!(summary.quality_note, None);
        let saved_video = get_video(&conn, "vid_manual_summary")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved_video.summary_status, ContentStatus::Ready);
        let auto_regen_attempts = get_summary_auto_regen_attempts(&conn, "vid_manual_summary")
            .await
            .unwrap();
        assert_eq!(auto_regen_attempts, 0);
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
            quality_score: None,
        };
        insert_video(&conn, &video).await.unwrap();

        conn.execute(
            "INSERT INTO summaries (video_id, content, model_used, quality_score, quality_note, quality_model_used)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                "vid_eval",
                "Summary body",
                "manual",
                8i64,
                "Missed nuance",
                "glm-5:cloud",
            ],
        )
        .await
        .unwrap();

        let summary = get_summary(&conn, "vid_eval").await.unwrap().unwrap();
        assert_eq!(summary.quality_score, Some(8));
        assert_eq!(summary.quality_note, Some("Missed nuance".to_string()));
        assert_eq!(summary.quality_model_used, Some("glm-5:cloud".to_string()));
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
            quality_score: None,
        };
        insert_video(&conn, &video).await.unwrap();

        conn.execute(
            "INSERT INTO summaries (video_id, content, model_used, quality_score, quality_note, quality_model_used)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                "vid_eval2",
                "Before",
                "manual",
                9i64,
                "Minor mismatch",
                "glm-5:cloud",
            ],
        )
        .await
        .unwrap();

        write_summary_content(&conn, "vid_eval2", "After edit", Some("manual"))
            .await
            .unwrap();
        let summary = get_summary(&conn, "vid_eval2").await.unwrap().unwrap();
        assert_eq!(summary.content, "After edit");
        assert_eq!(summary.quality_score, None);
        assert_eq!(summary.quality_note, None);
        assert_eq!(summary.quality_model_used, None);
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
            quality_score: None,
        };
        insert_video(&conn, &video).await.unwrap();

        upsert_transcript(
            &conn,
            &Transcript {
                video_id: "vid_eval3".to_string(),
                raw_text: Some("Transcript body".to_string()),
                formatted_markdown: None,
                render_mode: TranscriptRenderMode::PlainText,
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
                quality_model_used: None,
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

        update_summary_quality(
            &conn,
            "vid_eval3",
            Some(7),
            Some("Missed one claim"),
            Some("qwen3-235b-a22b:cloud"),
        )
        .await
        .unwrap();
        let updated = get_summary(&conn, "vid_eval3").await.unwrap().unwrap();
        assert_eq!(updated.quality_score, Some(7));
        assert_eq!(updated.quality_note, Some("Missed one claim".to_string()));
        assert_eq!(
            updated.quality_model_used,
            Some("qwen3-235b-a22b:cloud".to_string())
        );

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
            quality_score: None,
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
                quality_model_used: None,
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

        write_summary_content(&conn, "vid_regen", "Manual edit", Some("manual"))
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
                quality_score: None,
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
                quality_score: None,
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
                quality_score: None,
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
                quality_score: None,
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
                quality_score: None,
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
            quality_score: None,
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
            quality_score: None,
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

        let all_videos = list_videos_by_channel(&conn, "UCF", 10, 0, None, None, None)
            .await
            .unwrap();
        assert_eq!(all_videos.len(), 2);

        let long_only = list_videos_by_channel(&conn, "UCF", 10, 0, Some(false), None, None)
            .await
            .unwrap();
        assert_eq!(long_only.len(), 1);
        assert_eq!(long_only[0].id, "long_vid");

        let short_only = list_videos_by_channel(&conn, "UCF", 10, 0, Some(true), None, None)
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

        let all = list_videos_by_channel(&conn, "UCQ", 10, 0, None, None, None)
            .await
            .unwrap();
        assert_eq!(all.len(), 2);

        let queued_only = list_videos_by_channel(
            &conn,
            "UCQ",
            10,
            0,
            None,
            None,
            Some(QueueFilter::AnyIncomplete),
        )
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
            quality_score: None,
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
            quality_score: None,
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
                    quality_score: None,
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
                quality_score: None,
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
                quality_score: None,
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
                quality_score: None,
            },
        )
        .await
        .unwrap();

        let snapshot = load_channel_snapshot_data(&conn, &channel.id, 10, 0, None, None, None)
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
                quality_score: None,
            },
        )
        .await
        .unwrap();

        let bootstrap =
            load_workspace_bootstrap_data(&conn, Some(&first.id), 10, 0, None, None, None)
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

    #[tokio::test]
    async fn test_create_highlight_lists_video_highlights_newest_first_without_duplicates() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC_HL_SINGLE".to_string(),
            handle: None,
            name: "Highlights".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let video = Video {
            id: "vid_hl_single".to_string(),
            channel_id: channel.id.clone(),
            title: "Highlighted Video".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        };
        insert_video(&conn, &video).await.unwrap();

        let first = create_highlight(
            &conn,
            "vid_hl_single",
            HighlightSource::Transcript,
            "Important transcript passage",
            "Before",
            "After",
        )
        .await
        .unwrap();

        let duplicate = create_highlight(
            &conn,
            "vid_hl_single",
            HighlightSource::Transcript,
            "Important transcript passage",
            "Before",
            "After",
        )
        .await
        .unwrap();

        let second = create_highlight(
            &conn,
            "vid_hl_single",
            HighlightSource::Summary,
            "Important summary passage",
            "Intro",
            "Outro",
        )
        .await
        .unwrap();

        assert_eq!(duplicate.id, first.id);

        let highlights = list_video_highlights(&conn, "vid_hl_single").await.unwrap();
        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].id, second.id);
        assert_eq!(highlights[0].source, HighlightSource::Summary);
        assert_eq!(highlights[1].id, first.id);
        assert_eq!(highlights[1].source, HighlightSource::Transcript);
    }

    #[tokio::test]
    async fn test_list_highlights_grouped_by_channel_and_video_release_date() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let newest_channel = Channel {
            id: "UC_HL_NEW".to_string(),
            handle: Some("@new".to_string()),
            name: "New Channel".to_string(),
            thumbnail_url: Some("https://img.example.com/new-channel.jpg".to_string()),
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        let older_channel = Channel {
            id: "UC_HL_OLD".to_string(),
            handle: Some("@old".to_string()),
            name: "Old Channel".to_string(),
            thumbnail_url: Some("https://img.example.com/old-channel.jpg".to_string()),
            added_at: Utc::now() - chrono::Duration::days(1),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &newest_channel).await.unwrap();
        insert_channel(&conn, &older_channel).await.unwrap();

        let newest_video = Video {
            id: "vid_hl_newest".to_string(),
            channel_id: newest_channel.id.clone(),
            title: "Newest Video".to_string(),
            thumbnail_url: Some("https://img.example.com/newest.jpg".to_string()),
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        };
        let middle_video = Video {
            id: "vid_hl_middle".to_string(),
            channel_id: older_channel.id.clone(),
            title: "Middle Video".to_string(),
            thumbnail_url: Some("https://img.example.com/middle.jpg".to_string()),
            published_at: Utc::now() - chrono::Duration::days(2),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        };
        let oldest_video = Video {
            id: "vid_hl_oldest".to_string(),
            channel_id: older_channel.id.clone(),
            title: "Oldest Video".to_string(),
            thumbnail_url: Some("https://img.example.com/oldest.jpg".to_string()),
            published_at: Utc::now() - chrono::Duration::days(5),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        };
        insert_video(&conn, &newest_video).await.unwrap();
        insert_video(&conn, &middle_video).await.unwrap();
        insert_video(&conn, &oldest_video).await.unwrap();

        create_highlight(
            &conn,
            "vid_hl_oldest",
            HighlightSource::Transcript,
            "Oldest clip",
            "",
            "",
        )
        .await
        .unwrap();
        create_highlight(
            &conn,
            "vid_hl_middle",
            HighlightSource::Summary,
            "Middle clip",
            "",
            "",
        )
        .await
        .unwrap();
        create_highlight(
            &conn,
            "vid_hl_newest",
            HighlightSource::Transcript,
            "Newest clip",
            "",
            "",
        )
        .await
        .unwrap();

        let grouped = list_highlights_grouped(&conn).await.unwrap();
        assert_eq!(grouped.len(), 2);

        assert_eq!(grouped[0].channel_id, newest_channel.id);
        assert_eq!(grouped[0].channel_name, newest_channel.name);
        assert_eq!(grouped[0].videos.len(), 1);
        assert_eq!(grouped[0].videos[0].video_id, "vid_hl_newest");
        assert_eq!(grouped[0].videos[0].highlights[0].text, "Newest clip");

        assert_eq!(grouped[1].channel_id, older_channel.id);
        assert_eq!(grouped[1].videos.len(), 2);
        assert_eq!(grouped[1].videos[0].video_id, "vid_hl_middle");
        assert_eq!(grouped[1].videos[1].video_id, "vid_hl_oldest");
        assert_eq!(
            grouped[1].videos[0].highlights[0].source,
            HighlightSource::Summary
        );
    }

    #[tokio::test]
    async fn test_delete_highlight_removes_it_from_video_and_grouped_views() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let channel = Channel {
            id: "UC_HL_DELETE".to_string(),
            handle: None,
            name: "Delete".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&conn, &channel).await.unwrap();

        let video = Video {
            id: "vid_hl_delete".to_string(),
            channel_id: channel.id.clone(),
            title: "Delete Highlight".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        };
        insert_video(&conn, &video).await.unwrap();

        let first = create_highlight(
            &conn,
            &video.id,
            HighlightSource::Transcript,
            "First",
            "",
            "",
        )
        .await
        .unwrap();
        let second = create_highlight(&conn, &video.id, HighlightSource::Summary, "Second", "", "")
            .await
            .unwrap();

        assert!(delete_highlight(&conn, first.id).await.unwrap());
        assert!(!delete_highlight(&conn, 999_999).await.unwrap());

        let remaining = list_video_highlights(&conn, &video.id).await.unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, second.id);

        let grouped = list_highlights_grouped(&conn).await.unwrap();
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].videos.len(), 1);
        assert_eq!(grouped[0].videos[0].highlights.len(), 1);
        assert_eq!(grouped[0].videos[0].highlights[0].id, second.id);
    }

    #[tokio::test]
    async fn test_init_db_creates_scalability_indexes_for_video_lists() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        let mut rows = conn
            .query(
                "SELECT name, sql FROM sqlite_master WHERE type = 'index'",
                (),
            )
            .await
            .unwrap();

        let mut indexes = std::collections::HashMap::new();
        while let Some(row) = rows.next().await.unwrap() {
            let name: String = row.get(0).unwrap();
            let sql: Option<String> = row.get(1).unwrap();
            if let Some(sql) = sql {
                indexes.insert(name, sql);
            }
        }

        let ack_short_index = indexes
            .get("idx_videos_channel_ack_short_published")
            .expect("combined acknowledged/short index should exist");
        assert!(
            ack_short_index.contains("(channel_id, acknowledged, is_short, published_at DESC)")
        );

        let transcript_queue_index = indexes
            .get("idx_videos_channel_transcript_queue_published")
            .expect("channel transcript queue partial index should exist");
        assert!(transcript_queue_index.contains("WHERE transcript_status != 'ready'"));

        let summary_queue_index = indexes
            .get("idx_videos_channel_summary_queue_published")
            .expect("channel summary queue partial index should exist");
        assert!(
            summary_queue_index
                .contains("WHERE transcript_status = 'ready' AND summary_status != 'ready'")
        );

        let ready_index = indexes
            .get("idx_videos_channel_ready_published")
            .expect("ready-video partial index should exist");
        assert!(
            ready_index.contains("WHERE transcript_status = 'ready' AND summary_status = 'ready'")
        );

        let highlight_index = indexes
            .get("idx_highlights_video_created")
            .expect("video highlight index should exist");
        assert!(highlight_index.contains("highlights(video_id, created_at DESC"));
    }

    #[tokio::test]
    async fn test_search_source_roundtrip_and_fts_lookup() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        insert_channel(
            &conn,
            &Channel {
                id: "UC_SEARCH".to_string(),
                handle: Some("@search".to_string()),
                name: "Search Channel".to_string(),
                thumbnail_url: None,
                added_at: Utc::now(),
                earliest_sync_date: None,
                earliest_sync_date_user_set: false,
            },
        )
        .await
        .unwrap();
        insert_video(
            &conn,
            &Video {
                id: "vid_search".to_string(),
                channel_id: "UC_SEARCH".to_string(),
                title: "Semantic Search Basics".to_string(),
                thumbnail_url: None,
                published_at: Utc::now(),
                is_short: false,
                transcript_status: ContentStatus::Ready,
                summary_status: ContentStatus::Ready,
                acknowledged: false,
                retry_count: 0,
                quality_score: None,
            },
        )
        .await
        .unwrap();

        mark_search_source_pending(&conn, "vid_search", SearchSourceKind::Summary, "hash-1")
            .await
            .unwrap();
        assert!(
            mark_search_source_indexing(&conn, "vid_search", SearchSourceKind::Summary, "hash-1")
                .await
                .unwrap()
        );

        let vector = vector_to_json(&vec![0.25_f32; 512]);
        let stored = replace_search_chunks(
            &conn,
            "vid_search",
            SearchSourceKind::Summary,
            "hash-1",
            Some("embeddinggemma"),
            &[
                SearchIndexChunk {
                    chunk_index: 0,
                    section_title: Some("Overview".to_string()),
                    chunk_text:
                        "Semantic search combines vector retrieval with exact keyword lookup."
                            .to_string(),
                    embedding_json: Some(vector.clone()),
                    token_count: 10,
                },
                SearchIndexChunk {
                    chunk_index: 1,
                    section_title: Some("Details".to_string()),
                    chunk_text: "Semantic indexing can still surface related results.".to_string(),
                    embedding_json: Some(vector_to_json(&vec![0.75_f32; 512])),
                    token_count: 8,
                },
            ],
        )
        .await
        .unwrap();
        assert!(stored);

        let state = get_search_source_state(&conn, "vid_search", SearchSourceKind::Summary)
            .await
            .unwrap()
            .expect("search source state");
        assert_eq!(state.embedding_model.as_deref(), Some("embeddinggemma"));

        let counts = get_search_source_counts(&conn).await.unwrap();
        assert_eq!(counts.ready, 1);
        assert_eq!(counts.pending, 0);
        assert_eq!(counts.total_sources, 2);

        let matches = search_fts_candidates(
            &conn,
            "\"semantic\"",
            Some("embeddinggemma"),
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert_eq!(matches.len(), 2);
        assert!(matches.iter().all(|item| item.video_id == "vid_search"));
        assert!(
            matches
                .iter()
                .all(|item| item.source_kind == SearchSourceKind::Summary)
        );

        let exact_matches = search_exact_candidates(
            &conn,
            &vector,
            "embeddinggemma",
            &matches
                .iter()
                .map(|candidate| candidate.chunk_id.parse::<i64>().unwrap())
                .collect::<Vec<_>>(),
            10,
        )
        .await
        .unwrap();
        assert_eq!(exact_matches[0].section_title.as_deref(), Some("Overview"));

        let mismatched = search_fts_candidates(
            &conn,
            "\"semantic\"",
            Some("qwen3-embedding:8b"),
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert!(mismatched.is_empty());
    }

    #[tokio::test]
    async fn test_plain_fts_search_does_not_require_embedding_model() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        insert_channel(
            &conn,
            &Channel {
                id: "UC_SEARCH_FTS".to_string(),
                handle: Some("@fts-only".to_string()),
                name: "FTS Only".to_string(),
                thumbnail_url: None,
                added_at: Utc::now(),
                earliest_sync_date: None,
                earliest_sync_date_user_set: false,
            },
        )
        .await
        .unwrap();
        insert_video(
            &conn,
            &Video {
                id: "vid_search_fts".to_string(),
                channel_id: "UC_SEARCH_FTS".to_string(),
                title: "FTS Search".to_string(),
                thumbnail_url: None,
                published_at: Utc::now(),
                is_short: false,
                transcript_status: ContentStatus::Ready,
                summary_status: ContentStatus::Pending,
                acknowledged: false,
                retry_count: 0,
                quality_score: None,
            },
        )
        .await
        .unwrap();

        mark_search_source_pending(
            &conn,
            "vid_search_fts",
            SearchSourceKind::Transcript,
            "hash-fts-1",
        )
        .await
        .unwrap();
        mark_search_source_indexing(
            &conn,
            "vid_search_fts",
            SearchSourceKind::Transcript,
            "hash-fts-1",
        )
        .await
        .unwrap();

        let stored = replace_search_chunks(
            &conn,
            "vid_search_fts",
            SearchSourceKind::Transcript,
            "hash-fts-1",
            None,
            &[SearchIndexChunk {
                chunk_index: 0,
                section_title: Some("Overview".to_string()),
                chunk_text: "Keyword search should work without embeddings.".to_string(),
                embedding_json: None,
                token_count: 7,
            }],
        )
        .await
        .unwrap();
        assert!(stored);

        let state = get_search_source_state(&conn, "vid_search_fts", SearchSourceKind::Transcript)
            .await
            .unwrap()
            .expect("search source state");
        assert_eq!(state.embedding_model, None);

        let matches = search_fts_candidates(&conn, "\"keyword\"", None, None, None, 10)
            .await
            .unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].video_id, "vid_search_fts");
    }

    #[tokio::test]
    async fn test_mark_search_source_pending_preserves_chunks_until_pruned() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        insert_channel(
            &conn,
            &Channel {
                id: "UC_SEARCH_PRUNE".to_string(),
                handle: Some("@prune".to_string()),
                name: "Prune Search".to_string(),
                thumbnail_url: None,
                added_at: Utc::now(),
                earliest_sync_date: None,
                earliest_sync_date_user_set: false,
            },
        )
        .await
        .unwrap();
        insert_video(
            &conn,
            &Video {
                id: "vid_search_prune".to_string(),
                channel_id: "UC_SEARCH_PRUNE".to_string(),
                title: "Prune Search".to_string(),
                thumbnail_url: None,
                published_at: Utc::now(),
                is_short: false,
                transcript_status: ContentStatus::Ready,
                summary_status: ContentStatus::Ready,
                acknowledged: false,
                retry_count: 0,
                quality_score: None,
            },
        )
        .await
        .unwrap();

        mark_search_source_pending(
            &conn,
            "vid_search_prune",
            SearchSourceKind::Summary,
            "hash-prune-1",
        )
        .await
        .unwrap();
        mark_search_source_indexing(
            &conn,
            "vid_search_prune",
            SearchSourceKind::Summary,
            "hash-prune-1",
        )
        .await
        .unwrap();
        replace_search_chunks(
            &conn,
            "vid_search_prune",
            SearchSourceKind::Summary,
            "hash-prune-1",
            Some("embeddinggemma"),
            &[SearchIndexChunk {
                chunk_index: 0,
                section_title: None,
                chunk_text: "semantic prune test".to_string(),
                embedding_json: Some(vector_to_json(&vec![0.5_f32; 512])),
                token_count: 3,
            }],
        )
        .await
        .unwrap();

        mark_search_source_pending(
            &conn,
            "vid_search_prune",
            SearchSourceKind::Summary,
            "hash-prune-2",
        )
        .await
        .unwrap();

        let mut count_rows = conn
            .query(
                "SELECT COUNT(*)
                 FROM search_chunks sc
                 JOIN search_sources ss ON ss.id = sc.search_source_id
                 WHERE ss.video_id = ?1",
                params!["vid_search_prune"],
            )
            .await
            .unwrap();
        let chunk_count = if let Some(row) = count_rows.next().await.unwrap() {
            row.get::<i64>(0).unwrap()
        } else {
            0
        };
        assert_eq!(chunk_count, 1);

        let hidden_matches = search_fts_candidates(
            &conn,
            "\"semantic\"",
            Some("embeddinggemma"),
            None,
            None,
            10,
        )
        .await
        .unwrap();
        assert!(hidden_matches.is_empty());

        let pruned = prune_stale_search_rows(&conn, 10).await.unwrap();
        assert_eq!(pruned, 1);

        let mut remaining_rows = conn
            .query(
                "SELECT COUNT(*)
                 FROM search_chunks sc
                 JOIN search_sources ss ON ss.id = sc.search_source_id
                 WHERE ss.video_id = ?1",
                params!["vid_search_prune"],
            )
            .await
            .unwrap();
        let remaining = if let Some(row) = remaining_rows.next().await.unwrap() {
            row.get::<i64>(0).unwrap()
        } else {
            0
        };
        assert_eq!(remaining, 0);
    }

    #[tokio::test]
    async fn test_list_pending_search_sources_prioritizes_summaries() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        insert_channel(
            &conn,
            &Channel {
                id: "UC_SEARCH_PENDING".to_string(),
                handle: Some("@search-pending".to_string()),
                name: "Search Pending".to_string(),
                thumbnail_url: None,
                added_at: Utc::now(),
                earliest_sync_date: None,
                earliest_sync_date_user_set: false,
            },
        )
        .await
        .unwrap();

        for index in 0..6 {
            let video_id = format!("vid_search_pending_tx_{index}");
            insert_video(
                &conn,
                &Video {
                    id: video_id.clone(),
                    channel_id: "UC_SEARCH_PENDING".to_string(),
                    title: format!("Pending Transcript {index}"),
                    thumbnail_url: None,
                    published_at: Utc::now(),
                    is_short: false,
                    transcript_status: ContentStatus::Ready,
                    summary_status: ContentStatus::Pending,
                    acknowledged: false,
                    retry_count: 0,
                    quality_score: None,
                },
            )
            .await
            .unwrap();
            mark_search_source_pending(
                &conn,
                &video_id,
                SearchSourceKind::Transcript,
                &format!("hash-tx-{index}"),
            )
            .await
            .unwrap();
        }

        for index in 0..4 {
            let video_id = format!("vid_search_pending_summary_{index}");
            insert_video(
                &conn,
                &Video {
                    id: video_id.clone(),
                    channel_id: "UC_SEARCH_PENDING".to_string(),
                    title: format!("Pending Summary {index}"),
                    thumbnail_url: None,
                    published_at: Utc::now(),
                    is_short: false,
                    transcript_status: ContentStatus::Ready,
                    summary_status: ContentStatus::Ready,
                    acknowledged: false,
                    retry_count: 0,
                    quality_score: None,
                },
            )
            .await
            .unwrap();
            mark_search_source_pending(
                &conn,
                &video_id,
                SearchSourceKind::Summary,
                &format!("hash-summary-{index}"),
            )
            .await
            .unwrap();
        }

        let pending = list_pending_search_sources(&conn, 4).await.unwrap();
        let transcript_count = pending
            .iter()
            .filter(|state| state.source_kind == SearchSourceKind::Transcript)
            .count();
        let summary_count = pending
            .iter()
            .filter(|state| state.source_kind == SearchSourceKind::Summary)
            .count();

        assert_eq!(pending.len(), 4);
        assert_eq!(transcript_count, 0);
        assert_eq!(summary_count, 4);
    }

    #[tokio::test]
    async fn test_vector_index_detection_tracks_lifecycle() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        assert!(!has_vector_index(&conn).await.unwrap());

        ensure_vector_index(&conn).await.unwrap();

        assert!(has_vector_index(&conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_run_migrations_is_idempotent_on_existing_schema() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        run_migrations(&conn).await.unwrap();

        let mut rows = conn
            .query(
                "SELECT count(*) FROM sqlite_master WHERE type = 'trigger' AND name IN ('search_chunks_ai', 'search_chunks_ad', 'search_chunks_au')",
                (),
            )
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        let trigger_count: i64 = row.get(0).unwrap();
        assert_eq!(trigger_count, 3);
    }

    #[tokio::test]
    async fn test_list_search_backfill_materials_returns_ready_content_missing_sources() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        insert_channel(
            &conn,
            &Channel {
                id: "UC_SEARCH_BACKFILL".to_string(),
                handle: Some("@search-backfill".to_string()),
                name: "Search Backfill".to_string(),
                thumbnail_url: None,
                added_at: Utc::now(),
                earliest_sync_date: None,
                earliest_sync_date_user_set: false,
            },
        )
        .await
        .unwrap();

        let newest_published_at = Utc::now();
        let older_published_at = newest_published_at - chrono::Duration::days(1);
        let oldest_published_at = newest_published_at - chrono::Duration::days(2);

        insert_video(
            &conn,
            &Video {
                id: "vid_search_transcript_missing".to_string(),
                channel_id: "UC_SEARCH_BACKFILL".to_string(),
                title: "Transcript Missing".to_string(),
                thumbnail_url: None,
                published_at: newest_published_at,
                is_short: false,
                transcript_status: ContentStatus::Ready,
                summary_status: ContentStatus::Pending,
                acknowledged: false,
                retry_count: 0,
                quality_score: None,
            },
        )
        .await
        .unwrap();
        upsert_transcript(
            &conn,
            &Transcript {
                video_id: "vid_search_transcript_missing".to_string(),
                raw_text: Some("alpha beta gamma".to_string()),
                formatted_markdown: None,
                render_mode: TranscriptRenderMode::PlainText,
            },
        )
        .await
        .unwrap();

        insert_video(
            &conn,
            &Video {
                id: "vid_search_summary_missing".to_string(),
                channel_id: "UC_SEARCH_BACKFILL".to_string(),
                title: "Summary Missing".to_string(),
                thumbnail_url: None,
                published_at: older_published_at,
                is_short: false,
                transcript_status: ContentStatus::Ready,
                summary_status: ContentStatus::Ready,
                acknowledged: false,
                retry_count: 0,
                quality_score: None,
            },
        )
        .await
        .unwrap();
        upsert_summary(
            &conn,
            &Summary {
                video_id: "vid_search_summary_missing".to_string(),
                content: "delta epsilon zeta".to_string(),
                model_used: Some("model".to_string()),
                quality_score: None,
                quality_note: None,
                quality_model_used: None,
            },
        )
        .await
        .unwrap();

        insert_video(
            &conn,
            &Video {
                id: "vid_search_ready".to_string(),
                channel_id: "UC_SEARCH_BACKFILL".to_string(),
                title: "Already Indexed".to_string(),
                thumbnail_url: None,
                published_at: oldest_published_at,
                is_short: false,
                transcript_status: ContentStatus::Ready,
                summary_status: ContentStatus::Pending,
                acknowledged: false,
                retry_count: 0,
                quality_score: None,
            },
        )
        .await
        .unwrap();
        upsert_transcript(
            &conn,
            &Transcript {
                video_id: "vid_search_ready".to_string(),
                raw_text: Some("already indexed transcript".to_string()),
                formatted_markdown: None,
                render_mode: TranscriptRenderMode::PlainText,
            },
        )
        .await
        .unwrap();
        let ready_hash = crate::services::search::hash_search_content("already indexed transcript");
        mark_search_source_pending(
            &conn,
            "vid_search_ready",
            SearchSourceKind::Transcript,
            &ready_hash,
        )
        .await
        .unwrap();
        mark_search_source_indexing(
            &conn,
            "vid_search_ready",
            SearchSourceKind::Transcript,
            &ready_hash,
        )
        .await
        .unwrap();
        let stored = replace_search_chunks(
            &conn,
            "vid_search_ready",
            SearchSourceKind::Transcript,
            &ready_hash,
            Some("embeddinggemma"),
            &[SearchIndexChunk {
                chunk_index: 0,
                section_title: None,
                chunk_text: "already indexed transcript".to_string(),
                embedding_json: Some(vector_to_json(&vec![0.5_f32; 512])),
                token_count: 3,
            }],
        )
        .await
        .unwrap();
        assert!(stored);

        let materials = list_search_backfill_materials(&conn, 10).await.unwrap();
        let material_keys = materials
            .into_iter()
            .map(|material| (material.video_id, material.source_kind))
            .collect::<Vec<_>>();

        assert_eq!(material_keys.len(), 2);
        assert!(material_keys.contains(&(
            "vid_search_transcript_missing".to_string(),
            SearchSourceKind::Transcript
        )));
        assert!(material_keys.contains(&(
            "vid_search_summary_missing".to_string(),
            SearchSourceKind::Summary
        )));
        assert!(
            !material_keys
                .contains(&("vid_search_ready".to_string(), SearchSourceKind::Transcript))
        );
    }

    #[tokio::test]
    async fn test_list_search_backfill_materials_prioritizes_summaries() {
        let pool = init_db_memory().await.unwrap();
        let conn = pool.lock().await;

        insert_channel(
            &conn,
            &Channel {
                id: "UC_SEARCH_BALANCE".to_string(),
                handle: Some("@search-balance".to_string()),
                name: "Search Balance".to_string(),
                thumbnail_url: None,
                added_at: Utc::now(),
                earliest_sync_date: None,
                earliest_sync_date_user_set: false,
            },
        )
        .await
        .unwrap();

        for index in 0..6 {
            let video_id = format!("vid_search_balance_tx_{index}");
            insert_video(
                &conn,
                &Video {
                    id: video_id.clone(),
                    channel_id: "UC_SEARCH_BALANCE".to_string(),
                    title: format!("Transcript Balance {index}"),
                    thumbnail_url: None,
                    published_at: Utc::now() - chrono::Duration::minutes(index as i64 + 1),
                    is_short: false,
                    transcript_status: ContentStatus::Ready,
                    summary_status: ContentStatus::Pending,
                    acknowledged: false,
                    retry_count: 0,
                    quality_score: None,
                },
            )
            .await
            .unwrap();
            upsert_transcript(
                &conn,
                &Transcript {
                    video_id,
                    raw_text: Some(format!("transcript balance text {index}")),
                    formatted_markdown: None,
                    render_mode: TranscriptRenderMode::PlainText,
                },
            )
            .await
            .unwrap();
        }

        for index in 0..4 {
            let video_id = format!("vid_search_balance_summary_{index}");
            insert_video(
                &conn,
                &Video {
                    id: video_id.clone(),
                    channel_id: "UC_SEARCH_BALANCE".to_string(),
                    title: format!("Summary Balance {index}"),
                    thumbnail_url: None,
                    published_at: Utc::now() - chrono::Duration::seconds(index as i64),
                    is_short: false,
                    transcript_status: ContentStatus::Ready,
                    summary_status: ContentStatus::Ready,
                    acknowledged: false,
                    retry_count: 0,
                    quality_score: None,
                },
            )
            .await
            .unwrap();
            upsert_summary(
                &conn,
                &Summary {
                    video_id,
                    content: format!("summary balance text {index}"),
                    model_used: Some("model".to_string()),
                    quality_score: None,
                    quality_note: None,
                    quality_model_used: None,
                },
            )
            .await
            .unwrap();
        }

        let materials = list_search_backfill_materials(&conn, 4).await.unwrap();
        let transcript_count = materials
            .iter()
            .filter(|material| material.source_kind == SearchSourceKind::Transcript)
            .count();
        let summary_count = materials
            .iter()
            .filter(|material| material.source_kind == SearchSourceKind::Summary)
            .count();

        assert_eq!(materials.len(), 4);
        assert_eq!(transcript_count, 0);
        assert_eq!(summary_count, 4);
    }
}
