use crate::model::{
    CapabilityClass, ExpectedAnswerability, FAILURE_NO_SOURCES, PromptRunResult, PromptRunStatus,
};
use crate::report::build_summary;
use crate::sse::{SseAccumulator, parse_sse_block};

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
