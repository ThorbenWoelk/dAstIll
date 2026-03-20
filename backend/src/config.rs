use std::env;

use crate::services::SummaryEvaluatorService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OllamaRuntimeConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub chat_model: Option<String>,
    pub fallback_model: Option<String>,
    pub summary_evaluator_model: String,
    pub embedding_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRuntimeConfig {
    pub auto_create_vector_index: bool,
    pub semantic_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatRuntimeConfig {
    pub multi_pass_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityRuntimeConfig {
    pub proxy_token: String,
    pub allowed_origins: Vec<String>,
    pub baseline_rate_limit_per_minute: u32,
    pub expensive_rate_limit_per_minute: u32,
}

impl OllamaRuntimeConfig {
    pub fn from_env(search_semantic_enabled: bool) -> Result<Self, String> {
        let url = env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let api_key = optional_env("OLLAMA_API_KEY");
        let model = required_env("OLLAMA_MODEL")?;
        let chat_model = optional_env("OLLAMA_CHAT_MODEL");
        let fallback_model = optional_env("OLLAMA_FALLBACK_MODEL");
        let summary_evaluator_model = required_env("SUMMARY_EVALUATOR_MODEL")?;
        let embedding_model = if search_semantic_enabled {
            Some(required_env("OLLAMA_EMBEDDING_MODEL")?)
        } else {
            optional_env("OLLAMA_EMBEDDING_MODEL")
        };

        validate_distinct_model_roles(&model, &summary_evaluator_model)?;
        SummaryEvaluatorService::validate_model_policy(&summary_evaluator_model)?;
        validate_cloud_auth(&url, &api_key)?;

        Ok(Self {
            url,
            api_key,
            model,
            chat_model,
            fallback_model,
            summary_evaluator_model,
            embedding_model,
        })
    }
}

fn validate_distinct_model_roles(model: &str, summary_evaluator_model: &str) -> Result<(), String> {
    if model == summary_evaluator_model {
        return Err(format!(
            "OLLAMA_MODEL and SUMMARY_EVALUATOR_MODEL must differ so summaries are evaluated independently; got `{model}` for both"
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
        }
    }
}

impl SecurityRuntimeConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            proxy_token: required_env_with_local_default(
                "BACKEND_PROXY_TOKEN",
                "local-dev-backend-proxy-token",
            )?,
            allowed_origins: optional_csv_env("BACKEND_CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(default_backend_allowed_origins),
            baseline_rate_limit_per_minute: optional_u32_env("BASELINE_RATE_LIMIT_PER_MINUTE")
                .unwrap_or(60)
                .clamp(1, 1_000),
            expensive_rate_limit_per_minute: optional_u32_env("EXPENSIVE_RATE_LIMIT_PER_MINUTE")
                .unwrap_or(5)
                .clamp(1, 1_000),
        })
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

fn default_backend_allowed_origins() -> Vec<String> {
    vec![
        "http://localhost:3000".to_string(),
        "http://127.0.0.1:3000".to_string(),
        "http://localhost:3543".to_string(),
        "http://127.0.0.1:3543".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::sync::{Mutex, OnceLock};

    use super::{
        ChatRuntimeConfig, OllamaRuntimeConfig, SearchRuntimeConfig, SecurityRuntimeConfig,
    };

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    const OLLAMA_ENV_KEYS: &[&str] = &[
        "OLLAMA_URL",
        "OLLAMA_API_KEY",
        "OLLAMA_MODEL",
        "OLLAMA_CHAT_MODEL",
        "OLLAMA_FALLBACK_MODEL",
        "SUMMARY_EVALUATOR_MODEL",
        "OLLAMA_EMBEDDING_MODEL",
    ];
    const SECURITY_ENV_KEYS: &[&str] = &[
        "BACKEND_PROXY_TOKEN",
        "BACKEND_CORS_ALLOWED_ORIGINS",
        "BASELINE_RATE_LIMIT_PER_MINUTE",
        "EXPENSIVE_RATE_LIMIT_PER_MINUTE",
    ];

    #[test]
    fn from_env_requires_primary_model() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        remove_env("OLLAMA_URL");
        remove_env("OLLAMA_API_KEY");
        remove_env("OLLAMA_MODEL");
        set_env("SUMMARY_EVALUATOR_MODEL", "glm-5:cloud");

        let err = OllamaRuntimeConfig::from_env(true).expect_err("missing model should fail");
        assert!(err.contains("OLLAMA_MODEL"));
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
        set_env("OLLAMA_MODEL", "glm-5:cloud");
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
        set_env("OLLAMA_MODEL", "glm-5:cloud");
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
        set_env("OLLAMA_MODEL", "glm-5:cloud");
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
        set_env("OLLAMA_MODEL", "glm-5:cloud");
        set_env("OLLAMA_FALLBACK_MODEL", "   ");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        set_env("OLLAMA_EMBEDDING_MODEL", "embeddinggemma");

        let config = OllamaRuntimeConfig::from_env(true).expect("config");
        assert_eq!(config.fallback_model, None);
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
        set_env("OLLAMA_MODEL", "glm-5:cloud");
        set_env("OLLAMA_FALLBACK_MODEL", "qwen3-coder:30b");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        set_env("OLLAMA_EMBEDDING_MODEL", "embeddinggemma");

        let config = OllamaRuntimeConfig::from_env(true).expect("config");
        assert_eq!(config.model, "glm-5:cloud");
        assert_eq!(config.chat_model, None);
        assert_eq!(config.fallback_model.as_deref(), Some("qwen3-coder:30b"));
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
        remove_env("BASELINE_RATE_LIMIT_PER_MINUTE");
        remove_env("EXPENSIVE_RATE_LIMIT_PER_MINUTE");

        let config = SecurityRuntimeConfig::from_env().expect("security config");
        assert_eq!(config.proxy_token, "local-dev-backend-proxy-token");
        assert_eq!(config.baseline_rate_limit_per_minute, 60);
        assert_eq!(config.expensive_rate_limit_per_minute, 5);
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
        set_env("BASELINE_RATE_LIMIT_PER_MINUTE", "90");
        set_env("EXPENSIVE_RATE_LIMIT_PER_MINUTE", "7");

        let config = SecurityRuntimeConfig::from_env().expect("security config");
        assert_eq!(config.proxy_token, "proxy-secret");
        assert_eq!(
            config.allowed_origins,
            vec![
                "https://app.example.com".to_string(),
                "https://ops.example.com".to_string()
            ]
        );
        assert_eq!(config.baseline_rate_limit_per_minute, 90);
        assert_eq!(config.expensive_rate_limit_per_minute, 7);
    }

    #[test]
    fn from_env_loads_optional_chat_model() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(OLLAMA_ENV_KEYS);
        remove_env("OLLAMA_URL");
        remove_env("OLLAMA_API_KEY");
        set_env("OLLAMA_MODEL", "glm-5:cloud");
        set_env("OLLAMA_CHAT_MODEL", "qwen3-chat:latest");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        set_env("OLLAMA_EMBEDDING_MODEL", "embeddinggemma");

        let config = OllamaRuntimeConfig::from_env(true).expect("config");
        assert_eq!(config.chat_model.as_deref(), Some("qwen3-chat:latest"));
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
        set_env("OLLAMA_MODEL", "qwen3.5:397b-cloud");
        set_env("SUMMARY_EVALUATOR_MODEL", "qwen3.5:397b-cloud");
        set_env("OLLAMA_EMBEDDING_MODEL", "embeddinggemma");

        let err = OllamaRuntimeConfig::from_env(true)
            .expect_err("matching summary and evaluator models should fail");
        assert!(err.contains("OLLAMA_MODEL"));
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

        let _reset = EnvReset::capture(&["CHAT_MULTI_PASS_ENABLED"]);
        remove_env("CHAT_MULTI_PASS_ENABLED");

        let config = ChatRuntimeConfig::from_env();
        assert!(config.multi_pass_enabled);
    }

    #[test]
    fn chat_runtime_config_respects_explicit_disable() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(&["CHAT_MULTI_PASS_ENABLED"]);
        set_env("CHAT_MULTI_PASS_ENABLED", "false");

        let config = ChatRuntimeConfig::from_env();
        assert!(!config.multi_pass_enabled);
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
        set_env("OLLAMA_MODEL", "glm-5:cloud");
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
        set_env("OLLAMA_MODEL", "glm-5:cloud");
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
        set_env("OLLAMA_MODEL", "glm-5:cloud");
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
