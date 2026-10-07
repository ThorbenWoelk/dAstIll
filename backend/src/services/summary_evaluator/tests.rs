use super::{
    EvaluatorResponse, SummaryEvaluation, SummaryEvaluatorError, SummaryEvaluatorService,
    UnscorableCause, evaluation_preamble, evaluation_prompt, evaluation_result_from_response,
    evaluator_response_schema, transcript_for_evaluation,
};
use crate::models::{AiStatus, SummaryEvaluationResult, VocabularyReplacement};
use crate::services::ollama::OllamaCore;

fn parse_evaluation(raw: &str) -> Result<SummaryEvaluation, SummaryEvaluatorError> {
    let start = raw
        .find('{')
        .ok_or_else(|| SummaryEvaluatorError::ParseFailed("missing json object".to_string()))?;
    let end = raw
        .rfind('}')
        .ok_or_else(|| SummaryEvaluatorError::ParseFailed("missing json object".to_string()))?;

    let json = &raw[start..=end];
    let parsed: EvaluatorResponse = serde_json::from_str(json)
        .map_err(|err| SummaryEvaluatorError::ParseFailed(err.to_string()))?;

    evaluation_result_from_response(parsed)
}

fn parse_evaluation_response(raw: &str) -> Result<SummaryEvaluationResult, SummaryEvaluatorError> {
    parse_evaluation(raw).map(|evaluation| evaluation.result)
}

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
    assert!(SummaryEvaluatorService::validate_model_policy("glm-5.3:cloud").is_ok());
    assert!(SummaryEvaluatorService::validate_model_policy("deepseek-v4-pro:cloud").is_ok());
    assert!(SummaryEvaluatorService::validate_model_policy("kimi-k3:cloud").is_ok());
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
fn parse_evaluation_response_accepts_non_perfect_scores_without_defects() {
    // Production shape: "defects are required for scores below 10" used to drop the whole result.
    let parsed = parse_evaluation_response(
        r#"{
          "status": "scored",
          "faithfulness_score": 8,
          "completeness_score": 7,
          "final_score": 7,
          "defects": [],
          "evaluation_note": "Some issues exist."
        }"#,
    )
    .expect("a score without defect details is still usable");

    assert_eq!(parsed.quality_score, Some(7));
    let note = parsed.quality_note.expect("note");
    assert!(note.contains("The evaluator listed no defects."));
    assert!(note.contains("Some issues exist."));
}

#[test]
fn parse_evaluation_response_keeps_defect_with_blank_anchor() {
    let parsed = parse_evaluation_response(
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
    .expect("a defect with a claim but no anchor is still evidence");

    let note = parsed.quality_note.expect("note");
    assert!(note.contains("- **Major hallucination**: Title-derived claim"));
    assert!(!note.contains("transcript anchor:"));
}

#[test]
fn parse_evaluation_response_defaults_missing_defect_severity_to_minor() {
    // Production shape: "missing field `severity`" used to fail the whole decode.
    let parsed = parse_evaluation_response(
        r#"{
          "status": "scored",
          "unscorable_reason": null,
          "faithfulness_score": 9,
          "completeness_score": 8,
          "final_score": 8,
          "defects": [
            {
              "type": "omission",
              "summary_claim": "Skips the pricing section.",
              "transcript_anchor": "Let's talk about pricing."
            }
          ],
          "evaluation_note": null,
          "tags": ["Pricing"]
        }"#,
    )
    .expect("missing severity should default");

    assert_eq!(parsed.quality_score, Some(8));
    let note = parsed.quality_note.expect("note");
    assert!(note.contains(
        "- **Minor omission**: Skips the pricing section.; transcript anchor: Let's talk about pricing."
    ));
}

#[test]
fn parse_evaluation_response_accepts_missing_axis_scores() {
    // Production shape: "faithfulness_score is required" used to drop the whole result.
    let parsed = parse_evaluation_response(
        r#"{
          "status": "scored",
          "final_score": 9,
          "defects": [
            {
              "type": "omission",
              "severity": "minor",
              "summary_claim": "Leaves out the closing example.",
              "transcript_anchor": "One last example."
            }
          ],
          "tags": []
        }"#,
    )
    .expect("final score alone is usable");

    assert_eq!(parsed.quality_score, Some(9));
    let note = parsed.quality_note.expect("note");
    assert!(!note.contains("Faithfulness"));
    assert!(!note.contains("Completeness"));
    assert!(note.contains("- Final: 9/10"));
}

#[test]
fn parse_evaluation_response_uses_weaker_axis_when_final_score_missing() {
    let parsed = parse_evaluation_response(
        r#"{
          "status": "scored",
          "faithfulness_score": 9,
          "completeness_score": 6,
          "defects": [
            { "type": "omission", "summary_claim": "Misses section two." }
          ]
        }"#,
    )
    .unwrap();

    assert_eq!(parsed.quality_score, Some(6));
}

