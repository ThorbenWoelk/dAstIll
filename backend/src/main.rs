use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router, middleware,
    routing::{delete, get, post},
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{Layer, filter, layer::SubscriberExt, util::SubscriberInitExt};

use dastill::cache_headers::add_cache_control;
use dastill::config::{
    ChatRuntimeConfig, DatabricksRuntimeConfig, ElevenLabsTtsRuntimeConfig, OllamaRuntimeConfig,
    SearchRuntimeConfig, SecurityRuntimeConfig,
};
use dastill::db::init_store;
use dastill::handlers::{
    analytics, channels, chat, content, highlights, preferences, search, videos,
};
use dastill::read_cache::ReadCache;
use dastill::search_progress::SearchProgress;
use dastill::security::{
    build_cors_layer, enforce_anonymous_chat_quota, enforce_baseline_rate_limit,
    enforce_expensive_rate_limit, rate_limiter, require_operator_role, require_proxy_auth,
};
use dastill::services::{
    ChatService, Cooldown, DatabricksSqlService, ElevenLabsTtsService, FtsIndex, OllamaCore,
    SearchService, SummarizerService, SummaryEvaluatorService, TranscriptService, YouTubeService,
    build_http_client,
};
use dastill::state::AppState;
use dastill::workers::{
    spawn_gap_scan_worker, spawn_queue_worker, spawn_refresh_worker, spawn_search_index_worker,
    spawn_summary_evaluation_worker,
};

