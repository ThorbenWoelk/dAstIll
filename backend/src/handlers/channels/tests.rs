use std::sync::Arc;

use axum::{
    Extension,
    body::to_bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::RwLock;

use super::{
    backfill_channel_videos, refresh_channel_videos, require_authenticated_channel_mutation,
    workspace_bootstrap, BackfillParams,
};
use crate::{
    db::{
        Store, SourceProfileRecord, insert_channel, insert_video, list_search_progress_materials,
        put_source_profile, upsert_transcript,
    },
    handlers::query::WorkspaceBootstrapParams,
    models::{
        Channel, ContentSource, ContentSourceKind, ContentStatus, ProviderKind, SourceBackingKind,
        SubscriptionContainer, SubscriptionContainerKind, Transcript, TranscriptRenderMode, Video,
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

fn anonymous_seeded_access_context(channel_id: &str) -> AccessContext {
    AccessContext {
        user_id: None,
        auth_state: AuthState::Anonymous,
        access_role: AccessRole::Anonymous,
        allowed_channel_ids: vec![channel_id.to_string()],
        allowed_other_video_ids: Vec::new(),
    }
}

fn authenticated_access_context(user_id: &str, channel_id: &str) -> AccessContext {
    AccessContext {
        user_id: Some(user_id.to_string()),
        auth_state: AuthState::Authenticated,
        access_role: AccessRole::User,
        allowed_channel_ids: vec![channel_id.to_string()],
        allowed_other_video_ids: Vec::new(),
    }
}

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
fn channel_mutations_require_authenticated_context() {
    let authenticated = authenticated_access_context("user-a", "seeded-channel");
    assert_eq!(
        require_authenticated_channel_mutation(&authenticated).unwrap(),
        "user-a"
    );

    let anonymous = anonymous_seeded_access_context("seeded-channel");
    let error = require_authenticated_channel_mutation(&anonymous)
        .expect_err("anonymous access should be rejected");
    assert_eq!(error.0, StatusCode::FORBIDDEN);
    assert_eq!(error.1, "Sign-in required");
}

async fn seed_podcast_channel_with_shared_content(state: &AppState) -> (String, String) {
    let channel_id =
        crate::services::podcast_feed::podcast_source_id_for_feed_url(crate::config::DEFAULT_HARD_FORK_FEED_URL);
    let channel = Channel {
        id: channel_id.clone(),
        handle: None,
        name: "Hard Fork".to_string(),
        thumbnail_url: None,
        added_at: Utc::now(),
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    };
    insert_channel(&state.db, &channel).await.expect("insert channel");

    let container = SubscriptionContainer {
        id: format!("podcast:series:{}", channel_id),
        kind: SubscriptionContainerKind::Series,
        title: "Hard Fork".to_string(),
        provider: ProviderKind::PodcastRss,
        backing_kind: SourceBackingKind::Feed,
        user_editable: false,
        source_ids: vec![channel_id.clone()],
    };
    let profile = SourceProfileRecord {
        source: ContentSource {
            id: channel_id.clone(),
            provider: ProviderKind::PodcastRss,
            source_kind: ContentSourceKind::PodcastSeries,
            container_id: container.id.clone(),
            container_kind: SubscriptionContainerKind::Series,
            backing_kind: SourceBackingKind::Feed,
            title: "Hard Fork".to_string(),
            subtitle: Some(crate::config::DEFAULT_HARD_FORK_FEED_URL.to_string()),
            handle: None,
            thumbnail_url: None,
            requires_auth: false,
            public_content_available: true,
            entitled_content_available: false,
            external_ids: Vec::new(),
        },
        container,
        openalex_query: None,
    };
    put_source_profile(&state.db, &profile)
        .await
        .expect("put source profile");

    let video_id = format!("{channel_id}:episode-shared");
    insert_video(
        &state.db,
        &Video {
            id: video_id.clone(),
            channel_id: channel_id.clone(),
            title: "Shared episode".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
            quality_score: Some(9),
        },
    )
    .await
    .expect("insert video");
    crate::db::save_manual_transcript(
        &state.db,
        &video_id,
        "shared asr transcript",
        TranscriptRenderMode::PlainText,
    )
    .await
    .expect("save transcript");
    crate::db::save_manual_summary(&state.db, &video_id, "shared summary", Some("manual"))
        .await
        .expect("save summary");

    (channel_id, video_id)
}

#[tokio::test]
async fn anonymous_refresh_cannot_mutate_shared_podcast_catalog() {
    let store = Store::for_test().await;
    let state = test_app_state(store).await;
    let (channel_id, video_id) = seed_podcast_channel_with_shared_content(&state).await;

    let error = match refresh_channel_videos(
        State(state.clone()),
        Extension(anonymous_seeded_access_context(&channel_id)),
        Path(channel_id.clone()),
    )
    .await
    {
        Ok(_) => panic!("anonymous refresh must be rejected"),
        Err(error) => error,
    };
    assert_eq!(error.0, StatusCode::FORBIDDEN);
    assert_eq!(error.1, "Sign-in required");

    let transcript = crate::db::get_transcript(&state.db, &video_id)
        .await
        .expect("load transcript")
        .expect("transcript should remain");
    assert_eq!(
        transcript.raw_text.as_deref(),
        Some("shared asr transcript")
    );
    let summary = crate::db::get_summary(&state.db, &video_id)
        .await
        .expect("load summary")
        .expect("summary should remain");
    assert_eq!(summary.content, "shared summary");
}

#[tokio::test]
async fn anonymous_backfill_requires_sign_in() {
    let store = Store::for_test().await;
    let state = test_app_state(store).await;
    let (channel_id, _) = seed_podcast_channel_with_shared_content(&state).await;

    let error = match backfill_channel_videos(
        State(state),
        Extension(anonymous_seeded_access_context(&channel_id)),
        Path(channel_id),
        Query(BackfillParams {
            limit: Some(5),
            until: None,
        }),
    )
    .await
    {
        Ok(_) => panic!("anonymous backfill must be rejected"),
        Err(error) => error,
    };
    assert_eq!(error.0, StatusCode::FORBIDDEN);
    assert_eq!(error.1, "Sign-in required");
}
