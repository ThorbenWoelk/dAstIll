use std::{collections::HashMap, sync::Arc};

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use reqwest::Client;

use super::{
    regenerate_summary, require_authenticated_content_mutation, reset_video, update_summary,
    update_transcript,
};
use crate::config::SecurityRuntimeConfig;
use crate::db::{self, Store};
use crate::models::{Channel, ContentStatus, UpdateContentRequest, Video};
use crate::read_cache::ReadCache;
use crate::search::{FtsIndex, SearchProgress, SearchService};
use crate::security::{AccessContext, AccessRole, AuthState, RequestRateLimiter};
use crate::services::{
    ChatService, CloudCooldown, InputGuardrailService, OllamaCore, OpenAlexPlannerService,
    OpenAlexService, PodcastFeedService, SummarizerService, SummaryEvaluatorService,
    TranscriptCooldown, TranscriptService, UserActivity, WebsiteService, YouTubeQuotaCooldown,
    YouTubeService,
};
use crate::state::AppState;

fn anonymous_access_context() -> AccessContext {
    AccessContext {
        user_id: None,
        auth_state: AuthState::Anonymous,
        access_role: AccessRole::Anonymous,
        allowed_channel_ids: vec!["seeded-channel".to_string()],
        allowed_other_video_ids: Vec::new(),
    }
}

fn authenticated_access_context(user_id: &str) -> AccessContext {
    AccessContext {
        user_id: Some(user_id.to_string()),
        auth_state: AuthState::Authenticated,
        access_role: AccessRole::User,
        allowed_channel_ids: vec!["seeded-channel".to_string()],
        allowed_other_video_ids: Vec::new(),
    }
}

fn test_security_config() -> SecurityRuntimeConfig {
    SecurityRuntimeConfig {
        proxy_token: "proxy".to_string(),
        firebase_project_id: "demo-dastill".to_string(),
        allowed_origins: vec![],
        operator_email_allowlist: vec![],
        default_seeded_channel_id: "seeded-channel".to_string(),
        default_seeded_channel_ids: vec!["seeded-channel".to_string()],
        baseline_rate_limit_per_minute: 600,
        expensive_rate_limit_per_minute: 120,
        anonymous_chat_quota: 30,
    }
}