fn should_send_to_logfire(target: &str) -> bool {
    target.starts_with("dastill::services::chat") || target.starts_with("dastill::handlers::search")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install the rustls crypto provider before any TLS usage (required by rustls 0.23+)
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

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

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "dastill=info,tower_http=info".into());

    let _logfire_guard = if std::env::var("LOGFIRE_TOKEN").is_ok() {
        let logfire = logfire::configure().local().finish()?;
        let guard = logfire.clone().shutdown_guard();

        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .with(
                logfire
                    .tracing_layer()
                    .with_filter(filter::filter_fn(|metadata| {
                        should_send_to_logfire(metadata.target())
                    })),
            )
            .init();

        Some(guard)
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
        None
    };

    let search_runtime = SearchRuntimeConfig::from_env();
    let chat_runtime = ChatRuntimeConfig::from_env();
    let databricks_runtime =
        DatabricksRuntimeConfig::from_env().map_err(|err| anyhow::anyhow!(err))?;
    let elevenlabs_tts_runtime =
        ElevenLabsTtsRuntimeConfig::from_env().map_err(|err| anyhow::anyhow!(err))?;
    let security_runtime =
        Arc::new(SecurityRuntimeConfig::from_env().map_err(|err| anyhow::anyhow!(err))?);
    let summarize_path = std::env::var("SUMMARIZE_PATH")
        .unwrap_or_else(|_| "/opt/homebrew/bin/summarize".to_string());
    let ytdlp_path =
        std::env::var("YTDLP_PATH").unwrap_or_else(|_| "/usr/local/bin/yt-dlp".to_string());
    let ollama = OllamaRuntimeConfig::from_env(search_runtime.semantic_enabled)
        .map_err(|err| anyhow::anyhow!(err))?;
    if std::env::var("SUMMARY_EVALUATOR_FALLBACK_MODEL").is_ok() {
        tracing::warn!(
            "SUMMARY_EVALUATOR_FALLBACK_MODEL is ignored - summary evaluation is cloud-only"
        );
    }

    let data_bucket = std::env::var("S3_DATA_BUCKET")
        .map_err(|_| anyhow::anyhow!("S3_DATA_BUCKET must be set"))?;
    let vector_bucket = std::env::var("S3_VECTOR_BUCKET")
        .map_err(|_| anyhow::anyhow!("S3_VECTOR_BUCKET must be set"))?;
    let vector_index =
        std::env::var("S3_VECTOR_INDEX").unwrap_or_else(|_| "search-chunks".to_string());
    let aws_region = std::env::var("AWS_REGION").unwrap_or_else(|_| "eu-central-1".to_string());

    let aws_config = if let (Ok(role_arn), Ok(audience)) = (
        std::env::var("AWS_ROLE_ARN"),
        std::env::var("AWS_WIF_AUDIENCE"),
    ) {
        tracing::info!(role_arn = %role_arn, "using GCP Workload Identity Federation for AWS auth");
        let wif_provider = dastill::aws_auth::GcpWifCredentialProvider::new(
            role_arn,
            audience,
            aws_region.clone(),
        );
        aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(aws_region))
            .credentials_provider(wif_provider)
            .load()
            .await
    } else {
        tracing::info!("using default AWS credential chain (local dev)");
        aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(aws_region))
            .load()
            .await
    };

    tracing::info!(bucket = %data_bucket, vector_bucket = %vector_bucket, "connecting to AWS S3");
    let s3_client = aws_sdk_s3::Client::new(&aws_config);
    let s3v_client = aws_sdk_s3vectors::Client::new(&aws_config);

    let gcp_project_id = std::env::var("GCP_PROJECT_ID")
        .map_err(|_| anyhow::anyhow!("GCP_PROJECT_ID must be set"))?;
    tracing::info!(project = %gcp_project_id, "connecting to Firestore");
    let firestore_db = firestore::FirestoreDb::new(&gcp_project_id).await?;

    let pool = init_store(
        s3_client,
        s3v_client,
        firestore_db,
        data_bucket,
        vector_bucket,
        vector_index,
    )
    .await
    .map_err(|e| anyhow::anyhow!(e))?;

    let client = build_http_client();
    let analytics = databricks_runtime
        .map(|config| Arc::new(DatabricksSqlService::new(client.clone(), config)));
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
    let transcript = Arc::new(TranscriptService::with_paths(&summarize_path, &ytdlp_path));
    let ollama_semaphore = Arc::new(tokio::sync::Semaphore::new(1));
    let search_ollama_semaphore = Arc::new(tokio::sync::Semaphore::new(1));

    let summarizer_core = OllamaCore::with_client(client.clone(), &ollama.url, &ollama.model)
        .with_fallback_model(ollama.fallback_model.clone())
        .with_api_key(ollama.api_key.clone())
        .with_cloud_cooldown(cloud_cooldown.clone())
        .with_ollama_semaphore(ollama_semaphore.clone());
    let summarizer = Arc::new(SummarizerService::new(summarizer_core));

    let chat_model = ollama
        .chat_model
        .clone()
        .unwrap_or_else(|| ollama.model.clone());
    let chat_core = OllamaCore::with_client(build_http_client(), &ollama.url, &chat_model)
        .with_fallback_model(ollama.fallback_model.clone())
        .with_api_key(ollama.api_key.clone())
        .with_cloud_cooldown(cloud_cooldown.clone())
        .with_ollama_semaphore(ollama_semaphore.clone());
    let chat = Arc::new(
        ChatService::new(chat_core).with_multi_pass_enabled(chat_runtime.multi_pass_enabled),
    );

    let evaluator_core =
        OllamaCore::with_client(client, &ollama.url, &ollama.summary_evaluator_model)
            .with_api_key(ollama.api_key.clone())
            .with_cloud_cooldown(cloud_cooldown.clone())
            .with_ollama_semaphore(ollama_semaphore.clone());
    let summary_evaluator = Arc::new(SummaryEvaluatorService::new(evaluator_core));
    let search = Arc::new(
        SearchService::with_config(
            &ollama.url,
            ollama.embedding_model.as_deref(),
            dastill::services::search::SEARCH_EMBEDDING_DIMENSIONS,
            search_runtime.semantic_enabled,
        )
        .with_api_key(ollama.api_key)
        .with_ollama_semaphore(search_ollama_semaphore)
        .with_rerank_model(ollama.rerank_model)
        .with_hyde_model(ollama.hyde_model),
    );
    let search_progress = Arc::new(SearchProgress::new(
        search.model(),
        search.dimensions(),
        search.semantic_enabled(),
    ));

    let fts = Arc::new(FtsIndex::new().expect("failed to create in-memory FTS index"));
    let elevenlabs_tts = elevenlabs_tts_runtime.map(|cfg| {
        Arc::new(ElevenLabsTtsService::new(
            build_http_client(),
            cfg.api_key,
            cfg.voice_id,
            cfg.model_id,
            cfg.output_format,
        ))
    });

    let state = AppState {
        db: pool,
        read_cache: Arc::new(ReadCache::default()),
        security: security_runtime.clone(),
        request_rate_limiter: rate_limiter(security_runtime.as_ref()),
        search_auto_create_vector_index: search_runtime.auto_create_vector_index,
        search_projection_lock: Arc::new(tokio::sync::RwLock::new(())),
        search_progress,
        fts,
        youtube,
        transcript,
        elevenlabs_tts,
        summarizer,
        summary_evaluator,
        search,
        chat,
        analytics,
        active_chats: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        chat_store_lock: Arc::new(tokio::sync::Mutex::new(())),
        anonymous_chat_quota_lock: Arc::new(tokio::sync::Mutex::new(())),
        cloud_cooldown,
        youtube_quota_cooldown,
        transcript_cooldown,
    };

    let search_progress_state = state.clone();
    tokio::spawn(async move {
        tracing::info!("search progress hydration started");

        let search_progress_materials = {
            let conn = search_progress_state.db.connect();
            dastill::db::list_search_progress_materials(&conn).await
        };

        let search_progress_materials = match search_progress_materials {
            Ok(materials) => materials,
            Err(err) => {
                tracing::error!(error = %err, "search progress hydration failed to load materials");
                return;
            }
        };

        let vector_index_ready = if search_progress_state.search.semantic_enabled() {
            let conn = search_progress_state.db.connect();
            match dastill::db::has_vector_index(&conn).await {
                Ok(ready) => ready,
                Err(err) => {
                    tracing::error!(error = %err, "search progress hydration failed to inspect vector index");
                    false
                }
            }
        } else {
            false
        };

        let search_available = search_progress_state.search.is_available().await;
        search_progress_state
            .search_progress
            .initialize_from_materials(
                &search_progress_materials,
                search_available,
                vector_index_ready,
            )
            .await;

        tracing::info!(
            total_sources = search_progress_materials.len(),
            vector_index_ready,
            search_available,
            "search progress hydration complete"
        );
    });

    // Populate the in-memory FTS index from all existing S3 chunks at startup.
    let fts_hydration_state = state.clone();
    tokio::spawn(async move {
        dastill::workers::populate_fts_index_from_store(fts_hydration_state).await;
    });

    spawn_queue_worker(state.clone());
    spawn_refresh_worker(state.clone());
    spawn_gap_scan_worker(state.clone());
    spawn_summary_evaluation_worker(state.clone());
    spawn_search_index_worker(state.clone());

    let protected_api = Router::new()
        .route("/api/health/ai", get(content::health_ai))
        .route("/api/chat/config", get(chat::chat_client_config))
        .route(
            "/api/chat/conversations",
            get(chat::list_conversations).post(chat::create_conversation),
        )
        .route(
            "/api/chat/conversations/{id}",
            get(chat::get_conversation)
                .put(chat::update_conversation)
                .delete(chat::delete_conversation),
        )
        .route(
            "/api/chat/conversations/{id}/messages",
            post(chat::send_message)
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    enforce_expensive_rate_limit,
                ))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    enforce_anonymous_chat_quota,
                )),
        )
        .route(
            "/api/chat/conversations/{id}/stream",
            get(chat::reconnect_stream).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/chat/conversations/{id}/cancel",
            post(chat::cancel_message),
        )
        .route(
            "/api/preferences",
            get(preferences::get_preferences).put(preferences::save_preferences),
        )
        .route("/api/search", get(search::search))
        .route("/api/search/status", get(search::search_status))
        .route(
            "/api/search/status/stream",
            get(search::search_status_stream).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/search/rebuild",
            post(search::rebuild_search_projection).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
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
            get(channels::get_channel).put(channels::update_channel),
        )
        .route(
            "/api/channels/{id}",
            delete(channels::delete_channel).layer(middleware::from_fn(require_operator_role)),
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
            post(channels::refresh_channel_videos).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/channels/{id}/backfill",
            post(channels::backfill_channel_videos).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/channels/{id}/videos",
            get(videos::list_channel_videos),
        )
        .route("/api/videos", post(videos::add_manual_video))
        .route("/api/videos/{id}", get(videos::get_video))
        .route("/api/videos/{id}/info", get(videos::get_video_info))
        .route(
            "/api/videos/{id}/info/ensure",
            post(videos::ensure_video_info).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/videos/info/backfill",
            post(videos::backfill_video_info).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route("/api/videos/{id}/transcript", get(content::get_transcript))
        .route(
            "/api/videos/{id}/transcript/ensure",
            post(content::generate_transcript).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/videos/{id}/transcript",
            axum::routing::put(content::update_transcript),
        )
        .route(
            "/api/videos/{id}/acknowledged",
            axum::routing::put(videos::update_video_acknowledged),
        )
        .route(
            "/api/videos/{id}/transcript/clean",
            post(content::clean_transcript_formatting).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route("/api/videos/{id}/summary", get(content::get_summary))
        .route(
            "/api/videos/{id}/summary/ensure",
            post(content::generate_summary).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/videos/{id}/summary",
            axum::routing::put(content::update_summary),
        )
        .route(
            "/api/videos/{id}/summary/regenerate",
            post(content::regenerate_summary).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/videos/{id}/reset",
            post(content::reset_video).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route("/api/highlights", get(highlights::list_highlights))
        .route(
            "/api/videos/{id}/highlights",
            get(highlights::list_video_highlights).post(highlights::create_highlight),
        )
        .route("/api/highlights/{id}", delete(highlights::delete_highlight))
        .route("/api/analytics/events", post(analytics::ingest_events))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            enforce_baseline_rate_limit,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_proxy_auth,
        ));

    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .merge(protected_api)
        .layer(middleware::from_fn(add_cache_control))
        .layer(build_cors_layer(security_runtime.as_ref()).map_err(|err| anyhow::anyhow!(err))?)
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
