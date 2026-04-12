use crate::model::{
    CapabilityClass, ExpectedAnswerability, FAILURE_NO_SOURCES, PromptRunResult, PromptRunStatus,
};
use crate::report::build_summary;
use crate::runner::{
    apply_eval_identity_headers, has_explicit_scope, persistent_chat_requires_ephemeral,
    prompt_refers_to_current_channel, prompt_refers_to_current_video,
};
use crate::sse::{SseAccumulator, parse_sse_block};
use reqwest::header::HeaderMap;

#[test]
fn parse_sse_block_extracts_event_and_data() {
    let block = "event: status\ndata: {\"stage\":\"retrieving\"}\n";
    let event = parse_sse_block(block).expect("event should parse");
    assert_eq!(event.name, "status");
    assert_eq!(event.data, "{\"stage\":\"retrieving\"}");
}

#[test]
fn accumulator_handles_fragmented_chunks() {
    let mut parser = SseAccumulator::default();
    let first = parser.push("event: status\ndata: {\"stage\":\"ret");
    assert!(first.is_empty());
    let second = parser.push("rieving\"}\n\n");
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].name, "status");
}

#[test]
fn summary_groups_failures_by_class() {
    let results = vec![
        PromptRunResult {
            prompt_id: "q001".to_string(),
            prompt: "A".to_string(),
            capability_class: CapabilityClass::Recommendation,
            answerability_expected: ExpectedAnswerability::Yes,
            conversation_id: None,
            status: PromptRunStatus::Completed,
            assistant_content: "Answer".to_string(),
            source_count: 0,
            source_videos: Vec::new(),
            source_channels: Vec::new(),
            used_search_tool: false,
            used_db_tool: false,
            used_conversation_only: false,
            status_trace: Vec::new(),
            tool_calls: Vec::new(),
            latency_ms_total: 0,
            latency_ms_retrieval: None,
            latency_ms_generation: None,
            rubric_answerability_pass: false,
            rubric_grounding_pass: false,
            rubric_shape_pass: false,
            rubric_capability_score: 0,
            failure_class: Some(FAILURE_NO_SOURCES.to_string()),
            notes: Vec::new(),
            raw_error: None,
            raw_sse: None,
        },
        PromptRunResult {
            prompt_id: "q002".to_string(),
            prompt: "B".to_string(),
            capability_class: CapabilityClass::Recommendation,
            answerability_expected: ExpectedAnswerability::Yes,
            conversation_id: None,
            status: PromptRunStatus::Completed,
            assistant_content: "Good answer with enough content to pass all rubric checks."
                .repeat(4),
            source_count: 3,
            source_videos: vec!["Video 1".to_string(), "Video 2".to_string()],
            source_channels: vec!["Channel".to_string()],
            used_search_tool: true,
            used_db_tool: false,
            used_conversation_only: false,
            status_trace: Vec::new(),
            tool_calls: Vec::new(),
            latency_ms_total: 0,
            latency_ms_retrieval: None,
            latency_ms_generation: None,
            rubric_answerability_pass: true,
            rubric_grounding_pass: true,
            rubric_shape_pass: true,
            rubric_capability_score: 3,
            failure_class: None,
            notes: Vec::new(),
            raw_error: None,
            raw_sse: None,
        },
    ];

    let summary = build_summary(&results);
    assert_eq!(summary.total_prompts, 2);
    assert_eq!(summary.passed_prompts, 1);
    assert_eq!(
        summary.failure_counts.get(FAILURE_NO_SOURCES).copied(),
        Some(1)
    );
    assert_eq!(summary.by_capability_class.len(), 1);
    assert_eq!(summary.by_capability_class[0].total, 2);
}

#[test]
fn detects_signed_out_persistent_chat_guardrail() {
    assert!(persistent_chat_requires_ephemeral(
        403,
        "Sign-in required for persistent chat. Signed-out chat stays ephemeral."
    ));
    assert!(!persistent_chat_requires_ephemeral(
        401,
        "Sign-in required for persistent chat. Signed-out chat stays ephemeral."
    ));
    assert!(!persistent_chat_requires_ephemeral(
        403,
        "Operator access required"
    ));
}

#[test]
fn injects_authenticated_eval_identity_headers_when_user_is_set() {
    let mut headers = HeaderMap::new();
    apply_eval_identity_headers(&mut headers, Some("firebase-user-123"), Some("user"))
        .expect("headers should be applied");

    assert_eq!(
        headers
            .get("x-dastill-auth-state")
            .and_then(|value| value.to_str().ok()),
        Some("authenticated")
    );
    assert_eq!(
        headers
            .get("x-dastill-user-id")
            .and_then(|value| value.to_str().ok()),
        Some("firebase-user-123")
    );
    assert_eq!(
        headers
            .get("x-dastill-role")
            .and_then(|value| value.to_str().ok()),
        Some("user")
    );
}

#[test]
fn detects_deictic_prompt_scope_needs() {
    assert!(prompt_refers_to_current_video(
        "Give me a quick summary of this video in three bullets."
    ));
    assert!(prompt_refers_to_current_channel(
        "What does this creator think about OpenAI?"
    ));
    assert!(has_explicit_scope(
        "+{OpenAI just dropped their Cursor killer} summarize this video"
    ));
    assert!(!prompt_refers_to_current_video(
        "Find every video that mentions RAG."
    ));
}
