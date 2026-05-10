mod channels;
mod content;
mod conversations;
mod helpers;
mod highlights;
mod library;
mod libsql_snapshot;
mod local_libsql;
mod media_assets;
mod preferences;
mod search;
mod source_profiles;
pub mod sql_schema;
pub(crate) mod sql_videos;
mod stats;
mod tts_stats;
mod user_scope;
mod video_info;
mod videos;

pub(crate) use crate::read_cache::ReadCache;

/// Maximum number of concurrent S3 operations. Chosen for 1 vCPU / 512 MiB Cloud Run.
pub(crate) const MAX_CONCURRENT_S3_OPS: usize = 12;

pub use channels::*;
pub use content::*;
pub use conversations::*;
pub use highlights::*;
pub use library::*;
pub use libsql_snapshot::*;
pub use local_libsql::*;
pub use media_assets::*;
pub use preferences::*;
pub use search::*;
pub use source_profiles::*;
pub use sql_videos::*;
pub use stats::*;
pub use tts_stats::*;
pub use user_scope::*;
pub use video_info::*;
pub use videos::*;

use crate::models::{Channel, Video};
use crate::search::SearchSourceKind;
use aws_sdk_s3::error::SdkError;
use aws_smithy_types::error::metadata::ProvideErrorMetadata;

