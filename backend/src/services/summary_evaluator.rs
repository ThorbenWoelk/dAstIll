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

        let (raw, model_used) = self
            .prompt_model("summary_quality_evaluation", evaluation_preamble(), &prompt)
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

Return strict JSON only with this schema:
{{
  "status": "scored",
  "unscorable_reason": "<required when status is unscorable; otherwise null>",
  "faithfulness_score": <integer 0-10 or null>,
  "completeness_score": <integer 0-10 or null>,
  "final_score": <integer 0-10 or null>,
  "defects": [
    {{
      "type": "hallucination" | "omission" | "factual_error" | "transcript_quality",
      "severity": "minor" | "major",
      "summary_claim": "<affected summary claim or section>",
      "transcript_anchor": "<short transcript quote, section anchor, or explicit 'not found in transcript'>"
    }}
  ],
  "evaluation_note": "<concise human-readable note>",
  "tags": ["<topic>", "<frame>", "<stance>"]
}}

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

#[cfg(test)]
mod tests {
    use super::{
        SummaryEvaluatorService, evaluation_preamble, evaluation_prompt, parse_evaluation_response,
    };
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
        assert!(SummaryEvaluatorService::validate_model_policy("glm-5.1:cloud").is_ok());
        assert!(SummaryEvaluatorService::validate_model_policy("gemma4:31b-cloud").is_ok());
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
    fn evaluator_model_policy_rejects_models_below_31b() {
        let err = SummaryEvaluatorService::validate_model_policy("qwen3:30b-cloud")
            .expect_err("30b cloud evaluator model should be rejected");
        assert!(err.contains("at least 31B"));
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
            "{\"score\":8,\"incoherence_note\":\"**Omissions**:\\n- Overstates one claim\",\"tags\":[\"AI Security\",\"Tech Knowledge\",\"Blackpilled\"]}",
        )
        .unwrap();
        assert_eq!(parsed.quality_score, Some(8));
        assert_eq!(
            parsed.quality_note,
            Some("**Omissions**:\n- Overstates one claim".to_string())
        );
        assert_eq!(
            parsed.summary_tags,
            vec![
                "AI Security".to_string(),
                "Tech Knowledge".to_string(),
                "Blackpilled".to_string()
            ]
        );
    }

    #[test]
    fn parse_evaluation_response_handles_wrapped_json_and_empty_note() {
        let parsed = parse_evaluation_response(
            "```json\n{\n  \"score\": 10,\n  \"incoherence_note\": \"\"\n}\n```",
        )
        .unwrap();
        assert_eq!(parsed.quality_score, Some(10));
        assert_eq!(parsed.quality_note, None);
        assert!(parsed.summary_tags.is_empty());
    }

    #[test]
    fn parse_evaluation_response_rejects_score_outside_range() {
        let err = parse_evaluation_response("{\"score\":12,\"incoherence_note\":null}")
            .expect_err("out-of-range evaluator scores must be schema failures");
        assert!(err.to_string().contains("score must be between 0 and 10"));
    }

    #[test]
    fn parse_evaluation_response_normalizes_tags() {
        let parsed = parse_evaluation_response(
            "{\"score\":7,\"incoherence_note\":null,\"tags\":[\" AI Security. \",\"ai security\",\"Tech Knowledge\",\"Blackpilled\",\"Too Many\",\"Ignored\"]}",
        )
        .unwrap();
        assert_eq!(
            parsed.summary_tags,
            vec![
                "AI Security".to_string(),
                "Tech Knowledge".to_string(),
                "Blackpilled".to_string(),
                "Too Many".to_string()
            ]
        );
    }

    #[test]
    fn parse_evaluation_response_handles_structured_scored_schema() {
        let parsed = parse_evaluation_response(
            r#"{
              "status": "scored",
              "faithfulness_score": 7,
              "completeness_score": 6,
              "final_score": 6,
              "defects": [
                {
                  "type": "hallucination",
                  "severity": "major",
                  "summary_claim": "The summary says the model ordered pizza.",
                  "transcript_anchor": "Transcript only says it was about to place the order."
                }
              ],
              "evaluation_note": "The main problem is a title-derived action claim.",
              "tags": ["AI Agents", "Transcript Quality"]
            }"#,
        )
        .unwrap();

        assert_eq!(parsed.quality_score, Some(6));
        let note = parsed
            .quality_note
            .expect("structured defects should be preserved");
        assert!(note.contains("Faithfulness: 7/10"));
        assert!(note.contains("Completeness: 6/10"));
        assert!(note.contains("Major hallucination"));
        assert!(note.contains("The summary says the model ordered pizza."));
        assert!(note.contains("Transcript only says it was about to place the order."));
        assert!(note.contains("title-derived action claim"));
        assert_eq!(
            parsed.summary_tags,
            vec!["AI Agents".to_string(), "Transcript Quality".to_string()]
        );
    }

    #[test]
    fn parse_evaluation_response_requires_defects_for_non_perfect_structured_scores() {
        let err = parse_evaluation_response(
            r#"{
              "status": "scored",
              "faithfulness_score": 8,
              "completeness_score": 7,
              "final_score": 7,
              "defects": [],
              "evaluation_note": "Some issues exist."
            }"#,
        )
        .expect_err("non-perfect structured scores need evidence-backed defects");

        assert!(err.to_string().contains("defects are required"));
    }

    #[test]
    fn parse_evaluation_response_rejects_empty_defect_evidence() {
        let err = parse_evaluation_response(
            r#"{
              "status": "scored",
              "faithfulness_score": 8,
              "completeness_score": 7,
              "final_score": 7,
              "defects": [
                {
                  "type": "hallucination",
                  "severity": "major",
                  "summary_claim": "Title-derived claim",
                  "transcript_anchor": "   "
                }
              ],
              "evaluation_note": "The summary adds title context."
            }"#,
        )
        .expect_err("defect evidence anchors must not be blank");

        assert!(err.to_string().contains("transcript_anchor is required"));
    }

    #[test]
    fn parse_evaluation_response_handles_unscorable_schema_without_numeric_score() {
        let parsed = parse_evaluation_response(
            r#"{
              "status": "unscorable",
              "unscorable_reason": "Transcript is show notes, not spoken content.",
              "tags": ["Transcript Quality"]
            }"#,
        )
        .unwrap();

        assert_eq!(parsed.quality_score, None);
        assert_eq!(
            parsed.quality_note,
            Some("**Unscorable**:\n- Transcript is show notes, not spoken content.".to_string())
        );
        assert_eq!(parsed.summary_tags, vec!["Transcript Quality".to_string()]);
    }

    #[test]
    fn evaluation_prompt_sets_critical_but_realistic_tone() {
        let prompt = evaluation_prompt(
            "Example title",
            "This is a detailed transcript with several sections.",
            "- A short summary",
        );

        assert!(evaluation_preamble().contains("critical but realistic review"));
        assert!(prompt.contains("Write a critical but realistic review of the content."));
        assert!(prompt.contains("Do not sugar-coat obvious misses, but do not destroy the summary over minor phrasing issues."));
        assert!(prompt.contains("Focus on substantive problems; do not pad the note with praise and do not invent flaws."));
        assert!(prompt.contains("\"faithfulness_score\""));
        assert!(prompt.contains("\"completeness_score\""));
        assert!(prompt.contains("\"final_score\""));
        assert!(prompt.contains("\"defects\""));
        assert!(prompt.contains("\"unscorable\""));
        assert!(prompt.contains("scores below 10 require at least one defect"));
        assert!(prompt.contains("7 is acceptable"));
    }
}
