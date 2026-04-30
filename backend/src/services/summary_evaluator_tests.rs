use super::{
    EvaluatorResponse, SummaryEvaluationResult, SummaryEvaluatorError, SummaryEvaluatorService,
    evaluation_preamble, evaluation_prompt, evaluation_result_from_response,
    evaluator_response_schema,
};
use crate::models::AiStatus;
use crate::services::ollama::OllamaCore;

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

    evaluation_result_from_response(parsed)
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
