use std::sync::Arc;

use axum::{
    Extension,
    body::to_bytes,
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::Utc;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::RwLock;

use super::workspace_bootstrap;
use crate::{
    db::{Store, insert_channel, insert_video, list_search_progress_materials, upsert_transcript},
    handlers::query::WorkspaceBootstrapParams,
    models::{Channel, ContentStatus, Transcript, TranscriptRenderMode, Video},
    search_progress::SearchProgress,
    security::{AccessContext, AccessRole, AuthState},
    services::{
        ChatService, CloudCooldown, OllamaCore, OpenAlexService, PodcastFeedService, SearchService,
        SummarizerService, SummaryEvaluatorService, TranscriptCooldown, TranscriptService,
        UserActivity, WebsiteService, YouTubeQuotaCooldown, YouTubeService,
    },
    state::AppState,
};

async fn test_app_state(db: crate::db::Store) -> AppState {
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
            crate::services::search::SEARCH_EMBEDDING_DIMENSIONS,
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
            crate::services::search::SEARCH_EMBEDDING_DIMENSIONS,
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
        fts: Arc::new(crate::services::FtsIndex::new().await.expect("fts index")),
        anonymous_chat_quota_lock: Arc::new(tokio::sync::Mutex::new(())),
        mobile_auth_handoffs: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        cloud_cooldown: cooldown,
        youtube_quota_cooldown: Arc::new(YouTubeQuotaCooldown::youtube_quota()),
        transcript_cooldown: Arc::new(TranscriptCooldown::transcript()),
        user_activity: Arc::new(UserActivity::from_env()),
    }
}

#[tokio::test]
#[ignore] // requires live S3 backend
async fn workspace_bootstrap_includes_search_status_for_initial_render() {
    let store = Store::for_test().await;
    let channel = Channel {
        id: "UC_BOOT_SEARCH".to_string(),
        handle: None,
        name: "Bootstrap Search".to_string(),
        thumbnail_url: None,
        added_at: Utc::now(),
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    };
    insert_channel(&store, &channel).await.unwrap();
    insert_video(
        &store,
        &Video {
            id: "vid_boot_search".to_string(),
            channel_id: channel.id.clone(),
            title: "Ready transcript".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Pending,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        },
    )
    .await
    .unwrap();
    upsert_transcript(
        &store,
        &Transcript {
            video_id: "vid_boot_search".to_string(),
            raw_text: Some("bootstrap transcript content".to_string()),
            formatted_markdown: None,
            render_mode: TranscriptRenderMode::PlainText,
            timed_text: None,
        },
    )
    .await
    .unwrap();

    let state = test_app_state(store.clone()).await;
    let materials = list_search_progress_materials(&store).await.unwrap();
    state
        .search_progress
        .initialize_from_materials(&materials, false, false)
        .await;

    let response = workspace_bootstrap(
        State(state),
        Extension(AccessContext {
            user_id: None,
            auth_state: AuthState::Anonymous,
            access_role: AccessRole::Anonymous,
            allowed_channel_ids: vec![channel.id.clone()],
            allowed_other_video_ids: Vec::new(),
        }),
        Query(WorkspaceBootstrapParams::default()),
    )
    .await
    .unwrap()
    .into_response();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(payload["channels"].as_array().unwrap().len(), 1);
    assert_eq!(payload["search_status"]["total_sources"].as_u64(), Some(1));
    assert_eq!(payload["search_status"]["ready"].as_u64(), Some(0));
}
