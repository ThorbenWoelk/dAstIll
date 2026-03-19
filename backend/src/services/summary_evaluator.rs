use serde::Deserialize;
use thiserror::Error;

use crate::models::{AiStatus, SummaryEvaluationResult};
use crate::services::http::is_cloud_model;
use crate::services::ollama::{CooldownStatusPolicy, OllamaCore, OllamaPromptError};

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
    core: OllamaCore,
}

impl From<OllamaPromptError> for SummaryEvaluatorError {
    fn from(err: OllamaPromptError) -> Self {
        match err {
            OllamaPromptError::NotAvailable => Self::NotAvailable,
            OllamaPromptError::RequestFailed(e) => Self::RequestFailed(e),
            OllamaPromptError::GenerationFailed(s) => Self::EvaluationFailed(s),
            OllamaPromptError::EmptyResponse => {
                Self::EvaluationFailed("Empty response from evaluator model".to_string())
            }
        }
    }
}

impl SummaryEvaluatorService {
    pub const MIN_EVALUATOR_PARAMS_B: u16 = 41;

    pub fn new(core: OllamaCore) -> Self {
        Self { core }
    }

    pub fn validate_model_policy(model: &str) -> Result<(), String> {
        if !is_cloud_model(model) {
            return Err(format!(
                "summary evaluator model must be a cloud model, got `{model}`"
            ));
        }

        let params_b = parse_model_params_billions(model)
            .or_else(|| known_cloud_model_params_billions(model))
            .ok_or_else(|| {
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
        self.core.is_available().await
    }

    pub fn indicator_status(
        &self,
        cloud_cooldown_active: bool,
        endpoint_available: bool,
    ) -> AiStatus {
        self.core.indicator_status(
            cloud_cooldown_active,
            endpoint_available,
            CooldownStatusPolicy::Offline,
        )
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

        let transcript_word_count = transcript.split_whitespace().count();
        let prompt = format!(
            r#"Video Title: {video_title}

Transcript ({transcript_word_count} words):
{transcript}

Summary:
{summary}

Evaluate the summary against the transcript on two independent axes, then combine into a final score.

Axis 1 - Faithfulness (no hallucination):
- Every claim in the summary must be supported by the transcript.
- Penalize any invented names, numbers, claims, or conclusions not in the transcript.
- Penalize vague or generic statements that could apply to any video (e.g. "the speaker discusses interesting topics").

Axis 2 - Completeness (no omission):
- Every significant topic, argument, example, and conclusion in the transcript must appear in the summary, at minimum as a higher-level statement.
- For a {transcript_word_count}-word transcript, a summary with only 2-3 bullet points is almost certainly incomplete.
- Mentally walk through the transcript section by section and check each is represented.

Scoring guide:
- 10: Fully faithful AND fully complete. No hallucinations, no omissions.
- 8-9: Minor omissions or minor imprecisions, but all major points covered.
- 5-7: Several points missing or some unsupported claims.
- 3-4: Major gaps - large sections of transcript content not reflected in summary.
- 0-2: Summary is mostly hallucinated or almost entirely missing transcript content.

Return strict JSON only with this schema:
{{"score": <integer 0-10>, "incoherence_note": "<brief note listing specific hallucinations and/or omitted topics, or empty string if none>"}}

Rules:
- Be harsh. A short, generic summary of a long, detailed transcript should score low.
- List specific omitted topics or hallucinated claims in incoherence_note.
- Do not include markdown, comments, or extra keys."#
        );

        let (raw, model_used) = self
            .prompt_model(
                "summary_quality_evaluation",
                "You are a strict, skeptical evaluator. You check summaries for two failure modes: hallucination (claims not in the transcript) and omission (transcript content missing from the summary). You penalize both equally. A short generic summary of a long detailed transcript is a failing summary.",
                &prompt,
            )
            .await?;

        let mut evaluation = parse_evaluation_response(&raw)?;
        evaluation.quality_model_used = Some(model_used);
        Ok(evaluation)
    }

    pub fn model(&self) -> &str {
        self.core.model()
    }

    async fn prompt_model(
        &self,
        operation: &str,
        preamble: &str,
        prompt: &str,
    ) -> Result<(String, String), SummaryEvaluatorError> {
        self.core
            .prompt_with_fallback(operation, preamble, prompt, CooldownStatusPolicy::Offline)
            .await
            .map_err(Into::into)
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

fn known_cloud_model_params_billions(model: &str) -> Option<u16> {
    match model {
        "glm-5:cloud" => Some(744),
        _ => None,
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
        quality_model_used: None,
    })
}

#[cfg(test)]
mod tests {
    use super::{SummaryEvaluatorService, parse_evaluation_response};
    use crate::models::AiStatus;
    use crate::services::ollama::OllamaCore;

    #[tokio::test]
    async fn is_available_returns_false_for_invalid_url() {
        let service =
            SummaryEvaluatorService::new(OllamaCore::new("://invalid-url", "qwen3.5:397b-cloud"));
        assert!(!service.is_available().await);
    }

    #[test]
    fn indicator_status_reports_cloud_when_cloud_evaluator_is_available() {
        let service = SummaryEvaluatorService::new(OllamaCore::new(
            "http://localhost:11434",
            "qwen3.5:397b-cloud",
        ));
        assert_eq!(service.indicator_status(false, true), AiStatus::Cloud);
    }

    #[test]
    fn indicator_status_reports_local_only_when_local_evaluator_is_primary() {
        let service =
            SummaryEvaluatorService::new(OllamaCore::new("http://localhost:11434", "qwen3:8b"));
        assert_eq!(service.indicator_status(false, true), AiStatus::LocalOnly);
    }

    #[test]
    fn indicator_status_reports_offline_when_cloud_evaluator_is_in_cooldown() {
        let service = SummaryEvaluatorService::new(
            OllamaCore::new("http://localhost:11434", "qwen3.5:397b-cloud")
                .with_fallback_model(Some("qwen3:8b".to_string())),
        );
        assert_eq!(service.indicator_status(true, true), AiStatus::Offline);
    }

    #[test]
    fn evaluator_model_policy_accepts_large_cloud_models() {
        assert!(SummaryEvaluatorService::validate_model_policy("glm-5:cloud").is_ok());
        assert!(SummaryEvaluatorService::validate_model_policy("qwen3.5:397b-cloud").is_ok());
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
