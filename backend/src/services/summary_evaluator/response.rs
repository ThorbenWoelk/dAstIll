//! Turns the evaluator model's JSON into a stored evaluation.
//!
//! Cloud models do not always follow the JSON schema exactly. Parsing fills in
//! missing optional parts (defect severity, axis scores, defect lists) and only
//! rejects output that has no usable score or status.

use serde::{Deserialize, Deserializer};

use super::SummaryEvaluatorError;
use crate::models::SummaryEvaluationResult;

const DEFAULT_DEFECT_SEVERITY: &str = "minor";
const DEFAULT_DEFECT_TYPE: &str = "issue";
const MISSING_UNSCORABLE_REASON: &str = "The evaluator gave no reason.";

/// Why the evaluator could not score a summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnscorableCause {
    /// The source transcript cannot be judged (show notes, corrupted, wrong language).
    /// Regenerating the summary would not help.
    Transcript,
    /// The summary itself is too malformed to compare. A new summary may fix it.
    Summary,
}

/// One parsed evaluator answer.
#[derive(Debug, Clone)]
pub struct SummaryEvaluation {
    pub result: SummaryEvaluationResult,
    /// Set only when the evaluator returned `unscorable`.
    pub unscorable_cause: Option<UnscorableCause>,
}

#[derive(Deserialize)]
pub(super) struct EvaluatorResponse {
    #[serde(default)]
    status: Option<String>,
    #[serde(default, deserialize_with = "lenient_score")]
    score: Option<i64>,
    #[serde(default, deserialize_with = "lenient_score")]
    final_score: Option<i64>,
    #[serde(default, deserialize_with = "lenient_score")]
    faithfulness_score: Option<i64>,
    #[serde(default, deserialize_with = "lenient_score")]
    completeness_score: Option<i64>,
    #[serde(default)]
    incoherence_note: Option<String>,
    #[serde(default)]
    evaluation_note: Option<String>,
    #[serde(default)]
    unscorable_reason: Option<String>,
    #[serde(default)]
    unscorable_cause: Option<String>,
    #[serde(default)]
    defects: Option<Vec<EvaluatorDefect>>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct EvaluatorDefect {
    #[serde(default, rename = "type")]
    defect_type: Option<String>,
    #[serde(default)]
    severity: Option<String>,
    #[serde(default)]
    summary_claim: Option<String>,
    #[serde(default)]
    transcript_anchor: Option<String>,
}

/// A defect after defaults are filled in.
struct Defect {
    defect_type: String,
    severity: String,
    summary_claim: Option<String>,
    transcript_anchor: Option<String>,
}

/// Accepts integers, whole floats (`8.0`), and numeric strings (`"8"`).
/// Anything else becomes `None`, so the required-field checks decide.
fn lenient_score<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(match value {
        Some(serde_json::Value::Number(number)) => number.as_i64().or_else(|| {
            number
                .as_f64()
                .filter(|value| value.fract() == 0.0)
                .map(|value| value as i64)
        }),
        Some(serde_json::Value::String(text)) => text.trim().parse::<i64>().ok(),
        _ => None,
    })
}

fn clean_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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

/// Uses the explicit cause when the model sent one. Otherwise only a reason that
/// talks about the summary and not the transcript counts as a summary problem.
fn unscorable_cause(explicit: Option<&str>, reason: &str) -> UnscorableCause {
    match explicit.map(|value| value.trim().to_ascii_lowercase()) {
        Some(value) if value == "summary" => return UnscorableCause::Summary,
        Some(value) if value == "transcript" => return UnscorableCause::Transcript,
        _ => {}
    }
    let reason = reason.to_ascii_lowercase();
    if reason.contains("summary") && !reason.contains("transcript") {
        UnscorableCause::Summary
    } else {
        UnscorableCause::Transcript
    }
}

