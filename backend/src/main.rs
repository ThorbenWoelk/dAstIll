use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use dastill::db::init_db;
use dastill::handlers::{channels, content, videos};
use dastill::services::{
    SummarizerService, SummaryEvaluatorService, TranscriptService, YouTubeService,
    build_http_client,
};
use dastill::state::AppState;
use dastill::workers::{
    spawn_gap_scan_worker, spawn_queue_worker, spawn_refresh_worker,
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
    let youtube = Arc::new(YouTubeService::with_client(client.clone()));
    match youtube.validate_data_api_key().await {
        Ok(Some(true)) => tracing::info!("YOUTUBE_API_KEY is configured and valid"),
        Ok(Some(false)) => tracing::warn!("YOUTUBE_API_KEY is configured but invalid"),
        Ok(None) => tracing::info!("YOUTUBE_API_KEY is not configured - using fallback sources"),
        Err(err) => tracing::warn!(error = %err, "could not validate YOUTUBE_API_KEY on startup"),
    }

    let summarize_path = std::env::var("SUMMARIZE_PATH")
        .unwrap_or_else(|_| "/opt/homebrew/bin/summarize".to_string());

    let ollama_url =
        std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama_model =
        std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "minimax-m2.5:cloud".to_string());
    let ollama_fallback_model = std::env::var("OLLAMA_FALLBACK_MODEL")
        .ok()
        .or_else(|| Some("qwen3:8b".to_string()));
    let summary_evaluator_model = std::env::var("SUMMARY_EVALUATOR_MODEL")
        .unwrap_or_else(|_| "qwen3-coder:480b-cloud".to_string());
    let summary_evaluator_fallback_model = std::env::var("SUMMARY_EVALUATOR_FALLBACK_MODEL")
        .ok()
        .or_else(|| Some("qwen3:8b".to_string()));
    let transcript = Arc::new(TranscriptService::with_path(&summarize_path));
    let summarizer = Arc::new(
        SummarizerService::with_client(client.clone(), &ollama_url, &ollama_model)
            .with_fallback_model(ollama_fallback_model),
    );
    let summary_evaluator = Arc::new(
        SummaryEvaluatorService::with_client(client, &ollama_url, &summary_evaluator_model)
            .with_fallback_model(summary_evaluator_fallback_model),
    );

    let state = AppState {
        db: pool,
        youtube,
        transcript,
        summarizer,
        summary_evaluator,
    };
    spawn_queue_worker(state.clone());
    spawn_refresh_worker(state.clone());
    spawn_gap_scan_worker(state.clone());
    spawn_summary_evaluation_worker(state.clone());

    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/health/ai", get(content::health_ai))
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
