use std::env;

use crate::services::SummaryEvaluatorService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OllamaRuntimeConfig {
    pub url: String,
    pub model: String,
    pub fallback_model: Option<String>,
    pub summary_evaluator_model: String,
}

impl OllamaRuntimeConfig {
    pub fn from_env() -> Result<Self, String> {
        let url = env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model = required_env("OLLAMA_MODEL")?;
        let fallback_model = optional_env("OLLAMA_FALLBACK_MODEL");
        let summary_evaluator_model = required_env("SUMMARY_EVALUATOR_MODEL")?;

        SummaryEvaluatorService::validate_model_policy(&summary_evaluator_model)?;

        Ok(Self {
            url,
            model,
            fallback_model,
            summary_evaluator_model,
        })
    }
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

#[cfg(test)]
mod tests {
    use std::env;
    use std::sync::{Mutex, OnceLock};

    use super::OllamaRuntimeConfig;

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
        ]);
        remove_env("OLLAMA_MODEL");
        set_env("SUMMARY_EVALUATOR_MODEL", "glm-5:cloud");

        let err = OllamaRuntimeConfig::from_env().expect_err("missing model should fail");
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
        ]);
        set_env("OLLAMA_MODEL", "glm-5:cloud");
        remove_env("SUMMARY_EVALUATOR_MODEL");

        let err = OllamaRuntimeConfig::from_env().expect_err("missing evaluator should fail");
        assert!(err.contains("SUMMARY_EVALUATOR_MODEL"));
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
        ]);
        set_env("OLLAMA_MODEL", "glm-5:cloud");
        set_env("OLLAMA_FALLBACK_MODEL", "   ");
        set_env("SUMMARY_EVALUATOR_MODEL", "glm-5:cloud");

        let config = OllamaRuntimeConfig::from_env().expect("config");
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
        ]);
        set_env("OLLAMA_MODEL", "glm-5:cloud");
        set_env("OLLAMA_FALLBACK_MODEL", "qwen3-coder:30b");
        set_env("SUMMARY_EVALUATOR_MODEL", "glm-5:cloud");

        let config = OllamaRuntimeConfig::from_env().expect("config");
        assert_eq!(config.model, "glm-5:cloud");
        assert_eq!(config.fallback_model.as_deref(), Some("qwen3-coder:30b"));
        assert_eq!(config.summary_evaluator_model, "glm-5:cloud");
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
