use reqwest::Client;
use rig::client::Nothing;
use rig::completion::Prompt;
use rig::prelude::*;
use rig::providers::ollama;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tokio::sync::Semaphore;

use crate::models::{AiStatus, SummaryEvaluationResult};
use crate::services::http::CloudCooldown;
use crate::services::{build_http_client, is_cloud_model, is_rate_limited};

#[derive(Error, Debug)]
pub enum SummaryEvaluatorError {
    #[error("Ollama request failed: {0}")]
    RequestFailed(#[from] rig::completion::PromptError),
    #[error("Ollama not available")]
    NotAvailable,
    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),
    #[error("Failed to parse evaluator response: {0}")]
    ParseFailed(String),
}

pub struct SummaryEvaluatorService {
    client: Client,
    base_url: String,
    model: String,
    fallback_model: Option<String>,
    cloud_cooldown: Option<Arc<CloudCooldown>>,
    ollama_semaphore: Option<Arc<Semaphore>>,
}

impl SummaryEvaluatorService {
    pub const MIN_EVALUATOR_PARAMS_B: u16 = 41;

    pub fn new() -> Self {
        Self {
            client: build_http_client(),
            base_url: "http://localhost:11434".to_string(),
            model: "qwen3-coder:480b-cloud".to_string(),
            fallback_model: None,
            cloud_cooldown: None,
            ollama_semaphore: None,
        }
    }