async fn test_app_state() -> AppState {
    let store = Store::for_test().await;
    let cooldown = Arc::new(CloudCooldown::cloud());
    let security = Arc::new(test_security_config());
    let search = Arc::new(SearchService::with_config(
        "://invalid-url",
        None,
        crate::search::SEARCH_EMBEDDING_DIMENSIONS,
        false,
    ));

    AppState {
        db: store,
        read_cache: Arc::new(ReadCache::default()),
        security: security.clone(),
        request_rate_limiter: Arc::new(RequestRateLimiter::new(security.as_ref())),
        search_auto_create_vector_index: false,
        search_projection_lock: Arc::new(tokio::sync::RwLock::new(())),
        search_progress: Arc::new(SearchProgress::new(
            search.model(),
            search.dimensions(),
            search.semantic_enabled(),
        )),
        fts: Arc::new(FtsIndex::new().await.expect("fts index")),
        youtube: Arc::new(YouTubeService::with_client(Client::new())),
        openalex_planner: Arc::new(OpenAlexPlannerService::new(
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
        search,
        chat: Arc::new(ChatService::new(
            OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
        )),
        input_guardrails: Arc::new(InputGuardrailService::new(
            OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
            Vec::new(),
            Vec::new(),
        )),
        analytics: None,
        active_replies: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        conversation_store_lock: Arc::new(tokio::sync::Mutex::new(())),
        anonymous_chat_quota_lock: Arc::new(tokio::sync::Mutex::new(())),
        mobile_auth_handoffs: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        cloud_cooldown: cooldown,
        youtube_quota_cooldown: Arc::new(YouTubeQuotaCooldown::youtube_quota()),
        transcript_cooldown: Arc::new(TranscriptCooldown::transcript()),
        user_activity: Arc::new(UserActivity::from_env()),
    }
}

async fn seed_shared_video(state: &AppState) -> String {
    let channel = Channel {
        id: "seeded-channel".to_string(),
        handle: None,
        name: "Seeded".to_string(),
        thumbnail_url: None,
        added_at: Utc::now(),
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    };
    db::insert_channel(&state.db, &channel)
        .await
        .expect("insert channel");

    let video = Video {
        id: "seeded-video".to_string(),
        channel_id: channel.id,
        title: "Seeded video".to_string(),
        thumbnail_url: None,
        published_at: Utc::now(),
        is_short: false,
        transcript_status: ContentStatus::Ready,
        summary_status: ContentStatus::Ready,
        acknowledged: false,
        retry_count: 0,
        quality_score: Some(9),
    };
    db::insert_video(&state.db, &video)
        .await
        .expect("insert video");
    db::save_manual_transcript(
        &state.db,
        &video.id,
        "shared transcript",
        crate::models::TranscriptRenderMode::PlainText,
    )
    .await
    .expect("save transcript");
    db::save_manual_summary(&state.db, &video.id, "shared summary", Some("manual"))
        .await
        .expect("save summary");
    video.id
}

#[test]
fn content_mutations_require_authenticated_context() {
    let authenticated = authenticated_access_context("user-a");
    assert_eq!(
        require_authenticated_content_mutation(&authenticated).unwrap(),
        "user-a"
    );

    let anonymous = anonymous_access_context();
    let error = require_authenticated_content_mutation(&anonymous)
        .expect_err("anonymous access should be rejected");
    assert_eq!(error.0, StatusCode::FORBIDDEN);
    assert_eq!(error.1, "Sign-in required");
}

#[tokio::test]
async fn anonymous_reset_cannot_wipe_shared_catalog_content() {
    let state = test_app_state().await;
    let video_id = seed_shared_video(&state).await;

    let error = match reset_video(
        State(state.clone()),
        Extension(anonymous_access_context()),
        Path(video_id.clone()),
    )
    .await
    {
        Ok(_) => panic!("anonymous reset must be rejected"),
        Err(error) => error,
    };
    assert_eq!(error.0, StatusCode::FORBIDDEN);

    let transcript = db::get_transcript(&state.db, &video_id)
        .await
        .expect("load transcript")
        .expect("transcript should remain");
    assert_eq!(transcript.raw_text.as_deref(), Some("shared transcript"));
    let summary = db::get_summary(&state.db, &video_id)
        .await
        .expect("load summary")
        .expect("summary should remain");
    assert_eq!(summary.content, "shared summary");
}

#[tokio::test]
async fn anonymous_manual_edits_cannot_overwrite_shared_catalog_content() {
    let state = test_app_state().await;
    let video_id = seed_shared_video(&state).await;

    let transcript_error = match update_transcript(
        State(state.clone()),
        Extension(anonymous_access_context()),
        Path(video_id.clone()),
        Json(UpdateContentRequest {
            content: "guest vandalism".to_string(),
            render_mode: None,
        }),
    )
    .await
    {
        Ok(_) => panic!("anonymous transcript edit must be rejected"),
        Err(error) => error,
    };
    assert_eq!(transcript_error.0, StatusCode::FORBIDDEN);

    let summary_error = match update_summary(
        State(state.clone()),
        Extension(anonymous_access_context()),
        Path(video_id.clone()),
        Json(UpdateContentRequest {
            content: "guest vandalism".to_string(),
            render_mode: None,
        }),
    )
    .await
    {
        Ok(_) => panic!("anonymous summary edit must be rejected"),
        Err(error) => error,
    };
    assert_eq!(summary_error.0, StatusCode::FORBIDDEN);

    let regenerate_error = match regenerate_summary(
        State(state.clone()),
        Extension(anonymous_access_context()),
        Path(video_id.clone()),
    )
    .await
    {
        Ok(_) => panic!("anonymous regenerate must be rejected"),
        Err(error) => error,
    };
    assert_eq!(regenerate_error.0, StatusCode::FORBIDDEN);

    let transcript = db::get_transcript(&state.db, &video_id)
        .await
        .expect("load transcript")
        .expect("transcript should remain");
    assert_eq!(transcript.raw_text.as_deref(), Some("shared transcript"));
    let summary = db::get_summary(&state.db, &video_id)
        .await
        .expect("load summary")
        .expect("summary should remain");
    assert_eq!(summary.content, "shared summary");
}
