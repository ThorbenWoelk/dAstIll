use reqwest::Client;
use rig::client::Nothing;
use rig::completion::Prompt;
use rig::prelude::*;
use rig::providers::ollama;
use serde::Deserialize;
use std::time::Instant;
use thiserror::Error;

use crate::models::SummaryEvaluationResult;
use crate::services::{build_http_client, is_rate_limited};

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
}

impl SummaryEvaluatorService {
    pub fn new() -> Self {
        Self {
            client: build_http_client(),
            base_url: "http://localhost:11434".to_string(),
            model: "qwen3-coder:480b-cloud".to_string(),
            fallback_model: Some("qwen3:8b".to_string()),
        }
    }

    pub fn with_config(base_url: &str, model: &str) -> Self {
        Self {
            client: build_http_client(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            fallback_model: None,
        }
    }

    pub fn with_client(client: Client, base_url: &str, model: &str) -> Self {
        Self {
            client,
            base_url: base_url.to_string(),
            model: model.to_string(),
            fallback_model: None,
        }
    }

    pub fn with_fallback_model(mut self, fallback_model: Option<String>) -> Self {
        self.fallback_model = fallback_model;
        self
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
        let agent = ollama_client.agent(&self.model).preamble(preamble).build();
        let response = match agent.prompt(prompt).await {
            Ok(resp) => resp,
            Err(err) if is_rate_limited(&err) => {
                let fallback = self.fallback_model.as_deref().ok_or_else(|| {
                    SummaryEvaluatorError::EvaluationFailed(format!(
                        "rate limited by provider and no fallback model configured: {err}"
                    ))
                })?;
                tracing::warn!(
                    operation = operation,
                    primary_model = %self.model,
                    fallback_model = %fallback,
                    error = %err,
                    "rate limited - falling back to local model"
                );
                let fallback_agent =
                    ollama_client.agent(fallback).preamble(preamble).build();
                fallback_agent.prompt(prompt).await?
            }
            Err(err) => return Err(err.into()),
        };
        tracing::info!(
            operation = operation,
            model = %self.model,
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

    #[tokio::test]
    async fn is_available_returns_false_for_invalid_url() {
        let service =
            SummaryEvaluatorService::with_config("://invalid-url", "qwen3-coder:480b-cloud");
        assert!(!service.is_available().await);
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
