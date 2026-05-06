use std::env;
use std::sync::{Mutex, OnceLock};

use super::{
    ChatRuntimeConfig, DatabricksRuntimeConfig, LocalAsrAuthMode, LocalAsrRuntimeConfig,
    OllamaRuntimeConfig, SearchRuntimeConfig, SecurityRuntimeConfig,
};

#[test]
fn is_local_url_recognizes_local_addresses() {
    use super::is_local_url;

    assert!(is_local_url("http://localhost:11434"));
    assert!(is_local_url("http://127.0.0.1:11434"));
    assert!(is_local_url("http://0.0.0.0:11434"));
    assert!(!is_local_url("https://ollama.cloud.example.com"));
    assert!(!is_local_url("http://10.0.0.5:11434"));
}

fn set_env(key: &str, value: &str) {
    // SAFETY: test access is serialized with ENV_LOCK in this module.
    unsafe { env::set_var(key, value) };
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
fn search_runtime_config_reads_boolean_flag() {
    let _guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    let _reset = EnvReset::capture(&["SEARCH_AUTO_CREATE_VECTOR_INDEX", "SEARCH_SEMANTIC_ENABLED"]);
    set_env("SEARCH_AUTO_CREATE_VECTOR_INDEX", "true");
    set_env("SEARCH_SEMANTIC_ENABLED", "true");

    let config = SearchRuntimeConfig::from_env();
    assert!(config.auto_create_vector_index);
    assert!(config.semantic_enabled);
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

fn remove_env(key: &str) {
    // SAFETY: test access is serialized with ENV_LOCK in this module.
    unsafe { env::remove_var(key) };
}

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

    let _reset = EnvReset::capture(&["SEARCH_AUTO_CREATE_VECTOR_INDEX", "SEARCH_SEMANTIC_ENABLED"]);
    remove_env("SEARCH_AUTO_CREATE_VECTOR_INDEX");
    remove_env("SEARCH_SEMANTIC_ENABLED");

    let config = SearchRuntimeConfig::from_env();
    assert!(!config.auto_create_vector_index);
    assert_eq!(config.semantic_enabled, cfg!(debug_assertions));
}

#[test]
fn search_runtime_config_respects_explicit_disable() {
    let _guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    let _reset = EnvReset::capture(&["SEARCH_AUTO_CREATE_VECTOR_INDEX", "SEARCH_SEMANTIC_ENABLED"]);
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

    let err =
        OllamaRuntimeConfig::from_env(true).expect_err("remote URL without API key should fail");
    assert!(err.contains("OLLAMA_API_KEY"));
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
