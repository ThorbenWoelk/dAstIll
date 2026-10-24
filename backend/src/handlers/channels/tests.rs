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
        evaluation_work: Arc::new(crate::state::EvaluationWork::default()),
        processing_role: crate::config::ProcessingRole::Combined,
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
async fn remote_waits_for_local_transcript_then_summarizes_and_evaluates() {
    use crate::{config::ProcessingRole, db, handlers::content};
    use axum::{
        Json, Router,
        routing::{get, post},
    };
    let store = Store::for_test().await;
    let mut local = Store::for_test().await;
    local.objects = store.objects.clone();
    let video = Video {
        id: "handoff-video".into(),
        channel_id: "channel".into(),
        title: "Batching results".into(),
        thumbnail_url: None,
        published_at: Utc::now(),
        is_short: false,
        transcript_status: ContentStatus::Pending,
        summary_status: ContentStatus::Pending,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    };
    insert_video(&store, &video).await.unwrap();
    let mut waiting_video = video.clone();
    waiting_video.id = "newer-waiting-video".into();
    waiting_video.published_at += chrono::Duration::seconds(1);
    insert_video(&store, &waiting_video).await.unwrap();
    let mut remote = test_app_state(store.clone()).await;
    remote.processing_role = ProcessingRole::Remote;

    // No model or YouTube endpoint is usable yet. Waiting must still be a clean 503.
    let error = content::ensure_transcript(&remote, &video.id)
        .await
        .unwrap_err();
    assert_eq!(error.0, axum::http::StatusCode::SERVICE_UNAVAILABLE);
    assert!(error.1.contains("local app"));
    assert_eq!(
        crate::workers::count_pending_queue_work(&remote, 4)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        db::get_video(&store, &video.id, false)
            .await
            .unwrap()
            .unwrap()
            .retry_count,
        0
    );

    let request = db::pending_youtube_transcript_requests(&local)
        .await
        .unwrap()
        .into_iter()
        .find(|request| request.video_id == video.id)
        .unwrap();
    db::save_youtube_transcript_result(
        &local,
        &request,
        &db::YoutubeTranscriptResult::Ready(Transcript {
            video_id: video.id.clone(),
            raw_text: Some("Batching reduced latency from 80 ms to 35 ms in staging.".into()),
            formatted_markdown: None,
            render_mode: TranscriptRenderMode::PlainText,
            timed_text: None,
        }),
    )
    .await
    .unwrap();
    assert_eq!(db::sql_video_count(&local).await.unwrap(), 0);
    assert_eq!(
        crate::workers::count_pending_queue_work(&remote, 4)
            .await
            .unwrap(),
        1
    );

    let app = Router::new().route("/api/tags", get(|| async { Json(serde_json::json!({"models": []})) }))
        .route("/api/generate", post(|Json(request): Json<Value>| async move {
            assert!(request["prompt"].as_str().unwrap().contains("80 ms"));
            let response = if request.get("format").is_some() {
                serde_json::json!({"status":"scored", "faithfulness_score":10, "completeness_score":10, "final_score":10, "defects":[], "tags":["Batching"]}).to_string()
            } else {
                "## At a glance\n- Batching reduced latency.\n\n## Overview\nA staging result.\n\n## Key Points\nLatency fell from 80 ms to 35 ms.\n\n## Takeaways\nProduction behavior is untested.".into()
            };
            Json(serde_json::json!({"response":response,"done":true,"done_reason":"stop"}))
        }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async {
        axum::serve(listener, app).await.unwrap();
    });
    remote.summarizer = Arc::new(SummarizerService::new(OllamaCore::new(
        &url,
        "summary:cloud",
    )));
    remote.summary_evaluator = Arc::new(SummaryEvaluatorService::new(OllamaCore::new(
        &url,
        "judge:cloud",
    )));
    let summary = content::ensure_summary_for_queue(&remote, &video.id)
        .await
        .unwrap();
    assert!(summary.content.contains("35 ms"));
    let job = db::load_summary_evaluation_job(&store, &video.id)
        .await
        .unwrap()
        .unwrap();
    let evaluation = remote
        .summary_evaluator
        .evaluate(&job.transcript_text, &job.summary_content, &job.video_title)
        .await
        .unwrap();
    assert_eq!(evaluation.quality_score, Some(10));
    assert!(
        db::get_transcript(&store, &video.id)
            .await
            .unwrap()
            .is_some()
    );
    let waiting = db::requested_youtube_video_ids(&store).await.unwrap();
    assert!(!waiting.contains(&video.id));
    assert!(waiting.contains(&waiting_video.id));
    assert_eq!(
        db::get_video(&store, &video.id, false)
            .await
            .unwrap()
            .unwrap()
            .summary_status,
        ContentStatus::Ready
    );
    server.abort();
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
