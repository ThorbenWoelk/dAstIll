use std::env;

use crate::services::SummaryEvaluatorService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OllamaRuntimeConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub summary_model: String,
    pub default_chat_model: Option<String>,
    pub fallback_model: Option<String>,
    pub cloud_cooldown_secs: u64,
    pub summary_evaluator_model: String,
    pub embedding_model: Option<String>,
    /// Optional cross-encoder model for re-ranking search results (env: SEARCH_RERANK_MODEL).
    pub rerank_model: Option<String>,
    /// Optional generative model for HyDE passage synthesis (env: SEARCH_HYDE_MODEL).
    pub hyde_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRuntimeConfig {
    pub auto_create_vector_index: bool,
    pub semantic_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatRuntimeConfig {
    pub multi_pass_enabled: bool,
    pub guardrail_model: Option<String>,
    pub prompt_blocklist: Vec<String>,
    pub prompt_allowlist: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityRuntimeConfig {
    pub proxy_token: String,
    pub firebase_project_id: String,
    pub allowed_origins: Vec<String>,
    pub operator_email_allowlist: Vec<String>,
    pub default_seeded_channel_id: String,
    pub default_seeded_channel_ids: Vec<String>,
    pub baseline_rate_limit_per_minute: u32,
    pub expensive_rate_limit_per_minute: u32,
    pub anonymous_chat_quota: u32,
}

const LOCAL_DEV_FIREBASE_PROJECT_ID: &str = "demo-dastill";
const LOCAL_DEV_DEFAULT_SEEDED_CHANNEL_ID: &str = "UCbRP3c757lWg9M-U7TyEkXA";
pub const DEFAULT_HARD_FORK_FEED_URL: &str = "https://feeds.simplecast.com/6HKOhNgS";

fn default_seeded_channel_ids() -> Vec<String> {
    vec![
        LOCAL_DEV_DEFAULT_SEEDED_CHANNEL_ID.to_string(),
        crate::services::podcast_feed::podcast_source_id_for_feed_url(DEFAULT_HARD_FORK_FEED_URL),
    ]
}

fn configured_seeded_channel_ids() -> Vec<String> {
    if let Some(ids) = optional_csv_env("DEFAULT_SEEDED_CHANNEL_IDS").filter(|ids| !ids.is_empty())
    {
        return ids;
    }

    optional_env("DEFAULT_SEEDED_CHANNEL_ID")
        .filter(|id| !id.trim().is_empty())
        .map(|id| vec![id])
        .unwrap_or_else(default_seeded_channel_ids)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabricksRuntimeConfig {
    pub host: String,
    pub token: String,
    pub warehouse_id: String,
    pub catalog: String,
    pub schema: String,
    pub bronze_table: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollyTtsRuntimeConfig {
    pub voice_id: String,
    pub engine: String,
    pub output_format: String,
    pub sample_rate: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalAsrRuntimeConfig {
    pub base_url: String,
    pub api_key: String,
    pub auth_mode: LocalAsrAuthMode,
    pub model: String,
    pub max_audio_bytes: u64,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalAsrAuthMode {
    ApiKey,
    GoogleIdToken,
}

impl OllamaRuntimeConfig {
    pub fn from_env(search_semantic_enabled: bool) -> Result<Self, String> {
        let url = env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let api_key = optional_env("OLLAMA_API_KEY");
        let summary_model = required_env("OLLAMA_SUMMARY_MODEL")?;
        let default_chat_model = optional_env("OLLAMA_DEFAULT_CHAT_MODEL");
        let fallback_model = optional_env("OLLAMA_FALLBACK_MODEL");
        let cloud_cooldown_secs = optional_u64_env("OLLAMA_CLOUD_COOLDOWN_SECS")
            .unwrap_or(crate::services::http::DEFAULT_CLOUD_COOLDOWN_DURATION.as_secs());
        let summary_evaluator_model = required_env("SUMMARY_EVALUATOR_MODEL")?;
        let embedding_model = if search_semantic_enabled {
            Some(required_env("OLLAMA_EMBEDDING_MODEL")?)
        } else {
            optional_env("OLLAMA_EMBEDDING_MODEL")
        };

        validate_distinct_model_roles(&summary_model, &summary_evaluator_model)?;
        SummaryEvaluatorService::validate_model_policy(&summary_evaluator_model)?;
        validate_cloud_auth(&url, &api_key)?;

        Ok(Self {
            url,
            api_key,
            summary_model,
            default_chat_model,
            fallback_model,
            cloud_cooldown_secs,
            summary_evaluator_model,
            embedding_model,
            rerank_model: optional_env("SEARCH_RERANK_MODEL"),
            hyde_model: optional_env("SEARCH_HYDE_MODEL"),
        })
    }
}

fn validate_distinct_model_roles(
    summary_model: &str,
    summary_evaluator_model: &str,
) -> Result<(), String> {
    if summary_model == summary_evaluator_model {
        return Err(format!(
            "OLLAMA_SUMMARY_MODEL and SUMMARY_EVALUATOR_MODEL must differ so summaries are evaluated independently; got `{summary_model}` for both"
        ));
    }

    Ok(())
}

impl SearchRuntimeConfig {
    pub fn from_env() -> Self {
        Self {
            auto_create_vector_index: optional_bool_env("SEARCH_AUTO_CREATE_VECTOR_INDEX")
                .unwrap_or(false),
            semantic_enabled: optional_bool_env("SEARCH_SEMANTIC_ENABLED")
                .unwrap_or(default_search_semantic_enabled()),
        }
    }
}

impl ChatRuntimeConfig {
    pub fn from_env() -> Self {
        Self {
            multi_pass_enabled: optional_bool_env("CHAT_MULTI_PASS_ENABLED").unwrap_or(true),
            guardrail_model: optional_env("CHAT_GUARDRAIL_MODEL"),
            prompt_blocklist: optional_csv_env("CHAT_PROMPT_BLOCKLIST").unwrap_or_default(),
            prompt_allowlist: optional_csv_env("CHAT_PROMPT_ALLOWLIST").unwrap_or_default(),
        }
    }
}

impl SecurityRuntimeConfig {
    pub fn from_env() -> Result<Self, String> {
        let default_seeded_channel_ids = configured_seeded_channel_ids();
        let default_seeded_channel_id = default_seeded_channel_ids
            .first()
            .cloned()
            .unwrap_or_else(|| LOCAL_DEV_DEFAULT_SEEDED_CHANNEL_ID.to_string());
        Ok(Self {
            proxy_token: required_env_with_local_default(
                "BACKEND_PROXY_TOKEN",
                "local-dev-backend-proxy-token",
            )?,
            firebase_project_id: optional_env("FIREBASE_PROJECT_ID")
                .or_else(|| optional_env("PUBLIC_FIREBASE_PROJECT_ID"))
                .or_else(|| optional_env("GCP_PROJECT_ID"))
                .or_else(|| optional_env("GOOGLE_CLOUD_PROJECT"))
                .unwrap_or_else(|| LOCAL_DEV_FIREBASE_PROJECT_ID.to_string()),
            allowed_origins: optional_csv_env("BACKEND_CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(default_backend_allowed_origins),
            operator_email_allowlist: optional_csv_env("OPERATOR_EMAIL_ALLOWLIST")
                .unwrap_or_default()
                .into_iter()
                .map(|email| email.trim().to_lowercase())
                .filter(|email| !email.is_empty())
                .collect(),
            // Release builds do not use `cfg!(debug_assertions)`; use the same default as local dev
            // when unset so Cloud Run and Docker do not require a duplicate env var.
            default_seeded_channel_id,
            default_seeded_channel_ids,
            // Baseline applies to almost all API routes; SPAs with polling and parallel loads
            // need a generous default (120/min was routinely exceeded by a single user).
            baseline_rate_limit_per_minute: optional_u32_env("BASELINE_RATE_LIMIT_PER_MINUTE")
                .unwrap_or(600)
                .clamp(1, 1_000),
            // Expensive tier stacks with baseline for AI/chat/search mutations and streams.
            expensive_rate_limit_per_minute: optional_u32_env("EXPENSIVE_RATE_LIMIT_PER_MINUTE")
                .unwrap_or(120)
                .clamp(1, 1_000),
            anonymous_chat_quota: optional_u32_env("ANONYMOUS_CHAT_QUOTA")
                .unwrap_or(30)
                .clamp(1, 1_000),
        })
    }
}

impl DatabricksRuntimeConfig {
    pub fn from_env() -> Result<Option<Self>, String> {
        let host = optional_env("DATABRICKS_HOST");
        let token = optional_env("DATABRICKS_TOKEN");
        let warehouse_id = optional_env("DATABRICKS_WAREHOUSE_ID");

        if host.is_none() && token.is_none() && warehouse_id.is_none() {
            return Ok(None);
        }

        Ok(Some(Self {
            host: host.ok_or_else(|| "DATABRICKS_HOST must be set".to_string())?,
            token: token.ok_or_else(|| "DATABRICKS_TOKEN must be set".to_string())?,
            warehouse_id: warehouse_id
                .ok_or_else(|| "DATABRICKS_WAREHOUSE_ID must be set".to_string())?,
            catalog: optional_env("DATABRICKS_CATALOG").unwrap_or_else(|| "workspace".to_string()),
            schema: optional_env("DATABRICKS_SCHEMA").unwrap_or_else(|| "sandbox".to_string()),
            bronze_table: optional_env("DATABRICKS_BRONZE_TABLE")
                .unwrap_or_else(|| "bronze_app_events".to_string()),
        }))
    }
}

impl PollyTtsRuntimeConfig {
    pub fn from_env() -> Result<Option<Self>, String> {
        let enabled = optional_bool_env("POLLY_TTS_ENABLED").unwrap_or(false);
        if !enabled {
            return Ok(None);
        }

        Ok(Some(Self {
            voice_id: optional_env("POLLY_TTS_VOICE_ID").unwrap_or_else(|| "Joanna".to_string()),
            engine: optional_env("POLLY_TTS_ENGINE").unwrap_or_else(|| "neural".to_string()),
            output_format: optional_env("POLLY_TTS_OUTPUT_FORMAT")
                // `wav` maps to Polly `pcm` and then we wrap the result into a WAV container.
                .unwrap_or_else(|| "wav".to_string()),
            sample_rate: optional_env("POLLY_TTS_SAMPLE_RATE")
                .unwrap_or_else(|| "16000".to_string()),
        }))
    }
}

impl LocalAsrRuntimeConfig {
    pub fn from_env() -> Option<Self> {
        let enabled = optional_bool_env("LOCAL_ASR_ENABLED").unwrap_or(false);
        let base_url = optional_env("LOCAL_ASR_BASE_URL");
        if !enabled && base_url.is_none() {
            return None;
        }

        Some(Self {
            base_url: base_url.unwrap_or_else(|| "http://127.0.0.1:5092/v1".to_string()),
            api_key: optional_env("LOCAL_ASR_API_KEY")
                .unwrap_or_else(|| "sk-no-key-required".to_string()),
            auth_mode: LocalAsrAuthMode::from_env_value(optional_env("LOCAL_ASR_AUTH_MODE")),
            model: optional_env("LOCAL_ASR_MODEL").unwrap_or_else(|| "whisper-base.en".to_string()),
            max_audio_bytes: optional_u64_env("LOCAL_ASR_MAX_AUDIO_BYTES")
                .unwrap_or(250 * 1024 * 1024),
            timeout_secs: optional_u64_env("LOCAL_ASR_TIMEOUT_SECS").unwrap_or(60 * 60),
        })
    }

    pub fn transcription_url(&self) -> String {
        format!(
            "{}/audio/transcriptions",
            self.base_url.trim_end_matches('/')
        )
    }

    pub fn audience_url(&self) -> String {
        self.base_url
            .trim_end_matches('/')
            .strip_suffix("/v1")
            .unwrap_or_else(|| self.base_url.trim_end_matches('/'))
            .to_string()
    }
}

impl LocalAsrAuthMode {
    fn from_env_value(value: Option<String>) -> Self {
        match value.as_deref().map(str::trim) {
            Some("google_id_token") => Self::GoogleIdToken,
            _ => Self::ApiKey,
        }
    }
}

fn is_local_url(url: &str) -> bool {
    let host = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .and_then(|s| s.split('/').next())
        .and_then(|s| s.split(':').next())
        .unwrap_or(url);
    matches!(host, "localhost" | "127.0.0.1" | "0.0.0.0" | "::1")
}

fn validate_cloud_auth(url: &str, api_key: &Option<String>) -> Result<(), String> {
    if !is_local_url(url) && api_key.is_none() {
        return Err(format!(
            "OLLAMA_API_KEY is required when OLLAMA_URL points to a remote endpoint ({url})"
        ));
    }
    Ok(())
}

fn default_search_semantic_enabled() -> bool {
    cfg!(debug_assertions)
}

fn required_env(key: &str) -> Result<String, String> {
    match env::var(key) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(format!("{key} must be set and non-empty"))
            } else {
                Ok(trimmed.to_string())
            }
        }
        Err(_) => Err(format!("{key} must be set")),
    }
}

fn required_env_with_local_default(key: &str, local_default: &str) -> Result<String, String> {
    optional_env(key)
        .or_else(|| cfg!(debug_assertions).then(|| local_default.to_string()))
        .ok_or_else(|| format!("{key} must be set"))
}

fn optional_env(key: &str) -> Option<String> {
    env::var(key).ok().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn optional_csv_env(key: &str) -> Option<Vec<String>> {
    optional_env(key).map(|value| {
        value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    })
}

fn optional_bool_env(key: &str) -> Option<bool> {
    optional_env(key).map(|value| {
        matches!(
            value.as_str(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        )
    })
}

fn optional_u32_env(key: &str) -> Option<u32> {
    optional_env(key).and_then(|value| value.parse::<u32>().ok())
}

fn optional_u64_env(key: &str) -> Option<u64> {
    optional_env(key).and_then(|value| value.parse::<u64>().ok())
}

fn default_backend_allowed_origins() -> Vec<String> {
    vec![
        "http://localhost:3000".to_string(),
        "http://127.0.0.1:3000".to_string(),
        "http://localhost:3543".to_string(),
        "http://127.0.0.1:3543".to_string(),
        "http://tauri.localhost".to_string(),
        "https://tauri.localhost".to_string(),
        "tauri://localhost".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::sync::{Mutex, OnceLock};

    use super::{
        ChatRuntimeConfig, DatabricksRuntimeConfig, LocalAsrAuthMode, LocalAsrRuntimeConfig,
        OllamaRuntimeConfig, SearchRuntimeConfig, SecurityRuntimeConfig,
    };

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    const OLLAMA_ENV_KEYS: &[&str] = &[
        "OLLAMA_URL",
        "OLLAMA_API_KEY",
        "OLLAMA_SUMMARY_MODEL",
        "OLLAMA_DEFAULT_CHAT_MODEL",
        "OLLAMA_FALLBACK_MODEL",
        "OLLAMA_CLOUD_COOLDOWN_SECS",
        "SUMMARY_EVALUATOR_MODEL",
        "OLLAMA_EMBEDDING_MODEL",
    ];
    const SECURITY_ENV_KEYS: &[&str] = &[
        "BACKEND_PROXY_TOKEN",
        "BACKEND_CORS_ALLOWED_ORIGINS",
        "FIREBASE_PROJECT_ID",
        "PUBLIC_FIREBASE_PROJECT_ID",
        "GCP_PROJECT_ID",
        "GOOGLE_CLOUD_PROJECT",
        "OPERATOR_EMAIL_ALLOWLIST",
        "DEFAULT_SEEDED_CHANNEL_ID",
        "DEFAULT_SEEDED_CHANNEL_IDS",
        "BASELINE_RATE_LIMIT_PER_MINUTE",
        "EXPENSIVE_RATE_LIMIT_PER_MINUTE",
        "ANONYMOUS_CHAT_QUOTA",
    ];
    const DATABRICKS_ENV_KEYS: &[&str] = &[
        "DATABRICKS_HOST",
        "DATABRICKS_TOKEN",
        "DATABRICKS_WAREHOUSE_ID",
        "DATABRICKS_CATALOG",
        "DATABRICKS_SCHEMA",
        "DATABRICKS_BRONZE_TABLE",
    ];
    const LOCAL_ASR_ENV_KEYS: &[&str] = &[
        "LOCAL_ASR_ENABLED",
        "LOCAL_ASR_BASE_URL",
        "LOCAL_ASR_API_KEY",
        "LOCAL_ASR_AUTH_MODE",
        "LOCAL_ASR_MODEL",
        "LOCAL_ASR_MAX_AUDIO_BYTES",
        "LOCAL_ASR_TIMEOUT_SECS",
    ];

    #[test]
    fn local_asr_from_env_is_disabled_by_default() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(LOCAL_ASR_ENV_KEYS);
        for key in LOCAL_ASR_ENV_KEYS {
            remove_env(key);
        }

        assert_eq!(LocalAsrRuntimeConfig::from_env(), None);
    }

    #[test]
    fn local_asr_from_env_defaults_to_whisper_endpoint_when_enabled() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(LOCAL_ASR_ENV_KEYS);
        for key in LOCAL_ASR_ENV_KEYS {
            remove_env(key);
        }
        set_env("LOCAL_ASR_ENABLED", "true");

        let config = LocalAsrRuntimeConfig::from_env().expect("local ASR should be enabled");

        assert_eq!(config.base_url, "http://127.0.0.1:5092/v1");
        assert_eq!(config.auth_mode, LocalAsrAuthMode::ApiKey);
        assert_eq!(config.model, "whisper-base.en");
        assert_eq!(
            config.transcription_url(),
            "http://127.0.0.1:5092/v1/audio/transcriptions"
        );
        assert_eq!(config.audience_url(), "http://127.0.0.1:5092");
    }

    #[test]
    fn local_asr_from_env_supports_google_identity_auth() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(LOCAL_ASR_ENV_KEYS);
        for key in LOCAL_ASR_ENV_KEYS {
            remove_env(key);
        }
        set_env("LOCAL_ASR_ENABLED", "true");
        set_env("LOCAL_ASR_AUTH_MODE", "google_id_token");
        set_env("LOCAL_ASR_BASE_URL", "https://asr.example.run.app/v1");

        let config = LocalAsrRuntimeConfig::from_env().expect("local ASR should be enabled");

        assert_eq!(config.auth_mode, LocalAsrAuthMode::GoogleIdToken);
        assert_eq!(config.audience_url(), "https://asr.example.run.app");
    }
    #[test]
    fn from_env_requires_summary_model() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        remove_env("OLLAMA_URL");
        remove_env("OLLAMA_API_KEY");
        remove_env("OLLAMA_SUMMARY_MODEL");
        set_env("SUMMARY_EVALUATOR_MODEL", "glm-5.1:cloud");

        let err = OllamaRuntimeConfig::from_env(true).expect_err("missing model should fail");
        assert!(err.contains("OLLAMA_SUMMARY_MODEL"));
    }

    #[test]
    fn from_env_requires_summary_evaluator_model() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        remove_env("OLLAMA_URL");
        remove_env("OLLAMA_API_KEY");
        set_env("OLLAMA_SUMMARY_MODEL", "glm-5.1:cloud");
        remove_env("SUMMARY_EVALUATOR_MODEL");

        let err = OllamaRuntimeConfig::from_env(true).expect_err("missing evaluator should fail");
        assert!(err.contains("SUMMARY_EVALUATOR_MODEL"));
    }

    #[test]
    fn from_env_requires_embedding_model_when_semantic_search_is_enabled() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        remove_env("OLLAMA_URL");
        remove_env("OLLAMA_API_KEY");
        set_env("OLLAMA_SUMMARY_MODEL", "glm-5.1:cloud");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        remove_env("OLLAMA_EMBEDDING_MODEL");

        let err = OllamaRuntimeConfig::from_env(true)
            .expect_err("missing embedding model should fail when semantic search is enabled");
        assert!(err.contains("OLLAMA_EMBEDDING_MODEL"));
    }

    #[test]
    fn from_env_allows_missing_embedding_model_when_semantic_search_is_disabled() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        remove_env("OLLAMA_URL");
        remove_env("OLLAMA_API_KEY");
        set_env("OLLAMA_SUMMARY_MODEL", "glm-5.1:cloud");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        remove_env("OLLAMA_EMBEDDING_MODEL");

        let config = OllamaRuntimeConfig::from_env(false).expect("config");
        assert_eq!(config.embedding_model, None);
    }

    #[test]
    fn from_env_treats_blank_fallback_as_none() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        remove_env("OLLAMA_URL");
        remove_env("OLLAMA_API_KEY");
        set_env("OLLAMA_SUMMARY_MODEL", "glm-5.1:cloud");
        set_env("OLLAMA_FALLBACK_MODEL", "   ");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        set_env("OLLAMA_EMBEDDING_MODEL", "embeddinggemma");

        let config = OllamaRuntimeConfig::from_env(true).expect("config");
        assert_eq!(config.fallback_model, None);
        assert_eq!(
            config.cloud_cooldown_secs,
            crate::services::http::DEFAULT_CLOUD_COOLDOWN_DURATION.as_secs()
        );
    }

    #[test]
    fn from_env_loads_models_from_environment() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        remove_env("OLLAMA_URL");
        remove_env("OLLAMA_API_KEY");
        set_env("OLLAMA_SUMMARY_MODEL", "glm-5.1:cloud");
        set_env("OLLAMA_FALLBACK_MODEL", "qwen3-coder:30b");
        set_env("OLLAMA_CLOUD_COOLDOWN_SECS", "12345");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        set_env("OLLAMA_EMBEDDING_MODEL", "embeddinggemma");

        let config = OllamaRuntimeConfig::from_env(true).expect("config");
        assert_eq!(config.summary_model, "glm-5.1:cloud");
        assert_eq!(config.default_chat_model, None);
        assert_eq!(config.fallback_model.as_deref(), Some("qwen3-coder:30b"));
        assert_eq!(config.cloud_cooldown_secs, 12345);
        assert_eq!(config.summary_evaluator_model, "qwen3.5:397b-cloud");
        assert_eq!(config.embedding_model.as_deref(), Some("embeddinggemma"));
    }

    #[test]
    fn security_from_env_uses_local_defaults_for_dev() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(SECURITY_ENV_KEYS);
        remove_env("BACKEND_PROXY_TOKEN");
        remove_env("BACKEND_CORS_ALLOWED_ORIGINS");
        remove_env("FIREBASE_PROJECT_ID");
        remove_env("PUBLIC_FIREBASE_PROJECT_ID");
        remove_env("GCP_PROJECT_ID");
        remove_env("GOOGLE_CLOUD_PROJECT");
        remove_env("OPERATOR_EMAIL_ALLOWLIST");
        remove_env("DEFAULT_SEEDED_CHANNEL_ID");
        remove_env("DEFAULT_SEEDED_CHANNEL_IDS");
        remove_env("BASELINE_RATE_LIMIT_PER_MINUTE");
        remove_env("EXPENSIVE_RATE_LIMIT_PER_MINUTE");

        let config = SecurityRuntimeConfig::from_env().expect("security config");
        assert_eq!(config.proxy_token, "local-dev-backend-proxy-token");
        assert_eq!(config.firebase_project_id, "demo-dastill");
        assert_eq!(config.default_seeded_channel_id, "UCbRP3c757lWg9M-U7TyEkXA");
        assert_eq!(
            config.default_seeded_channel_ids,
            vec![
                "UCbRP3c757lWg9M-U7TyEkXA".to_string(),
                "podcast:rss:https-feeds-simplecast-com-6hkohngs".to_string()
            ]
        );
        assert_eq!(config.baseline_rate_limit_per_minute, 600);
        assert_eq!(config.expensive_rate_limit_per_minute, 120);
        assert_eq!(config.anonymous_chat_quota, 30);
        assert!(
            config
                .allowed_origins
                .contains(&"http://localhost:3543".to_string())
        );
    }

    #[test]
    fn security_from_env_honors_configured_values() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(SECURITY_ENV_KEYS);
        set_env("BACKEND_PROXY_TOKEN", "proxy-secret");
        set_env(
            "BACKEND_CORS_ALLOWED_ORIGINS",
            "https://app.example.com,https://ops.example.com",
        );
        set_env("FIREBASE_PROJECT_ID", "prod-project");
        set_env(
            "OPERATOR_EMAIL_ALLOWLIST",
            "operator@example.com, OWNER@example.com ",
        );
        set_env("DEFAULT_SEEDED_CHANNEL_ID", "seeded-channel-123");
        set_env("BASELINE_RATE_LIMIT_PER_MINUTE", "90");
        set_env("EXPENSIVE_RATE_LIMIT_PER_MINUTE", "7");
        set_env("ANONYMOUS_CHAT_QUOTA", "12");

        let config = SecurityRuntimeConfig::from_env().expect("security config");
        assert_eq!(config.proxy_token, "proxy-secret");
        assert_eq!(config.firebase_project_id, "prod-project");
        assert_eq!(config.default_seeded_channel_id, "seeded-channel-123");
        assert_eq!(
            config.default_seeded_channel_ids,
            vec!["seeded-channel-123".to_string()]
        );
        assert_eq!(
            config.allowed_origins,
            vec![
                "https://app.example.com".to_string(),
                "https://ops.example.com".to_string()
            ]
        );
        assert_eq!(
            config.operator_email_allowlist,
            vec![
                "operator@example.com".to_string(),
                "owner@example.com".to_string()
            ]
        );
        assert_eq!(config.baseline_rate_limit_per_minute, 90);
        assert_eq!(config.expensive_rate_limit_per_minute, 7);
        assert_eq!(config.anonymous_chat_quota, 12);
    }

    #[test]
    fn security_from_env_supports_multiple_seeded_channels() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(SECURITY_ENV_KEYS);
        set_env("BACKEND_PROXY_TOKEN", "proxy-secret");
        set_env(
            "DEFAULT_SEEDED_CHANNEL_IDS",
            "youtube-seed,podcast:rss:podcast-seed",
        );
        remove_env("DEFAULT_SEEDED_CHANNEL_ID");

        let config = SecurityRuntimeConfig::from_env().expect("security config");

        assert_eq!(config.default_seeded_channel_id, "youtube-seed");
        assert_eq!(
            config.default_seeded_channel_ids,
            vec![
                "youtube-seed".to_string(),
                "podcast:rss:podcast-seed".to_string()
            ]
        );
    }

    #[test]
    fn from_env_loads_optional_default_chat_model() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        remove_env("OLLAMA_URL");
        remove_env("OLLAMA_API_KEY");
        set_env("OLLAMA_SUMMARY_MODEL", "glm-5.1:cloud");
        set_env("OLLAMA_DEFAULT_CHAT_MODEL", "qwen3-chat:latest");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        set_env("OLLAMA_EMBEDDING_MODEL", "embeddinggemma");

        let config = OllamaRuntimeConfig::from_env(true).expect("config");
        assert_eq!(
            config.default_chat_model.as_deref(),
            Some("qwen3-chat:latest")
        );
    }

    #[test]
    fn from_env_rejects_matching_summary_and_evaluator_models() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        remove_env("OLLAMA_URL");
        remove_env("OLLAMA_API_KEY");
        set_env("OLLAMA_SUMMARY_MODEL", "qwen3.5:397b-cloud");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        set_env("OLLAMA_EMBEDDING_MODEL", "embeddinggemma");

        let err = OllamaRuntimeConfig::from_env(true)
            .expect_err("matching summary and evaluator models should fail");
        assert!(err.contains("OLLAMA_SUMMARY_MODEL"));
        assert!(err.contains("SUMMARY_EVALUATOR_MODEL"));
    }

    #[test]
    fn search_runtime_config_defaults_vector_index_creation_off() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset =
            EnvReset::capture(&["SEARCH_AUTO_CREATE_VECTOR_INDEX", "SEARCH_SEMANTIC_ENABLED"]);
        remove_env("SEARCH_AUTO_CREATE_VECTOR_INDEX");
        remove_env("SEARCH_SEMANTIC_ENABLED");

        let config = SearchRuntimeConfig::from_env();
        assert!(!config.auto_create_vector_index);
        assert_eq!(config.semantic_enabled, cfg!(debug_assertions));
    }

    #[test]
    fn search_runtime_config_reads_boolean_flag() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset =
            EnvReset::capture(&["SEARCH_AUTO_CREATE_VECTOR_INDEX", "SEARCH_SEMANTIC_ENABLED"]);
        set_env("SEARCH_AUTO_CREATE_VECTOR_INDEX", "true");
        set_env("SEARCH_SEMANTIC_ENABLED", "true");

        let config = SearchRuntimeConfig::from_env();
        assert!(config.auto_create_vector_index);
        assert!(config.semantic_enabled);
    }

    #[test]
    fn search_runtime_config_respects_explicit_disable() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset =
            EnvReset::capture(&["SEARCH_AUTO_CREATE_VECTOR_INDEX", "SEARCH_SEMANTIC_ENABLED"]);
        remove_env("SEARCH_AUTO_CREATE_VECTOR_INDEX");
        set_env("SEARCH_SEMANTIC_ENABLED", "false");

        let config = SearchRuntimeConfig::from_env();
        assert!(!config.auto_create_vector_index);
        assert!(!config.semantic_enabled);
    }

    #[test]
    fn chat_runtime_config_defaults_multi_pass_on() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(&[
            "CHAT_MULTI_PASS_ENABLED",
            "CHAT_GUARDRAIL_MODEL",
            "CHAT_PROMPT_BLOCKLIST",
            "CHAT_PROMPT_ALLOWLIST",
        ]);
        remove_env("CHAT_MULTI_PASS_ENABLED");
        remove_env("CHAT_GUARDRAIL_MODEL");
        remove_env("CHAT_PROMPT_BLOCKLIST");
        remove_env("CHAT_PROMPT_ALLOWLIST");

        let config = ChatRuntimeConfig::from_env();
        assert!(config.multi_pass_enabled);
        assert_eq!(config.guardrail_model, None);
        assert!(config.prompt_blocklist.is_empty());
        assert!(config.prompt_allowlist.is_empty());
    }

    #[test]
    fn chat_runtime_config_respects_explicit_disable() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(&[
            "CHAT_MULTI_PASS_ENABLED",
            "CHAT_GUARDRAIL_MODEL",
            "CHAT_PROMPT_BLOCKLIST",
            "CHAT_PROMPT_ALLOWLIST",
        ]);
        set_env("CHAT_MULTI_PASS_ENABLED", "false");
        set_env("CHAT_GUARDRAIL_MODEL", "llama-guard:8b");
        set_env(
            "CHAT_PROMPT_BLOCKLIST",
            "ignore previous instructions,reveal system prompt",
        );
        set_env(
            "CHAT_PROMPT_ALLOWLIST",
            "security training,prompt injection examples",
        );

        let config = ChatRuntimeConfig::from_env();
        assert!(!config.multi_pass_enabled);
        assert_eq!(config.guardrail_model.as_deref(), Some("llama-guard:8b"));
        assert_eq!(
            config.prompt_blocklist,
            vec![
                "ignore previous instructions".to_string(),
                "reveal system prompt".to_string()
            ]
        );
        assert_eq!(
            config.prompt_allowlist,
            vec![
                "security training".to_string(),
                "prompt injection examples".to_string()
            ]
        );
    }

    #[test]
    fn from_env_rejects_remote_url_without_api_key() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        set_env("OLLAMA_URL", "https://ollama.cloud.example.com");
        remove_env("OLLAMA_API_KEY");
        set_env("OLLAMA_SUMMARY_MODEL", "glm-5.1:cloud");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        set_env("OLLAMA_EMBEDDING_MODEL", "embeddinggemma");

        let err = OllamaRuntimeConfig::from_env(true)
            .expect_err("remote URL without API key should fail");
        assert!(err.contains("OLLAMA_API_KEY"));
    }

    #[test]
    fn from_env_accepts_remote_url_with_api_key() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        set_env("OLLAMA_URL", "https://ollama.cloud.example.com");
        set_env("OLLAMA_API_KEY", "sk-test-key");
        set_env("OLLAMA_SUMMARY_MODEL", "glm-5.1:cloud");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        set_env("OLLAMA_EMBEDDING_MODEL", "embeddinggemma");

        let config = OllamaRuntimeConfig::from_env(true).expect("config");
        assert_eq!(config.api_key.as_deref(), Some("sk-test-key"));
    }

    #[test]
    fn from_env_allows_localhost_without_api_key() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        set_env("OLLAMA_URL", "http://localhost:11434");
        remove_env("OLLAMA_API_KEY");
        set_env("OLLAMA_SUMMARY_MODEL", "glm-5.1:cloud");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        set_env("OLLAMA_EMBEDDING_MODEL", "embeddinggemma");

        OllamaRuntimeConfig::from_env(true).expect("localhost without API key should succeed");
    }

    #[test]
    fn is_local_url_recognizes_local_addresses() {
        use super::is_local_url;

        assert!(is_local_url("http://localhost:11434"));
        assert!(is_local_url("http://127.0.0.1:11434"));
        assert!(is_local_url("http://0.0.0.0:11434"));
        assert!(!is_local_url("https://ollama.cloud.example.com"));
        assert!(!is_local_url("http://10.0.0.5:11434"));
    }

    #[test]
    fn databricks_config_is_optional_when_unset() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(DATABRICKS_ENV_KEYS);
        for key in DATABRICKS_ENV_KEYS {
            remove_env(key);
        }

        let config = DatabricksRuntimeConfig::from_env().expect("config parse");
        assert!(config.is_none());
    }

    #[test]
    fn databricks_config_requires_complete_credentials() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(DATABRICKS_ENV_KEYS);
        set_env("DATABRICKS_HOST", "https://dbc.example.com");
        remove_env("DATABRICKS_TOKEN");
        set_env("DATABRICKS_WAREHOUSE_ID", "warehouse-123");

        let err = DatabricksRuntimeConfig::from_env().expect_err("missing token should fail");
        assert!(err.contains("DATABRICKS_TOKEN"));
    }

    #[test]
    fn databricks_config_uses_defaults_for_catalog_schema_and_table() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(DATABRICKS_ENV_KEYS);
        set_env("DATABRICKS_HOST", "https://dbc.example.com");
        set_env("DATABRICKS_TOKEN", "dapi-test");
        set_env("DATABRICKS_WAREHOUSE_ID", "warehouse-123");
        remove_env("DATABRICKS_CATALOG");
        remove_env("DATABRICKS_SCHEMA");
        remove_env("DATABRICKS_BRONZE_TABLE");

        let config = DatabricksRuntimeConfig::from_env()
            .expect("config parse")
            .expect("config should be present");
        assert_eq!(config.catalog, "workspace");
        assert_eq!(config.schema, "sandbox");
        assert_eq!(config.bronze_table, "bronze_app_events");
    }

    struct EnvReset {
        saved: Vec<(String, Option<String>)>,
    }

    impl EnvReset {
        fn capture(keys: &[&str]) -> Self {
            let saved = keys
                .iter()
                .map(|key| ((*key).to_string(), env::var(key).ok()))
                .collect();
            Self { saved }
        }
    }

    impl Drop for EnvReset {
        fn drop(&mut self) {
            for (key, value) in &self.saved {
                match value {
                    Some(value) => set_env(key, value),
                    None => remove_env(key),
                }
            }
        }
    }

    fn set_env(key: &str, value: &str) {
        // SAFETY: test access is serialized with ENV_LOCK in this module.
        unsafe { env::set_var(key, value) };
    }

    fn remove_env(key: &str) {
        // SAFETY: test access is serialized with ENV_LOCK in this module.
        unsafe { env::remove_var(key) };
    }
}
