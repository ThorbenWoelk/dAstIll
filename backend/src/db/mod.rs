mod channels;
mod chat;
mod content;
mod helpers;
mod highlights;
mod search;
mod video_info;
mod videos;

/// Maximum number of concurrent S3 operations. Chosen for 1 vCPU / 512 MiB Cloud Run.
pub(crate) const MAX_CONCURRENT_S3_OPS: usize = 12;

pub use channels::*;
pub use chat::*;
pub use content::*;
pub use highlights::*;
pub use search::*;
pub use video_info::*;
pub use videos::*;

use crate::models::{Channel, Video};
use crate::services::search::SearchSourceKind;

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
    pub(crate) data_bucket: String,
    pub(crate) vector_bucket: String,
    pub(crate) vector_index: String,
}

impl Store {
    pub fn connect(&self) -> Store {
        self.clone()
    }

    #[cfg(test)]
    pub async fn for_test() -> Store {
        let config = aws_config::load_from_env().await;
        let s3 = aws_sdk_s3::Client::new(&config);
        let s3v = aws_sdk_s3vectors::Client::new(&config);
        Store {
            s3,
            s3v,
            data_bucket: std::env::var("S3_DATA_BUCKET")
                .unwrap_or_else(|_| "dastill-test".to_string()),
            vector_bucket: std::env::var("S3_VECTOR_BUCKET")
                .unwrap_or_else(|_| "dastill-vectors-test".to_string()),
            vector_index: std::env::var("S3_VECTOR_INDEX")
                .unwrap_or_else(|_| "search-chunks".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChannelSnapshotData {
    pub channel: Channel,
    pub derived_earliest_ready_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Total videos stored for this channel (no type / read / queue filters).
    pub channel_video_count: usize,
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
    data_bucket: String,
    vector_bucket: String,
    vector_index: String,
) -> Result<Store, StoreError> {
    Ok(Store {
        s3,
        s3v,
        data_bucket,
        vector_bucket,
        vector_index,
    })
}

#[cfg(test)]
pub async fn init_store_memory() -> Result<Store, StoreError> {
    Err(StoreError::Other(
        "in-memory store not yet implemented".to_string(),
    ))
}
