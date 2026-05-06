use std::sync::Arc;

use crate::config::{
    ChatRuntimeConfig, DatabricksRuntimeConfig, LocalAsrRuntimeConfig, OllamaRuntimeConfig,
    PollyTtsRuntimeConfig, SearchRuntimeConfig, SecurityRuntimeConfig,
};

pub(super) struct RuntimeConfig {
    pub(super) search: SearchRuntimeConfig,
    pub(super) chat: ChatRuntimeConfig,
    pub(super) databricks: Option<DatabricksRuntimeConfig>,
    pub(super) polly_tts: Option<PollyTtsRuntimeConfig>,
    pub(super) local_asr: Option<LocalAsrRuntimeConfig>,
    pub(super) security: Arc<SecurityRuntimeConfig>,
    pub(super) summarize_path: String,
    pub(super) ytdlp_path: String,
    pub(super) ollama: OllamaRuntimeConfig,
    pub(super) data_bucket: String,
    pub(super) vector_bucket: String,
    pub(super) vector_index: String,
    pub(super) aws_region: String,
}

pub(super) fn load_runtime_config() -> anyhow::Result<RuntimeConfig> {
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
