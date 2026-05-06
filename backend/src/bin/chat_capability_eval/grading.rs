use std::collections::{BTreeMap, BTreeSet, HashSet};

use dastill::models::ChatSource;

use crate::model::{
    CapabilityClass, ExpectedAnswerability, FAILURE_GENERIC, FAILURE_NO_SOURCES, FAILURE_SHAPE,
    FAILURE_SINGLE_VIDEO, FAILURE_STREAM, FAILURE_UNSUPPORTED, PromptRunResult, PromptRunStatus,
    PromptSpec, TimedStatus, ToolCallReport,
};
use crate::sse::ParsedStream;

fn capability_score(
    content: &str,
    answerability_pass: bool,
    grounding_pass: bool,
    shape_pass: bool,
    stream_completed: bool,
) -> u8 {
    if !stream_completed || content.trim().is_empty() {
        return 0;
    }
    let mut score = 0;
    if answerability_pass {
        score += 1;
    }
    if grounding_pass {
        score += 1;
    }
    if shape_pass && content.trim().len() >= 160 {
        score += 1;
    }
    score
}

fn looks_like_leading_refusal(normalized: &str, phrase: &str) -> bool {
    let trimmed = normalized.trim_start();
    trimmed.starts_with(phrase)
        || trimmed
            .lines()
            .take(2)
            .any(|line| line.trim_start().starts_with(phrase))
}

fn unique_video_ids(sources: &[ChatSource]) -> HashSet<String> {
    sources
        .iter()
        .map(|source| source.video_id.clone())
        .collect()
}

fn classify_grounding_failure(spec: &PromptSpec, sources: &[ChatSource]) -> String {
    if sources.is_empty() {
        FAILURE_NO_SOURCES.to_string()
    } else if spec.requires_cross_video_synthesis && unique_video_ids(sources).len() < 2 {
        FAILURE_SINGLE_VIDEO.to_string()
    } else {
        FAILURE_NO_SOURCES.to_string()
    }
}

fn unique_video_titles(sources: &[ChatSource]) -> Vec<String> {
    let mut values = BTreeSet::new();
    for source in sources {
        values.insert(source.video_title.clone());
    }
    values.into_iter().collect()
}

fn unique_channel_names(sources: &[ChatSource]) -> Vec<String> {
    let mut values = BTreeSet::new();
    for source in sources {
        values.insert(source.channel_name.clone());
    }
    values.into_iter().collect()
}

fn retrieval_latency_ms(statuses: &[TimedStatus]) -> Option<u64> {
    statuses
        .iter()
        .find(|status| status.payload.stage == "retrieving_complete")
        .map(|status| status.received_at_ms)
        .or_else(|| {
            statuses
                .iter()
                .find(|status| status.payload.stage == "tool_complete")
                .map(|status| status.received_at_ms)
        })
}

fn generation_latency_ms(statuses: &[TimedStatus]) -> Option<u64> {
    let generation_start = statuses
        .iter()
        .find(|status| status.payload.stage == "generating")
        .map(|status| status.received_at_ms)?;
    let done = statuses.last().map(|status| status.received_at_ms)?;
    Some(done.saturating_sub(generation_start))
}

fn contains_timestamp(content: &str) -> bool {
    let bytes = content.as_bytes();
    for window in bytes.windows(5) {
        if window[0].is_ascii_digit()
            && window[1].is_ascii_digit()
            && window[2] == b':'
            && window[3].is_ascii_digit()
            && window[4].is_ascii_digit()
        {
            return true;
        }
    }
    false
}

fn grounding_pass(
    spec: &PromptSpec,
    content: &str,
    sources: &[ChatSource],
    used_db_tool: bool,
    used_highlight_tool: bool,
    notes: &mut Vec<String>,
) -> bool {
    if spec.requires_highlights && !used_highlight_tool {
        notes.push("highlight prompt did not use the saved highlights tool".to_string());
        return false;
    }

    if sources.is_empty() && !used_db_tool && !spec.requires_highlights {
        notes.push("no grounding sources were attached".to_string());
        return false;
    }

    let unique_videos = unique_video_ids(sources);
    if spec.requires_cross_video_synthesis && unique_videos.len() < 2 {
        if spec.requires_highlights && used_highlight_tool {
            return true;
        }
        notes.push("cross-video prompt drew from fewer than two source videos".to_string());
        return false;
    }

    if spec.requires_timestamp {
        let normalized = content.to_ascii_lowercase();
        let has_timestamp = contains_timestamp(content)
            || normalized.contains("timestamp")
            || normalized.contains("time code")
            || normalized.contains("timed captions unavailable")
            || normalized.contains("no timestamp")
            || normalized.contains("couldn't find a timestamp");
        if !has_timestamp {
            notes.push(
                "timestamp-oriented answer did not surface timestamp information or a timing caveat"
                    .to_string(),
            );
            return false;
        }
    }

    true
}

fn has_list_shape(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("1. ")
            || trimmed.starts_with("2. ")
            || trimmed.starts_with("3. ")
    })
}

