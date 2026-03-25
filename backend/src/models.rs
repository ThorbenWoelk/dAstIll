use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::services::search::SearchSourceKind;

pub const OTHERS_CHANNEL_ID: &str = "__others__";
pub const OTHERS_CHANNEL_NAME: &str = "Others";

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum ContentStatus {
    Pending,
    Loading,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
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
    #[ts(optional)]
    pub quality_score: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct Transcript {
    pub video_id: String,
    pub raw_text: Option<String>,
    pub formatted_markdown: Option<String>,
    #[serde(default)]
    pub render_mode: TranscriptRenderMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct Summary {
    pub video_id: String,
    pub content: String,
    pub model_used: Option<String>,
    pub quality_score: Option<u8>,
    pub quality_note: Option<String>,
    pub quality_model_used: Option<String>,
}

#[derive(Debug, Clone, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct SummaryEvaluationJob {
    pub video_id: String,
    pub video_title: String,
    pub transcript_text: String,
    pub summary_content: String,
}

#[derive(Debug, Clone, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct SummaryEvaluationResult {
    pub quality_score: u8,
    pub quality_note: Option<String>,
    pub quality_model_used: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct UserPreferences {
    /// Ordered list of channel IDs for the "custom" sort mode.
    #[serde(default)]
    pub channel_order: Vec<String>,
    /// Which sort mode is active: "custom", "alpha", or "newest".
    #[serde(default = "default_channel_sort_mode")]
    pub channel_sort_mode: String,
}

fn default_channel_sort_mode() -> String {
    "custom".to_string()
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct AddChannelRequest {
    pub input: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct AddVideoRequest {
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct AddVideoResponse {
    pub video: Video,
    pub target_channel_id: String,
    pub already_exists: bool,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct UpdateChannelRequest {
    pub earliest_sync_date: Option<DateTime<Utc>>,
    pub earliest_sync_date_user_set: Option<bool>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct UpdateContentRequest {
    pub content: String,
    #[serde(default)]
    pub render_mode: Option<TranscriptRenderMode>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct UpdateAcknowledgedRequest {
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum HighlightSource {
    Transcript,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct Highlight {
    pub id: i64,
    pub video_id: String,
    pub source: HighlightSource,
    pub text: String,
    pub prefix_context: String,
    pub suffix_context: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct CreateHighlightRequest {
    pub source: HighlightSource,
    pub text: String,
    #[serde(default)]
    pub prefix_context: String,
    #[serde(default)]
    pub suffix_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct HighlightVideoGroup {
    pub video_id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: DateTime<Utc>,
    pub highlights: Vec<Highlight>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct HighlightChannelGroup {
    pub channel_id: String,
    pub channel_name: String,
    pub channel_thumbnail_url: Option<String>,
    pub videos: Vec<HighlightVideoGroup>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct CleanTranscriptResponse {
    pub content: String,
    pub preserved_text: bool,
    pub attempts_used: u8,
    pub max_attempts: u8,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum TranscriptRenderMode {
    PlainText,
    Markdown,
}

impl Default for TranscriptRenderMode {
    fn default() -> Self {
        Self::PlainText
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum AiStatus {
    Cloud,
    LocalOnly,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct AiHealthPayload {
    pub available: bool,
    pub status: AiStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct SyncDepthPayload {
    pub earliest_sync_date: Option<String>,
    pub earliest_sync_date_user_set: bool,
    pub derived_earliest_ready_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct ChannelSnapshotPayload {
    pub channel_id: String,
    pub sync_depth: SyncDepthPayload,
    /// Total videos stored for this channel (no type / read / queue filters).
    pub channel_video_count: usize,
    pub videos: Vec<Video>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct WorkspaceBootstrapPayload {
    pub ai_available: bool,
    pub ai_status: AiStatus,
    pub channels: Vec<Channel>,
    pub selected_channel_id: Option<String>,
    pub snapshot: Option<ChannelSnapshotPayload>,
    pub search_status: SearchStatusPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct SearchMatchPayload {
    pub source: SearchSourceKind,
    pub section_title: Option<String>,
    pub snippet: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct SearchVideoResultPayload {
    pub video_id: String,
    pub channel_id: String,
    pub channel_name: String,
    pub video_title: String,
    pub published_at: String,
    pub matches: Vec<SearchMatchPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct SearchResponsePayload {
    pub query: String,
    pub source: String,
    pub results: Vec<SearchVideoResultPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct SearchStatusPayload {
    pub available: bool,
    pub model: String,
    pub dimensions: usize,
    pub pending: usize,
    pub indexing: usize,
    pub ready: usize,
    pub failed: usize,
    pub total_sources: usize,
    pub total_chunk_count: usize,
    pub embedded_chunk_count: usize,
    pub vector_index_ready: bool,
    pub retrieval_mode: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum ChatMessageStatus {
    Completed,
    Streaming,
    Cancelled,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum ChatTitleStatus {
    Idle,
    Generating,
    Ready,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct ChatSource {
    pub video_id: String,
    pub channel_id: String,
    pub channel_name: String,
    pub video_title: String,
    pub source_kind: SearchSourceKind,
    pub section_title: Option<String>,
    pub snippet: String,
    pub score: f32,
    /// Stable id for the indexed transcript/summary chunk (search excerpt).
    #[serde(default)]
    pub chunk_id: String,
    #[serde(default)]
    #[ts(optional)]
    pub retrieval_pass: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct ChatMessage {
    pub id: String,
    pub role: ChatRole,
    pub content: String,
    #[serde(default)]
    pub sources: Vec<ChatSource>,
    pub status: ChatMessageStatus,
    pub created_at: DateTime<Utc>,
    /// Ollama model id used for this assistant turn (final answer), when applicable.
    #[serde(default)]
    #[ts(optional)]
    pub model: Option<String>,
    /// Prompt token count from the streaming API final chunk, when provided.
    #[serde(default)]
    #[ts(optional)]
    pub prompt_tokens: Option<u64>,
    /// Generated token count from the streaming API final chunk, when provided.
    #[serde(default)]
    #[ts(optional)]
    pub completion_tokens: Option<u64>,
    /// Wall time reported by Ollama for the generate call (nanoseconds), when provided.
    #[serde(default)]
    #[ts(optional)]
    pub total_duration_ns: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct ChatConversationSummary {
    pub id: String,
    pub title: Option<String>,
    pub title_status: ChatTitleStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct ChatConversation {
    pub id: String,
    pub title: Option<String>,
    pub title_status: ChatTitleStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
}

impl From<&ChatConversation> for ChatConversationSummary {
    fn from(value: &ChatConversation) -> Self {
        Self {
            id: value.id.clone(),
            title: value.title.clone(),
            title_status: value.title_status,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct CreateConversationRequest {
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct UpdateConversationRequest {
    pub title: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct SendChatMessageRequest {
    pub content: String,
    /// When true, retrieval uses the maximum excerpt budget and multi-query passes so the model can synthesize across much more of the library.
    #[serde(default)]
    pub deep_research: bool,
    /// Ollama cloud model id from [`ChatClientConfig::models`]. When omitted, the server default cloud model is used.
    #[serde(default)]
    #[ts(optional)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct ChatModelOption {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
pub struct ChatClientConfig {
    /// Default cloud model id when the client omits `model` on send.
    pub default_model: String,
    /// Curated Ollama cloud models the client may offer in a selector.
    pub models: Vec<ChatModelOption>,
}
