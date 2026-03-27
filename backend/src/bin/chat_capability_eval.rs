use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use dastill::models::{ChatConversation, ChatMessage, ChatSource};
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

const DEFAULT_BASE_URL: &str = "http://localhost:3544";
const DEFAULT_PROXY_TOKEN: &str = "local-dev-backend-proxy-token";
const FAILURE_NO_SOURCES: &str = "no_sources";
const FAILURE_SINGLE_VIDEO: &str = "single_video_overfit";
const FAILURE_GENERIC: &str = "generic_answer";
const FAILURE_SHAPE: &str = "shape_mismatch";
const FAILURE_STREAM: &str = "stream_error";
const FAILURE_UNSUPPORTED: &str = "unsupported_capability";

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PromptSpec {
    id: String,
    prompt: String,
    search_strategy_expected: String,
    answerability_expected: ExpectedAnswerability,
    good_answer_shape: String,
    capability_class: CapabilityClass,
    requires_timestamp: bool,
    requires_highlights: bool,
    requires_quality_score: bool,
    requires_cross_video_synthesis: bool,
    requires_opinion_inference: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ExpectedAnswerability {
    Yes,
    Partial,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
enum CapabilityClass {
    DirectLookup,
    TopicAggregation,
    CrossVideoSynthesis,
    Comparison,
    Recommendation,
    CreatorStance,
    HighlightLookup,
    HighlightClustering,
    TranscriptSummaryAlignment,
    TimestampNavigation,
    ToneOrStyleInference,
    MetaLearningOrNextStep,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct StreamStatusPayload {
    stage: String,
    label: Option<String>,
    detail: Option<String>,
    decision: Option<String>,
    plan: Option<StreamPlanPayload>,
    tool: Option<StreamToolPayload>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct StreamPlanPayload {
    intent: Option<String>,
    label: String,
    budget: usize,
    max_per_video: usize,
    queries: Vec<String>,
    expansion_queries: Vec<String>,
    rationale: Option<String>,
    skip_retrieval: Option<bool>,
    deep_research: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct StreamToolPayload {
    name: String,
    label: String,
    state: String,
    input: String,
    output: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SourcesEventPayload {
    sources: Vec<ChatSource>,
}

#[derive(Debug, Clone, Deserialize)]
struct TokenEventPayload {
    #[serde(rename = "token")]
    _token: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DoneEventPayload {
    message: ChatMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct ErrorEventPayload {
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct TimedStatus {
    received_at_ms: u64,
    payload: StreamStatusPayload,
}

#[derive(Debug, Clone, Serialize)]
struct ToolCallReport {
    name: String,
    label: String,
    state: String,
    input: String,
    output: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum PromptRunStatus {
    Completed,
    StreamError,
    HttpError,
    ParseError,
}

#[derive(Debug, Clone, Serialize)]
struct PromptRunResult {
    prompt_id: String,
    prompt: String,
    capability_class: CapabilityClass,
    answerability_expected: ExpectedAnswerability,
    conversation_id: Option<String>,
    status: PromptRunStatus,
    assistant_content: String,
    source_count: usize,
    source_videos: Vec<String>,
    source_channels: Vec<String>,
    used_search_tool: bool,
    used_db_tool: bool,
    used_conversation_only: bool,
    status_trace: Vec<TimedStatus>,
    tool_calls: Vec<ToolCallReport>,
    latency_ms_total: u64,
    latency_ms_retrieval: Option<u64>,
    latency_ms_generation: Option<u64>,
    rubric_answerability_pass: bool,
    rubric_grounding_pass: bool,
    rubric_shape_pass: bool,
    rubric_capability_score: u8,
    failure_class: Option<String>,
    notes: Vec<String>,
    raw_error: Option<String>,
    raw_sse: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CapabilitySummary {
    capability_class: CapabilityClass,
    total: usize,
    passed: usize,
    answerability_passed: usize,
    grounding_passed: usize,
    shape_passed: usize,
    average_score: f32,
    common_failure_classes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SweepSummary {
    total_prompts: usize,
    passed_prompts: usize,
    answerability_passed: usize,
    grounding_passed: usize,
    shape_passed: usize,
    average_score: f32,
    prompts_without_sources: Vec<String>,
    single_video_prompts: Vec<String>,
    failure_counts: BTreeMap<String, usize>,
    by_capability_class: Vec<CapabilitySummary>,
}

#[derive(Debug, Clone, Serialize)]
struct SweepReport {
    generated_at_utc: String,
    base_url: String,
    dataset_path: String,
    prompt_count: usize,
    summary: SweepSummary,
    results: Vec<PromptRunResult>,
}

#[derive(Debug, Default)]
struct CliConfig {
    base_url: String,
    dataset_path: PathBuf,
    output_dir: PathBuf,
    timeout: Duration,
    deep_research: bool,
    model: Option<String>,
    class_filters: HashSet<CapabilityClass>,
    prompt_id_filters: HashSet<String>,
}

#[derive(Debug)]
struct SseEvent {
    name: String,
    data: String,
}

#[derive(Debug, Default)]
struct SseAccumulator {
    buffer: String,
}

#[derive(Debug)]
struct ParsedStream {
    statuses: Vec<TimedStatus>,
    latest_sources: Vec<ChatSource>,
    final_message: Option<ChatMessage>,
    error_message: Option<String>,
    raw_sse: String,
}

#[derive(Debug)]
struct SweepRunner {
    client: Client,
    base_url: String,
    default_headers: HeaderMap,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_args()?;
    let prompts = load_prompt_specs(&config.dataset_path)?;
    let filtered = filter_prompts(prompts, &config.class_filters, &config.prompt_id_filters);
    if filtered.is_empty() {
        bail!("no prompts matched the provided filters");
    }

    ensure_backend_ready(&config.base_url).await?;
    fs::create_dir_all(&config.output_dir)
        .with_context(|| format!("failed to create {}", config.output_dir.display()))?;

    let runner = SweepRunner::new(&config.base_url, config.timeout)?;
    let mut results = Vec::with_capacity(filtered.len());

    for (index, spec) in filtered.iter().enumerate() {
        println!(
            "[{}/{}] {} {}",
            index + 1,
            filtered.len(),
            spec.id,
            spec.prompt
        );
        let result = runner
            .run_prompt(spec, config.deep_research, config.model.as_deref())
            .await
            .with_context(|| format!("failed prompt {}", spec.id))?;
        let status_label = if prompt_passed(&result) {
            "PASS"
        } else {
            "FAIL"
        };
        println!(
            "  -> {} score={} sources={} failure={}",
            status_label,
            result.rubric_capability_score,
            result.source_count,
            result.failure_class.as_deref().unwrap_or("-")
        );
        results.push(result);
    }

    let summary = build_summary(&results);
    let report = SweepReport {
        generated_at_utc: chrono::Utc::now().to_rfc3339(),
        base_url: config.base_url.clone(),
        dataset_path: config.dataset_path.display().to_string(),
        prompt_count: results.len(),
        summary,
        results,
    };

    write_reports(&config.output_dir, &report)?;
    println!("Wrote reports to {}", config.output_dir.as_path().display());

    Ok(())
}

impl SweepRunner {
    fn new(base_url: &str, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .context("failed to build HTTP client")?;
        let mut default_headers = HeaderMap::new();
        default_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        default_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        default_headers.insert(
            "x-dastill-proxy-auth",
            HeaderValue::from_str(
                &env::var("BACKEND_PROXY_TOKEN")
                    .unwrap_or_else(|_| DEFAULT_PROXY_TOKEN.to_string()),
            )
            .context("invalid BACKEND_PROXY_TOKEN header value")?,
        );
        default_headers.insert("x-dastill-role", HeaderValue::from_static("operator"));
        default_headers.insert("x-dastill-client-ip", HeaderValue::from_static("127.0.0.1"));

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            default_headers,
        })
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn create_conversation(&self) -> Result<ChatConversation> {
        let response = self
            .client
            .post(self.api_url("/api/chat/conversations"))
            .headers(self.default_headers.clone())
            .json(&serde_json::json!({ "title": null }))
            .send()
            .await
            .context("failed to create conversation")?;

        if response.status() != StatusCode::CREATED {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("conversation create failed: {status} {body}");
        }

        response
            .json::<ChatConversation>()
            .await
            .context("failed to decode conversation create response")
    }

    async fn send_prompt(
        &self,
        conversation_id: &str,
        prompt: &str,
        deep_research: bool,
        model: Option<&str>,
    ) -> Result<ParsedStream> {
        let mut request_body = serde_json::json!({
            "content": prompt,
            "deep_research": deep_research,
        });
        if let Some(model) = model {
            request_body["model"] = serde_json::Value::String(model.to_string());
        }

        let mut headers = self.default_headers.clone();
        headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));

        let response = self
            .client
            .post(self.api_url(&format!(
                "/api/chat/conversations/{conversation_id}/messages"
            )))
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .context("failed to start chat stream")?;

        if response.status() != StatusCode::OK {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("chat stream start failed: {status} {body}");
        }

        let started_at = Instant::now();
        let mut response = response;
        let mut parser = SseAccumulator::default();
        let mut raw_sse = String::new();
        let mut statuses = Vec::new();
        let mut latest_sources = Vec::new();
        let mut final_message = None;
        let mut error_message = None;

        while let Some(chunk) = response
            .chunk()
            .await
            .context("failed to read stream chunk")?
        {
            let text = String::from_utf8_lossy(&chunk);
            raw_sse.push_str(&text);
            for event in parser.push(&text) {
                let received_at_ms = started_at.elapsed().as_millis() as u64;
                match event.name.as_str() {
                    "status" => {
                        let payload = serde_json::from_str::<StreamStatusPayload>(&event.data)
                            .with_context(|| {
                                format!("failed to parse status payload: {}", event.data)
                            })?;
                        statuses.push(TimedStatus {
                            received_at_ms,
                            payload,
                        });
                    }
                    "sources" => {
                        let payload = serde_json::from_str::<SourcesEventPayload>(&event.data)
                            .with_context(|| {
                                format!("failed to parse sources payload: {}", event.data)
                            })?;
                        latest_sources = payload.sources;
                    }
                    "token" => {
                        let _ = serde_json::from_str::<TokenEventPayload>(&event.data)
                            .with_context(|| {
                                format!("failed to parse token payload: {}", event.data)
                            })?;
                    }
                    "done" => {
                        let payload = serde_json::from_str::<DoneEventPayload>(&event.data)
                            .with_context(|| {
                                format!("failed to parse done payload: {}", event.data)
                            })?;
                        final_message = Some(payload.message);
                    }
                    "error" => {
                        let payload = serde_json::from_str::<ErrorEventPayload>(&event.data)
                            .with_context(|| {
                                format!("failed to parse error payload: {}", event.data)
                            })?;
                        error_message = Some(payload.message);
                    }
                    _ => {}
                }
            }
        }

        for event in parser.finish() {
            let received_at_ms = started_at.elapsed().as_millis() as u64;
            match event.name.as_str() {
                "status" => {
                    let payload = serde_json::from_str::<StreamStatusPayload>(&event.data)
                        .with_context(|| {
                            format!("failed to parse status payload: {}", event.data)
                        })?;
                    statuses.push(TimedStatus {
                        received_at_ms,
                        payload,
                    });
                }
                "sources" => {
                    let payload = serde_json::from_str::<SourcesEventPayload>(&event.data)
                        .with_context(|| {
                            format!("failed to parse sources payload: {}", event.data)
                        })?;
                    latest_sources = payload.sources;
                }
                "token" => {
                    let _ = serde_json::from_str::<TokenEventPayload>(&event.data).with_context(
                        || format!("failed to parse token payload: {}", event.data),
                    )?;
                }
                "done" => {
                    let payload = serde_json::from_str::<DoneEventPayload>(&event.data)
                        .with_context(|| format!("failed to parse done payload: {}", event.data))?;
                    final_message = Some(payload.message);
                }
                "error" => {
                    let payload = serde_json::from_str::<ErrorEventPayload>(&event.data)
                        .with_context(|| {
                            format!("failed to parse error payload: {}", event.data)
                        })?;
                    error_message = Some(payload.message);
                }
                _ => {}
            }
        }

        Ok(ParsedStream {
            statuses,
            latest_sources,
            final_message,
            error_message,
            raw_sse,
        })
    }

    async fn run_prompt(
        &self,
        spec: &PromptSpec,
        deep_research: bool,
        model: Option<&str>,
    ) -> Result<PromptRunResult> {
        let started_at = Instant::now();
        let conversation = self.create_conversation().await?;
        let conversation_id = conversation.id.clone();

        let stream = self
            .send_prompt(&conversation_id, &spec.prompt, deep_research, model)
            .await;

        let total_ms = started_at.elapsed().as_millis() as u64;

        match stream {
            Ok(parsed) => {
                let result = grade_prompt_result(spec, Some(conversation_id), parsed, total_ms);
                Ok(result)
            }
            Err(error) => Ok(PromptRunResult {
                prompt_id: spec.id.clone(),
                prompt: spec.prompt.clone(),
                capability_class: spec.capability_class,
                answerability_expected: spec.answerability_expected,
                conversation_id: Some(conversation_id),
                status: PromptRunStatus::HttpError,
                assistant_content: String::new(),
                source_count: 0,
                source_videos: Vec::new(),
                source_channels: Vec::new(),
                used_search_tool: false,
                used_db_tool: false,
                used_conversation_only: false,
                status_trace: Vec::new(),
                tool_calls: Vec::new(),
                latency_ms_total: total_ms,
                latency_ms_retrieval: None,
                latency_ms_generation: None,
                rubric_answerability_pass: false,
                rubric_grounding_pass: false,
                rubric_shape_pass: false,
                rubric_capability_score: 0,
                failure_class: Some(FAILURE_STREAM.to_string()),
                notes: vec!["failed to obtain a complete stream".to_string()],
                raw_error: Some(error.to_string()),
                raw_sse: None,
            }),
        }
    }
}

fn grade_prompt_result(
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

fn answerability_pass(spec: &PromptSpec, content: &str, notes: &mut Vec<String>) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        notes.push("assistant content was empty".to_string());
        return false;
    }

    let normalized = trimmed.to_ascii_lowercase();
    if unsupported_library_phrases()
        .iter()
        .any(|phrase| normalized.contains(phrase))
    {
        notes.push("assistant claimed missing library access".to_string());
        return false;
    }

    if generic_failure_phrases()
        .iter()
        .any(|phrase| normalized.contains(phrase))
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
            notes.push("timestamp-oriented answer did not surface timestamp information or a timing caveat".to_string());
            return false;
        }
    }

    true
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

fn classify_answerability_failure(content: &str) -> String {
    let normalized = content.to_ascii_lowercase();
    if unsupported_library_phrases()
        .iter()
        .any(|phrase| normalized.contains(phrase))
    {
        FAILURE_UNSUPPORTED.to_string()
    } else {
        FAILURE_GENERIC.to_string()
    }
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

fn unique_video_ids(sources: &[ChatSource]) -> HashSet<String> {
    sources
        .iter()
        .map(|source| source.video_id.clone())
        .collect()
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

fn contains_caveat_marker(normalized: &str) -> bool {
    caveat_markers()
        .iter()
        .any(|marker| normalized.contains(marker))
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

fn build_summary(results: &[PromptRunResult]) -> SweepSummary {
    let total_prompts = results.len();
    let passed_prompts = results
        .iter()
        .filter(|result| prompt_passed(result))
        .count();
    let answerability_passed = results
        .iter()
        .filter(|result| result.rubric_answerability_pass)
        .count();
    let grounding_passed = results
        .iter()
        .filter(|result| result.rubric_grounding_pass)
        .count();
    let shape_passed = results
        .iter()
        .filter(|result| result.rubric_shape_pass)
        .count();
    let average_score = if total_prompts == 0 {
        0.0
    } else {
        results
            .iter()
            .map(|result| result.rubric_capability_score as f32)
            .sum::<f32>()
            / total_prompts as f32
    };

    let prompts_without_sources = results
        .iter()
        .filter(|result| result.source_count == 0)
        .map(|result| result.prompt_id.clone())
        .collect();

    let single_video_prompts = results
        .iter()
        .filter(|result| {
            result.failure_class.as_deref() == Some(FAILURE_SINGLE_VIDEO)
                || (result.capability_class != CapabilityClass::DirectLookup
                    && result.source_videos.len() == 1)
        })
        .map(|result| result.prompt_id.clone())
        .collect();

    let mut failure_counts = BTreeMap::<String, usize>::new();
    for result in results {
        if let Some(failure) = &result.failure_class {
            *failure_counts.entry(failure.clone()).or_insert(0) += 1;
        }
    }

    let mut grouped = HashMap::<CapabilityClass, Vec<&PromptRunResult>>::new();
    for result in results {
        grouped
            .entry(result.capability_class)
            .or_default()
            .push(result);
    }

    let mut by_capability_class = grouped
        .into_iter()
        .map(|(capability_class, grouped_results)| {
            let total = grouped_results.len();
            let passed = grouped_results
                .iter()
                .filter(|result| prompt_passed(result))
                .count();
            let answerability_passed = grouped_results
                .iter()
                .filter(|result| result.rubric_answerability_pass)
                .count();
            let grounding_passed = grouped_results
                .iter()
                .filter(|result| result.rubric_grounding_pass)
                .count();
            let shape_passed = grouped_results
                .iter()
                .filter(|result| result.rubric_shape_pass)
                .count();
            let average_score = grouped_results
                .iter()
                .map(|result| result.rubric_capability_score as f32)
                .sum::<f32>()
                / total as f32;
            let mut common_failure_counts = BTreeMap::<String, usize>::new();
            for result in grouped_results {
                if let Some(failure) = &result.failure_class {
                    *common_failure_counts.entry(failure.clone()).or_insert(0) += 1;
                }
            }
            CapabilitySummary {
                capability_class,
                total,
                passed,
                answerability_passed,
                grounding_passed,
                shape_passed,
                average_score,
                common_failure_classes: common_failure_counts.keys().cloned().collect(),
            }
        })
        .collect::<Vec<_>>();
    by_capability_class.sort_by_key(|summary| summary.capability_class);

    SweepSummary {
        total_prompts,
        passed_prompts,
        answerability_passed,
        grounding_passed,
        shape_passed,
        average_score,
        prompts_without_sources,
        single_video_prompts,
        failure_counts,
        by_capability_class,
    }
}

fn prompt_passed(result: &PromptRunResult) -> bool {
    result.rubric_answerability_pass
        && result.rubric_grounding_pass
        && result.rubric_shape_pass
        && result.rubric_capability_score >= 2
}

fn write_reports(output_dir: &Path, report: &SweepReport) -> Result<()> {
    let results_json = output_dir.join("results.json");
    let results_md = output_dir.join("results.md");
    let failures_json = output_dir.join("failures-by-class.json");

    fs::write(
        &results_json,
        serde_json::to_vec_pretty(report).context("failed to encode report JSON")?,
    )
    .with_context(|| format!("failed to write {}", results_json.display()))?;

    let failure_map = grouped_failures(&report.results);
    fs::write(
        &failures_json,
        serde_json::to_vec_pretty(&failure_map).context("failed to encode failure JSON")?,
    )
    .with_context(|| format!("failed to write {}", failures_json.display()))?;

    fs::write(&results_md, render_markdown_report(report))
        .with_context(|| format!("failed to write {}", results_md.display()))?;

    Ok(())
}

fn grouped_failures(results: &[PromptRunResult]) -> BTreeMap<String, Vec<String>> {
    let mut grouped = BTreeMap::<String, Vec<String>>::new();
    for result in results {
        if let Some(failure) = &result.failure_class {
            grouped
                .entry(failure.clone())
                .or_default()
                .push(result.prompt_id.clone());
        }
    }
    grouped
}

fn render_markdown_report(report: &SweepReport) -> String {
    let mut md = String::new();
    md.push_str("# Chat Capability Sweep Results\n\n");
    md.push_str(&format!(
        "- Generated: `{}`\n- Base URL: `{}`\n- Dataset: `{}`\n- Prompt count: `{}`\n\n",
        report.generated_at_utc, report.base_url, report.dataset_path, report.prompt_count
    ));

    md.push_str("## Summary\n\n");
    md.push_str(&format!(
        "- Passed prompts: `{}/{}`
- Answerability pass: `{}/{}`
- Grounding pass: `{}/{}`
- Shape pass: `{}/{}`
- Average score: `{:.2}`\n\n",
        report.summary.passed_prompts,
        report.summary.total_prompts,
        report.summary.answerability_passed,
        report.summary.total_prompts,
        report.summary.grounding_passed,
        report.summary.total_prompts,
        report.summary.shape_passed,
        report.summary.total_prompts,
        report.summary.average_score
    ));

    md.push_str("## Capability Classes\n\n");
    for summary in &report.summary.by_capability_class {
        md.push_str(&format!(
            "- `{}`: passed `{}/{}`, avg score `{:.2}`, failures `{}`\n",
            capability_class_name(summary.capability_class),
            summary.passed,
            summary.total,
            summary.average_score,
            if summary.common_failure_classes.is_empty() {
                "-".to_string()
            } else {
                summary.common_failure_classes.join(", ")
            }
        ));
    }
    md.push('\n');

    md.push_str("## Failures By Class\n\n");
    for (failure, ids) in grouped_failures(&report.results) {
        md.push_str(&format!("- `{failure}`: {}\n", ids.join(", ")));
    }
    md.push('\n');

    md.push_str("## Prompt Results\n\n");
    for result in &report.results {
        md.push_str(&format!(
            "### {} {}\n\n",
            result.prompt_id,
            if prompt_passed(result) {
                "PASS"
            } else {
                "FAIL"
            }
        ));
        md.push_str(&format!(
            "- Prompt: {}\n- Class: `{}`\n- Status: `{:?}`\n- Score: `{}`\n- Sources: `{}`\n- Failure: `{}`\n",
            result.prompt,
            capability_class_name(result.capability_class),
            result.status,
            result.rubric_capability_score,
            result.source_count,
            result.failure_class.as_deref().unwrap_or("-")
        ));
        if !result.source_videos.is_empty() {
            md.push_str(&format!(
                "- Source videos: {}\n",
                result.source_videos.join(" | ")
            ));
        }
        if !result.tool_calls.is_empty() {
            let tools = result
                .tool_calls
                .iter()
                .map(|tool| format!("{} ({})", tool.label, tool.name))
                .collect::<Vec<_>>()
                .join(", ");
            md.push_str(&format!("- Tools: {tools}\n"));
        }
        if !result.notes.is_empty() {
            md.push_str(&format!("- Notes: {}\n", result.notes.join(" | ")));
        }
        md.push_str("\n#### Answer\n\n");
        if result.assistant_content.trim().is_empty() {
            md.push_str("_No assistant content._\n\n");
        } else {
            md.push_str(&result.assistant_content);
            md.push_str("\n\n");
        }
    }

    md
}

fn capability_class_name(class_name: CapabilityClass) -> &'static str {
    match class_name {
        CapabilityClass::DirectLookup => "direct_lookup",
        CapabilityClass::TopicAggregation => "topic_aggregation",
        CapabilityClass::CrossVideoSynthesis => "cross_video_synthesis",
        CapabilityClass::Comparison => "comparison",
        CapabilityClass::Recommendation => "recommendation",
        CapabilityClass::CreatorStance => "creator_stance",
        CapabilityClass::HighlightLookup => "highlight_lookup",
        CapabilityClass::HighlightClustering => "highlight_clustering",
        CapabilityClass::TranscriptSummaryAlignment => "transcript_summary_alignment",
        CapabilityClass::TimestampNavigation => "timestamp_navigation",
        CapabilityClass::ToneOrStyleInference => "tone_or_style_inference",
        CapabilityClass::MetaLearningOrNextStep => "meta_learning_or_next_step",
    }
}

fn filter_prompts(
    prompts: Vec<PromptSpec>,
    class_filters: &HashSet<CapabilityClass>,
    prompt_id_filters: &HashSet<String>,
) -> Vec<PromptSpec> {
    prompts
        .into_iter()
        .filter(|prompt| {
            (class_filters.is_empty() || class_filters.contains(&prompt.capability_class))
                && (prompt_id_filters.is_empty() || prompt_id_filters.contains(&prompt.id))
        })
        .collect()
}

fn load_prompt_specs(path: &Path) -> Result<Vec<PromptSpec>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str::<Vec<PromptSpec>>(&content)
        .with_context(|| format!("failed to decode {}", path.display()))
}

async fn ensure_backend_ready(base_url: &str) -> Result<()> {
    let url = format!("{}/api/health", base_url.trim_end_matches('/'));
    let status = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .context("failed to build health-check client")?
        .get(url)
        .send()
        .await
        .context("failed to reach backend health endpoint")?;
    if !status.status().is_success() {
        bail!("backend health check failed with {}", status.status());
    }
    Ok(())
}

fn default_dataset_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("chat_capability_prompts.json")
}

fn default_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(".artifacts")
        .join("chat-capability")
}

fn parse_args() -> Result<CliConfig> {
    let mut config = CliConfig {
        base_url: DEFAULT_BASE_URL.to_string(),
        dataset_path: default_dataset_path(),
        output_dir: default_output_dir(),
        timeout: Duration::from_secs(240),
        deep_research: false,
        model: None,
        class_filters: HashSet::new(),
        prompt_id_filters: HashSet::new(),
    };

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--base-url" => {
                config.base_url = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --base-url"))?;
            }
            "--dataset" => {
                config.dataset_path = PathBuf::from(
                    args.next()
                        .ok_or_else(|| anyhow!("missing value for --dataset"))?,
                );
            }
            "--output-dir" => {
                config.output_dir = PathBuf::from(
                    args.next()
                        .ok_or_else(|| anyhow!("missing value for --output-dir"))?,
                );
            }
            "--timeout-seconds" => {
                let seconds = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --timeout-seconds"))?
                    .parse::<u64>()
                    .context("invalid timeout seconds")?;
                config.timeout = Duration::from_secs(seconds.max(1));
            }
            "--class" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --class"))?;
                config.class_filters.insert(parse_capability_class(&value)?);
            }
            "--prompt-id" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --prompt-id"))?;
                config.prompt_id_filters.insert(value);
            }
            "--model" => {
                config.model = Some(
                    args.next()
                        .ok_or_else(|| anyhow!("missing value for --model"))?,
                );
            }
            "--deep-research" => {
                config.deep_research = true;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => bail!("unknown argument `{other}`"),
        }
    }

    Ok(config)
}

fn print_help() {
    println!(
        "chat_capability_eval

Options:
  --base-url <url>          Backend base URL. Default: {DEFAULT_BASE_URL}
  --dataset <path>          Prompt dataset path.
  --output-dir <path>       Output directory for reports.
  --timeout-seconds <n>     Per-request timeout. Default: 240
  --class <name>            Filter by capability class. Repeatable.
  --prompt-id <id>          Filter by prompt id. Repeatable.
  --model <id>              Optional chat model id to send with the prompt.
  --deep-research           Set deep_research=true for every prompt.
  --help                    Show this help.
"
    );
}

fn parse_capability_class(value: &str) -> Result<CapabilityClass> {
    match value.trim() {
        "direct_lookup" => Ok(CapabilityClass::DirectLookup),
        "topic_aggregation" => Ok(CapabilityClass::TopicAggregation),
        "cross_video_synthesis" => Ok(CapabilityClass::CrossVideoSynthesis),
        "comparison" => Ok(CapabilityClass::Comparison),
        "recommendation" => Ok(CapabilityClass::Recommendation),
        "creator_stance" => Ok(CapabilityClass::CreatorStance),
        "highlight_lookup" => Ok(CapabilityClass::HighlightLookup),
        "highlight_clustering" => Ok(CapabilityClass::HighlightClustering),
        "transcript_summary_alignment" => Ok(CapabilityClass::TranscriptSummaryAlignment),
        "timestamp_navigation" => Ok(CapabilityClass::TimestampNavigation),
        "tone_or_style_inference" => Ok(CapabilityClass::ToneOrStyleInference),
        "meta_learning_or_next_step" => Ok(CapabilityClass::MetaLearningOrNextStep),
        other => bail!("unknown capability class `{other}`"),
    }
}

impl SseAccumulator {
    fn push(&mut self, chunk: &str) -> Vec<SseEvent> {
        self.buffer.push_str(&chunk.replace('\r', ""));
        let mut events = Vec::new();
        while let Some(index) = self.buffer.find("\n\n") {
            let block = self.buffer[..index].to_string();
            self.buffer.drain(..index + 2);
            if let Some(event) = parse_sse_block(&block) {
                events.push(event);
            }
        }
        events
    }

    fn finish(&mut self) -> Vec<SseEvent> {
        if self.buffer.trim().is_empty() {
            return Vec::new();
        }
        let block = std::mem::take(&mut self.buffer);
        parse_sse_block(&block).into_iter().collect()
    }
}

fn parse_sse_block(block: &str) -> Option<SseEvent> {
    let mut event_name = None::<String>;
    let mut data_lines = Vec::new();
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event_name = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start().to_string());
        }
    }
    let name = event_name?;
    let data = data_lines.join("\n");
    Some(SseEvent { name, data })
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

fn generic_failure_phrases() -> &'static [&'static str] {
    &[
        "i can't answer that",
        "i cannot answer that",
        "not enough information",
        "i don't know",
        "i do not know",
    ]
}

fn caveat_markers() -> &'static [&'static str] {
    &[
        "it seems",
        "appears",
        "likely",
        "probably",
        "inference",
        "based on the excerpts",
        "from the available evidence",
    ]
}

fn contrast_markers() -> &'static [&'static str] {
    &[
        "however",
        "in contrast",
        "while",
        "whereas",
        "on the other hand",
        "compared with",
        "difference",
        "similarity",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
