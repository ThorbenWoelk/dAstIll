use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};

use crate::config::SecurityRuntimeConfig;
use crate::db::Store;
use crate::read_cache::ReadCache;
use crate::search_progress::SearchProgress;
use crate::security::RequestRateLimiter;
use crate::services::{
    ActiveChatHandle, ChatService, CloudCooldown, DatabricksSqlService, SearchService,
    SummarizerService, SummaryEvaluatorService, TranscriptCooldown, TranscriptService,
    YouTubeQuotaCooldown, YouTubeService,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Store,
    pub read_cache: Arc<ReadCache>,
    pub security: Arc<SecurityRuntimeConfig>,
    pub request_rate_limiter: Arc<RequestRateLimiter>,
    pub search_auto_create_vector_index: bool,
    pub search_projection_lock: Arc<RwLock<()>>,
    pub search_progress: Arc<SearchProgress>,
    pub youtube: Arc<YouTubeService>,
    pub transcript: Arc<TranscriptService>,
    pub summarizer: Arc<SummarizerService>,
    pub summary_evaluator: Arc<SummaryEvaluatorService>,
    pub search: Arc<SearchService>,
    pub chat: Arc<ChatService>,
    pub analytics: Option<Arc<DatabricksSqlService>>,
    pub active_chats: Arc<Mutex<HashMap<String, ActiveChatHandle>>>,
    pub chat_store_lock: Arc<Mutex<()>>,
    pub anonymous_chat_quota_lock: Arc<Mutex<()>>,
    pub cloud_cooldown: Arc<CloudCooldown>,
    pub youtube_quota_cooldown: Arc<YouTubeQuotaCooldown>,
    pub transcript_cooldown: Arc<TranscriptCooldown>,
}
