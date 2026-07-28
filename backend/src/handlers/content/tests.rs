use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, State},
};
use chrono::Utc;
use libsql::params;
use reqwest::Client;
use tokio::sync::RwLock;

use super::{get_summary, update_summary};
use crate::{
    db::Store,
    models::UpdateContentRequest,
    search::{SearchProgress, SearchService},
    security::{AccessContext, AccessRole, AuthState},
    services::{
        ChatService, CloudCooldown, OllamaCore, OpenAlexService, PodcastFeedService,
        SummarizerService, SummaryEvaluatorService, TranscriptCooldown, TranscriptService,
        UserActivity, WebsiteService, YouTubeQuotaCooldown, YouTubeService,
    },
    state::AppState,
};

async fn test_app_state(db: Store) -> AppState {
    let cooldown = Arc::new(CloudCooldown::cloud());
    let security =
        Arc::new(crate::config::SecurityRuntimeConfig::from_env().expect("security config"));

    AppState {
        db,
        read_cache: Arc::new(crate::read_cache::ReadCache::default()),
        security: security.clone(),
        request_rate_limiter: crate::security::rate_limiter(security.as_ref()),
        search_auto_create_vector_index: false,
        search_projection_lock: Arc::new(RwLock::new(())),
        search_progress: Arc::new(SearchProgress::new(
            None,
            crate::search::SEARCH_EMBEDDING_DIMENSIONS,
            false,
        )),
        youtube: Arc::new(YouTubeService::with_client(Client::new())),
        openalex_planner: Arc::new(crate::services::OpenAlexPlannerService::new(
            OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
        )),
        openalex: Arc::new(OpenAlexService::with_client(Client::new())),
        podcast_feed: Arc::new(PodcastFeedService::with_client(Client::new())),
        website: Arc::new(WebsiteService::with_client(Client::new())),
        transcript: Arc::new(TranscriptService::with_path("/usr/bin/false")),
        tts: None,
        summarizer: Arc::new(SummarizerService::new(
            OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
        )),
        summary_evaluator: Arc::new(SummaryEvaluatorService::new(
            OllamaCore::new("://invalid-url", "qwen3.5:397b-cloud")
                .with_cloud_cooldown(cooldown.clone()),
        )),
        search: Arc::new(SearchService::with_config(
            "://invalid-url",
            None,
            crate::search::SEARCH_EMBEDDING_DIMENSIONS,
            false,
        )),
        chat: Arc::new(ChatService::new(
            OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
        )),
        input_guardrails: Arc::new(crate::services::InputGuardrailService::new(
            OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
            Vec::new(),
            Vec::new(),
        )),
        analytics: None,
        active_replies: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        conversation_store_lock: Arc::new(tokio::sync::Mutex::new(())),
        fts: Arc::new(crate::search::FtsIndex::new().await.expect("fts index")),
        anonymous_chat_quota_lock: Arc::new(tokio::sync::Mutex::new(())),
        mobile_auth_handoffs: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        cloud_cooldown: cooldown,
        youtube_quota_cooldown: Arc::new(YouTubeQuotaCooldown::youtube_quota()),
        transcript_cooldown: Arc::new(TranscriptCooldown::transcript()),
        user_activity: Arc::new(UserActivity::from_env()),
    }
}

fn unscoped_access_context() -> AccessContext {
    AccessContext {
        user_id: Some("user-a".to_string()),
        auth_state: AuthState::Authenticated,
        access_role: AccessRole::User,
        allowed_channel_ids: vec!["allowed-channel".to_string()],
        allowed_other_video_ids: Vec::new(),
    }
}

async fn state_with_foreign_video() -> AppState {
    let store = Store::for_test().await;
    let published_at = Utc::now();
    store
        .sql
        .execute(
            r#"INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, retry_count, quality_score)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                "foreign-video",
                "foreign-channel",
                "Foreign video",
                Option::<String>::None,
                published_at.to_rfc3339(),
                0_i64,
                "ready",
                "ready",
                0_i64,
                Option::<i64>::None,
            ],
        )
        .await
        .expect("seed video");

    test_app_state(store).await
}

#[tokio::test]
async fn get_summary_rejects_videos_outside_access_scope() {
    let state = state_with_foreign_video().await;

    let err = match get_summary(
        State(state),
        Extension(unscoped_access_context()),
        Path("foreign-video".to_string()),
    )
    .await
    {
        Ok(_) => panic!("foreign video should be hidden"),
        Err(err) => err,
    };

    assert_eq!(err.0, axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_summary_rejects_videos_outside_access_scope() {
    let state = state_with_foreign_video().await;

    let err = match update_summary(
        State(state),
        Extension(unscoped_access_context()),
        Path("foreign-video".to_string()),
        Json(UpdateContentRequest {
            content: "replacement".to_string(),
            render_mode: None,
        }),
    )
    .await
    {
        Ok(_) => panic!("foreign video should not be writable"),
        Err(err) => err,
    };

    assert_eq!(err.0, axum::http::StatusCode::NOT_FOUND);
}
