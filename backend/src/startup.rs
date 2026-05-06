use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::config::{
    ChatRuntimeConfig, DatabricksRuntimeConfig, LocalAsrRuntimeConfig, OllamaRuntimeConfig,
    PollyTtsRuntimeConfig, SearchRuntimeConfig, SecurityRuntimeConfig,
};
use crate::search_progress::SearchProgress;
use crate::security::rate_limiter;
use crate::services::{
    ChatService, Cooldown, DatabricksSqlService, FtsIndex, OllamaCore, OpenAlexPlannerService,
    OpenAlexService, PodcastFeedService, PollyTtsService, SearchService, SummarizerService,
    SummaryEvaluatorService, TranscriptService, UserActivity, WebsiteService, YouTubeService,
    build_http_client,
};
use crate::state::AppState;
use crate::workers::{
    spawn_fts_hydration_if_empty, spawn_gap_scan_worker, spawn_queue_worker, spawn_refresh_worker,
    spawn_search_index_worker, spawn_search_progress_hydration, spawn_summary_evaluation_worker,
};

pub struct Runtime {
    pub state: AppState,
    pub security: Arc<SecurityRuntimeConfig>,
    pub fts_dir: PathBuf,
}

struct RuntimeConfig {
    search: SearchRuntimeConfig,
    chat: ChatRuntimeConfig,
    databricks: Option<DatabricksRuntimeConfig>,
    polly_tts: Option<PollyTtsRuntimeConfig>,
    local_asr: Option<LocalAsrRuntimeConfig>,
    security: Arc<SecurityRuntimeConfig>,
    summarize_path: String,
    ytdlp_path: String,
    ollama: OllamaRuntimeConfig,
    data_bucket: String,
    vector_bucket: String,
    vector_index: String,
    aws_region: String,
}

struct AwsClients {
    config: aws_config::SdkConfig,
    s3: aws_sdk_s3::Client,
    s3v: aws_sdk_s3vectors::Client,
}

pub fn install_crypto_providers() {
    // Install crypto providers for all rustls versions in the dependency tree.
    // libsql/hyper-rustls uses rustls 0.22, AWS SDKs use rustls 0.23.
    // Installing both ensures TLS works across the entire tree.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls 0.23 crypto provider");
    // Note: rustls 0.22 (via libsql) uses a global default approach and will pick up
    // the ring crypto features via its dependency on rustls-webpki.
}

pub async fn bind_startup_listener() -> anyhow::Result<(u16, tokio::net::TcpListener)> {
    // Bind the port immediately so Cloud Run's TCP startup probe succeeds
    // before the rest of initialization runs. The OS queues incoming
    // connections in the backlog until axum::serve() processes them.
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("port {} bound - waiting for initialization", port);
    Ok((port, listener))
}

