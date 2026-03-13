use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::services::search::SearchSourceKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub handle: Option<String>,
    pub name: String,
    pub thumbnail_url: Option<String>,
    pub added_at: DateTime<Utc>,
    pub earliest_sync_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub earliest_sync_date_user_set: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentStatus {
    Pending,
    Loading,
    Ready,
    Failed,
}

impl ContentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Loading => "loading",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }

    pub fn from_db_value(s: &str) -> Self {
        match s {
            "loading" => Self::Loading,
            "ready" => Self::Ready,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub id: String,
    pub channel_id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: DateTime<Utc>,
    pub is_short: bool,
    pub transcript_status: ContentStatus,
    pub summary_status: ContentStatus,
    pub acknowledged: bool,
    #[serde(default)]
    pub retry_count: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_score: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub video_id: String,
    pub watch_url: String,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub channel_name: Option<String>,
    pub channel_id: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub duration_iso8601: Option<String>,
    pub duration_seconds: Option<u64>,
    pub view_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub video_id: String,
    pub raw_text: Option<String>,
    pub formatted_markdown: Option<String>,
    #[serde(default)]
    pub render_mode: TranscriptRenderMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub video_id: String,
    pub content: String,
    pub model_used: Option<String>,
    pub quality_score: Option<u8>,
    pub quality_note: Option<String>,
    pub quality_model_used: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SummaryEvaluationJob {
    pub video_id: String,
    pub video_title: String,
    pub transcript_text: String,
    pub summary_content: String,
}

#[derive(Debug, Clone)]
pub struct SummaryEvaluationResult {
    pub quality_score: u8,
    pub quality_note: Option<String>,
    pub quality_model_used: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddChannelRequest {
    pub input: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    pub earliest_sync_date: Option<DateTime<Utc>>,
    pub earliest_sync_date_user_set: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContentRequest {
    pub content: String,
    #[serde(default)]
    pub render_mode: Option<TranscriptRenderMode>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAcknowledgedRequest {
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HighlightSource {
    Transcript,
    Summary,
}

impl HighlightSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transcript => "transcript",
            Self::Summary => "summary",
        }
    }

    pub fn from_db_value(value: &str) -> Self {
        match value {
            "summary" => Self::Summary,
            _ => Self::Transcript,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub id: i64,
    pub video_id: String,
    pub source: HighlightSource,
    pub text: String,
    pub prefix_context: String,
    pub suffix_context: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHighlightRequest {
    pub source: HighlightSource,
    pub text: String,
    #[serde(default)]
    pub prefix_context: String,
    #[serde(default)]
    pub suffix_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightVideoGroup {
    pub video_id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: DateTime<Utc>,
    pub highlights: Vec<Highlight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightChannelGroup {
    pub channel_id: String,
    pub channel_name: String,
    pub channel_thumbnail_url: Option<String>,
    pub videos: Vec<HighlightVideoGroup>,
}

#[derive(Debug, Serialize)]
pub struct CleanTranscriptResponse {
    pub content: String,
    pub preserved_text: bool,
    pub attempts_used: u8,
    pub max_attempts: u8,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptRenderMode {
    PlainText,
    Markdown,
}

impl TranscriptRenderMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PlainText => "plain_text",
            Self::Markdown => "markdown",
        }
    }

    pub fn from_db_value(value: &str) -> Self {
        match value {
            "markdown" => Self::Markdown,
            _ => Self::PlainText,
        }
    }
}

impl Default for TranscriptRenderMode {
    fn default() -> Self {
        Self::PlainText
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiStatus {
    Cloud,
    LocalOnly,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiHealthPayload {
    pub available: bool,
    pub status: AiStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDepthPayload {
    pub earliest_sync_date: Option<String>,
    pub earliest_sync_date_user_set: bool,
    pub derived_earliest_ready_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSnapshotPayload {
    pub channel_id: String,
    pub sync_depth: SyncDepthPayload,
    pub videos: Vec<Video>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBootstrapPayload {
    pub ai_available: bool,
    pub ai_status: AiStatus,
    pub channels: Vec<Channel>,
    pub selected_channel_id: Option<String>,
    pub snapshot: Option<ChannelSnapshotPayload>,
    pub search_status: SearchStatusPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatchPayload {
    pub source: SearchSourceKind,
    pub section_title: Option<String>,
    pub snippet: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchVideoResultPayload {
    pub video_id: String,
    pub channel_id: String,
    pub channel_name: String,
    pub video_title: String,
    pub published_at: String,
    pub matches: Vec<SearchMatchPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponsePayload {
    pub query: String,
    pub source: String,
    pub results: Vec<SearchVideoResultPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStatusPayload {
    pub available: bool,
    pub model: String,
    pub dimensions: usize,
    pub pending: usize,
    pub indexing: usize,
    pub ready: usize,
    pub failed: usize,
    pub total_sources: usize,
    pub vector_index_ready: bool,
    pub retrieval_mode: String,
}
