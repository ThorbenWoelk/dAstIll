use std::sync::Arc;

use crate::config::{
    ChatRuntimeConfig, DatabricksRuntimeConfig, GoogleTtsRuntimeConfig, LocalAsrRuntimeConfig,
    OllamaRuntimeConfig, SearchRuntimeConfig, SecurityRuntimeConfig,
};

pub(super) struct RuntimeConfig {
    pub(super) search: SearchRuntimeConfig,
    pub(super) chat: ChatRuntimeConfig,
    pub(super) databricks: Option<DatabricksRuntimeConfig>,
    pub(super) google_tts: Option<GoogleTtsRuntimeConfig>,
    pub(super) local_asr: Option<LocalAsrRuntimeConfig>,
    pub(super) security: Arc<SecurityRuntimeConfig>,
    pub(super) summarize_path: String,
    pub(super) ytdlp_path: String,
    pub(super) ollama: OllamaRuntimeConfig,
    pub(super) object_store_provider: String,
    pub(super) gcs_data_bucket: Option<String>,
}

pub(super) fn load_runtime_config() -> anyhow::Result<RuntimeConfig> {
    let search = SearchRuntimeConfig::from_env();
    if search.semantic_enabled {
        anyhow::bail!("SEARCH_SEMANTIC_ENABLED must be false in the GCS-only runtime");
    }
    if search.auto_create_vector_index {
        anyhow::bail!("SEARCH_AUTO_CREATE_VECTOR_INDEX must be false in the GCS-only runtime");
    }
    let chat = ChatRuntimeConfig::from_env();
    let databricks = DatabricksRuntimeConfig::from_env().map_err(|err| anyhow::anyhow!(err))?;
    let google_tts = GoogleTtsRuntimeConfig::from_env().map_err(|err| anyhow::anyhow!(err))?;
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

    let object_store_provider =
        std::env::var("OBJECT_STORE_PROVIDER").unwrap_or_else(|_| "gcs".to_string());
    let memory_store_allowed = std::env::var("ALLOW_IN_MEMORY_OBJECT_STORE")
        .ok()
        .as_deref()
        == Some("true");
    let gcs_data_bucket = match object_store_provider.as_str() {
        "gcs" => Some(
            std::env::var("GCS_DATA_BUCKET")
                .map_err(|_| anyhow::anyhow!("GCS_DATA_BUCKET must be set"))?,
        ),
        "memory" if memory_store_allowed => None,
        "memory" => {
            anyhow::bail!(
                "OBJECT_STORE_PROVIDER=memory requires ALLOW_IN_MEMORY_OBJECT_STORE=true"
            );
        }
        other => {
            anyhow::bail!("unsupported OBJECT_STORE_PROVIDER `{other}`; use gcs or memory");
        }
    };

    Ok(RuntimeConfig {
        search,
        chat,
        databricks,
        google_tts,
        local_asr,
        security,
        summarize_path,
        ytdlp_path,
        ollama,
        object_store_provider,
        gcs_data_bucket,
    })
}