    pub fn with_config(base_url: &str, model: &str) -> Self {
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

    pub fn with_fallback_model(mut self, fallback_model: Option<String>) -> Self {
        self.fallback_model = fallback_model;
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

    pub fn validate_model_policy(model: &str) -> Result<(), String> {
        if !is_cloud_model(model) {
            return Err(format!(
                "summary evaluator model must be a cloud model, got `{model}`"
            ));
        }

        let params_b = parse_model_params_billions(model).ok_or_else(|| {
            format!(
                "summary evaluator model must include a parseable parameter size, got `{model}`"
            )
        })?;

        if params_b < Self::MIN_EVALUATOR_PARAMS_B {
            return Err(format!(
                "summary evaluator model must be >40B parameters, got `{model}`"
            ));
        }

        Ok(())
    }

    pub async fn is_available(&self) -> bool {
        let base_url = &self.base_url;
        self.client
            .get(format!("{base_url}/api/tags"))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub fn indicator_status(
        &self,
        cloud_cooldown_active: bool,
        endpoint_available: bool,
    ) -> AiStatus {
        if !endpoint_available {
            return AiStatus::Offline;
        }

        if !is_cloud_model(&self.model) {
            return AiStatus::LocalOnly;
        }

        if !cloud_cooldown_active {
            return AiStatus::Cloud;
        }

        AiStatus::Offline
    }

    pub async fn evaluate(
        &self,
        transcript: &str,
        summary: &str,
        video_title: &str,
    ) -> Result<SummaryEvaluationResult, SummaryEvaluatorError> {
        if transcript.trim().is_empty() || summary.trim().is_empty() {
            return Err(SummaryEvaluatorError::EvaluationFailed(
                "Transcript or summary is empty".to_string(),
            ));
        }

        let prompt = format!(
            "Video Title: {video_title}\n\nTranscript:\n{transcript}\n\nSummary:\n{summary}\n\nEvaluate summary coherence against the transcript.\nReturn strict JSON only with this schema:\n{{\"score\": <integer 0-10>, \"incoherence_note\": \"<brief note or empty string>\"}}\n\nRules:\n- Score 10 means fully faithful and coherent.\n- Penalize hallucinations, contradictions, and missing core claims.\n- Keep incoherence_note very brief.\n- If no incoherence exists, return an empty string as incoherence_note.\n- Do not include markdown, comments, or extra keys."
        );

        let raw = self
            .prompt_model(
                "summary_quality_evaluation",
                "You are a strict evaluator that compares a summary against a transcript.",
                &prompt,
            )
            .await?;

        parse_evaluation_response(&raw)
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    fn build_ollama_client(&self) -> Result<ollama::Client, SummaryEvaluatorError> {
        ollama::Client::builder()
            .api_key(Nothing)
            .base_url(&self.base_url)
            .build()
            .map_err(|err| SummaryEvaluatorError::EvaluationFailed(err.to_string()))
    }

    async fn prompt_model(
        &self,
        operation: &str,
        preamble: &str,
        prompt: &str,
    ) -> Result<String, SummaryEvaluatorError> {
        tracing::info!(
            operation = operation,
            model = %self.model,
            base_url = %self.base_url,
            prompt_chars = prompt.len(),
            "starting ollama summary evaluation prompt"
        );
        let started = Instant::now();
        let ollama_client = self.build_ollama_client()?;

        let is_cloud = is_cloud_model(&self.model);
        let cooldown_active = is_cloud
            && self
                .cloud_cooldown
                .as_ref()
                .is_some_and(|cd| cd.is_active());

        let (response, model_used) = if cooldown_active {
            return Err(SummaryEvaluatorError::NotAvailable);
        } else {
            // Acquire semaphore for primary model if it's not cloud
            let _permit = if !is_cloud {
                if let Some(sem) = &self.ollama_semaphore {
                    Some(
                        sem.acquire()
                            .await
                            .map_err(|e| SummaryEvaluatorError::EvaluationFailed(e.to_string()))?,
                    )
                } else {
                    None
                }
            } else {
                None
            };

            let agent = ollama_client.agent(&self.model).preamble(preamble).build();
            match agent.prompt(prompt).await {
                Ok(resp) => (resp, self.model.clone()),
                Err(err) if is_rate_limited(&err) => {
                    if let Some(cd) = &self.cloud_cooldown {
                        if is_cloud {
                            cd.activate();
                        }
                    }
                    if is_cloud {
                        tracing::warn!(
                            operation = operation,
                            primary_model = %self.model,
                            error = %err,
                            "rate limited - deferring summary evaluation to preserve local capacity"
                        );
                        return Err(SummaryEvaluatorError::NotAvailable);
                    }

                    return Err(err.into());
                }
                Err(err) => return Err(err.into()),
            }
        };
        tracing::info!(
            operation = operation,
            model = %model_used,
            response_chars = response.len(),
            elapsed_ms = started.elapsed().as_millis() as u64,
            "completed ollama summary evaluation prompt"
        );

        if response.trim().is_empty() {
            return Err(SummaryEvaluatorError::EvaluationFailed(
                "Empty response from evaluator model".to_string(),
            ));
        }

        Ok(response)
    }
}

fn parse_model_params_billions(model: &str) -> Option<u16> {
    let chars: Vec<char> = model.chars().collect();
    let mut index = 0usize;
    let mut found = None;

    while index < chars.len() {
        if !chars[index].is_ascii_digit() {
            index += 1;
            continue;
        }

        let start = index;
        while index < chars.len() && chars[index].is_ascii_digit() {
            index += 1;
        }

        if index < chars.len() && chars[index].eq_ignore_ascii_case(&'b') {
            let digits: String = chars[start..index].iter().collect();
            if let Ok(value) = digits.parse::<u16>() {
                found = Some(value);
            }
        }
    }

    found
}

impl Default for SummaryEvaluatorService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct EvaluatorResponse {
    score: i64,
    incoherence_note: Option<String>,
}

fn parse_evaluation_response(raw: &str) -> Result<SummaryEvaluationResult, SummaryEvaluatorError> {
    let start = raw
        .find('{')
        .ok_or_else(|| SummaryEvaluatorError::ParseFailed("missing json object".to_string()))?;
    let end = raw
        .rfind('}')
        .ok_or_else(|| SummaryEvaluatorError::ParseFailed("missing json object".to_string()))?;

    let json = &raw[start..=end];
    let parsed: EvaluatorResponse = serde_json::from_str(json)
        .map_err(|err| SummaryEvaluatorError::ParseFailed(err.to_string()))?;

    let score = parsed.score.clamp(0, 10) as u8;
    let note = parsed
        .incoherence_note
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    Ok(SummaryEvaluationResult {
        quality_score: score,
        quality_note: note,
    })
}

#[cfg(test)]
mod tests {
    use super::{SummaryEvaluatorService, parse_evaluation_response};
    use crate::models::AiStatus;

    #[tokio::test]
    async fn is_available_returns_false_for_invalid_url() {
        let service =
            SummaryEvaluatorService::with_config("://invalid-url", "qwen3-coder:480b-cloud");
        assert!(!service.is_available().await);
    }

    #[test]
    fn indicator_status_reports_cloud_when_cloud_evaluator_is_available() {
        let service = SummaryEvaluatorService::with_config(
            "http://localhost:11434",
            "qwen3-coder:480b-cloud",
        );
        assert_eq!(service.indicator_status(false, true), AiStatus::Cloud);
    }

    #[test]
    fn indicator_status_reports_local_only_when_local_evaluator_is_primary() {
        let service = SummaryEvaluatorService::with_config("http://localhost:11434", "qwen3:8b");
        assert_eq!(service.indicator_status(false, true), AiStatus::LocalOnly);
    }

    #[test]
    fn indicator_status_reports_offline_when_cloud_evaluator_is_in_cooldown() {
        let service = SummaryEvaluatorService::with_config(
            "http://localhost:11434",
            "qwen3-coder:480b-cloud",
        )
        .with_fallback_model(Some("qwen3:8b".to_string()));
        assert_eq!(service.indicator_status(true, true), AiStatus::Offline);
    }

    #[test]
    fn evaluator_model_policy_accepts_large_cloud_models() {
        assert!(SummaryEvaluatorService::validate_model_policy("qwen3-coder:480b-cloud").is_ok());
        assert!(SummaryEvaluatorService::validate_model_policy("llama3.3:70b-cloud").is_ok());
    }

    #[test]
    fn evaluator_model_policy_rejects_local_models() {
        let err = SummaryEvaluatorService::validate_model_policy("qwen3:32b")
            .expect_err("local evaluator model should be rejected");
        assert!(err.contains("cloud"));
    }

    #[test]
    fn evaluator_model_policy_rejects_models_at_or_below_40b() {
        let err = SummaryEvaluatorService::validate_model_policy("qwen3-coder:40b-cloud")
            .expect_err("40b cloud evaluator model should be rejected");
        assert!(err.contains(">40B"));
    }

    #[test]
    fn evaluator_model_policy_rejects_models_without_parseable_size() {
        let err = SummaryEvaluatorService::validate_model_policy("custom-evaluator:cloud")
            .expect_err("size-less cloud evaluator model should be rejected");
        assert!(err.contains("parameter size"));
    }

    #[test]
    fn parse_evaluation_response_handles_plain_json() {
        let parsed = parse_evaluation_response(
            "{\"score\":8,\"incoherence_note\":\"Overstates one claim\"}",
        )
        .unwrap();
        assert_eq!(parsed.quality_score, 8);
        assert_eq!(
            parsed.quality_note,
            Some("Overstates one claim".to_string())
        );
    }

    #[test]
    fn parse_evaluation_response_handles_wrapped_json_and_empty_note() {
        let parsed = parse_evaluation_response(
            "```json\n{\n  \"score\": 10,\n  \"incoherence_note\": \"\"\n}\n```",
        )
        .unwrap();
        assert_eq!(parsed.quality_score, 10);
        assert_eq!(parsed.quality_note, None);
    }

    #[test]
    fn parse_evaluation_response_clamps_score_range() {
        let parsed = parse_evaluation_response("{\"score\":12,\"incoherence_note\":null}").unwrap();
        assert_eq!(parsed.quality_score, 10);
    }
}
