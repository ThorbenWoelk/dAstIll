use std::sync::Arc;

use crate::db::DbPool;
use crate::services::{
    SummarizerService, SummaryEvaluatorService, TranscriptService, YouTubeService,
};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub youtube: Arc<YouTubeService>,
    pub transcript: Arc<TranscriptService>,
    pub summarizer: Arc<SummarizerService>,
    pub summary_evaluator: Arc<SummaryEvaluatorService>,
}