fn merge_tool_calls(statuses: &[TimedStatus]) -> Vec<ToolCallReport> {
    let mut merged = BTreeMap::<String, ToolCallReport>::new();
    for status in statuses {
        let Some(tool) = &status.payload.tool else {
            continue;
        };
        let key = format!("{}:{}", tool.name, tool.input);
        let existing = merged.get(&key).cloned();
        merged.insert(
            key,
            ToolCallReport {
                name: tool.name.clone(),
                label: tool.label.clone(),
                state: tool.state.clone(),
                input: tool.input.clone(),
                output: tool
                    .output
                    .clone()
                    .or(existing.and_then(|value| value.output)),
            },
        );
    }
    merged.into_values().collect()
}

fn unsupported_library_phrases() -> &'static [&'static str] {
    &[
        "i don't have access to your library",
        "i do not have access to your library",
        "i can't access your library",
        "i cannot access your library",
        "i don't have access to your videos",
        "i do not have direct access",
        "without access to your account data",
        "i cannot list your saved highlights",
    ]
}

fn classify_answerability_failure(content: &str) -> String {
    let normalized = content.to_ascii_lowercase();
    if unsupported_library_phrases()
        .iter()
        .any(|phrase| looks_like_leading_refusal(&normalized, phrase))
    {
        FAILURE_UNSUPPORTED.to_string()
    } else {
        FAILURE_GENERIC.to_string()
    }
}

fn generic_failure_phrases() -> &'static [&'static str] {
    &[
        "i can't answer that",
        "i cannot answer that",
        "not enough information",
        "i don't know",
        "i do not know",
    ]
}

fn answerability_pass(spec: &PromptSpec, content: &str, notes: &mut Vec<String>) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        notes.push("assistant content was empty".to_string());
        return false;
    }

    let normalized = trimmed.to_ascii_lowercase();
    if unsupported_library_phrases()
        .iter()
        .any(|phrase| looks_like_leading_refusal(&normalized, phrase))
    {
        notes.push("assistant claimed missing library access".to_string());
        return false;
    }

    if generic_failure_phrases()
        .iter()
        .any(|phrase| looks_like_leading_refusal(&normalized, phrase))
    {
        notes.push("assistant returned a generic failure or refusal".to_string());
        return false;
    }

    let min_len = match spec.answerability_expected {
        ExpectedAnswerability::Yes => 80,
        ExpectedAnswerability::Partial => 45,
    };
    if trimmed.len() < min_len {
        notes.push(format!(
            "assistant answer was too short for the expected prompt type ({} chars)",
            trimmed.len()
        ));
        return false;
    }

    true
}

fn caveat_markers() -> &'static [&'static str] {
    &[
        "it seems",
        "appears",
        "likely",
        "probably",
        "inference",
        "without knowing",
        "if you are asking",
        "if you're referring",
        "there is no direct statement",
        "none explicitly",
        "cannot determine",
        "based on the excerpts",
        "from the available evidence",
    ]
}

fn contains_caveat_marker(normalized: &str) -> bool {
    caveat_markers()
        .iter()
        .any(|marker| normalized.contains(marker))
}

fn contrast_markers() -> &'static [&'static str] {
    &[
        "however",
        "in contrast",
        "while",
        "whereas",
        "on the other hand",
        "compared with",
        "different",
        "disagree",
        "unrelated",
        "counterargument",
        "opposing",
        "difference",
        "similarity",
    ]
}

fn shape_pass(
    spec: &PromptSpec,
    content: &str,
    sources: &[ChatSource],
    notes: &mut Vec<String>,
) -> bool {
    let normalized = content.to_ascii_lowercase();
    match spec.capability_class {
        CapabilityClass::Recommendation => {
            if sources.is_empty() {
                notes.push("recommendation answer had no supporting sources".to_string());
                return false;
            }
            if !has_list_shape(content) && unique_video_ids(sources).len() > 1 {
                notes.push("recommendation answer did not present a list-like ranking".to_string());
                return false;
            }
        }
        CapabilityClass::Comparison => {
            if unique_video_ids(sources).len() < 2 {
                notes.push("comparison answer did not draw from at least two videos".to_string());
                return false;
            }
            if !contrast_markers()
                .iter()
                .any(|marker| normalized.contains(marker))
            {
                notes.push("comparison answer lacked explicit contrast language".to_string());
                return false;
            }
        }
        CapabilityClass::TopicAggregation | CapabilityClass::CrossVideoSynthesis => {
            if unique_video_ids(sources).len() < 2 {
                notes.push("aggregation answer did not cover enough distinct videos".to_string());
                return false;
            }
            if !has_list_shape(content)
                && !normalized.contains("theme")
                && !normalized.contains("pattern")
                && !normalized.contains("across")
            {
                notes.push("aggregation answer did not look grouped or thematic".to_string());
                return false;
            }
        }
        CapabilityClass::HighlightLookup | CapabilityClass::HighlightClustering => {
            if !normalized.contains("highlight") && !normalized.contains("snippet") {
                notes.push(
                    "highlight answer did not explicitly reference highlights or snippets"
                        .to_string(),
                );
                return false;
            }
        }
        CapabilityClass::TranscriptSummaryAlignment => {
            if !normalized.contains("summary") || !normalized.contains("transcript") {
                notes.push(
                    "alignment answer did not explicitly discuss both summary and transcript"
                        .to_string(),
                );
                return false;
            }
        }
        CapabilityClass::ToneOrStyleInference => {
            if !contains_caveat_marker(&normalized)
                && spec.answerability_expected == ExpectedAnswerability::Partial
            {
                notes.push(
                    "tone or style inference answer did not include a visible caveat".to_string(),
                );
                return false;
            }
        }
        CapabilityClass::MetaLearningOrNextStep => {
            if !normalized.contains("next")
                && !normalized.contains("follow-up")
                && !normalized.contains("learn")
                && !normalized.contains("question")
            {
                notes.push(
                    "next-step answer did not present a clear next step or follow-up".to_string(),
                );
                return false;
            }
        }
        CapabilityClass::TimestampNavigation => {
            if !contains_timestamp(content)
                && !normalized.contains("timestamp")
                && !normalized.contains("section")
            {
                notes.push(
                    "timestamp-navigation answer did not identify a section or time".to_string(),
                );
                return false;
            }
        }
        CapabilityClass::DirectLookup | CapabilityClass::CreatorStance => {}
    }

    true
}