#[test]
fn parse_evaluation_response_accepts_numeric_strings_and_whole_floats() {
    let parsed = parse_evaluation_response(
        r#"{ "status": "Scored", "faithfulness_score": "10", "completeness_score": 10.0, "final_score": 10 }"#,
    )
    .unwrap();

    assert_eq!(parsed.quality_score, Some(10));
}

#[test]
fn parse_evaluation_response_rejects_scored_output_without_any_score() {
    let err = parse_evaluation_response(
        r#"{ "status": "scored", "defects": [], "evaluation_note": "Looks fine." }"#,
    )
    .expect_err("a scored answer with no score is unusable");

    assert!(err.to_string().contains("final_score is required"));
}

#[test]
fn parse_evaluation_response_rejects_unknown_status() {
    let err = parse_evaluation_response(r#"{ "status": "maybe", "final_score": 8 }"#)
        .expect_err("unknown status is unusable");

    assert!(err.to_string().contains("unsupported evaluation status"));
}

#[test]
fn parse_evaluation_response_drops_defects_without_any_evidence() {
    let parsed = parse_evaluation_response(
        r#"{
          "status": "scored",
          "final_score": 8,
          "defects": [ { "type": "omission", "severity": "minor" } ]
        }"#,
    )
    .unwrap();

    let note = parsed.quality_note.expect("note");
    assert!(note.contains("The evaluator listed no defects."));
}

#[test]
fn unscorable_cause_uses_explicit_field() {
    let evaluation = parse_evaluation(
        r#"{
          "status": "unscorable",
          "unscorable_reason": "Cannot compare.",
          "unscorable_cause": "summary"
        }"#,
    )
    .unwrap();

    assert_eq!(evaluation.result.quality_score, None);
    assert_eq!(evaluation.unscorable_cause, Some(UnscorableCause::Summary));
}

#[test]
fn unscorable_cause_falls_back_to_reason_text() {
    let summary_problem = parse_evaluation(
        r#"{ "status": "unscorable", "unscorable_reason": "The summary is too malformed to compare." }"#,
    )
    .unwrap();
    assert_eq!(
        summary_problem.unscorable_cause,
        Some(UnscorableCause::Summary)
    );

    let transcript_problem = parse_evaluation(
        r#"{ "status": "unscorable", "unscorable_reason": "Transcript is show notes, not spoken content." }"#,
    )
    .unwrap();
    assert_eq!(
        transcript_problem.unscorable_cause,
        Some(UnscorableCause::Transcript)
    );

    let no_reason = parse_evaluation(r#"{ "status": "unscorable" }"#).unwrap();
    assert_eq!(
        no_reason.unscorable_cause,
        Some(UnscorableCause::Transcript)
    );
    assert_eq!(
        no_reason.result.quality_note.as_deref(),
        Some("**Unscorable**:\n- The evaluator gave no reason.")
    );
}

#[test]
fn scored_evaluation_has_no_unscorable_cause() {
    let evaluation = parse_evaluation(r#"{ "status": "scored", "final_score": 10 }"#).unwrap();
    assert_eq!(evaluation.unscorable_cause, None);
}

#[test]
fn transcript_for_evaluation_applies_vocabulary_replacements() {
    let replacements = vec![VocabularyReplacement {
        from: "clawed".to_string(),
        to: "Claude".to_string(),
        added_at: chrono::Utc::now(),
    }];

    let transcript = transcript_for_evaluation("I asked clawed about it.", &replacements);

    assert!(transcript.contains("Claude"));
    assert!(!transcript.contains("clawed"));
}

#[test]
fn evaluator_schema_offers_unscorable_cause() {
    let schema = evaluator_response_schema();
    assert!(schema["properties"]["unscorable_cause"].is_object());
    let required = schema["required"].as_array().expect("required list");
    assert!(required.iter().any(|value| value == "unscorable_cause"));
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
    assert!(prompt.contains(
        "Focus on substantive problems; do not pad the note with praise and do not invent flaws."
    ));
    assert!(prompt.contains("Return one JSON object matching the runtime schema."));
    assert!(!prompt.contains("\"faithfulness_score\""));
    let schema = evaluator_response_schema();
    assert!(schema["properties"]["faithfulness_score"].is_object());
    assert!(schema["properties"]["completeness_score"].is_object());
    assert!(schema["properties"]["final_score"].is_object());
    assert!(schema["properties"]["defects"].is_object());
    assert!(prompt.contains("\"unscorable\""));
    assert!(prompt.contains("scores below 10 require at least one defect"));
    assert!(prompt.contains("7 is acceptable"));
}

#[test]
fn evaluation_prompt_asks_for_the_fields_models_tend_to_drop() {
    let prompt = evaluation_prompt("Title", "Transcript text.", "- Summary");

    assert!(prompt.contains("a severity (minor or major)"));
    assert!(prompt.contains("always give all three scores"));
    assert!(prompt.contains("unscorable cause"));
}
