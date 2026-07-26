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

use super::{
    persist_and_sync_source_profile, should_rollback_channel_after_sync_failure,
    workspace_bootstrap,
};
use crate::{
    db::{
        SourceProfileRecord, Store, get_channel, get_transcript, get_video, insert_channel,
        insert_video, list_search_progress_materials, upsert_transcript,
    },
    handlers::query::WorkspaceBootstrapParams,
    models::{
        Channel, ContentSource, ContentSourceKind, ContentStatus, ProviderIdentity, ProviderKind,
        SourceBackingKind, SubscriptionContainer, SubscriptionContainerKind, Transcript,
        TranscriptRenderMode, Video,
    },
    search::{SearchProgress, SearchService},
    security::{AccessContext, AccessRole, AuthState},
    services::{
        ChatService, CloudCooldown, OllamaCore, OpenAlexService, PodcastFeedService,
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

#[tokio::test]
#[ignore] // requires live object-store backend
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

#[test]
fn failed_sync_rollback_only_targets_empty_newly_created_channels() {
    assert!(should_rollback_channel_after_sync_failure(false, false));
    assert!(!should_rollback_channel_after_sync_failure(true, false));
    assert!(!should_rollback_channel_after_sync_failure(false, true));
    assert!(!should_rollback_channel_after_sync_failure(true, true));
}

#[tokio::test]
async fn failed_sync_rolls_back_empty_newly_created_channel() {
    let store = Store::for_test().await;
    let channel_id = "website:new-empty-rollback".to_string();
    let state = test_app_state(store.clone()).await;
    let profile = SourceProfileRecord {
        source: ContentSource {
            id: channel_id.clone(),
            provider: ProviderKind::Website,
            source_kind: ContentSourceKind::Website,
            container_id: "websites".to_string(),
            container_kind: SubscriptionContainerKind::StandaloneTrackedSource,
            backing_kind: SourceBackingKind::Manual,
            title: "New Empty Site".to_string(),
            subtitle: Some("https://127.0.0.1:9/missing-page".to_string()),
            handle: Some("https://127.0.0.1:9/missing-page".to_string()),
            thumbnail_url: None,
            requires_auth: false,
            public_content_available: true,
            entitled_content_available: true,
            external_ids: vec![ProviderIdentity {
                provider: ProviderKind::Website,
                external_id: channel_id.clone(),
            }],
        },
        container: SubscriptionContainer {
            id: "websites".to_string(),
            kind: SubscriptionContainerKind::StandaloneTrackedSource,
            title: "Websites".to_string(),
            provider: ProviderKind::Website,
            backing_kind: SourceBackingKind::Manual,
            user_editable: true,
            source_ids: vec![channel_id.clone()],
        },
        openalex_query: None,
    };

    let err = persist_and_sync_source_profile(&state, &profile)
        .await
        .expect_err("sync against unreachable page should fail");
    assert_eq!(err.0, axum::http::StatusCode::BAD_GATEWAY);
    assert!(
        get_channel(&store, &channel_id).await.unwrap().is_none(),
        "empty newly created channel should be rolled back after sync failure"
    );
}

#[tokio::test]
async fn failed_sync_does_not_wipe_existing_shared_channel_content() {
    let store = Store::for_test().await;
    let channel_id = "website:shared-existing".to_string();
    let video_id = "website:page:shared-existing".to_string();
    let channel = Channel {
        id: channel_id.clone(),
        handle: Some("https://example.com/existing".to_string()),
        name: "Existing Shared Site".to_string(),
        thumbnail_url: None,
        added_at: Utc::now(),
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    };
    insert_channel(&store, &channel).await.unwrap();
    insert_video(
        &store,
        &Video {
            id: video_id.clone(),
            channel_id: channel_id.clone(),
            title: "Existing page".to_string(),
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
            video_id: video_id.clone(),
            raw_text: Some("keep this shared transcript".to_string()),
            formatted_markdown: None,
            render_mode: TranscriptRenderMode::PlainText,
            timed_text: None,
        },
    )
    .await
    .unwrap();

    let state = test_app_state(store.clone()).await;
    let profile = SourceProfileRecord {
        source: ContentSource {
            id: channel_id.clone(),
            provider: ProviderKind::Website,
            source_kind: ContentSourceKind::Website,
            container_id: "websites".to_string(),
            container_kind: SubscriptionContainerKind::StandaloneTrackedSource,
            backing_kind: SourceBackingKind::Manual,
            title: "Existing Shared Site".to_string(),
            // Force sync_source_profile to fail after persist.
            subtitle: Some("https://127.0.0.1:9/missing-page".to_string()),
            handle: Some("https://127.0.0.1:9/missing-page".to_string()),
            thumbnail_url: None,
            requires_auth: false,
            public_content_available: true,
            entitled_content_available: true,
            external_ids: vec![ProviderIdentity {
                provider: ProviderKind::Website,
                external_id: channel_id.clone(),
            }],
        },
        container: SubscriptionContainer {
            id: "websites".to_string(),
            kind: SubscriptionContainerKind::StandaloneTrackedSource,
            title: "Websites".to_string(),
            provider: ProviderKind::Website,
            backing_kind: SourceBackingKind::Manual,
            user_editable: true,
            source_ids: vec![channel_id.clone()],
        },
        openalex_query: None,
    };

    let err = persist_and_sync_source_profile(&state, &profile)
        .await
        .expect_err("sync against unreachable page should fail");
    assert_eq!(err.0, axum::http::StatusCode::BAD_GATEWAY);

    assert!(
        get_channel(&store, &channel_id)
            .await
            .unwrap()
            .is_some(),
        "existing shared channel must survive failed subscribe sync rollback"
    );
    assert!(
        get_video(&store, &video_id, false)
            .await
            .unwrap()
            .is_some(),
        "existing shared videos must survive failed subscribe sync rollback"
    );
    let transcript = get_transcript(&store, &video_id).await.unwrap().unwrap();
    assert_eq!(
        transcript.raw_text.as_deref(),
        Some("keep this shared transcript")
    );
}
