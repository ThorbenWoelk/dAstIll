use std::sync::Arc;

use axum::{
    Extension, Json,
    body::to_bytes,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use reqwest::Client;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;

use super::{should_roll_back_new_channel, workspace_bootstrap};
use crate::{
    db::{
        Store, get_canonical_channel, get_summary, get_video, insert_channel, insert_video,
        list_search_progress_materials, upsert_summary, upsert_transcript,
    },
    handlers::query::WorkspaceBootstrapParams,
    models::{
        AddChannelRequest, Channel, ContentStatus, Summary, Transcript, TranscriptRenderMode, Video,
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
        catch_up: None,
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
fn failed_sync_rolls_back_only_a_new_channel() {
    assert!(should_roll_back_new_channel(false));
    assert!(!should_roll_back_new_channel(true));
}

const FEED_BODY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Example Podcast</title>
    <link>https://example.com/podcast</link>
    <description>Weekly deep dives</description>
    <item>
      <title>Episode 1</title>
      <guid>episode-1</guid>
      <pubDate>Tue, 07 Jan 2025 10:00:00 GMT</pubDate>
      <description>Episode 1 show notes</description>
    </item>
  </channel>
</rss>"#;

/// Serves one successful feed response, then fails later fetches.
async fn feed_that_fails_on_the_second_fetch() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let mut served = 0usize;
        while served < 2 {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            served += 1;
            let mut buf = [0u8; 2048];
            let _ = socket.read(&mut buf).await;
            let (status, body) = if served == 1 {
                ("200 OK", FEED_BODY)
            } else {
                ("500 Internal Server Error", "")
            };
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\nContent-Type: application/rss+xml\r\n\r\n{body}",
                body.len()
            );
            let _ = socket.write_all(response.as_bytes()).await;
        }
    });
    format!("http://127.0.0.1:{port}/feed.xml")
}

fn signed_in() -> AccessContext {
    AccessContext {
        user_id: Some("reader".to_string()),
        auth_state: AuthState::Authenticated,
        access_role: AccessRole::User,
        allowed_channel_ids: Vec::new(),
        allowed_other_video_ids: Vec::new(),
    }
}

#[tokio::test]
async fn failed_resubscribe_keeps_stored_summaries() {
    let store = Store::for_test().await;
    let feed_url = feed_that_fails_on_the_second_fetch().await;
    let channel_id = crate::services::podcast_feed::podcast_source_id_for_feed_url(&feed_url);
    let channel = Channel {
        id: channel_id.clone(),
        handle: Some(feed_url.clone()),
        name: "Example Podcast".to_string(),
        thumbnail_url: None,
        added_at: Utc::now(),
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    };
    insert_channel(&store, &channel).await.unwrap();
    insert_video(
        &store,
        &Video {
            id: "kept-episode".to_string(),
            channel_id: channel_id.clone(),
            title: "Kept episode".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
            quality_score: Some(8),
        },
    )
    .await
    .unwrap();
    upsert_summary(
        &store,
        &Summary {
            video_id: "kept-episode".to_string(),
            content: "Keep this summary.".to_string(),
            model_used: None,
            quality_score: Some(8),
            quality_note: None,
            quality_model_used: None,
            summary_tags: Vec::new(),
            summary_tags_evaluated: true,
        },
    )
    .await
    .unwrap();

    let state = test_app_state(store.clone()).await;
    let error = match super::add_channel(
        State(state),
        Extension(signed_in()),
        Json(AddChannelRequest {
            input: feed_url,
            openalex_query: None,
        }),
    )
    .await
    {
        Ok(_) => panic!("resubscribe should fail when the second feed fetch fails"),
        Err(error) => error,
    };

    assert_eq!(error.0, StatusCode::BAD_GATEWAY);
    assert!(
        get_canonical_channel(&store, &channel_id)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        get_video(&store, "kept-episode", false)
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        get_summary(&store, "kept-episode")
            .await
            .unwrap()
            .expect("summary stays stored")
            .content,
        "Keep this summary."
    );
}

#[tokio::test]
async fn failed_first_subscribe_removes_the_new_channel() {
    let store = Store::for_test().await;
    let feed_url = feed_that_fails_on_the_second_fetch().await;
    let channel_id = crate::services::podcast_feed::podcast_source_id_for_feed_url(&feed_url);
    let state = test_app_state(store.clone()).await;

    let error = match super::add_channel(
        State(state),
        Extension(signed_in()),
        Json(AddChannelRequest {
            input: feed_url,
            openalex_query: None,
        }),
    )
    .await
    {
        Ok(_) => panic!("first subscribe should fail when the second feed fetch fails"),
        Err(error) => error,
    };

    assert_eq!(error.0, StatusCode::BAD_GATEWAY);
    assert!(
        get_canonical_channel(&store, &channel_id)
            .await
            .unwrap()
            .is_none()
    );
}
