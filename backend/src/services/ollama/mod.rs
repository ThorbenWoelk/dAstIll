use reqwest::Client;
use rig::client::Nothing;
use rig::completion::Prompt;
use rig::prelude::*;
use rig::providers::ollama;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;
use tracing::Instrument;

use crate::models::AiStatus;
use crate::services::http::{
    CloudCooldown, build_http_client, is_cloud_model, is_provider_capacity_limited_message,
    is_rate_limited,
};

pub const CLOUD_PROMPT_TIMEOUT_SECS: u64 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CooldownStatusPolicy {
    UseLocalFallback,
    Offline,
}

/// Error returned by [`OllamaCore::prompt_with_fallback`].
#[derive(Debug)]
pub enum OllamaPromptError {
    /// Cloud cooldown active and no fallback configured, or cancelled by policy.
    NotAvailable,
    RequestFailed(rig::completion::PromptError),
    GenerationFailed(String),
    /// Model returned an empty response.
    EmptyResponse,
    /// Model returned content that could not be decoded into the requested structured type.
    InvalidStructuredResponse(String),
}

#[derive(Debug)]
enum OllamaGenerateCallError {
    RateLimited(String),
    Failed(String),
}

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    system: &'a str,
    prompt: &'a str,
    stream: bool,
    format: &'a Value,
    options: OllamaGenerateOptions,
}

#[derive(Debug, Serialize)]
struct OllamaGenerateOptions {
    temperature: f32,
}

#[derive(Debug, serde::Deserialize)]
struct OllamaGenerateResponse {
    response: Option<String>,
    error: Option<String>,
}

