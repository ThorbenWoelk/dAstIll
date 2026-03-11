use reqwest::Client;
use rig::client::Nothing;
use rig::providers::ollama;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::models::AiStatus;
use crate::services::http::{CloudCooldown, build_http_client, is_cloud_model};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CooldownStatusPolicy {
    UseLocalFallback,
    Offline,
}

/// Shared configuration and low-level helpers for Ollama-backed services.
pub(crate) struct OllamaCore {
    client: Client,
    base_url: String,
    model: String,
    fallback_model: Option<String>,
    cloud_cooldown: Option<Arc<CloudCooldown>>,
    ollama_semaphore: Option<Arc<Semaphore>>,
}

impl OllamaCore {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            client: build_http_client(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            fallback_model: None,
            cloud_cooldown: None,
            ollama_semaphore: None,
        }
    }

    pub fn with_client(client: Client, base_url: &str, model: &str) -> Self {
        Self {
            client,
            base_url: base_url.to_string(),
            model: model.to_string(),
            fallback_model: None,
            cloud_cooldown: None,
            ollama_semaphore: None,
        }
    }

    pub fn with_fallback_model(mut self, model: Option<String>) -> Self {
        self.fallback_model = model;
        self
    }

    pub fn with_cloud_cooldown(mut self, cooldown: Arc<CloudCooldown>) -> Self {
        self.cloud_cooldown = Some(cooldown);
        self
    }

    pub fn with_ollama_semaphore(mut self, semaphore: Arc<Semaphore>) -> Self {
        self.ollama_semaphore = Some(semaphore);
        self
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn fallback_model(&self) -> Option<&str> {
        self.fallback_model.as_deref()
    }

    pub fn cloud_cooldown(&self) -> Option<&Arc<CloudCooldown>> {
        self.cloud_cooldown.as_ref()
    }

    pub fn ollama_semaphore(&self) -> Option<&Arc<Semaphore>> {
        self.ollama_semaphore.as_ref()
    }

    pub fn uses_cloud_model(&self) -> bool {
        is_cloud_model(self.model())
    }

    pub async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub fn is_cloud_cooldown_active(&self) -> bool {
        self.uses_cloud_model() && self.cloud_cooldown().is_some_and(|cd| cd.is_active())
    }

    pub fn activate_cloud_cooldown(&self) {
        if self.uses_cloud_model() {
            if let Some(cooldown) = self.cloud_cooldown() {
                cooldown.activate();
            }
        }
    }

    pub fn indicator_status(
        &self,
        cloud_cooldown_active: bool,
        endpoint_available: bool,
        cooldown_status_policy: CooldownStatusPolicy,
    ) -> AiStatus {
        if !endpoint_available {
            return AiStatus::Offline;
        }
        if !self.uses_cloud_model() {
            return AiStatus::LocalOnly;
        }
        if !cloud_cooldown_active {
            return AiStatus::Cloud;
        }
        match (cooldown_status_policy, self.fallback_model()) {
            (CooldownStatusPolicy::UseLocalFallback, Some(fallback_model))
                if !is_cloud_model(fallback_model) =>
            {
                AiStatus::LocalOnly
            }
            _ => AiStatus::Offline,
        }
    }

    pub fn build_ollama_client(&self) -> Result<ollama::Client, String> {
        ollama::Client::builder()
            .api_key(Nothing)
            .base_url(&self.base_url)
            .build()
            .map_err(|err| err.to_string())
    }

    /// Acquire the local-model semaphore if `model` is not a cloud model.
    pub async fn acquire_local_permit(
        &self,
        model: &str,
    ) -> Result<Option<OwnedSemaphorePermit>, String> {
        if !is_cloud_model(model) {
            if let Some(sem) = self.ollama_semaphore() {
                return Ok(Some(
                    sem.clone()
                        .acquire_owned()
                        .await
                        .map_err(|err| err.to_string())?,
                ));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::Semaphore;

    use super::{CooldownStatusPolicy, OllamaCore};
    use crate::models::AiStatus;
    use crate::services::http::Cooldown;

    #[test]
    fn indicator_status_uses_local_fallback_when_policy_allows_it() {
        let core = OllamaCore::new("http://localhost:11434", "glm-5:cloud")
            .with_fallback_model(Some("qwen3-coder:30b".to_string()));

        assert_eq!(
            core.indicator_status(true, true, CooldownStatusPolicy::UseLocalFallback),
            AiStatus::LocalOnly
        );
    }

    #[test]
    fn indicator_status_reports_offline_when_policy_disallows_local_fallback() {
        let core = OllamaCore::new("http://localhost:11434", "glm-5:cloud")
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
}
