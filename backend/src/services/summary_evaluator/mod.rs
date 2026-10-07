use thiserror::Error;

use crate::models::{AiStatus, SummaryEvaluationResult, VocabularyReplacement};
use crate::services::http::is_cloud_model;
use crate::services::ollama::{CooldownStatusPolicy, OllamaCore, OllamaPromptError};

mod response;

use response::{EvaluatorResponse, evaluation_result_from_response};
pub use response::{SummaryEvaluation, UnscorableCause};

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

fn evaluation_preamble() -> &'static str {
    "You are a strict evaluator writing a critical but realistic review of a summary against its transcript. Judge only what the transcript supports. Penalize hallucinations and substantive omissions equally. Do not sugar-coat weak summaries, but calibrate scores so 7 is acceptable and 6 or below should be regenerated. Omission of confidently identifiable sponsor or ad segments does not count against completeness. A short generic summary of a long detailed editorial transcript is a failing summary."
}

fn evaluation_prompt(video_title: &str, transcript: &str, summary: &str) -> String {
    let transcript_word_count = transcript.split_whitespace().count();
    format!(
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

Axis 2 - Completeness (no omission of editorial content):
- Every significant topic, argument, example, and conclusion in the substantive (non-ad) parts of the transcript must appear in the summary, at minimum as a higher-level statement.
- Do not treat omission as incomplete when the summary skips transcript portions you confidently identify as paid promotions, sponsor reads, discount pitches, or standalone ad segments (e.g. explicit sponsorship framing, isolated product pitch, use-code style copy) while the main editorial arc is covered.
- For a {transcript_word_count}-word transcript, a summary with only 2-3 bullet points is almost certainly incomplete (unless almost the entire transcript is clearly non-editorial ad copy).
- Mentally walk through the transcript section by section and check each editorial segment is represented.

Scoring guide:
- 10: Fully faithful AND fully complete on editorial substance. No defects.
- 9: Strong summary with only one or two minor defects; no major hallucinations or major omissions.
- 8: Useful summary with minor defects, but all major transcript points are still represented.
- 7: Acceptable summary. It can have a few minor defects, but no major hallucination, no major factual error, and no missing main arc.
- 6: Regenerate. Several minor defects or one major defect materially reduce trust.
- 3-5: Poor. Multiple major omissions, factual errors, or unsupported claims.
- 0-2: Broken, mostly hallucinated, wrong-source, or almost entirely missing transcript content.

Use status "unscorable" instead of a numeric score when the source cannot be judged reliably:
- transcript is show notes, a description, or not spoken/source content
- transcript or summary appears corrupted, mismatched, language-incompatible, or mostly unreadable
- the summary is too malformed to compare
When status is "unscorable", say whether the summary (malformed, wrong source, unreadable) or the transcript is the problem in the unscorable cause field. Otherwise leave that field null.

Return one JSON object matching the runtime schema.

Rules:
- Write a critical but realistic review of the content.
- Do not sugar-coat obvious misses, but do not destroy the summary over minor phrasing issues.
- Focus on substantive problems; do not pad the note with praise and do not invent flaws.
- Reserve the lowest scores for genuinely broken summaries and acknowledge when the summary is mostly sound apart from limited issues.
- Set "status" to exactly "scored" or "unscorable".
- scores below 10 require at least one defect with a transcript_anchor.
- Every defect needs a type, a severity (minor or major), the summary claim, and a transcript anchor.
- When status is "scored", always give all three scores: faithfulness, completeness, and final.
- 7 is acceptable; 6 or below means the summary should be regenerated.
- Tags are metadata only. Return 0-4 short Title Case tags supported by the transcript; do not use tags to explain defects.
- Do not include extra keys, comments, or explain your reasoning outside the JSON."#
    )
}

