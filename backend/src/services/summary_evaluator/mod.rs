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

Return one JSON object matching the runtime schema.

Rules:
- Write a critical but realistic review of the content.
- Do not sugar-coat obvious misses, but do not destroy the summary over minor phrasing issues.
- Focus on substantive problems; do not pad the note with praise and do not invent flaws.
- Reserve the lowest scores for genuinely broken summaries and acknowledge when the summary is mostly sound apart from limited issues.
- Set "status" to exactly "scored" or "unscorable".
- scores below 10 require at least one defect with a transcript_anchor.
- 7 is acceptable; 6 or below means the summary should be regenerated.
- Tags are metadata only. Return 0-4 short Title Case tags supported by the transcript; do not use tags to explain defects.
- Do not include extra keys, comments, or explain your reasoning outside the JSON."#
    )
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

fn known_cloud_model_params_billions(model: &str) -> Option<u16> {
    match model {
        "glm-5.1:cloud" => Some(744),
        _ => None,
    }
}

#[derive(Deserialize)]
struct EvaluatorResponse {
    status: Option<String>,
    score: Option<i64>,
    final_score: Option<i64>,
    faithfulness_score: Option<i64>,
    completeness_score: Option<i64>,
    incoherence_note: Option<String>,
    evaluation_note: Option<String>,
    unscorable_reason: Option<String>,
    defects: Option<Vec<EvaluatorDefect>>,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct EvaluatorDefect {
    #[serde(rename = "type")]
    defect_type: String,
    severity: String,
    summary_claim: String,
    transcript_anchor: String,
}

fn normalize_tags(tags: Option<Vec<String>>) -> Vec<String> {
    let mut normalized = Vec::new();

    for tag in tags.unwrap_or_default() {
        let cleaned = tag.trim().trim_matches('.').to_string();
        if cleaned.is_empty() {
            continue;
        }
        if normalized
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&cleaned))
        {
            continue;
        }
        normalized.push(cleaned);
        if normalized.len() >= 4 {
            break;
        }
    }

    normalized
}

fn evaluation_result_from_response(
    parsed: EvaluatorResponse,
) -> Result<SummaryEvaluationResult, SummaryEvaluatorError> {
    let tags = normalize_tags(parsed.tags);
    let status = parsed.status.as_deref().unwrap_or("scored");
    if status == "unscorable" {
        let reason = clean_required_text(parsed.unscorable_reason, "unscorable_reason")?;
        return Ok(SummaryEvaluationResult {
            quality_score: None,
            quality_note: Some(format!("**Unscorable**:\n- {reason}")),
            quality_model_used: None,
            summary_tags: tags,
        });
    }
    if status != "scored" {
        return Err(SummaryEvaluatorError::ParseFailed(format!(
            "unsupported evaluation status `{status}`"
        )));
    }

    let score = parse_score(parsed.final_score.or(parsed.score), "final_score")?;
    let legacy_note = parsed
        .incoherence_note
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let is_structured = parsed.status.is_some() || parsed.final_score.is_some();
    let note = if is_structured {
        let faithfulness_score = parse_score(parsed.faithfulness_score, "faithfulness_score")?;
        let completeness_score = parse_score(parsed.completeness_score, "completeness_score")?;
        let defects = parsed.defects.unwrap_or_default();
        if score < 10 && defects.is_empty() {
            return Err(SummaryEvaluatorError::ParseFailed(
                "defects are required for scores below 10".to_string(),
            ));
        }
        validate_defects(&defects)?;
        build_structured_note(
            faithfulness_score,
            completeness_score,
            score,
            &defects,
            parsed.evaluation_note,
        )
    } else {
        legacy_note
    };

    Ok(SummaryEvaluationResult {
        quality_score: Some(score),
        quality_note: note,
        quality_model_used: None,
        summary_tags: tags,
    })
}

fn parse_score(value: Option<i64>, field: &str) -> Result<u8, SummaryEvaluatorError> {
    let value =
        value.ok_or_else(|| SummaryEvaluatorError::ParseFailed(format!("{field} is required")))?;
    if !(0..=10).contains(&value) {
        return Err(SummaryEvaluatorError::ParseFailed(format!(
            "{field} score must be between 0 and 10"
        )));
    }
    Ok(value as u8)
}

fn clean_required_text(
    value: Option<String>,
    field: &str,
) -> Result<String, SummaryEvaluatorError> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| SummaryEvaluatorError::ParseFailed(format!("{field} is required")))
}

fn validate_defects(defects: &[EvaluatorDefect]) -> Result<(), SummaryEvaluatorError> {
    for defect in defects {
        require_defect_field(&defect.defect_type, "defect.type")?;
        require_defect_field(&defect.severity, "defect.severity")?;
        require_defect_field(&defect.summary_claim, "summary_claim")?;
        require_defect_field(&defect.transcript_anchor, "transcript_anchor")?;
    }
    Ok(())
}

fn require_defect_field(value: &str, field: &str) -> Result<(), SummaryEvaluatorError> {
    if value.trim().is_empty() {
        return Err(SummaryEvaluatorError::ParseFailed(format!(
            "{field} is required"
        )));
    }
    Ok(())
}

fn build_structured_note(
    faithfulness_score: u8,
    completeness_score: u8,
    final_score: u8,
    defects: &[EvaluatorDefect],
    evaluation_note: Option<String>,
) -> Option<String> {
    let mut sections = vec![
        "**Scores**:".to_string(),
        format!("- Faithfulness: {faithfulness_score}/10"),
        format!("- Completeness: {completeness_score}/10"),
        format!("- Final: {final_score}/10"),
    ];

    if !defects.is_empty() {
        sections.push("\n**Defects**:".to_string());
        for defect in defects {
            sections.push(format!(
                "- **{} {}**: {}; transcript anchor: {}",
                title_case(defect.severity.trim()),
                defect.defect_type.trim(),
                defect.summary_claim.trim(),
                defect.transcript_anchor.trim()
            ));
        }
    }

    if let Some(note) = evaluation_note
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        sections.push("\n**Evaluation**:".to_string());
        sections.push(format!("- {note}"));
    }

    Some(sections.join("\n"))
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
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