pub async fn build_runtime(port: u16) -> anyhow::Result<Runtime> {
    let config = load_runtime_config()?;
    let aws = build_aws_clients(&config.aws_region).await?;
    let local_libsql = crate::db::initialize_local_libsql_store(
        aws.s3,
        aws.s3v,
        config.data_bucket,
        config.vector_bucket,
        config.vector_index,
        port,
    )
    .await?;
    let store = local_libsql.store;
    let read_cache = local_libsql.read_cache;
    let fts_dir = local_libsql.fts_dir;

    let client = build_http_client();
    let analytics = config
        .databricks
        .map(|config| Arc::new(DatabricksSqlService::new(client.clone(), config)));
    let cloud_cooldown = Arc::new(Cooldown::cloud_with_duration(Duration::from_secs(
        config.ollama.cloud_cooldown_secs,
    )));
    let youtube_quota_cooldown = Arc::new(Cooldown::youtube_quota());
    let transcript_cooldown = Arc::new(Cooldown::transcript());

    let youtube = Arc::new(
        YouTubeService::with_client(client.clone())
            .with_quota_cooldown(youtube_quota_cooldown.clone()),
    );
    validate_youtube_api_key(&youtube).await;

    let openalex = Arc::new(OpenAlexService::with_client(client.clone()));
    let podcast_feed = Arc::new(PodcastFeedService::with_client(client.clone()));
    let website = Arc::new(WebsiteService::with_client(client.clone()));
    let transcript_semaphore = Arc::new(tokio::sync::Semaphore::new(1));
    let transcript = Arc::new(
        TranscriptService::with_paths(&config.summarize_path, &config.ytdlp_path)
            .with_local_asr(config.local_asr)
            .with_concurrency_semaphore(transcript_semaphore),
    );
    let ollama_semaphore = Arc::new(tokio::sync::Semaphore::new(1));
    let search_ollama_semaphore = Arc::new(tokio::sync::Semaphore::new(1));

    let summarizer_core = OllamaCore::with_client(
        client.clone(),
        &config.ollama.url,
        &config.ollama.summary_model,
    )
    .with_fallback_model(config.ollama.fallback_model.clone())
    .with_api_key(config.ollama.api_key.clone())
    .with_cloud_cooldown(cloud_cooldown.clone())
    .with_ollama_semaphore(ollama_semaphore.clone());
    let summarizer = Arc::new(SummarizerService::new(summarizer_core));

    let chat_model = config
        .ollama
        .default_chat_model
        .clone()
        .unwrap_or_else(|| config.ollama.summary_model.clone());
    let openalex_planner = Arc::new(OpenAlexPlannerService::new(
        OllamaCore::with_client(build_http_client(), &config.ollama.url, &chat_model)
            .with_fallback_model(config.ollama.fallback_model.clone())
            .with_api_key(config.ollama.api_key.clone())
            .with_cloud_cooldown(cloud_cooldown.clone())
            .with_ollama_semaphore(ollama_semaphore.clone()),
    ));
    let chat_core = OllamaCore::with_client(build_http_client(), &config.ollama.url, &chat_model)
        .with_fallback_model(config.ollama.fallback_model.clone())
        .with_api_key(config.ollama.api_key.clone())
        .with_cloud_cooldown(cloud_cooldown.clone())
        .with_ollama_semaphore(ollama_semaphore.clone());
    let chat = Arc::new(
        ChatService::new(chat_core).with_multi_pass_enabled(config.chat.multi_pass_enabled),
    );
    let guardrail_model = config
        .chat
        .guardrail_model
        .clone()
        .or_else(|| config.ollama.fallback_model.clone())
        .or_else(|| config.ollama.default_chat_model.clone())
        .unwrap_or_else(|| config.ollama.summary_model.clone());
    let input_guardrails = Arc::new(crate::services::InputGuardrailService::new(
        OllamaCore::with_client(build_http_client(), &config.ollama.url, &guardrail_model)
            .with_fallback_model(config.ollama.fallback_model.clone())
            .with_api_key(config.ollama.api_key.clone())
            .with_cloud_cooldown(cloud_cooldown.clone())
            .with_ollama_semaphore(ollama_semaphore.clone()),
        config.chat.prompt_blocklist.clone(),
        config.chat.prompt_allowlist.clone(),
    ));

    let evaluator_core = OllamaCore::with_client(
        client,
        &config.ollama.url,
        &config.ollama.summary_evaluator_model,
    )
    .with_api_key(config.ollama.api_key.clone())
    .with_cloud_cooldown(cloud_cooldown.clone())
    .with_ollama_semaphore(ollama_semaphore.clone());
    let summary_evaluator = Arc::new(SummaryEvaluatorService::new(evaluator_core));
    let search = Arc::new(
        SearchService::with_config(
            &config.ollama.url,
            config.ollama.embedding_model.as_deref(),
            crate::services::search::SEARCH_EMBEDDING_DIMENSIONS,
            config.search.semantic_enabled,
        )
        .with_api_key(config.ollama.api_key)
        .with_ollama_semaphore(search_ollama_semaphore)
        .with_rerank_model(config.ollama.rerank_model)
        .with_hyde_model(config.ollama.hyde_model),
    );
    let search_progress = Arc::new(SearchProgress::new(
        search.model(),
        search.dimensions(),
        search.semantic_enabled(),
    ));

    let fts = Arc::new(
        FtsIndex::new_with_db(local_libsql.database, local_libsql.shared_db_path)
            .await
            .expect("failed to create shared libSQL FTS index"),
    );
    let polly_tts = config.polly_tts.map(|cfg| {
        Arc::new(PollyTtsService::new(
            aws_sdk_polly::Client::new(&aws.config),
            cfg.voice_id,
            cfg.engine,
            cfg.output_format,
            cfg.sample_rate,
        ))
    });

    let user_activity = Arc::new(UserActivity::from_env());
    let security = config.security.clone();
    let state = AppState {
        db: store.clone(),
        read_cache: Arc::new(read_cache),
        security: security.clone(),
        request_rate_limiter: rate_limiter(security.as_ref()),
        search_auto_create_vector_index: config.search.auto_create_vector_index,
        search_projection_lock: Arc::new(tokio::sync::RwLock::new(())),
        search_progress,
        fts,
        youtube,
        openalex_planner,
        openalex,
        podcast_feed,
        website,
        transcript,
        tts: polly_tts,
        summarizer,
        summary_evaluator,
        search,
        chat,
        input_guardrails,
        analytics,
        active_replies: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        conversation_store_lock: Arc::new(tokio::sync::Mutex::new(())),
        anonymous_chat_quota_lock: Arc::new(tokio::sync::Mutex::new(())),
        mobile_auth_handoffs: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        cloud_cooldown,
        youtube_quota_cooldown,
        transcript_cooldown,
        user_activity,
    };

    Ok(Runtime {
        state,
        security,
        fts_dir,
    })
}