/// The transcript text the evaluator compares against.
///
/// The summarizer reads a vocabulary-normalized transcript, so canonical names in
/// the summary (for example a corrected product name) must also appear in what the
/// evaluator reads. Otherwise correct names get flagged as hallucinations.
pub(crate) fn transcript_for_evaluation(
    transcript: &str,
    vocabulary_replacements: &[VocabularyReplacement],
) -> String {
    crate::services::summarizer::apply_vocabulary_replacements(transcript, vocabulary_replacements)
}

fn evaluator_response_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "status": {
                "type": "string",
                "enum": ["scored", "unscorable"]
            },
            "unscorable_reason": {
                "anyOf": [
                    { "type": "string" },
                    { "type": "null" }
                ]
            },
            "unscorable_cause": {
                "anyOf": [
                    {
                        "type": "string",
                        "enum": ["transcript", "summary"]
                    },
                    { "type": "null" }
                ]
            },
            "faithfulness_score": nullable_score_schema(),
            "completeness_score": nullable_score_schema(),
            "final_score": nullable_score_schema(),
            "defects": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": [
                                "hallucination",
                                "omission",
                                "factual_error",
                                "transcript_quality"
                            ]
                        },
                        "severity": {
                            "type": "string",
                            "enum": ["minor", "major"]
                        },
                        "summary_claim": { "type": "string" },
                        "transcript_anchor": { "type": "string" }
                    },
                    "required": [
                        "type",
                        "severity",
                        "summary_claim",
                        "transcript_anchor"
                    ]
                }
            },
            "evaluation_note": {
                "anyOf": [
                    { "type": "string" },
                    { "type": "null" }
                ]
            },
            "tags": {
                "type": "array",
                "items": { "type": "string" },
                "maxItems": 4
            }
        },
        "required": [
            "status",
            "unscorable_reason",
            "unscorable_cause",
            "faithfulness_score",
            "completeness_score",
            "final_score",
            "defects",
            "evaluation_note",
            "tags"
        ]
    })
}

fn nullable_score_schema() -> serde_json::Value {
    serde_json::json!({
        "anyOf": [
            {
                "type": "integer",
                "minimum": 0,
                "maximum": 10
            },
            { "type": "null" }
        ]
    })
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

/// Total parameters, in billions, for cloud models whose tag carries no size.
/// Values come from the model pages on ollama.com/library.
fn known_cloud_model_params_billions(model: &str) -> Option<u16> {
    match model {
        "glm-5.1:cloud" => Some(744),
        "glm-5.3:cloud" => Some(753),
        "deepseek-v4-pro:cloud" => Some(1600),
        "kimi-k3:cloud" => Some(2810),
        _ => None,
    }
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
            OllamaPromptError::InvalidStructuredResponse(s) => Self::ParseFailed(s),
        }
    }
}

impl SummaryEvaluatorService {
    pub const MIN_EVALUATOR_PARAMS_B: u16 = 31;

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
                "summary evaluator model must be at least 31B parameters, got `{model}`"
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
        self.evaluate_with_cause(transcript, summary, video_title)
            .await
            .map(|evaluation| evaluation.result)
    }

    /// Like [`Self::evaluate`], but also says why a summary was unscorable.
    pub async fn evaluate_with_cause(
        &self,
        transcript: &str,
        summary: &str,
        video_title: &str,
    ) -> Result<SummaryEvaluation, SummaryEvaluatorError> {
        if transcript.trim().is_empty() || summary.trim().is_empty() {
            return Err(SummaryEvaluatorError::EvaluationFailed(
                "Transcript or summary is empty".to_string(),
            ));
        }

        let prompt = evaluation_prompt(video_title, transcript, summary);

        let (parsed, model_used) = self
            .prompt_model("summary_quality_evaluation", evaluation_preamble(), &prompt)
            .await?;

        let mut evaluation = evaluation_result_from_response(parsed)?;
        evaluation.result.quality_model_used = Some(model_used);
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
    ) -> Result<(EvaluatorResponse, String), SummaryEvaluatorError> {
        self.core
            .prompt_json_schema(
                operation,
                preamble,
                prompt,
                &evaluator_response_schema(),
                CooldownStatusPolicy::Offline,
            )
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
