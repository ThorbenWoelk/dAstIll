use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{Mutex, RwLock};

use crate::config::SecurityRuntimeConfig;
use crate::db::Store;
use crate::read_cache::ReadCache;
use crate::search::{FtsIndex, SearchProgress, SearchService};
use crate::security::RequestRateLimiter;
use crate::services::PollyTtsService;
use crate::services::{
    ActiveChatHandle, ChatService, CloudCooldown, DatabricksSqlService, InputGuardrailService,
    OpenAlexPlannerService, OpenAlexService, PodcastFeedService, SummarizerService,
    SummaryEvaluatorService, TranscriptCooldown, TranscriptService, UserActivity, WebsiteService,
    YouTubeQuotaCooldown, YouTubeService,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActiveChatKey {
    pub scope_key: String,
    pub conversation_id: String,
}

impl ActiveChatKey {
    pub fn new(scope_key: impl Into<String>, conversation_id: impl Into<String>) -> Self {
        Self {
            scope_key: scope_key.into(),
            conversation_id: conversation_id.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MobileAuthHandoff {
    pub created_at: Instant,
    pub creator_binding_hash: String,
    pub complete_token_hash: String,
    pub redeem_token_hash: String,
    pub google_id_token: Option<String>,
    pub google_access_token: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Store,
    pub read_cache: Arc<ReadCache>,
    pub security: Arc<SecurityRuntimeConfig>,
    pub request_rate_limiter: Arc<RequestRateLimiter>,
    pub search_auto_create_vector_index: bool,
    pub search_projection_lock: Arc<RwLock<()>>,
    pub search_progress: Arc<SearchProgress>,
    pub fts: Arc<FtsIndex>,
    pub youtube: Arc<YouTubeService>,
    pub openalex_planner: Arc<OpenAlexPlannerService>,
    pub openalex: Arc<OpenAlexService>,
    pub podcast_feed: Arc<PodcastFeedService>,
    pub website: Arc<WebsiteService>,
    pub transcript: Arc<TranscriptService>,
    pub tts: Option<Arc<PollyTtsService>>,
    pub summarizer: Arc<SummarizerService>,
    pub summary_evaluator: Arc<SummaryEvaluatorService>,
    pub search: Arc<SearchService>,
    pub chat: Arc<ChatService>,
    pub input_guardrails: Arc<InputGuardrailService>,
    pub analytics: Option<Arc<DatabricksSqlService>>,
    pub active_replies: Arc<Mutex<HashMap<ActiveChatKey, ActiveChatHandle>>>,
    pub conversation_store_lock: Arc<Mutex<()>>,
    pub anonymous_chat_quota_lock: Arc<Mutex<()>>,
    pub mobile_auth_handoffs: Arc<Mutex<HashMap<String, MobileAuthHandoff>>>,
    pub cloud_cooldown: Arc<CloudCooldown>,
    pub youtube_quota_cooldown: Arc<YouTubeQuotaCooldown>,
    pub transcript_cooldown: Arc<TranscriptCooldown>,
    pub user_activity: Arc<UserActivity>,
}
