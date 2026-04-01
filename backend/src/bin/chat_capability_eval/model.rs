use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

use dastill::models::{ChatMessage, ChatSource};
use reqwest::Client;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

pub(crate) const DEFAULT_BASE_URL: &str = "http://localhost:3544";
pub(crate) const DEFAULT_PROXY_TOKEN: &str = "local-dev-backend-proxy-token";
pub(crate) const FAILURE_NO_SOURCES: &str = "no_sources";
pub(crate) const FAILURE_SINGLE_VIDEO: &str = "single_video_overfit";
pub(crate) const FAILURE_GENERIC: &str = "generic_answer";
pub(crate) const FAILURE_SHAPE: &str = "shape_mismatch";
pub(crate) const FAILURE_STREAM: &str = "stream_error";
pub(crate) const FAILURE_UNSUPPORTED: &str = "unsupported_capability";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct PromptSpec {
    pub(crate) id: String,
    pub(crate) prompt: String,
    pub(crate) search_strategy_expected: String,
    pub(crate) answerability_expected: ExpectedAnswerability,
    pub(crate) good_answer_shape: String,
    pub(crate) capability_class: CapabilityClass,
    pub(crate) requires_timestamp: bool,
    pub(crate) requires_highlights: bool,
    pub(crate) requires_quality_score: bool,
    pub(crate) requires_cross_video_synthesis: bool,
    pub(crate) requires_opinion_inference: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExpectedAnswerability {
    Yes,
    Partial,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CapabilityClass {
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
pub(crate) struct StreamStatusPayload {
    pub(crate) stage: String,
    pub(crate) label: Option<String>,
    pub(crate) detail: Option<String>,
    pub(crate) decision: Option<String>,
    pub(crate) plan: Option<StreamPlanPayload>,
    pub(crate) tool: Option<StreamToolPayload>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct StreamPlanPayload {
    pub(crate) intent: Option<String>,
    pub(crate) label: String,
    pub(crate) budget: usize,
    pub(crate) max_per_video: usize,
    pub(crate) queries: Vec<String>,
    pub(crate) expansion_queries: Vec<String>,
    pub(crate) rationale: Option<String>,
    pub(crate) skip_retrieval: Option<bool>,
    pub(crate) deep_research: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct StreamToolPayload {
    pub(crate) name: String,
    pub(crate) label: String,
    pub(crate) state: String,
    pub(crate) input: String,
    pub(crate) output: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SourcesEventPayload {
    pub(crate) sources: Vec<ChatSource>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TokenEventPayload {
    #[serde(rename = "token")]
    pub(crate) token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DoneEventPayload {
    pub(crate) message: ChatMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ErrorEventPayload {
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TimedStatus {
    pub(crate) received_at_ms: u64,
    pub(crate) payload: StreamStatusPayload,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolCallReport {
    pub(crate) name: String,
    pub(crate) label: String,
    pub(crate) state: String,
    pub(crate) input: String,
    pub(crate) output: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PromptRunStatus {
    Completed,
    StreamError,
    HttpError,
    ParseError,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PromptRunResult {
    pub(crate) prompt_id: String,
    pub(crate) prompt: String,
    pub(crate) capability_class: CapabilityClass,
    pub(crate) answerability_expected: ExpectedAnswerability,
    pub(crate) conversation_id: Option<String>,
    pub(crate) status: PromptRunStatus,
    pub(crate) assistant_content: String,
    pub(crate) source_count: usize,
    pub(crate) source_videos: Vec<String>,
    pub(crate) source_channels: Vec<String>,
    pub(crate) used_search_tool: bool,
    pub(crate) used_db_tool: bool,
    pub(crate) used_conversation_only: bool,
    pub(crate) status_trace: Vec<TimedStatus>,
    pub(crate) tool_calls: Vec<ToolCallReport>,
    pub(crate) latency_ms_total: u64,
    pub(crate) latency_ms_retrieval: Option<u64>,
    pub(crate) latency_ms_generation: Option<u64>,
    pub(crate) rubric_answerability_pass: bool,
    pub(crate) rubric_grounding_pass: bool,
    pub(crate) rubric_shape_pass: bool,
    pub(crate) rubric_capability_score: u8,
    pub(crate) failure_class: Option<String>,
    pub(crate) notes: Vec<String>,
    pub(crate) raw_error: Option<String>,
    pub(crate) raw_sse: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CapabilitySummary {
    pub(crate) capability_class: CapabilityClass,
    pub(crate) total: usize,
    pub(crate) passed: usize,
    pub(crate) answerability_passed: usize,
    pub(crate) grounding_passed: usize,
    pub(crate) shape_passed: usize,
    pub(crate) average_score: f32,
    pub(crate) common_failure_classes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SweepSummary {
    pub(crate) total_prompts: usize,
    pub(crate) passed_prompts: usize,
    pub(crate) answerability_passed: usize,
    pub(crate) grounding_passed: usize,
    pub(crate) shape_passed: usize,
    pub(crate) average_score: f32,
    pub(crate) prompts_without_sources: Vec<String>,
    pub(crate) single_video_prompts: Vec<String>,
    pub(crate) failure_counts: BTreeMap<String, usize>,
    pub(crate) by_capability_class: Vec<CapabilitySummary>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SweepReport {
    pub(crate) generated_at_utc: String,
    pub(crate) base_url: String,
    pub(crate) dataset_path: String,
    pub(crate) prompt_count: usize,
    pub(crate) summary: SweepSummary,
    pub(crate) results: Vec<PromptRunResult>,
}

#[derive(Debug, Default)]
pub(crate) struct CliConfig {
    pub(crate) base_url: String,
    pub(crate) dataset_path: PathBuf,
    pub(crate) output_dir: PathBuf,
    pub(crate) timeout: Duration,
    pub(crate) deep_research: bool,
    pub(crate) model: Option<String>,
    pub(crate) class_filters: HashSet<CapabilityClass>,
    pub(crate) prompt_id_filters: HashSet<String>,
}

#[derive(Debug)]
pub(crate) struct SweepRunner {
    pub(crate) client: Client,
    pub(crate) base_url: String,
    pub(crate) default_headers: HeaderMap,
}