#[derive(Debug)]
pub enum StoreError {
    S3(String),
    S3Vectors(String),
    Serialization(String),
    NotFound(String),
    Other(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S3(msg) => write!(f, "S3 error: {msg}"),
            Self::S3Vectors(msg) => write!(f, "S3 Vectors error: {msg}"),
            Self::Serialization(msg) => write!(f, "serialization error: {msg}"),
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<serde_json::Error> for StoreError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

#[derive(Clone)]
pub struct Store {
    pub(crate) s3: aws_sdk_s3::Client,
    pub(crate) s3v: aws_sdk_s3vectors::Client,
    pub(crate) sql: libsql::Connection,
    pub(crate) data_bucket: String,
    pub(crate) vector_bucket: String,
    pub(crate) vector_index: String,
    pub(crate) read_cache: ReadCache,
    pub(crate) source_generation_tracker: Option<LibsqlSourceGenerationTracker>,
    pub(crate) snapshot_publisher: Option<LibsqlSnapshotPublisher>,
}

impl Store {
    pub fn connect(&self) -> Store {
        self.clone()
    }

    pub(crate) fn schedule_libsql_snapshot_publish(&self) {
        if let Some(publisher) = &self.snapshot_publisher {
            publisher.schedule();
        }
    }

    pub(crate) async fn record_libsql_snapshot_delta(
        &self,
        operations: Vec<LibsqlSnapshotDeltaOperation>,
    ) -> Result<(), StoreError> {
        if operations.is_empty() {
            return Ok(());
        }
        if let Some(tracker) = &self.source_generation_tracker {
            tracker.append_delta(operations).await?;
        }
        self.schedule_libsql_snapshot_publish();
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SqlCacheReconcileReport {
    pub bootstrapped_videos: usize,
    pub exported_videos: usize,
    pub bootstrapped_preferences: usize,
    pub exported_preferences: usize,
    pub bootstrapped_tts_stats: bool,
    pub exported_tts_stats: bool,
}

pub async fn reconcile_sql_cache_with_store(
    store: &Store,
) -> Result<SqlCacheReconcileReport, StoreError> {
    let mut report = SqlCacheReconcileReport::default();

    let sql_videos = sql_video_count(store).await?;
    if sql_videos == 0 {
        report.bootstrapped_videos = bootstrap_sql_videos_from_store(store).await?;
    } else if snapshot_video_count(store).await? == 0 {
        report.exported_videos = export_sql_videos_to_store(store).await?;
    }

    let sql_preferences = sql_preferences_count(store).await?;
    if sql_preferences == 0 {
        report.bootstrapped_preferences = bootstrap_sql_preferences_from_store(store).await?;
    } else if snapshot_preferences_count(store).await? == 0 {
        report.exported_preferences = export_sql_preferences_to_store(store).await?;
    }

    let sql_tts_stats = has_sql_tts_stats(store).await?;
    if !sql_tts_stats {
        report.bootstrapped_tts_stats = bootstrap_sql_tts_stats_from_store(store).await?;
    } else if !has_snapshot_tts_stats(store).await? {
        report.exported_tts_stats = export_sql_tts_stats_to_store(store).await?;
    }

    Ok(report)
}

#[derive(Debug, Clone)]
pub struct ChannelSnapshotData {
    pub channel: Channel,
    pub derived_earliest_ready_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Total videos stored for this channel when cheaply available.
    pub channel_video_count: Option<usize>,
    pub has_more: bool,
    pub next_offset: Option<usize>,
    pub videos: Vec<Video>,
}

#[derive(Debug, Clone)]
pub struct ChannelVideoPageData {
    pub videos: Vec<Video>,
    pub has_more: bool,
    pub next_offset: Option<usize>,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct SearchSourceRecord {
    pub id: i64,
    pub source_generation: i64,
    pub video_id: String,
    pub source_kind: String,
    pub content_hash: String,
    pub embedding_model: Option<String>,
    pub index_status: String,
    pub last_indexed_at: Option<String>,
    pub last_error: Option<String>,
}

impl From<SearchSourceRecord> for SearchSourceState {
    fn from(r: SearchSourceRecord) -> Self {
        Self {
            id: r.id,
            source_generation: r.source_generation,
            video_id: r.video_id,
            source_kind: SearchSourceKind::from_db_value(&r.source_kind),
            content_hash: r.content_hash,
            embedding_model: r.embedding_model,
            index_status: r.index_status,
            last_indexed_at: r.last_indexed_at,
            last_error: r.last_error,
        }
    }
}

impl From<&SearchSourceState> for SearchSourceRecord {
    fn from(s: &SearchSourceState) -> Self {
        Self {
            id: s.id,
            source_generation: s.source_generation,
            video_id: s.video_id.clone(),
            source_kind: s.source_kind.as_str().to_string(),
            content_hash: s.content_hash.clone(),
            embedding_model: s.embedding_model.clone(),
            index_status: s.index_status.clone(),
            last_indexed_at: s.last_indexed_at.clone(),
            last_error: s.last_error.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchMaterial {
    pub video_id: String,
    pub channel_id: String,
    pub channel_name: String,
    pub video_title: String,
    pub published_at: String,
    pub source_kind: SearchSourceKind,
    pub content: String,
    /// Timed caption segments. Present only for transcripts extracted via yt-dlp.
    pub timed_segments: Option<Vec<crate::models::TimedSegment>>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoInsertOutcome {
    Inserted,
    Existing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueFilter {
    AnyIncomplete,
    TranscriptsOnly,
    SummariesOnly,
    EvaluationsOnly,
}

pub async fn init_store(
    s3: aws_sdk_s3::Client,
    s3v: aws_sdk_s3vectors::Client,
    sql: libsql::Connection,
    data_bucket: String,
    vector_bucket: String,
    vector_index: String,
    read_cache: ReadCache,
    source_generation_tracker: Option<LibsqlSourceGenerationTracker>,
    snapshot_publisher: Option<LibsqlSnapshotPublisher>,
) -> Result<Store, StoreError> {
    Ok(Store {
        s3,
        s3v,
        sql,
        data_bucket,
        vector_bucket,
        vector_index,
        read_cache,
        source_generation_tracker,
        snapshot_publisher,
    })
}

pub(crate) fn format_aws_error<E, R>(err: &SdkError<E, R>) -> String
where
    E: ProvideErrorMetadata + std::fmt::Display,
{
    use aws_sdk_s3::operation::RequestId;
    match err {
        SdkError::ServiceError(context) => {
            let meta = context.err().meta();
            let code = meta.code().unwrap_or("unknown_code");
            let message = meta.message().unwrap_or("service error");
            let request_id = meta.request_id().unwrap_or("unknown_request_id");
            format!("{code}: {message} (Request ID: {request_id})")
        }
        _ => format!("{err:#}"),
    }
}

#[cfg(test)]
mod db_tests;
