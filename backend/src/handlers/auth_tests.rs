use std::{collections::HashMap, sync::Arc};

use axum::{
    Extension, Json,
    body::to_bytes,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use reqwest::Client;

use super::{
    CompleteMobileAuthHandoffPayload, CreateMobileAuthHandoffPayload,
    MobileAuthHandoffStatusPayload, RedeemMobileAuthHandoffPayload, complete_mobile_auth_handoff,
    create_mobile_auth_handoff, redeem_mobile_auth_handoff,
};
use crate::{
    config::SecurityRuntimeConfig,
    db::Store,
    read_cache::ReadCache,
    search_progress::SearchProgress,
    security::{AccessContext, AccessRole, AuthState, RequestRateLimiter},
    services::{
        ChatService, CloudCooldown, FtsIndex, InputGuardrailService, OllamaCore,
        OpenAlexPlannerService, OpenAlexService, PodcastFeedService, SearchService,
        SummarizerService, SummaryEvaluatorService, TranscriptCooldown, TranscriptService,
        UserActivity, WebsiteService, YouTubeQuotaCooldown, YouTubeService,
    },
    state::AppState,
};

fn anonymous_access_context() -> AccessContext {
    AccessContext {
        user_id: None,
        auth_state: AuthState::Anonymous,
        access_role: AccessRole::Anonymous,
        allowed_channel_ids: Vec::new(),
        allowed_other_video_ids: Vec::new(),
    }
}

fn authenticated_access_context(user_id: &str) -> AccessContext {
    AccessContext {
        user_id: Some(user_id.to_string()),
        auth_state: AuthState::Authenticated,
        access_role: AccessRole::User,
        allowed_channel_ids: Vec::new(),
        allowed_other_video_ids: Vec::new(),
    }
}

fn authorization_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(token).expect("valid auth header"),
    );
    headers
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
        crate::services::search::SEARCH_EMBEDDING_DIMENSIONS,
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

async fn json_body<T: serde::de::DeserializeOwned>(response: impl IntoResponse) -> T {
    let response = response.into_response();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should read");
    serde_json::from_slice(&body).expect("payload should deserialize")
}

#[tokio::test]
async fn mobile_auth_handoff_requires_matching_creator_binding_for_redeem() {
    let state = test_app_state().await;
    let created: CreateMobileAuthHandoffPayload = json_body(
        create_mobile_auth_handoff(
            State(state.clone()),
            Extension(anonymous_access_context()),
            authorization_headers("Bearer anon-a"),
        )
        .await
        .expect("handoff should be created"),
    )
    .await;

    complete_mobile_auth_handoff(
        State(state.clone()),
        Extension(authenticated_access_context("google-user")),
        Path(created.handoff_id.clone()),
        Json(CompleteMobileAuthHandoffPayload {
            complete_token: created.complete_token.clone(),
            google_id_token: "google-id".to_string(),
            google_access_token: "google-access".to_string(),
        }),
    )
    .await
    .expect("handoff should complete");

    let wrong_redeem = redeem_mobile_auth_handoff(
        State(state.clone()),
        Extension(anonymous_access_context()),
        authorization_headers("Bearer anon-b"),
        Path(created.handoff_id.clone()),
        Json(RedeemMobileAuthHandoffPayload {
            redeem_token: created.redeem_token.clone(),
        }),
    )
    .await;
    assert!(matches!(
        wrong_redeem,
        Err((StatusCode::NOT_FOUND, message)) if message == "Mobile auth handoff not found"
    ));

    let redeemed: MobileAuthHandoffStatusPayload = json_body(
        redeem_mobile_auth_handoff(
            State(state.clone()),
            Extension(anonymous_access_context()),
            authorization_headers("Bearer anon-a"),
            Path(created.handoff_id.clone()),
            Json(RedeemMobileAuthHandoffPayload {
                redeem_token: created.redeem_token.clone(),
            }),
        )
        .await
        .expect("matching creator should redeem"),
    )
    .await;

    assert_eq!(redeemed.status, "complete");
    assert_eq!(redeemed.google_id_token.as_deref(), Some("google-id"));
    assert_eq!(
        redeemed.google_access_token.as_deref(),
        Some("google-access")
    );

    let second_redeem = redeem_mobile_auth_handoff(
        State(state),
        Extension(anonymous_access_context()),
        authorization_headers("Bearer anon-a"),
        Path(created.handoff_id),
        Json(RedeemMobileAuthHandoffPayload {
            redeem_token: created.redeem_token,
        }),
    )
    .await;
    assert!(matches!(
        second_redeem,
        Err((StatusCode::NOT_FOUND, message)) if message == "Mobile auth handoff not found"
    ));
}

#[tokio::test]
async fn mobile_auth_handoff_redeem_stays_pending_until_browser_completes() {
    let state = test_app_state().await;
    let created: CreateMobileAuthHandoffPayload = json_body(
        create_mobile_auth_handoff(
            State(state.clone()),
            Extension(anonymous_access_context()),
            authorization_headers("Bearer anon-pending"),
        )
        .await
        .expect("handoff should be created"),
    )
    .await;

    let response = redeem_mobile_auth_handoff(
        State(state),
        Extension(anonymous_access_context()),
        authorization_headers("Bearer anon-pending"),
        Path(created.handoff_id),
        Json(RedeemMobileAuthHandoffPayload {
            redeem_token: created.redeem_token,
        }),
    )
    .await
    .expect("pending handoff should return status");
    let response = response.into_response();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should read");
    let payload: MobileAuthHandoffStatusPayload =
        serde_json::from_slice(&body).expect("payload should deserialize");
    assert_eq!(payload.status, "pending");
    assert!(payload.google_id_token.is_none());
    assert!(payload.google_access_token.is_none());
}

#[tokio::test]
async fn mobile_auth_handoff_completion_requires_authenticated_browser_session() {
    let state = test_app_state().await;
    let created: CreateMobileAuthHandoffPayload = json_body(
        create_mobile_auth_handoff(
            State(state.clone()),
            Extension(anonymous_access_context()),
            authorization_headers("Bearer anon-complete"),
        )
        .await
        .expect("handoff should be created"),
    )
    .await;

    let completion = complete_mobile_auth_handoff(
        State(state),
        Extension(anonymous_access_context()),
        Path(created.handoff_id),
        Json(CompleteMobileAuthHandoffPayload {
            complete_token: created.complete_token,
            google_id_token: "google-id".to_string(),
            google_access_token: "google-access".to_string(),
        }),
    )
    .await;

    assert!(matches!(
        completion,
        Err((StatusCode::FORBIDDEN, message))
            if message == "Authenticated handoff completion required"
    ));
}
