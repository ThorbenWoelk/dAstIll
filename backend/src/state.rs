use std::sync::Arc;

use crate::db::DbPool;
use crate::services::{
    CloudCooldown, SummarizerService, SummaryEvaluatorService, TranscriptService, YouTubeQuotaCooldown,
    YouTubeService,
};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub youtube: Arc<YouTubeService>,
    pub transcript: Arc<TranscriptService>,
    pub summarizer: Arc<SummarizerService>,
    pub summary_evaluator: Arc<SummaryEvaluatorService>,
    pub cloud_cooldown: Arc<CloudCooldown>,
    pub youtube_quota_cooldown: Arc<YouTubeQuotaCooldown>,
}
