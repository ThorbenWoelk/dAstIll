use std::sync::Arc;

use tokio::sync::RwLock;

use crate::db::DbPool;
use crate::read_cache::ReadCache;
use crate::search_progress::SearchProgress;
use crate::services::{
    CloudCooldown, SearchService, SummarizerService, SummaryEvaluatorService, TranscriptCooldown,
    TranscriptService, YouTubeQuotaCooldown, YouTubeService,
};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub read_cache: Arc<ReadCache>,
    pub search_auto_create_vector_index: bool,
    pub search_projection_lock: Arc<RwLock<()>>,
    pub search_progress: Arc<SearchProgress>,
    pub youtube: Arc<YouTubeService>,
    pub transcript: Arc<TranscriptService>,
    pub summarizer: Arc<SummarizerService>,
    pub summary_evaluator: Arc<SummaryEvaluatorService>,
    pub search: Arc<SearchService>,
    pub cloud_cooldown: Arc<CloudCooldown>,
    pub youtube_quota_cooldown: Arc<YouTubeQuotaCooldown>,
    pub transcript_cooldown: Arc<TranscriptCooldown>,
}
