mod chat_heuristics;
mod chat_prompt;
mod chat_ranking;

pub mod chat;
pub mod databricks;
pub mod fts;
pub mod fusion;
pub mod http;
pub mod ollama;
pub mod search;
pub mod summarizer;
pub mod summary_evaluator;
pub mod text;
pub mod tts;
pub mod transcript;
pub mod youtube;

pub use chat::{ActiveChatHandle, ChatService, SpawnReplyJob};
pub use databricks::DatabricksSqlService;
pub use fts::{FtsChunk, FtsIndex};
pub use http::{
    CloudCooldown, Cooldown, TranscriptCooldown, YouTubeQuotaCooldown, build_http_client,
    is_cloud_model, is_rate_limited,
};
pub use ollama::{OllamaCore, OllamaPromptError};
pub use search::SearchService;
pub use search::SearchSourceKind;
pub use summarizer::SummarizerService;
pub use summary_evaluator::SummaryEvaluatorService;
pub use tts::ElevenLabsTtsService;
pub use transcript::TranscriptService;
pub use youtube::YouTubeService;
