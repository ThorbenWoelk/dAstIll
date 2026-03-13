use std::env;

use crate::services::SummaryEvaluatorService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OllamaRuntimeConfig {
    pub url: String,
    pub model: String,
    pub fallback_model: Option<String>,
    pub summary_evaluator_model: String,
    pub embedding_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRuntimeConfig {
    pub auto_create_vector_index: bool,
    pub semantic_enabled: bool,
}

impl OllamaRuntimeConfig {
    pub fn from_env(search_semantic_enabled: bool) -> Result<Self, String> {
        let url = env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model = required_env("OLLAMA_MODEL")?;
        let fallback_model = optional_env("OLLAMA_FALLBACK_MODEL");
        let summary_evaluator_model = required_env("SUMMARY_EVALUATOR_MODEL")?;
        let embedding_model = if search_semantic_enabled {
            Some(required_env("OLLAMA_EMBEDDING_MODEL")?)
        } else {
            optional_env("OLLAMA_EMBEDDING_MODEL")
        };

        SummaryEvaluatorService::validate_model_policy(&summary_evaluator_model)?;

        Ok(Self {
            url,
            model,
            fallback_model,
            summary_evaluator_model,
            embedding_model,
        })
    }
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

fn optional_bool_env(key: &str) -> Option<bool> {
    optional_env(key).map(|value| {
        matches!(
            value.as_str(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        )
    })
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::sync::{Mutex, OnceLock};

    use super::{OllamaRuntimeConfig, SearchRuntimeConfig};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[test]
    fn from_env_requires_primary_model() {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());

        let _reset = EnvReset::capture(&[
            "OLLAMA_MODEL",
            "OLLAMA_FALLBACK_MODEL",
            "SUMMARY_EVALUATOR_MODEL",
            "OLLAMA_EMBEDDING_MODEL",
        ]);
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

        let _reset = EnvReset::capture(&[
            "OLLAMA_MODEL",
            "OLLAMA_FALLBACK_MODEL",
            "SUMMARY_EVALUATOR_MODEL",
            "OLLAMA_EMBEDDING_MODEL",
        ]);
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

        let _reset = EnvReset::capture(&[
            "OLLAMA_MODEL",
            "OLLAMA_FALLBACK_MODEL",
            "SUMMARY_EVALUATOR_MODEL",
            "OLLAMA_EMBEDDING_MODEL",
        ]);
        set_env("OLLAMA_MODEL", "glm-5:cloud");
        set_env("SUMMARY_EVALUATOR_MODEL", "glm-5:cloud");
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

        let _reset = EnvReset::capture(&[
            "OLLAMA_MODEL",
            "OLLAMA_FALLBACK_MODEL",
            "SUMMARY_EVALUATOR_MODEL",
            "OLLAMA_EMBEDDING_MODEL",
        ]);
        set_env("OLLAMA_MODEL", "glm-5:cloud");
        set_env("SUMMARY_EVALUATOR_MODEL", "glm-5:cloud");
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

        let _reset = EnvReset::capture(&[
            "OLLAMA_MODEL",
            "OLLAMA_FALLBACK_MODEL",
            "SUMMARY_EVALUATOR_MODEL",
            "OLLAMA_EMBEDDING_MODEL",
        ]);
        set_env("OLLAMA_MODEL", "glm-5:cloud");
        set_env("OLLAMA_FALLBACK_MODEL", "   ");
        set_env("SUMMARY_EVALUATOR_MODEL", "glm-5:cloud");
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

        let _reset = EnvReset::capture(&[
            "OLLAMA_MODEL",
            "OLLAMA_FALLBACK_MODEL",
            "SUMMARY_EVALUATOR_MODEL",
            "OLLAMA_EMBEDDING_MODEL",
        ]);
        set_env("OLLAMA_MODEL", "glm-5:cloud");
        set_env("OLLAMA_FALLBACK_MODEL", "qwen3-coder:30b");
        set_env("SUMMARY_EVALUATOR_MODEL", "glm-5:cloud");
        set_env("OLLAMA_EMBEDDING_MODEL", "embeddinggemma");

        let config = OllamaRuntimeConfig::from_env(true).expect("config");
        assert_eq!(config.model, "glm-5:cloud");
        assert_eq!(config.fallback_model.as_deref(), Some("qwen3-coder:30b"));
        assert_eq!(config.summary_evaluator_model, "glm-5:cloud");
        assert_eq!(config.embedding_model.as_deref(), Some("embeddinggemma"));
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
