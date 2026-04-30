use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;

use super::{CLOUD_PROMPT_TIMEOUT_SECS, CooldownStatusPolicy, OllamaCore};
use crate::models::AiStatus;
use crate::services::http::Cooldown;

#[test]
fn indicator_status_uses_local_fallback_when_policy_allows_it() {
    let core = OllamaCore::new("http://localhost:11434", "glm-5.1:cloud")
        .with_fallback_model(Some("qwen3-coder:30b".to_string()));

    assert_eq!(
        core.indicator_status(true, true, CooldownStatusPolicy::UseLocalFallback),
        AiStatus::LocalOnly
    );
}

#[test]
fn indicator_status_reports_offline_when_policy_disallows_local_fallback() {
    let core = OllamaCore::new("http://localhost:11434", "glm-5.1:cloud")
        .with_fallback_model(Some("qwen3-coder:30b".to_string()));

    assert_eq!(
        core.indicator_status(true, true, CooldownStatusPolicy::Offline),
        AiStatus::Offline
    );
}

#[test]
fn builder_methods_store_shared_runtime_dependencies() {
    let cooldown = Arc::new(Cooldown::cloud());
    let semaphore = Arc::new(Semaphore::new(1));

    let core = OllamaCore::new("http://localhost:11434", "qwen3-coder:30b")
        .with_cloud_cooldown(cooldown.clone())
        .with_ollama_semaphore(semaphore.clone());

    assert!(core.cloud_cooldown().is_some());
    assert!(core.ollama_semaphore().is_some());
    assert_eq!(core.base_url(), "http://localhost:11434");
    assert_eq!(core.model(), "qwen3-coder:30b");
}

#[test]
fn cloud_cooldown_defers_without_fallback_model() {
    let cooldown = Arc::new(Cooldown::cloud_with_duration(Duration::from_secs(60)));
    let core = OllamaCore::new("http://localhost:11434", "glm-5.1:cloud")
        .with_cloud_cooldown(cooldown.clone());

    assert!(!core.defers_without_fallback_during_cloud_cooldown());
    cooldown.activate();
    assert!(core.defers_without_fallback_during_cloud_cooldown());
}

#[test]
fn cloud_cooldown_does_not_defer_when_fallback_model_exists() {
    let cooldown = Arc::new(Cooldown::cloud_with_duration(Duration::from_secs(60)));
    cooldown.activate();
    let core = OllamaCore::new("http://localhost:11434", "glm-5.1:cloud")
        .with_fallback_model(Some("qwen3-coder:8b".to_string()))
        .with_cloud_cooldown(cooldown);

    assert!(!core.defers_without_fallback_during_cloud_cooldown());
}

#[test]
fn build_ollama_client_succeeds_without_api_key() {
    let core = OllamaCore::new("http://localhost:11434", "qwen3-coder:30b");
    assert!(core.build_ollama_client().is_ok());
}

#[test]
fn build_ollama_client_succeeds_with_api_key() {
    let core = OllamaCore::new("https://cloud.example.com", "glm-5.1:cloud")
        .with_api_key(Some("test-key-123".to_string()));
    assert!(core.build_ollama_client().is_ok());
}

#[test]
fn cloud_prompt_timeout_matches_production_request_budget() {
    assert_eq!(CLOUD_PROMPT_TIMEOUT_SECS, 300);
}

#[test]
fn auth_adds_bearer_header_when_api_key_is_set() {
    let core = OllamaCore::new("http://localhost:11434", "qwen3-coder:30b")
        .with_api_key(Some("test-key".to_string()));
    let client = reqwest::Client::new();
    let req = core.auth(client.get("http://localhost:11434/api/tags"));
    let built = req.build().expect("request should build");
    let auth = built
        .headers()
        .get(reqwest::header::AUTHORIZATION)
        .expect("should have Authorization header");
    assert_eq!(auth.to_str().unwrap(), "Bearer test-key");
}

#[test]
fn auth_omits_header_when_no_api_key() {
    let core = OllamaCore::new("http://localhost:11434", "qwen3-coder:30b");
    let client = reqwest::Client::new();
    let req = core.auth(client.get("http://localhost:11434/api/tags"));
    let built = req.build().expect("request should build");
    assert!(
        built
            .headers()
            .get(reqwest::header::AUTHORIZATION)
            .is_none()
    );
}

#[test]
fn structured_response_parser_rejects_wrapped_json() {
    #[derive(Debug, serde::Deserialize)]
    struct Payload {
        #[serde(rename = "value")]
        _value: String,
    }

    let err = super::parse_structured_response::<Payload>("```json\n{\"value\":\"ok\"}\n```")
        .expect_err("schema-backed structured responses should be strict JSON");
    assert!(format!("{err:?}").contains("failed to decode structured response"));
}

#[test]
fn structured_response_parser_decodes_plain_json() {
    #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
    struct Payload {
        value: String,
    }

    let parsed =
        super::parse_structured_response::<Payload>("{\"value\":\"ok\"}").expect("valid json");
    assert_eq!(
        parsed,
        Payload {
            value: "ok".to_string()
        }
    );
}