pub async fn spawn_runtime_workers(state: AppState, fts_dir: PathBuf) {
    spawn_search_progress_hydration(state.clone());
    spawn_fts_hydration_if_empty(state.clone(), &fts_dir).await;
    spawn_queue_worker(state.clone());
    spawn_refresh_worker(state.clone());
    spawn_gap_scan_worker(state.clone());
    spawn_summary_evaluation_worker(state.clone());
    spawn_search_index_worker(state);
}

fn load_runtime_config() -> anyhow::Result<RuntimeConfig> {
    let search = SearchRuntimeConfig::from_env();
    let chat = ChatRuntimeConfig::from_env();
    let databricks = DatabricksRuntimeConfig::from_env().map_err(|err| anyhow::anyhow!(err))?;
    let polly_tts = PollyTtsRuntimeConfig::from_env().map_err(|err| anyhow::anyhow!(err))?;
    let local_asr = LocalAsrRuntimeConfig::from_env();
    let security = Arc::new(SecurityRuntimeConfig::from_env().map_err(|err| anyhow::anyhow!(err))?);
    let summarize_path = std::env::var("SUMMARIZE_PATH")
        .unwrap_or_else(|_| "/opt/homebrew/bin/summarize".to_string());
    let ytdlp_path =
        std::env::var("YTDLP_PATH").unwrap_or_else(|_| "/usr/local/bin/yt-dlp".to_string());
    let ollama = OllamaRuntimeConfig::from_env(search.semantic_enabled)
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

    Ok(RuntimeConfig {
        search,
        chat,
        databricks,
        polly_tts,
        local_asr,
        security,
        summarize_path,
        ytdlp_path,
        ollama,
        data_bucket,
        vector_bucket,
        vector_index,
        aws_region,
    })
}

async fn build_aws_clients(aws_region: &str) -> anyhow::Result<AwsClients> {
    let config = crate::aws_auth::load_aws_sdk_config(aws_region.to_string())
        .await
        .map_err(|err| anyhow::anyhow!(err))?;

    let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&config);
    if let Ok(endpoint) = std::env::var("S3_ENDPOINT_URL") {
        tracing::info!(endpoint = %endpoint, "using custom S3 endpoint");
        s3_config_builder = s3_config_builder
            .endpoint_url(endpoint)
            .force_path_style(true);
    }
    let s3 = aws_sdk_s3::Client::from_conf(s3_config_builder.build());

    let mut s3v_config_builder = aws_sdk_s3vectors::config::Builder::from(&config);
    if let Ok(endpoint) = std::env::var("S3_VECTOR_ENDPOINT_URL") {
        tracing::info!(endpoint = %endpoint, "using custom S3 Vectors endpoint");
        s3v_config_builder = s3v_config_builder.endpoint_url(endpoint);
    }
    let s3v = aws_sdk_s3vectors::Client::from_conf(s3v_config_builder.build());

    Ok(AwsClients { config, s3, s3v })
}

async fn validate_youtube_api_key(youtube: &YouTubeService) {
    match youtube.validate_data_api_key().await {
        Ok(crate::services::DataApiKeyValidation::Valid) => {
            tracing::info!("YOUTUBE_API_KEY is configured and valid")
        }
        Ok(crate::services::DataApiKeyValidation::QuotaExceeded { message }) => {
            tracing::warn!(
                message = message.as_deref().unwrap_or("unknown"),
                "YOUTUBE_API_KEY is configured but YouTube Data API quota is currently exceeded"
            )
        }
        Ok(crate::services::DataApiKeyValidation::ServiceDisabled { reason, message }) => {
            tracing::warn!(
                reason = reason.as_deref().unwrap_or("unknown"),
                message = message.as_deref().unwrap_or("unknown"),
                "YOUTUBE_API_KEY is configured but YouTube Data API v3 is disabled for the active GCP project or the key belongs to a different project"
            )
        }
        Ok(crate::services::DataApiKeyValidation::Rejected { reason, message }) => {
            tracing::warn!(
                reason = reason.as_deref().unwrap_or("unknown"),
                message = message.as_deref().unwrap_or("unknown"),
                "YOUTUBE_API_KEY is configured but rejected by YouTube Data API"
            )
        }
        Ok(crate::services::DataApiKeyValidation::NotConfigured) => {
            tracing::info!("YOUTUBE_API_KEY is not configured - using fallback sources")
        }
        Err(err) => tracing::warn!(error = %err, "could not validate YOUTUBE_API_KEY on startup"),
    }
}
