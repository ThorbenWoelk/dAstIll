mod chat_heuristics;
mod chat_prompt;
mod chat_ranking;

pub mod chat;
pub mod databricks;
pub mod fts;
pub mod fusion;
pub mod http;
pub mod input_guardrails;
pub mod ollama;
pub mod openalex;
pub mod openalex_planner;
pub mod podcast_feed;
pub mod providers;
pub mod search;
pub mod source_sync;
pub mod summarizer;
pub mod summary_evaluator;
pub mod text;
pub mod transcript;
pub mod tts;
pub mod website;
pub mod youtube;

pub use chat::{ActiveChatHandle, ChatService, ReplyWorkflowRequest};
pub use databricks::DatabricksSqlService;
pub use fts::{FtsChunk, FtsIndex};
pub use http::{
    CloudCooldown, Cooldown, TranscriptCooldown, UserActivity, YouTubeQuotaCooldown,
    build_http_client, is_cloud_model, is_rate_limited,
};
pub use input_guardrails::{CHAT_INPUT_BLOCK_MESSAGE, InputGuardrailService};
pub use ollama::{OllamaCore, OllamaPromptError};
pub use openalex::{OpenAlexPublicationMaterial, OpenAlexService};
pub use openalex_planner::OpenAlexPlannerService;
pub use podcast_feed::{PodcastEpisodeMaterial, PodcastFeedService};
pub use providers::{
    FeedSourceAdapter, ManualWebsiteAdapter, ManualWebsiteAdapterContract, ProviderAdapterError,
    QuerySourceAdapter, ResolvedSourceDraft, SyncedSourceBatch,
};
pub use search::SearchService;
pub use search::SearchSourceKind;
pub use source_sync::{persist_source_profile_and_channel, sync_source_profile};
pub use summarizer::SummarizerService;
pub use summary_evaluator::SummaryEvaluatorService;
pub use transcript::TranscriptService;
pub use tts::PollyTtsService;
pub use website::{WebsitePageMaterial, WebsiteService};
pub use youtube::{DataApiKeyValidation, YouTubeService};
