use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use dastill::config::{OllamaRuntimeConfig, SearchRuntimeConfig};
use dastill::db::init_db;
use dastill::handlers::{channels, content, highlights, search, videos};
use dastill::services::{
    Cooldown, SearchService, SummarizerService, SummaryEvaluatorService, TranscriptService,
    YouTubeService, build_http_client,
};
use dastill::state::AppState;
use dastill::workers::{
    spawn_gap_scan_worker, spawn_queue_worker, spawn_refresh_worker, spawn_search_index_worker,
    spawn_summary_evaluation_worker,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dastill=info,tower_http=info".into()),
        )
        .init();

    // Load .env if present (simple key=value parsing, no external crate)
    if let Ok(contents) = std::fs::read_to_string(".env") {
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                if std::env::var(key).is_err() {
                    // SAFETY: called during single-threaded init before any
                    // worker threads are spawned.
                    unsafe { std::env::set_var(key, value) };
                }
            }
        }
    }

    let db_url = std::env::var("DB_URL")
        .map_err(|_| anyhow::anyhow!("DB_URL must be set (Turso database URL)"))?;
    let db_pass = std::env::var("DB_PASS").unwrap_or_default();

    tracing::info!(url = %db_url, "connecting to Turso database");
    let database = libsql::Builder::new_remote(db_url, db_pass).build().await?;

    let pool = init_db(database).await.map_err(|e| anyhow::anyhow!(e))?;

    let client = build_http_client();
    let cloud_cooldown = Arc::new(Cooldown::cloud());
    let youtube_quota_cooldown = Arc::new(Cooldown::youtube_quota());
    let transcript_cooldown = Arc::new(Cooldown::transcript());

    let youtube = Arc::new(
        YouTubeService::with_client(client.clone())
            .with_quota_cooldown(youtube_quota_cooldown.clone()),
    );
    match youtube.validate_data_api_key().await {
        Ok(Some(true)) => tracing::info!("YOUTUBE_API_KEY is configured and valid"),
        Ok(Some(false)) => {
            tracing::warn!("YOUTUBE_API_KEY is configured but invalid (or quota exceeded)")
        }
        Ok(None) => tracing::info!("YOUTUBE_API_KEY is not configured - using fallback sources"),
        Err(err) => tracing::warn!(error = %err, "could not validate YOUTUBE_API_KEY on startup"),
    }

    let search_runtime = SearchRuntimeConfig::from_env();
    let summarize_path = std::env::var("SUMMARIZE_PATH")
        .unwrap_or_else(|_| "/opt/homebrew/bin/summarize".to_string());
    let ollama = OllamaRuntimeConfig::from_env(search_runtime.semantic_enabled)
        .map_err(|err| anyhow::anyhow!(err))?;
    if std::env::var("SUMMARY_EVALUATOR_FALLBACK_MODEL").is_ok() {
        tracing::warn!(
            "SUMMARY_EVALUATOR_FALLBACK_MODEL is ignored - summary evaluation is cloud-only"
        );
    }
    let transcript = Arc::new(TranscriptService::with_path(&summarize_path));
    let ollama_semaphore = Arc::new(tokio::sync::Semaphore::new(1));
    let search_ollama_semaphore = Arc::new(tokio::sync::Semaphore::new(1));

    let summarizer = Arc::new(
        SummarizerService::with_client(client.clone(), &ollama.url, &ollama.model)
            .with_fallback_model(ollama.fallback_model)
            .with_cloud_cooldown(cloud_cooldown.clone())
            .with_ollama_semaphore(ollama_semaphore.clone()),
    );
    let summary_evaluator = Arc::new(
        SummaryEvaluatorService::with_client(client, &ollama.url, &ollama.summary_evaluator_model)
            .with_cloud_cooldown(cloud_cooldown.clone())
            .with_ollama_semaphore(ollama_semaphore.clone()),
    );
    let search = Arc::new(
        SearchService::with_config(
            &ollama.url,
            ollama.embedding_model.as_deref(),
            dastill::services::search::SEARCH_EMBEDDING_DIMENSIONS,
            search_runtime.semantic_enabled,
        )
        .with_ollama_semaphore(search_ollama_semaphore),
    );

    let state = AppState {
        db: pool,
        search_auto_create_vector_index: search_runtime.auto_create_vector_index,
        search_projection_lock: Arc::new(tokio::sync::RwLock::new(())),
        youtube,
        transcript,
        summarizer,
        summary_evaluator,
        search,
        cloud_cooldown,
        youtube_quota_cooldown,
        transcript_cooldown,
    };
    spawn_queue_worker(state.clone());
    spawn_refresh_worker(state.clone());
    spawn_gap_scan_worker(state.clone());
    spawn_summary_evaluation_worker(state.clone());
    spawn_search_index_worker(state.clone());

    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/health/ai", get(content::health_ai))
        .route("/api/search", get(search::search))
        .route("/api/search/status", get(search::search_status))
        .route(
            "/api/search/rebuild",
            post(search::rebuild_search_projection),
        )
        .route(
            "/api/workspace/bootstrap",
            get(channels::workspace_bootstrap),
        )
        .route(
            "/api/channels",
            get(channels::list_channels).post(channels::add_channel),
        )
        .route(
            "/api/channels/{id}",
            get(channels::get_channel)
                .delete(channels::delete_channel)
                .put(channels::update_channel),
        )
        .route(
            "/api/channels/{id}/sync-depth",
            get(channels::get_channel_sync_depth),
        )
        .route(
            "/api/channels/{id}/snapshot",
            get(channels::get_channel_snapshot),
        )
        .route(
            "/api/channels/{id}/refresh",
            post(channels::refresh_channel_videos),
        )
        .route(
            "/api/channels/{id}/backfill",
            post(channels::backfill_channel_videos),
        )
        .route(
            "/api/channels/{id}/videos",
            get(videos::list_channel_videos),
        )
        .route("/api/videos/{id}", get(videos::get_video))
        .route("/api/videos/{id}/info", get(videos::get_video_info))
        .route(
            "/api/videos/info/backfill",
            post(videos::backfill_video_info),
        )
        .route(
            "/api/videos/{id}/transcript",
            get(content::get_transcript).put(content::update_transcript),
        )
        .route(
            "/api/videos/{id}/acknowledged",
            axum::routing::put(videos::update_video_acknowledged),
        )
        .route(
            "/api/videos/{id}/transcript/clean",
            post(content::clean_transcript_formatting),
        )
        .route(
            "/api/videos/{id}/summary",
            get(content::get_summary).put(content::update_summary),
        )
        .route(
            "/api/videos/{id}/summary/regenerate",
            post(content::regenerate_summary),
        )
        .route("/api/highlights", get(highlights::list_highlights))
        .route(
            "/api/videos/{id}/highlights",
            get(highlights::list_video_highlights).post(highlights::create_highlight),
        )
        .route("/api/highlights/{id}", delete(highlights::delete_highlight))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("backend listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
