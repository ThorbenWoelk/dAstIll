use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::services::search::SearchSourceKind;

pub const OTHERS_CHANNEL_ID: &str = "__others__";
pub const OTHERS_CHANNEL_NAME: &str = "Others";

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalChannelRecord {
    pub id: String,
    pub handle: Option<String>,
    pub name: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserChannelSubscription {
    pub channel_id: String,
    pub added_at: DateTime<Utc>,
    pub earliest_sync_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub earliest_sync_date_user_set: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    YouTube,
    NewYorkTimes,
    PodcastRss,
    OpenAlex,
    Arxiv,
    SemanticScholar,
    Website,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum SourceBackingKind {
    Feed,
    Query,
    Authenticated,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionContainerKind {
    Series,
    SavedSearch,
    Folder,
    StandaloneTrackedSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum ContentSourceKind {
    YouTubeChannel,
    PodcastSeries,
    PublicationSeries,
    SavedSearch,
    AuthenticatedPublisherSource,
    Website,
    StandaloneTrackedSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum ContentItemKind {
    PodcastEpisode,
    Publication,
    Article,
    Webpage,
    Video,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum ContentPartKind {
    FullText,
    Abstract,
    Transcript,
    ShowNotes,
    Chapters,
    GeneratedSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum MediaAssetKind {
    SourceAudio,
    GeneratedSummaryAudio,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct ProviderIdentity {
    pub provider: ProviderKind,
    pub external_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct SubscriptionContainer {
    pub id: String,
    pub kind: SubscriptionContainerKind,
    pub title: String,
    pub provider: ProviderKind,
    pub backing_kind: SourceBackingKind,
    #[serde(default)]
    pub user_editable: bool,
    #[serde(default)]
    pub source_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct ContentSource {
    pub id: String,
    pub provider: ProviderKind,
    pub source_kind: ContentSourceKind,
    pub container_id: String,
    pub container_kind: SubscriptionContainerKind,
    pub backing_kind: SourceBackingKind,
    pub title: String,
    #[serde(default)]
    #[ts(optional)]
    pub subtitle: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub handle: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub requires_auth: bool,
    #[serde(default)]
    pub public_content_available: bool,
    #[serde(default)]
    pub entitled_content_available: bool,
    #[serde(default)]
    pub external_ids: Vec<ProviderIdentity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct ContentItem {
    pub id: String,
    pub source_id: String,
    pub provider: ProviderKind,
    pub item_kind: ContentItemKind,
    pub title: String,
    #[serde(default)]
    #[ts(optional)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub external_ids: Vec<ProviderIdentity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct ContentPart {
    pub id: String,
    pub source_id: String,
    pub item_id: String,
    pub provider: ProviderKind,
    pub part_kind: ContentPartKind,
    pub status: ContentStatus,
    pub text_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct MediaAsset {
    pub id: String,
    pub source_id: String,
    pub item_id: String,
    pub provider: ProviderKind,
    pub asset_kind: MediaAssetKind,
    pub title: String,
    #[serde(default)]
    #[ts(optional)]
    pub url: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub mime_type: Option<String>,
}

pub fn youtube_series_container(channel: &Channel) -> SubscriptionContainer {
    SubscriptionContainer {
        id: format!("youtube:series:{}", channel.id),
        kind: SubscriptionContainerKind::Series,
        title: channel.name.clone(),
        provider: ProviderKind::YouTube,
        backing_kind: SourceBackingKind::Feed,
        user_editable: false,
        source_ids: vec![channel.id.clone()],
    }
}

pub fn youtube_content_source(channel: &Channel) -> ContentSource {
    let container = youtube_series_container(channel);

    ContentSource {
        id: channel.id.clone(),
        provider: ProviderKind::YouTube,
        source_kind: ContentSourceKind::YouTubeChannel,
        container_id: container.id,
        container_kind: container.kind,
        backing_kind: SourceBackingKind::Feed,
        title: channel.name.clone(),
        subtitle: None,
        handle: channel.handle.clone(),
        thumbnail_url: channel.thumbnail_url.clone(),
        requires_auth: false,
        public_content_available: true,
        entitled_content_available: true,
        external_ids: vec![ProviderIdentity {
            provider: ProviderKind::YouTube,
            external_id: channel.id.clone(),
        }],
    }
}

pub fn youtube_content_item(video: &Video) -> ContentItem {
    ContentItem {
        id: video.id.clone(),
        source_id: video.channel_id.clone(),
        provider: ProviderKind::YouTube,
        item_kind: ContentItemKind::Video,
        title: video.title.clone(),
        thumbnail_url: video.thumbnail_url.clone(),
        published_at: Some(video.published_at),
        external_ids: vec![ProviderIdentity {
            provider: ProviderKind::YouTube,
            external_id: video.id.clone(),
        }],
    }
}

pub fn youtube_content_parts(video: &Video) -> Vec<ContentPart> {
    vec![
        ContentPart {
            id: format!("transcript:{}", video.id),
            source_id: video.channel_id.clone(),
            item_id: video.id.clone(),
            provider: ProviderKind::YouTube,
            part_kind: ContentPartKind::Transcript,
            status: video.transcript_status,
            text_available: video.transcript_status == ContentStatus::Ready,
        },
        ContentPart {
            id: format!("summary:{}", video.id),
            source_id: video.channel_id.clone(),
            item_id: video.id.clone(),
            provider: ProviderKind::YouTube,
            part_kind: ContentPartKind::GeneratedSummary,
            status: video.summary_status,
            text_available: video.summary_status == ContentStatus::Ready,
        },
    ]
}

pub fn infer_provider_kind_for_source_id(source_id: &str) -> ProviderKind {
    if source_id.starts_with("openalex:query:") {
        ProviderKind::OpenAlex
    } else if source_id.starts_with("podcast:rss:") {
        ProviderKind::PodcastRss
    } else if source_id.starts_with("website:") {
        ProviderKind::Website
    } else {
        ProviderKind::YouTube
    }
}

pub fn infer_source_kind_for_source_id(source_id: &str) -> ContentSourceKind {
    if source_id.starts_with("openalex:query:") {
        ContentSourceKind::SavedSearch
    } else if source_id.starts_with("podcast:rss:") {
        ContentSourceKind::PodcastSeries
    } else if source_id.starts_with("website:") {
        ContentSourceKind::Website
    } else {
        ContentSourceKind::YouTubeChannel
    }
}

pub fn infer_item_kind_for_source_kind(source_kind: ContentSourceKind) -> ContentItemKind {
    match source_kind {
        ContentSourceKind::PodcastSeries => ContentItemKind::PodcastEpisode,
        ContentSourceKind::SavedSearch | ContentSourceKind::PublicationSeries => {
            ContentItemKind::Publication
        }
        ContentSourceKind::Website
        | ContentSourceKind::AuthenticatedPublisherSource
        | ContentSourceKind::StandaloneTrackedSource => ContentItemKind::Webpage,
        ContentSourceKind::YouTubeChannel => ContentItemKind::Video,
    }
}

pub fn infer_primary_text_part_kind_for_source_kind(
    source_kind: ContentSourceKind,
) -> ContentPartKind {
    match source_kind {
        ContentSourceKind::PodcastSeries => ContentPartKind::ShowNotes,
        ContentSourceKind::SavedSearch | ContentSourceKind::PublicationSeries => {
            ContentPartKind::Abstract
        }
        ContentSourceKind::Website
        | ContentSourceKind::AuthenticatedPublisherSource
        | ContentSourceKind::StandaloneTrackedSource => ContentPartKind::FullText,
        ContentSourceKind::YouTubeChannel => ContentPartKind::Transcript,
    }
}

pub fn fallback_source_from_channel(channel: &Channel) -> ContentSource {
    match infer_provider_kind_for_source_id(&channel.id) {
        ProviderKind::OpenAlex => ContentSource {
            id: channel.id.clone(),
            provider: ProviderKind::OpenAlex,
            source_kind: ContentSourceKind::SavedSearch,
            container_id: format!("openalex:saved-search:{}", channel.id),
            container_kind: SubscriptionContainerKind::SavedSearch,
            backing_kind: SourceBackingKind::Query,
            title: channel.name.clone(),
            subtitle: channel.handle.clone(),
            handle: channel.handle.clone(),
            thumbnail_url: channel.thumbnail_url.clone(),
            requires_auth: false,
            public_content_available: true,
            entitled_content_available: true,
            external_ids: vec![ProviderIdentity {
                provider: ProviderKind::OpenAlex,
                external_id: channel.id.clone(),
            }],
        },
        ProviderKind::PodcastRss => ContentSource {
            id: channel.id.clone(),
            provider: ProviderKind::PodcastRss,
            source_kind: ContentSourceKind::PodcastSeries,
            container_id: format!("podcast:series:{}", channel.id),
            container_kind: SubscriptionContainerKind::Series,
            backing_kind: SourceBackingKind::Feed,
            title: channel.name.clone(),
            subtitle: channel.handle.clone(),
            handle: channel.handle.clone(),
            thumbnail_url: channel.thumbnail_url.clone(),
            requires_auth: false,
            public_content_available: true,
            entitled_content_available: true,
            external_ids: vec![ProviderIdentity {
                provider: ProviderKind::PodcastRss,
                external_id: channel.id.clone(),
            }],
        },
        ProviderKind::Website => ContentSource {
            id: channel.id.clone(),
            provider: ProviderKind::Website,
            source_kind: ContentSourceKind::Website,
            container_id: "websites".to_string(),
            container_kind: SubscriptionContainerKind::StandaloneTrackedSource,
            backing_kind: SourceBackingKind::Manual,
            title: channel.name.clone(),
            subtitle: channel.handle.clone(),
            handle: channel.handle.clone(),
            thumbnail_url: channel.thumbnail_url.clone(),
            requires_auth: false,
            public_content_available: true,
            entitled_content_available: true,
            external_ids: vec![ProviderIdentity {
                provider: ProviderKind::Website,
                external_id: channel.id.clone(),
            }],
        },
        ProviderKind::YouTube
        | ProviderKind::NewYorkTimes
        | ProviderKind::Arxiv
        | ProviderKind::SemanticScholar
        | ProviderKind::Manual => youtube_content_source(channel),
    }
}

pub fn fallback_container_from_source(source: &ContentSource) -> SubscriptionContainer {
    SubscriptionContainer {
        id: source.container_id.clone(),
        kind: source.container_kind,
        title: source.title.clone(),
        provider: source.provider,
        backing_kind: source.backing_kind,
        user_editable: source.backing_kind == SourceBackingKind::Manual,
        source_ids: vec![source.id.clone()],
    }
}

pub fn content_item_from_video(video: &Video, source: &ContentSource) -> ContentItem {
    ContentItem {
        id: video.id.clone(),
        source_id: source.id.clone(),
        provider: source.provider,
        item_kind: infer_item_kind_for_source_kind(source.source_kind),
        title: video.title.clone(),
        thumbnail_url: video.thumbnail_url.clone(),
        published_at: Some(video.published_at),
        external_ids: vec![ProviderIdentity {
            provider: source.provider,
            external_id: video.id.clone(),
        }],
    }
}

pub fn content_parts_from_video(video: &Video, source: &ContentSource) -> Vec<ContentPart> {
    let primary_kind = infer_primary_text_part_kind_for_source_kind(source.source_kind);
    let primary_part_id_prefix = match primary_kind {
        ContentPartKind::FullText => "full-text",
        ContentPartKind::Abstract => "abstract",
        ContentPartKind::Transcript => "transcript",
        ContentPartKind::ShowNotes => "show-notes",
        ContentPartKind::Chapters => "chapters",
        ContentPartKind::GeneratedSummary => "summary",
    };

    vec![
        ContentPart {
            id: format!("{primary_part_id_prefix}:{}", video.id),
            source_id: source.id.clone(),
            item_id: video.id.clone(),
            provider: source.provider,
            part_kind: primary_kind,
            status: video.transcript_status,
            text_available: video.transcript_status == ContentStatus::Ready,
        },
        ContentPart {
            id: format!("summary:{}", video.id),
            source_id: source.id.clone(),
            item_id: video.id.clone(),
            provider: source.provider,
            part_kind: ContentPartKind::GeneratedSummary,
            status: video.summary_status,
            text_available: video.summary_status == ContentStatus::Ready,
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum ContentStatus {
    Pending,
    Loading,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserVideoMembership {
    pub video_id: String,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserVideoState {
    pub video_id: String,
    pub acknowledged: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
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

/// A time-stamped caption segment from yt-dlp json3 output.
/// Only present on transcripts extracted via the yt-dlp fallback path.
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct TimedSegment {
    /// Start position in the video, in seconds.
    pub start_sec: f32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct Transcript {
    pub video_id: String,
    pub raw_text: Option<String>,
    pub formatted_markdown: Option<String>,
    #[serde(default)]
    pub render_mode: TranscriptRenderMode,
    /// Timed segments from yt-dlp. Present only when the yt-dlp fallback path ran.
    #[serde(default)]
    #[ts(optional)]
    pub timed_text: Option<Vec<TimedSegment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct Summary {
    pub video_id: String,
    pub content: String,
    pub model_used: Option<String>,
    pub quality_score: Option<u8>,
    pub quality_note: Option<String>,
    pub quality_model_used: Option<String>,
}

#[derive(Debug, Clone, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct SummaryEvaluationJob {
    pub video_id: String,
    pub video_title: String,
    pub transcript_text: String,
    pub summary_content: String,
}

#[derive(Debug, Clone, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct SummaryEvaluationResult {
    pub quality_score: u8,
    pub quality_note: Option<String>,
    pub quality_model_used: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct UserPreferences {
    /// Ordered list of channel IDs for the "custom" sort mode.
    #[serde(default)]
    pub channel_order: Vec<String>,
    /// Which sort mode is active: "custom", "alpha", or "newest".
    #[serde(default = "default_channel_sort_mode")]
    pub channel_sort_mode: String,
    /// User-defined exact replacements applied before summary generation.
    #[serde(default)]
    pub vocabulary_replacements: Vec<VocabularyReplacement>,
}

fn default_channel_sort_mode() -> String {
    "custom".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, PartialEq, Eq, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct VocabularyReplacement {
    pub from: String,
    pub to: String,
    #[serde(default = "chrono::Utc::now")]
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum OpenAlexSearchScope {
    GeneralSearch,
    TitleAndAbstract,
}

impl Default for OpenAlexSearchScope {
    fn default() -> Self {
        Self::TitleAndAbstract
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum OpenAlexSort {
    PublicationDateDesc,
    RelevanceScoreDesc,
}

impl Default for OpenAlexSort {
    fn default() -> Self {
        Self::PublicationDateDesc
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct OpenAlexSavedSearchQuery {
    pub natural_language_query: String,
    pub query_text: String,
    #[serde(default)]
    pub from_publication_date: Option<String>,
    #[serde(default)]
    pub to_publication_date: Option<String>,
    #[serde(default)]
    pub work_type: Option<String>,
    #[serde(default)]
    pub open_access_only: Option<bool>,
    #[serde(default)]
    pub search_scope: OpenAlexSearchScope,
    #[serde(default)]
    pub sort: OpenAlexSort,
}

#[derive(Debug, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct OpenAlexPlanRequest {
    pub natural_language_query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct OpenAlexPlanResponse {
    pub query: OpenAlexSavedSearchQuery,
    #[serde(default)]
    pub notes: Vec<String>,
    pub display_label: String,
}

#[derive(Debug, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct AddChannelRequest {
    #[serde(default)]
    pub input: String,
    #[serde(default)]
    pub openalex_query: Option<OpenAlexSavedSearchQuery>,
}

#[derive(Debug, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct AddVideoRequest {
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct AddVideoResponse {
    pub video: Video,
    pub target_channel_id: String,
    pub already_exists: bool,
}

#[derive(Debug, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct UpdateChannelRequest {
    pub earliest_sync_date: Option<DateTime<Utc>>,
    pub earliest_sync_date_user_set: Option<bool>,
}

#[derive(Debug, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct UpdateContentRequest {
    pub content: String,
    #[serde(default)]
    pub render_mode: Option<TranscriptRenderMode>,
}

#[derive(Debug, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct UpdateAcknowledgedRequest {
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum HighlightSource {
    Transcript,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct Highlight {
    pub id: i64,
    pub video_id: String,
    pub source: HighlightSource,
    pub text: String,
    pub prefix_context: String,
    pub suffix_context: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct CreateHighlightRequest {
    pub source: HighlightSource,
    pub text: String,
    #[serde(default)]
    pub prefix_context: String,
    #[serde(default)]
    pub suffix_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct HighlightVideoGroup {
    pub source_id: String,
    pub video_id: String,
    pub item_id: String,
    pub provider: ProviderKind,
    pub item_kind: ContentItemKind,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: DateTime<Utc>,
    pub highlights: Vec<Highlight>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct HighlightChannelGroup {
    pub source_id: String,
    pub channel_id: String,
    pub provider: ProviderKind,
    pub source_kind: ContentSourceKind,
    pub channel_name: String,
    pub channel_thumbnail_url: Option<String>,
    pub videos: Vec<HighlightVideoGroup>,
}

#[derive(Debug, Serialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct CleanTranscriptResponse {
    pub content: String,
    pub preserved_text: bool,
    pub attempts_used: u8,
    pub max_attempts: u8,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum AiStatus {
    Cloud,
    LocalOnly,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct AiHealthPayload {
    pub available: bool,
    pub status: AiStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct SyncDepthPayload {
    pub earliest_sync_date: Option<String>,
    pub earliest_sync_date_user_set: bool,
    pub derived_earliest_ready_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct ChannelSnapshotPayload {
    pub channel_id: String,
    pub source_id: String,
    pub container: SubscriptionContainer,
    pub source: ContentSource,
    pub sync_depth: SyncDepthPayload,
    /// Total videos stored for this channel when cheaply available.
    pub channel_video_count: Option<usize>,
    pub has_more: bool,
    pub next_offset: Option<usize>,
    pub videos: Vec<Video>,
    pub items: Vec<ContentItem>,
    pub parts: Vec<ContentPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct ChannelVideoPagePayload {
    pub source_id: String,
    pub videos: Vec<Video>,
    pub items: Vec<ContentItem>,
    pub parts: Vec<ContentPart>,
    pub has_more: bool,
    pub next_offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct WorkspaceBootstrapPayload {
    pub ai_available: bool,
    pub ai_status: AiStatus,
    pub containers: Vec<SubscriptionContainer>,
    pub sources: Vec<ContentSource>,
    pub channels: Vec<Channel>,
    pub selected_source_id: Option<String>,
    pub selected_channel_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub selected_item_id: Option<String>,
    pub snapshot: Option<ChannelSnapshotPayload>,
    pub search_status: SearchStatusPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct SearchMatchPayload {
    pub source: SearchSourceKind,
    pub section_title: Option<String>,
    pub snippet: String,
    pub score: f32,
    /// Start position in the video for deep-link playback. Only present for
    /// transcripts extracted via yt-dlp (the summarize CLI path has no timestamps).
    #[serde(default)]
    #[ts(optional)]
    pub start_sec: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct SearchVideoResultPayload {
    pub source_id: String,
    pub video_id: String,
    pub item_id: String,
    pub provider: ProviderKind,
    pub source_kind: ContentSourceKind,
    pub item_kind: ContentItemKind,
    pub channel_id: String,
    pub channel_name: String,
    pub video_title: String,
    pub published_at: String,
    pub matches: Vec<SearchMatchPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct SearchResponsePayload {
    pub query: String,
    pub source: String,
    pub results: Vec<SearchVideoResultPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum ChatMessageStatus {
    Completed,
    Streaming,
    Cancelled,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
#[serde(rename_all = "snake_case")]
pub enum ChatTitleStatus {
    Idle,
    Generating,
    Ready,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct ChatSource {
    pub source_id: String,
    pub video_id: String,
    pub item_id: String,
    pub provider: ProviderKind,
    pub content_source_kind: ContentSourceKind,
    pub item_kind: ContentItemKind,
    pub part_kind: ContentPartKind,
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

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct ChatConversationSummary {
    pub id: String,
    pub title: Option<String>,
    pub title_status: ChatTitleStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
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

#[derive(Debug, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct CreateConversationRequest {
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct UpdateConversationRequest {
    pub title: String,
}

/// Anonymous-only chat turn: full conversation state is carried by the client; nothing is written to the store.
#[derive(Debug, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct EphemeralChatMessageRequest {
    pub conversation: ChatConversation,
    pub content: String,
    #[serde(default)]
    pub deep_research: bool,
    #[serde(default)]
    #[ts(optional)]
    pub model: Option<String>,
}

#[derive(Debug, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct ChatModelOption {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "frontend/src/lib/bindings/")]
pub struct ChatClientConfig {
    /// Default cloud model id when the client omits `model` on send.
    pub default_model: String,
    /// Curated Ollama cloud models the client may offer in a selector.
    pub models: Vec<ChatModelOption>,
}