/// Shared configuration and low-level helpers for Ollama-backed services.
#[derive(Clone)]
pub struct OllamaCore {
    client: Client,
    base_url: String,
    model: String,
    fallback_model: Option<String>,
    api_key: Option<String>,
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
            api_key: None,
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
            api_key: None,
            cloud_cooldown: None,
            ollama_semaphore: None,
        }
    }

    pub fn with_fallback_model(mut self, model: Option<String>) -> Self {
        self.fallback_model = model;
        self
    }

    pub fn with_api_key(mut self, key: Option<String>) -> Self {
        self.api_key = key;
        self
    }

    /// Add Authorization header to a request builder if an API key is configured.
    pub fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.api_key {
            Some(key) => req.bearer_auth(key),
            None => req,
        }
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

    pub fn client(&self) -> Client {
        self.client.clone()
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
        self.auth(self.client.get(format!("{}/api/tags", self.base_url)))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub fn is_cloud_cooldown_active(&self) -> bool {
        self.uses_cloud_model() && self.cloud_cooldown().is_some_and(|cd| cd.is_active())
    }

    pub fn defers_without_fallback_during_cloud_cooldown(&self) -> bool {
        self.is_cloud_cooldown_active() && self.fallback_model().is_none()
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
        let builder = ollama::Client::builder()
            .base_url(&self.base_url)
            .api_key(Nothing);

        let builder = if let Some(key) = &self.api_key {
            let mut headers = reqwest::header::HeaderMap::new();
            let val = reqwest::header::HeaderValue::from_str(&format!("Bearer {key}"))
                .map_err(|e| e.to_string())?;
            headers.insert(reqwest::header::AUTHORIZATION, val);
            let http_client = reqwest::Client::builder()
                .user_agent("dastill/0.1")
                .timeout(std::time::Duration::from_secs(CLOUD_PROMPT_TIMEOUT_SECS))
                .default_headers(headers)
                .build()
                .map_err(|e| e.to_string())?;
            builder.http_client(http_client)
        } else {
            builder
        };

        builder.build().map_err(|err| err.to_string())
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

    /// Prompt the configured model with automatic fallback and cooldown handling.
    ///
    /// Returns `(response_text, model_used)`.
    /// `policy` controls whether a local fallback is used or `NotAvailable` is
    /// returned when the cloud model is in cooldown or rate-limited.
    pub async fn prompt_with_fallback(
        &self,
        operation: &str,
        preamble: &str,
        prompt: &str,
        policy: CooldownStatusPolicy,
    ) -> Result<(String, String), OllamaPromptError> {
        let span = logfire::span!(
            "ollama.prompt",
            operation = operation,
            model = self.model().to_string(),
            base_url = self.base_url().to_string(),
            prompt_chars = prompt.chars().count(),
            cooldown_policy = format!("{policy:?}"),
            fallback_configured = self.fallback_model().is_some(),
        );

        async move {
            let started = Instant::now();
            let result: Result<(String, String), OllamaPromptError> = async {
                let ollama_client = self
                    .build_ollama_client()
                    .map_err(OllamaPromptError::GenerationFailed)?;

                let is_cloud = self.uses_cloud_model();
                let cooldown_active = self.is_cloud_cooldown_active();

                let (response, model_used) = if cooldown_active {
                    match policy {
                        CooldownStatusPolicy::UseLocalFallback => {
                            let fallback = self.fallback_model().ok_or_else(|| {
                                OllamaPromptError::NotAvailable
                            })?;
                            tracing::info!(
                                operation = operation,
                                skipped_model = %self.model(),
                                fallback_model = %fallback,
                                "skipping cloud model due to active cooldown"
                            );
                            let _permit = self
                                .acquire_local_permit(fallback)
                                .await
                                .map_err(OllamaPromptError::GenerationFailed)?;
                            let agent = ollama_client.agent(fallback).preamble(preamble).build();
                            let resp = match timeout(
                                std::time::Duration::from_secs(CLOUD_PROMPT_TIMEOUT_SECS),
                                agent.prompt(prompt),
                            )
                            .await
                            {
                                Ok(res) => res.map_err(OllamaPromptError::RequestFailed)?,
                                Err(_) => {
                                    return Err(OllamaPromptError::GenerationFailed(format!(
                                        "Ollama prompt timed out after {CLOUD_PROMPT_TIMEOUT_SECS}s"
                                    )))
                                }
                            };
                            (resp, fallback.to_string())
                        }
                        CooldownStatusPolicy::Offline => return Err(OllamaPromptError::NotAvailable),
                    }
                } else {
                    let _permit = self
                        .acquire_local_permit(self.model())
                        .await
                        .map_err(OllamaPromptError::GenerationFailed)?;
                    let agent = ollama_client.agent(self.model()).preamble(preamble).build();
                    match timeout(
                        std::time::Duration::from_secs(CLOUD_PROMPT_TIMEOUT_SECS),
                        agent.prompt(prompt),
                    )
                    .await
                    {
                        Ok(Ok(resp)) => (resp, self.model().to_string()),
                        Ok(Err(err)) if is_rate_limited(&err) => {
                            if is_cloud {
                                self.activate_cloud_cooldown();
                            }
                            match policy {
                                CooldownStatusPolicy::UseLocalFallback => {
                                    let fallback = self.fallback_model().ok_or_else(|| {
                                        OllamaPromptError::GenerationFailed(format!(
                                            "rate limited by provider and no fallback model configured: {err}"
                                        ))
                                    })?;
                                    tracing::warn!(
                                        operation = operation,
                                        primary_model = %self.model(),
                                        fallback_model = %fallback,
                                        error = %err,
                                        "rate limited - falling back to local model"
                                    );
                                    let _permit = self
                                        .acquire_local_permit(fallback)
                                        .await
                                        .map_err(OllamaPromptError::GenerationFailed)?;
                                    let fallback_agent =
                                        ollama_client.agent(fallback).preamble(preamble).build();
                                    let resp = match timeout(
                                        std::time::Duration::from_secs(CLOUD_PROMPT_TIMEOUT_SECS),
                                        fallback_agent.prompt(prompt),
                                    )
                                    .await
                                    {
                                        Ok(res) => res.map_err(OllamaPromptError::RequestFailed)?,
                                        Err(_) => {
                                            return Err(OllamaPromptError::GenerationFailed(format!(
                                                "Ollama prompt timed out after {CLOUD_PROMPT_TIMEOUT_SECS}s"
                                            )))
                                        }
                                    };
                                    (resp, fallback.to_string())
                                }
                                CooldownStatusPolicy::Offline => {
                                    if is_cloud {
                                        tracing::warn!(
                                            operation = operation,
                                            primary_model = %self.model(),
                                            error = %err,
                                            "rate limited - deferring to preserve local capacity"
                                        );
                                    }
                                    return Err(OllamaPromptError::NotAvailable);
                                }
                            }
                        }
                        Ok(Err(err)) => return Err(OllamaPromptError::RequestFailed(err)),
                        Err(_) => {
                            return Err(OllamaPromptError::GenerationFailed(format!(
                                "Ollama prompt timed out after {CLOUD_PROMPT_TIMEOUT_SECS}s"
                            )))
                        }
                    }
                };

                if response.trim().is_empty() {
                    return Err(OllamaPromptError::EmptyResponse);
                }

                Ok((response, model_used))
            }
            .await;

            match result {
                Ok((response, model_used)) => {
                    tracing::info!(
                        operation = operation,
                        model = %model_used,
                        response_chars = response.len(),
                        elapsed_ms = started.elapsed().as_millis() as u64,
                        "completed ollama prompt"
                    );
                    Ok((response, model_used))
                }
                Err(error) => {
                    tracing::error!(
                        operation = operation,
                        primary_model = %self.model(),
                        elapsed_ms = started.elapsed().as_millis() as u64,
                        error = ?error,
                        "ollama prompt failed"
                    );
                    Err(error)
                }
            }
        }
        .instrument(span)
        .await
    }

    /// Prompt the configured model with an Ollama structured-output schema and decode the reply.
    ///
    /// The raw prompt should describe the task, not the JSON contract; `schema` is sent to
    /// Ollama's native `format` field and the returned content is validated by serde.
    pub async fn prompt_json_schema<T>(
        &self,
        operation: &str,
        preamble: &str,
        prompt: &str,
        schema: &Value,
        policy: CooldownStatusPolicy,
    ) -> Result<(T, String), OllamaPromptError>
    where
        T: DeserializeOwned,
    {
        let (response, model_used) = self
            .prompt_generate_with_schema(operation, preamble, prompt, schema, policy)
            .await?;
        let parsed = parse_structured_response(&response)?;
        Ok((parsed, model_used))
    }

    async fn prompt_generate_with_schema(
        &self,
        operation: &str,
        preamble: &str,
        prompt: &str,
        schema: &Value,
        policy: CooldownStatusPolicy,
    ) -> Result<(String, String), OllamaPromptError> {
        let span = logfire::span!(
            "ollama.prompt.schema",
            operation = operation,
            model = self.model().to_string(),
            base_url = self.base_url().to_string(),
            prompt_chars = prompt.chars().count(),
            cooldown_policy = format!("{policy:?}"),
            fallback_configured = self.fallback_model().is_some(),
        );

        async move {
            let started = Instant::now();
            let result: Result<(String, String), OllamaPromptError> = async {
                let is_cloud = self.uses_cloud_model();
                let cooldown_active = self.is_cloud_cooldown_active();

                let (response, model_used) = if cooldown_active {
                    match policy {
                        CooldownStatusPolicy::UseLocalFallback => {
                            let fallback = self.fallback_model().ok_or_else(|| {
                                OllamaPromptError::NotAvailable
                            })?;
                            tracing::info!(
                                operation = operation,
                                skipped_model = %self.model(),
                                fallback_model = %fallback,
                                "skipping cloud model due to active cooldown"
                            );
                            let resp = self
                                .prompt_generate_once(fallback, preamble, prompt, schema)
                                .await
                                .map_err(|error| match error {
                                    OllamaGenerateCallError::RateLimited(message)
                                    | OllamaGenerateCallError::Failed(message) => {
                                        OllamaPromptError::GenerationFailed(message)
                                    }
                                })?;
                            (resp, fallback.to_string())
                        }
                        CooldownStatusPolicy::Offline => return Err(OllamaPromptError::NotAvailable),
                    }
                } else {
                    match self
                        .prompt_generate_once(self.model(), preamble, prompt, schema)
                        .await
                    {
                        Ok(resp) => (resp, self.model().to_string()),
                        Err(OllamaGenerateCallError::RateLimited(message)) => {
                            if is_cloud {
                                self.activate_cloud_cooldown();
                            }
                            match policy {
                                CooldownStatusPolicy::UseLocalFallback => {
                                    let fallback = self.fallback_model().ok_or_else(|| {
                                        OllamaPromptError::GenerationFailed(format!(
                                            "rate limited by provider and no fallback model configured: {message}"
                                        ))
                                    })?;
                                    tracing::warn!(
                                        operation = operation,
                                        primary_model = %self.model(),
                                        fallback_model = %fallback,
                                        error = %message,
                                        "rate limited - falling back to local model"
                                    );
                                    let resp = self
                                        .prompt_generate_once(fallback, preamble, prompt, schema)
                                        .await
                                        .map_err(|error| match error {
                                            OllamaGenerateCallError::RateLimited(message)
                                            | OllamaGenerateCallError::Failed(message) => {
                                                OllamaPromptError::GenerationFailed(message)
                                            }
                                        })?;
                                    (resp, fallback.to_string())
                                }
                                CooldownStatusPolicy::Offline => {
                                    if is_cloud {
                                        tracing::warn!(
                                            operation = operation,
                                            primary_model = %self.model(),
                                            error = %message,
                                            "rate limited - deferring to preserve local capacity"
                                        );
                                    }
                                    return Err(OllamaPromptError::NotAvailable);
                                }
                            }
                        }
                        Err(OllamaGenerateCallError::Failed(message)) => {
                            return Err(OllamaPromptError::GenerationFailed(message));
                        }
                    }
                };

                if response.trim().is_empty() {
                    return Err(OllamaPromptError::EmptyResponse);
                }

                Ok((response, model_used))
            }
            .await;

            match result {
                Ok((response, model_used)) => {
                    tracing::info!(
                        operation = operation,
                        model = %model_used,
                        response_chars = response.len(),
                        elapsed_ms = started.elapsed().as_millis() as u64,
                        "completed structured ollama prompt"
                    );
                    Ok((response, model_used))
                }
                Err(error) => {
                    tracing::error!(
                        operation = operation,
                        primary_model = %self.model(),
                        elapsed_ms = started.elapsed().as_millis() as u64,
                        error = ?error,
                        "structured ollama prompt failed"
                    );
                    Err(error)
                }
            }
        }
        .instrument(span)
        .await
    }

    async fn prompt_generate_once(
        &self,
        model: &str,
        preamble: &str,
        prompt: &str,
        schema: &Value,
    ) -> Result<String, OllamaGenerateCallError> {
        let _permit = self
            .acquire_local_permit(model)
            .await
            .map_err(OllamaGenerateCallError::Failed)?;

        let request = OllamaGenerateRequest {
            model,
            system: preamble,
            prompt,
            stream: false,
            format: schema,
            options: OllamaGenerateOptions { temperature: 0.0 },
        };

        let response = self
            .auth(
                self.client
                    .post(format!("{}/api/generate", self.base_url))
                    .timeout(std::time::Duration::from_secs(CLOUD_PROMPT_TIMEOUT_SECS))
                    .json(&request),
            )
            .send()
            .await
            .map_err(|error| OllamaGenerateCallError::Failed(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| OllamaGenerateCallError::Failed(error.to_string()))?;
        if !status.is_success() {
            let message = format!("Ollama generate request failed ({status}): {body}");
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS
                || is_provider_capacity_limited_message(&message)
            {
                return Err(OllamaGenerateCallError::RateLimited(message));
            }
            return Err(OllamaGenerateCallError::Failed(message));
        }

        let payload = serde_json::from_str::<OllamaGenerateResponse>(&body)
            .map_err(|error| OllamaGenerateCallError::Failed(error.to_string()))?;
        if let Some(error) = payload.error.filter(|value| !value.trim().is_empty()) {
            return Err(OllamaGenerateCallError::Failed(error));
        }
        payload
            .response
            .ok_or_else(|| OllamaGenerateCallError::Failed("missing response".to_string()))
    }
}

fn parse_structured_response<T: DeserializeOwned>(response: &str) -> Result<T, OllamaPromptError> {
    serde_json::from_str(response).map_err(|error| {
        OllamaPromptError::InvalidStructuredResponse(format!(
            "failed to decode structured response: {error}"
        ))
    })
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