pub(super) fn evaluation_result_from_response(
    parsed: EvaluatorResponse,
) -> Result<SummaryEvaluation, SummaryEvaluatorError> {
    let tags = normalize_tags(parsed.tags);
    let status = parsed
        .status
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "scored".to_string());

    if status == "unscorable" {
        let reason = clean_text(parsed.unscorable_reason)
            .unwrap_or_else(|| MISSING_UNSCORABLE_REASON.to_string());
        let cause = unscorable_cause(parsed.unscorable_cause.as_deref(), &reason);
        return Ok(SummaryEvaluation {
            result: SummaryEvaluationResult {
                quality_score: None,
                quality_note: Some(format!("**Unscorable**:\n- {reason}")),
                quality_model_used: None,
                summary_tags: tags,
            },
            unscorable_cause: Some(cause),
        });
    }
    if status != "scored" {
        return Err(SummaryEvaluatorError::ParseFailed(format!(
            "unsupported evaluation status `{status}`"
        )));
    }

    let faithfulness_score = optional_score(parsed.faithfulness_score, "faithfulness_score")?;
    let completeness_score = optional_score(parsed.completeness_score, "completeness_score")?;
    let final_score = match parsed.final_score.or(parsed.score) {
        Some(value) => checked_score(value, "final_score")?,
        // Both axes present but no final: use the weaker axis.
        None => match (faithfulness_score, completeness_score) {
            (Some(faithfulness), Some(completeness)) => faithfulness.min(completeness),
            _ => {
                return Err(SummaryEvaluatorError::ParseFailed(
                    "final_score is required".to_string(),
                ));
            }
        },
    };

    let is_structured = parsed.status.is_some()
        || parsed.final_score.is_some()
        || faithfulness_score.is_some()
        || completeness_score.is_some()
        || parsed.defects.is_some();
    let note = if is_structured {
        let defects = parsed
            .defects
            .unwrap_or_default()
            .into_iter()
            .filter_map(defect_with_defaults)
            .collect::<Vec<_>>();
        Some(build_structured_note(
            faithfulness_score,
            completeness_score,
            final_score,
            &defects,
            clean_text(parsed.evaluation_note),
        ))
    } else {
        clean_text(parsed.incoherence_note)
    };

    Ok(SummaryEvaluation {
        result: SummaryEvaluationResult {
            quality_score: Some(final_score),
            quality_note: note,
            quality_model_used: None,
            summary_tags: tags,
        },
        unscorable_cause: None,
    })
}

fn checked_score(value: i64, field: &str) -> Result<u8, SummaryEvaluatorError> {
    if !(0..=10).contains(&value) {
        return Err(SummaryEvaluatorError::ParseFailed(format!(
            "{field} score must be between 0 and 10"
        )));
    }
    Ok(value as u8)
}

fn optional_score(value: Option<i64>, field: &str) -> Result<Option<u8>, SummaryEvaluatorError> {
    value.map(|value| checked_score(value, field)).transpose()
}

/// Fills in a missing type or severity. Drops a defect that has neither a claim
/// nor an anchor, because it carries no evidence.
fn defect_with_defaults(defect: EvaluatorDefect) -> Option<Defect> {
    let summary_claim = clean_text(defect.summary_claim);
    let transcript_anchor = clean_text(defect.transcript_anchor);
    if summary_claim.is_none() && transcript_anchor.is_none() {
        return None;
    }
    Some(Defect {
        defect_type: clean_text(defect.defect_type)
            .unwrap_or_else(|| DEFAULT_DEFECT_TYPE.to_string()),
        severity: clean_text(defect.severity)
            .map(|value| value.to_ascii_lowercase())
            .unwrap_or_else(|| DEFAULT_DEFECT_SEVERITY.to_string()),
        summary_claim,
        transcript_anchor,
    })
}

fn build_structured_note(
    faithfulness_score: Option<u8>,
    completeness_score: Option<u8>,
    final_score: u8,
    defects: &[Defect],
    evaluation_note: Option<String>,
) -> String {
    let mut sections = vec!["**Scores**:".to_string()];
    if let Some(score) = faithfulness_score {
        sections.push(format!("- Faithfulness: {score}/10"));
    }
    if let Some(score) = completeness_score {
        sections.push(format!("- Completeness: {score}/10"));
    }
    sections.push(format!("- Final: {final_score}/10"));

    if !defects.is_empty() {
        sections.push("\n**Defects**:".to_string());
        for defect in defects {
            let mut line = format!(
                "- **{} {}**: {}",
                title_case(&defect.severity),
                defect.defect_type,
                defect
                    .summary_claim
                    .as_deref()
                    .unwrap_or("(no claim given)")
            );
            if let Some(anchor) = &defect.transcript_anchor {
                line.push_str(&format!("; transcript anchor: {anchor}"));
            }
            sections.push(line);
        }
    } else if final_score < 10 {
        sections.push("\n**Defects**:".to_string());
        sections.push("- The evaluator listed no defects.".to_string());
    }

    if let Some(note) = evaluation_note {
        sections.push("\n**Evaluation**:".to_string());
        sections.push(format!("- {note}"));
    }

    sections.join("\n")
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