pub(crate) fn grade_prompt_result(
    spec: &PromptSpec,
    conversation_id: Option<String>,
    parsed: ParsedStream,
    total_ms: u64,
) -> PromptRunResult {
    let tool_calls = merge_tool_calls(&parsed.statuses);
    let used_search_tool = tool_calls.iter().any(|tool| tool.name == "search_library");
    let used_db_tool = tool_calls.iter().any(|tool| tool.name == "db_inspect");
    let used_highlight_tool = tool_calls
        .iter()
        .any(|tool| tool.name == "highlight_lookup");
    let used_conversation_only = parsed.statuses.iter().any(|status| {
        status
            .payload
            .plan
            .as_ref()
            .and_then(|plan| plan.skip_retrieval)
            .unwrap_or(false)
    }) || parsed.statuses.iter().any(|status| {
        status
            .payload
            .label
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains("conversation")
    });

    let assistant_message = parsed.final_message.clone();
    let assistant_content = assistant_message
        .as_ref()
        .map(|message| message.content.trim().to_string())
        .unwrap_or_default();
    let final_sources = assistant_message
        .as_ref()
        .map(|message| message.sources.clone())
        .filter(|sources| !sources.is_empty())
        .unwrap_or(parsed.latest_sources.clone());

    let source_videos = unique_video_titles(&final_sources);
    let source_channels = unique_channel_names(&final_sources);
    let source_count = final_sources.len();
    let latency_ms_retrieval = retrieval_latency_ms(&parsed.statuses);
    let latency_ms_generation = generation_latency_ms(&parsed.statuses);

    let mut notes = Vec::new();
    let mut failure_class = None;

    if parsed.error_message.is_some() {
        notes.push("stream ended with an explicit error event".to_string());
        failure_class = Some(FAILURE_STREAM.to_string());
    }

    let answerability_pass = answerability_pass(spec, &assistant_content, &mut notes);
    if !answerability_pass && failure_class.is_none() {
        failure_class = Some(classify_answerability_failure(&assistant_content));
    }

    let grounding_pass = grounding_pass(
        spec,
        &assistant_content,
        &final_sources,
        used_db_tool,
        used_highlight_tool,
        &mut notes,
    );
    if !grounding_pass && failure_class.is_none() {
        failure_class = Some(classify_grounding_failure(spec, &final_sources));
    }

    let shape_pass = shape_pass(spec, &assistant_content, &final_sources, &mut notes);
    if !shape_pass && failure_class.is_none() {
        failure_class = Some(FAILURE_SHAPE.to_string());
    }

    let capability_score = capability_score(
        &assistant_content,
        answerability_pass,
        grounding_pass,
        shape_pass,
        parsed.error_message.is_none(),
    );

    PromptRunResult {
        prompt_id: spec.id.clone(),
        prompt: spec.prompt.clone(),
        capability_class: spec.capability_class,
        answerability_expected: spec.answerability_expected,
        conversation_id,
        status: if parsed.error_message.is_some() {
            PromptRunStatus::StreamError
        } else if assistant_message.is_some() {
            PromptRunStatus::Completed
        } else {
            PromptRunStatus::ParseError
        },
        assistant_content,
        source_count,
        source_videos,
        source_channels,
        used_search_tool,
        used_db_tool,
        used_conversation_only,
        status_trace: parsed.statuses,
        tool_calls,
        latency_ms_total: total_ms,
        latency_ms_retrieval,
        latency_ms_generation,
        rubric_answerability_pass: answerability_pass,
        rubric_grounding_pass: grounding_pass,
        rubric_shape_pass: shape_pass,
        rubric_capability_score: capability_score,
        failure_class,
        notes,
        raw_error: parsed.error_message,
        raw_sse: Some(parsed.raw_sse),
    }
}
