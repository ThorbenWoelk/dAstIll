use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

mod aws_clients;
mod settings;
mod youtube_validation;

use aws_clients::build_aws_clients;
use settings::load_runtime_config;
use youtube_validation::validate_youtube_api_key;

use crate::config::SecurityRuntimeConfig;
use crate::search_progress::SearchProgress;
use crate::security::rate_limiter;
use crate::services::{
    ChatService, Cooldown, DatabricksSqlService, FtsIndex, OllamaCore, OpenAlexPlannerService,
    OpenAlexService, PodcastFeedService, PollyTtsService, SearchService, SummarizerService,
    SummaryEvaluatorService, TranscriptService, UserActivity, WebsiteService, YouTubeService,
    build_http_client,
};
use crate::state::AppState;

pub struct Runtime {
    pub state: AppState,
    pub security: Arc<SecurityRuntimeConfig>,
    pub fts_dir: PathBuf,
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
