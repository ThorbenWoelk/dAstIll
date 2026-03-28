mod cloud_models;
mod constants;
mod intent;
mod recent;
mod tools;

pub use cloud_models::{default_chat_cloud_model_id, is_chat_cloud_model_choice};
pub(crate) use constants::*;
pub use intent::ChatQueryIntent;

use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use axum::response::sse::Event;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, broadcast, mpsc, watch};
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;
use tracing::Instrument;

use crate::db;
use crate::models::{
    ChatConversation, ChatMessage, ChatMessageStatus, ChatRole, ChatSource, ChatTitleStatus,
};
use crate::services::ollama::OllamaCore;
use crate::services::search::SearchCandidate;
use crate::services::text::limit_text;
use crate::state::AppState;

use super::chat::recent::{
    execute_recent_library_activity_query, is_explicit_realtime_status_query,
    is_recent_activity_query,
};
use super::chat_heuristics::{
    build_plan_label, collect_focus_terms, heuristic_expansion_queries, heuristic_query_variants,
    is_attributed_preference_query, push_unique_query, recommendation_query_variants,
    sanitize_queries,
};
use super::chat_prompt::{
    build_conversation_only_grounding, build_grounding_context, build_ollama_messages,
    build_synthesis_grounding_context, build_tool_grounding_context, synthesis_raw_limit_for_plan,
};
use super::chat_ranking::{
    accumulate_ranked_candidates, assess_coverage, build_video_observation_inputs,
    count_unique_videos, rank_chat_sources, retrieval_candidate_limit,
};

#[derive(Debug, Clone, Serialize)]
struct ChatRetrievalPlanVisibility {
    intent: ChatQueryIntent,
    label: String,
    budget: usize,
    max_per_video: usize,
    queries: Vec<String>,
    expansion_queries: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rationale: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    skip_retrieval: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    deep_research: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatToolStatusPayload {
    name: String,
    label: String,
    state: String,
    input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
}

impl ChatToolStatusPayload {
    fn new(
        name: impl Into<String>,
        label: impl Into<String>,
        state: impl Into<String>,
        input: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            state: state.into(),
            input: input.into(),
            output: None,
        }
    }

    fn with_output(mut self, output: impl Into<String>) -> Self {
        self.output = Some(output.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatStatusPayload {
    stage: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plan: Option<ChatRetrievalPlanVisibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool: Option<ChatToolStatusPayload>,
}

impl ChatStatusPayload {
    fn new(stage: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            stage: stage.into(),
            label: Some(label.into()),
            detail: None,
            decision: None,
            plan: None,
            tool: None,
        }
    }

    fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    fn with_decision(mut self, decision: impl Into<String>) -> Self {
        self.decision = Some(decision.into());
        self
    }

    fn with_plan(mut self, plan: ChatRetrievalPlanVisibility) -> Self {
        self.plan = Some(plan);
        self
    }

    fn with_tool(mut self, tool: ChatToolStatusPayload) -> Self {
        self.tool = Some(tool);
        self
    }
}

#[derive(Debug, Clone)]
pub(super) struct ChatRetrievalPlan {
    pub(super) intent: ChatQueryIntent,
    pub(super) label: String,
    pub(super) budget: usize,
    pub(super) max_per_video: usize,
    pub(super) queries: Vec<String>,
    pub(super) expansion_queries: Vec<String>,
    pub(super) focus_terms: Vec<String>,
    pub(super) channel_focus_ids: Vec<String>,
    pub(super) video_focus_ids: Vec<String>,
    pub(super) attributed_preference: bool,
    pub(super) rationale: Option<String>,
    /// When true, retrieval is skipped and the model answers from conversation history only.
    pub(super) skip_retrieval: bool,
    /// User requested maximum library coverage for this turn.
    pub(super) deep_research: bool,
}

impl ChatRetrievalPlan {
    fn fallback(prompt: &str, rationale: Option<String>) -> Self {
        let attributed_preference = is_attributed_preference_query(prompt);
        let recent_activity =
            is_recent_activity_query(prompt) && !is_explicit_realtime_status_query(prompt);
        let intent = if recent_activity {
            ChatQueryIntent::RecentActivity
        } else {
            ChatQueryIntent::Synthesis
        };
        let (budget, max_per_video) = if attributed_preference {
            (CHAT_RECOMMENDATION_SOURCE_LIMIT, 3)
        } else {
            match intent {
                ChatQueryIntent::RecentActivity => (CHAT_RECENT_ACTIVITY_SOURCE_LIMIT, 2),
                _ => (CHAT_SYNTHESIS_SOURCE_LIMIT, 4),
            }
        };
        Self {
            intent,
            label: build_plan_label(intent, attributed_preference).to_string(),
            budget,
            max_per_video,
            queries: vec![prompt.trim().to_string()],
            expansion_queries: Vec::new(),
            focus_terms: collect_focus_terms(prompt),
            channel_focus_ids: Vec::new(),
            video_focus_ids: Vec::new(),
            attributed_preference,
            rationale,
            skip_retrieval: false,
            deep_research: false,
        }
    }

    fn from_response(prompt: &str, response: ChatQueryPlanResponse) -> Self {
        let planner_intent = response
            .intent
            .as_deref()
            .and_then(ChatQueryIntent::from_str)
            .unwrap_or(ChatQueryIntent::Synthesis);
        let mut intent = planner_intent;
        let attributed_preference = is_attributed_preference_query(prompt);
        let recent_activity =
            is_recent_activity_query(prompt) && !is_explicit_realtime_status_query(prompt);
        if attributed_preference && matches!(intent, ChatQueryIntent::Fact) {
            intent = ChatQueryIntent::Synthesis;
        }
        if recent_activity && !matches!(intent, ChatQueryIntent::Comparison) {
            intent = ChatQueryIntent::RecentActivity;
        }

        let skip_retrieval = response.needs_retrieval == Some(false);

        let (mut budget, mut max_per_video) = match intent {
            ChatQueryIntent::Fact => (CHAT_SOURCE_LIMIT, 3),
            ChatQueryIntent::Synthesis => (CHAT_SYNTHESIS_SOURCE_LIMIT, 4),
            ChatQueryIntent::Pattern => (CHAT_PATTERN_SOURCE_LIMIT, 3),
            ChatQueryIntent::Comparison => (CHAT_COMPARISON_SOURCE_LIMIT, 5),
            ChatQueryIntent::RecentActivity => (CHAT_RECENT_ACTIVITY_SOURCE_LIMIT, 2),
        };
        if attributed_preference && !matches!(intent, ChatQueryIntent::Comparison) {
            budget = budget.max(CHAT_RECOMMENDATION_SOURCE_LIMIT);
            max_per_video = max_per_video.min(3);
        }

        let mut queries = sanitize_queries(response.sub_queries.unwrap_or_default());
        let mut expansion_queries =
            sanitize_queries(response.expansion_queries.unwrap_or_default());

        if queries.is_empty() {
            queries.push(prompt.trim().to_string());
        }

        let heuristic_queries = if attributed_preference {
            recommendation_query_variants(prompt)
        } else {
            heuristic_query_variants(prompt, intent)
        };
        for query in heuristic_queries {
            if queries.len() < CHAT_QUERY_LIMIT_PER_PASS {
                push_unique_query(&mut queries, query);
            } else if expansion_queries.len() < CHAT_QUERY_LIMIT_TOTAL - queries.len() {
                push_unique_query(&mut expansion_queries, query);
            }
        }

        if matches!(intent, ChatQueryIntent::Fact) && !attributed_preference {
            expansion_queries.clear();
            queries.truncate(1);
        } else {
            queries.truncate(CHAT_QUERY_LIMIT_PER_PASS);
            expansion_queries.retain(|query| !queries.contains(query));
            expansion_queries.truncate(CHAT_QUERY_LIMIT_TOTAL.saturating_sub(queries.len()));
        }

        Self {
            intent,
            label: build_plan_label(intent, attributed_preference).to_string(),
            budget,
            max_per_video,
            queries,
            expansion_queries,
            focus_terms: collect_focus_terms(prompt),
            channel_focus_ids: Vec::new(),
            video_focus_ids: Vec::new(),
            attributed_preference,
            rationale: if attributed_preference && matches!(planner_intent, ChatQueryIntent::Fact) {
                Some(
                    "The wording asks for someone’s recommendation/opinion, so the search was widened beyond a narrow fact lookup."
                        .to_string(),
                )
            } else {
                response.rationale.and_then(|value| trim_to_option(&value))
            },
            skip_retrieval,
            deep_research: false,
        }
    }

    fn visibility(&self) -> ChatRetrievalPlanVisibility {
        ChatRetrievalPlanVisibility {
            intent: self.intent,
            label: self.label.clone(),
            budget: self.budget,
            max_per_video: self.max_per_video,
            queries: self.queries.clone(),
            expansion_queries: self.expansion_queries.clone(),
            rationale: self.rationale.clone(),
            skip_retrieval: self.skip_retrieval,
            deep_research: self.deep_research,
        }
    }

    fn apply_scope(&mut self, scope: &tools::MentionScope) {
        if !scope.has_scope() {
            return;
        }

        self.channel_focus_ids = scope.channel_focus_ids.clone();
        self.video_focus_ids = scope.video_focus_ids.clone();

        self.queries = self
            .queries
            .iter()
            .map(|query| scope.scoped_query(query))
            .filter(|query| !query.trim().is_empty())
            .collect();
        self.expansion_queries = self
            .expansion_queries
            .iter()
            .map(|query| scope.scoped_query(query))
            .filter(|query| !query.trim().is_empty())
            .collect();

        if self.queries.is_empty() {
            let scoped = scope.prompt_for_retrieval(&scope.cleaned_prompt);
            if !scoped.trim().is_empty() {
                self.queries.push(scoped);
            }
        }

        if let Some(scope_detail) = scope.scope_detail() {
            let scope_note = format!("Scoped to {scope_detail}.");
            self.rationale = Some(match self.rationale.take() {
                Some(existing) if !existing.is_empty() => format!("{scope_note} {existing}"),
                _ => scope_note,
            });
        }
    }

    fn queries_per_pass_cap(&self) -> usize {
        if self.deep_research {
            CHAT_DEEP_RESEARCH_QUERIES_PER_PASS
        } else {
            CHAT_QUERY_LIMIT_PER_PASS
        }
    }

    /// Widens retrieval after planner output so this turn uses the app’s maximum excerpt budget and richer query fan-out.
    pub(super) fn apply_deep_research(&mut self, prompt: &str) {
        let prompt = prompt.trim();
        self.deep_research = true;
        self.skip_retrieval = false;
        self.intent = ChatQueryIntent::Pattern;
        self.budget = CHAT_DEEP_RESEARCH_SOURCE_LIMIT;
        self.max_per_video = self.max_per_video.max(4);
        self.label = "Deep research".to_string();

        let mut combined: Vec<String> = Vec::new();
        push_unique_query(&mut combined, prompt.to_string());
        for query in self.queries.iter().chain(self.expansion_queries.iter()) {
            push_unique_query(&mut combined, query.clone());
        }
        let heur = if self.attributed_preference {
            recommendation_query_variants(prompt)
        } else {
            heuristic_query_variants(prompt, ChatQueryIntent::Pattern)
        };
        for query in heur {
            push_unique_query(&mut combined, query);
        }
        push_unique_query(&mut combined, format!("{prompt} themes"));
        push_unique_query(&mut combined, format!("{prompt} overview"));

        self.queries = combined
            .iter()
            .take(CHAT_DEEP_RESEARCH_PRIMARY_QUERIES)
            .cloned()
            .collect();
        self.expansion_queries = combined
            .iter()
            .skip(CHAT_DEEP_RESEARCH_PRIMARY_QUERIES)
            .take(CHAT_DEEP_RESEARCH_EXPANSION_QUERIES)
            .cloned()
            .collect();

        let note = "Deep research: using the maximum excerpt budget for this app and broader search passes.";
        self.rationale = Some(match self.rationale.take() {
            Some(previous) if !previous.is_empty() => format!("{note} {previous}"),
            _ => note.to_string(),
        });
    }

    fn queries_for_pass(&self, pass: usize) -> Vec<String> {
        let cap = self.queries_per_pass_cap();
        match pass {
            1 => self.queries.clone(),
            2 => {
                let mut queries = self.expansion_queries.clone();
                if queries.is_empty() {
                    queries = heuristic_expansion_queries(self);
                }
                queries.truncate(cap);
                queries
            }
            3 => {
                let mut extra = heuristic_expansion_queries(self);
                let mut seen: HashSet<String> = self
                    .queries
                    .iter()
                    .chain(self.expansion_queries.iter())
                    .map(|q| q.trim().to_ascii_lowercase())
                    .collect();
                extra.retain(|q| {
                    let key = q.trim().to_ascii_lowercase();
                    if key.is_empty() || seen.contains(&key) {
                        return false;
                    }
                    seen.insert(key);
                    true
                });
                extra.truncate(cap);
                extra
            }
            _ => Vec::new(),
        }
    }

    pub(super) fn supports_second_pass(&self) -> bool {
        !self.queries_for_pass(2).is_empty()
    }

    pub(super) fn supports_third_pass(&self) -> bool {
        self.budget >= 16
            && matches!(
                self.intent,
                ChatQueryIntent::Pattern | ChatQueryIntent::Comparison
            )
            && !self.queries_for_pass(3).is_empty()
    }

    pub(super) fn synthesis_video_cap(&self) -> usize {
        let scaled = (self.budget / 3).max(CHAT_SYNTHESIS_VIDEO_LIMIT);
        scaled.min(24)
    }
}

#[derive(Debug, Deserialize)]
struct ChatQueryPlanResponse {
    needs_retrieval: Option<bool>,
    intent: Option<String>,
    rationale: Option<String>,
    sub_queries: Option<Vec<String>>,
    expansion_queries: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct ChatToolLoopResponse {
    action: Option<String>,
    rationale: Option<String>,
    tool_name: Option<String>,
    search_library_input: Option<tools::SearchLibraryToolInput>,
    db_inspect_input: Option<tools::DbInspectToolInput>,
    highlight_lookup_input: Option<tools::HighlightLookupToolInput>,
    recent_library_activity_input: Option<tools::RecentLibraryActivityToolInput>,
}

#[derive(Debug)]
struct ToolLoopStepOutcome {
    action: ToolLoopAction,
    rationale: Option<String>,
}

#[derive(Debug)]
enum ToolLoopAction {
    Respond,
    ToolCall(PlannedChatToolCall),
}

#[derive(Debug, Clone)]
enum PlannedChatToolCall {
    SearchLibrary(tools::SearchLibraryQuery),
    DbInspect(tools::DbInspectQuery),
    HighlightLookup(tools::HighlightLookupQuery),
    RecentLibraryActivity(tools::RecentLibraryActivityQuery),
}

impl PlannedChatToolCall {
    fn tool_name(&self) -> &'static str {
        match self {
            Self::SearchLibrary(_) => "search_library",
            Self::DbInspect(_) => "db_inspect",
            Self::HighlightLookup(_) => "highlight_lookup",
            Self::RecentLibraryActivity(_) => "recent_library_activity",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::SearchLibrary(_) => "Library search",
            Self::DbInspect(_) => "Database lookup",
            Self::HighlightLookup(_) => "Saved highlights lookup",
            Self::RecentLibraryActivity(_) => "Recent library activity",
        }
    }

    fn input_summary(&self) -> String {
        match self {
            Self::SearchLibrary(query) => describe_search_library_query(query.clone()),
            Self::DbInspect(query) => tools::describe_db_inspect_query(*query),
            Self::HighlightLookup(query) => describe_highlight_lookup_query(query.clone()),
            Self::RecentLibraryActivity(query) => {
                describe_recent_library_activity_query(query.clone())
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ToolEvidenceRecord {
    summary: String,
    output: String,
}

#[derive(Debug, Clone)]
struct ToolLoopOutcome {
    conversation_only: bool,
    rationale: Option<String>,
    tool_outputs: Vec<ToolEvidenceRecord>,
    sources: Vec<RetrievedChatSource>,
}

#[derive(Debug, Clone)]
struct SearchLibraryExecutionResult {
    summary: String,
    output: String,
    sources: Vec<RetrievedChatSource>,
}

impl ChatToolLoopResponse {
    fn into_step_outcome(self) -> Result<ToolLoopStepOutcome, String> {
        let rationale = self.rationale.and_then(|value| trim_to_option(&value));
        let action = self
            .action
            .as_deref()
            .ok_or_else(|| "missing tool loop action".to_string())?;

        match action.trim() {
            "respond" => Ok(ToolLoopStepOutcome {
                action: ToolLoopAction::Respond,
                rationale,
            }),
            "search_library" => {
                let query = tools::build_search_library_query(
                    Some("search_library"),
                    self.search_library_input,
                )?
                .ok_or_else(|| {
                    "search_library action did not include a valid search request".to_string()
                })?;
                Ok(ToolLoopStepOutcome {
                    action: ToolLoopAction::ToolCall(PlannedChatToolCall::SearchLibrary(query)),
                    rationale,
                })
            }
            "db_inspect" => {
                let query =
                    tools::build_db_inspect_query(Some("db_inspect"), self.db_inspect_input)?
                        .ok_or_else(|| {
                            "db_inspect action did not include a valid database request".to_string()
                        })?;
                Ok(ToolLoopStepOutcome {
                    action: ToolLoopAction::ToolCall(PlannedChatToolCall::DbInspect(query)),
                    rationale,
                })
            }
            "highlight_lookup" => {
                let query = tools::build_highlight_lookup_query(
                    Some("highlight_lookup"),
                    self.highlight_lookup_input,
                )?
                .ok_or_else(|| {
                    "highlight_lookup action did not include a valid highlights request".to_string()
                })?;
                Ok(ToolLoopStepOutcome {
                    action: ToolLoopAction::ToolCall(PlannedChatToolCall::HighlightLookup(query)),
                    rationale,
                })
            }
            "recent_library_activity" => {
                let query = tools::build_recent_library_activity_query(
                    Some("recent_library_activity"),
                    self.recent_library_activity_input,
                )?
                .ok_or_else(|| {
                    "recent_library_activity action did not include a valid request".to_string()
                })?;
                Ok(ToolLoopStepOutcome {
                    action: ToolLoopAction::ToolCall(PlannedChatToolCall::RecentLibraryActivity(
                        query,
                    )),
                    rationale,
                })
            }
            "tool_call" => {
                let tool_name = self
                    .tool_name
                    .as_deref()
                    .ok_or_else(|| "missing tool name for tool_call action".to_string())?;
                match tool_name {
                    "search_library" => {
                        let query = tools::build_search_library_query(
                            Some(tool_name),
                            self.search_library_input,
                        )?
                        .ok_or_else(|| {
                            "tool_call action did not include a valid search_library request"
                                .to_string()
                        })?;
                        Ok(ToolLoopStepOutcome {
                            action: ToolLoopAction::ToolCall(PlannedChatToolCall::SearchLibrary(
                                query,
                            )),
                            rationale,
                        })
                    }
                    "db_inspect" => {
                        let query =
                            tools::build_db_inspect_query(Some(tool_name), self.db_inspect_input)?
                                .ok_or_else(|| {
                                    "tool_call action did not include a valid db_inspect request"
                                        .to_string()
                                })?;
                        Ok(ToolLoopStepOutcome {
                            action: ToolLoopAction::ToolCall(PlannedChatToolCall::DbInspect(query)),
                            rationale,
                        })
                    }
                    "highlight_lookup" => {
                        let query = tools::build_highlight_lookup_query(
                            Some(tool_name),
                            self.highlight_lookup_input,
                        )?
                        .ok_or_else(|| {
                            "tool_call action did not include a valid highlight_lookup request"
                                .to_string()
                        })?;
                        Ok(ToolLoopStepOutcome {
                            action: ToolLoopAction::ToolCall(PlannedChatToolCall::HighlightLookup(
                                query,
                            )),
                            rationale,
                        })
                    }
                    "recent_library_activity" => {
                        let query = tools::build_recent_library_activity_query(
                            Some(tool_name),
                            self.recent_library_activity_input,
                        )?
                        .ok_or_else(|| {
                            "tool_call action did not include a valid recent_library_activity request"
                                .to_string()
                        })?;
                        Ok(ToolLoopStepOutcome {
                            action: ToolLoopAction::ToolCall(
                                PlannedChatToolCall::RecentLibraryActivity(query),
                            ),
                            rationale,
                        })
                    }
                    other => Err(format!("unsupported tool `{other}`")),
                }
            }
            other => Err(format!("unsupported tool loop action `{other}`")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChatStreamEvent {
    Status { status: ChatStatusPayload },
    Sources { sources: Vec<ChatSource> },
    Token { token: String },
    Done { message: ChatMessage },
    Error { message: String },
}

impl ChatStreamEvent {
    pub fn event_name(&self) -> &'static str {
        match self {
            Self::Status { .. } => "status",
            Self::Sources { .. } => "sources",
            Self::Token { .. } => "token",
            Self::Done { .. } => "done",
            Self::Error { .. } => "error",
        }
    }

    pub fn to_sse_event(&self) -> Event {
        let data = match self {
            Self::Status { status } => {
                serde_json::to_value(status).expect("chat status payload should serialize")
            }
            Self::Sources { sources } => serde_json::json!({ "sources": sources }),
            Self::Token { token } => serde_json::json!({ "token": token }),
            Self::Done { message } => serde_json::json!({ "message": message }),
            Self::Error { message } => serde_json::json!({ "message": message }),
        };

        Event::default()
            .event(self.event_name())
            .data(serde_json::to_string(&data).expect("chat SSE payload should serialize"))
    }
}

#[derive(Debug, Clone)]
struct SequencedChatEvent {
    sequence: u64,
    event: ChatStreamEvent,
}

#[derive(Debug)]
struct ActiveChatState {
    next_sequence: AtomicU64,
    cancel_tx: watch::Sender<bool>,
    events_tx: broadcast::Sender<SequencedChatEvent>,
    buffered_events: Mutex<Vec<SequencedChatEvent>>,
}

#[derive(Debug, Clone)]
pub struct ActiveChatHandle {
    inner: Arc<ActiveChatState>,
}

impl ActiveChatHandle {
    pub fn new() -> Self {
        let (cancel_tx, _) = watch::channel(false);
        let (events_tx, _) = broadcast::channel(256);
        Self {
            inner: Arc::new(ActiveChatState {
                next_sequence: AtomicU64::new(1),
                cancel_tx,
                events_tx,
                buffered_events: Mutex::new(Vec::new()),
            }),
        }
    }

    pub async fn emit(&self, event: ChatStreamEvent) {
        let sequence = self.inner.next_sequence.fetch_add(1, Ordering::Relaxed);
        let envelope = SequencedChatEvent { sequence, event };
        self.inner
            .buffered_events
            .lock()
            .await
            .push(envelope.clone());
        let _ = self.inner.events_tx.send(envelope);
    }

    pub fn cancel(&self) {
        self.inner.cancel_tx.send_replace(true);
    }

    fn is_cancelled(&self) -> bool {
        *self.inner.cancel_tx.borrow()
    }

    fn ensure_not_cancelled(&self) -> Result<(), String> {
        if self.is_cancelled() {
            Err(cancelled_error())
        } else {
            Ok(())
        }
    }

    fn subscribe_cancel(&self) -> watch::Receiver<bool> {
        self.inner.cancel_tx.subscribe()
    }

    fn subscribe_events(&self) -> broadcast::Receiver<SequencedChatEvent> {
        self.inner.events_tx.subscribe()
    }

    async fn buffered_events(&self) -> Vec<SequencedChatEvent> {
        self.inner.buffered_events.lock().await.clone()
    }

    pub async fn into_sse_stream(&self) -> ReceiverStream<Result<Event, Infallible>> {
        let buffered_events = self.buffered_events().await;
        let mut receiver = self.subscribe_events();
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut last_sequence = 0;

            for event in buffered_events {
                last_sequence = event.sequence;
                if tx.send(Ok(event.event.to_sse_event())).await.is_err() {
                    return;
                }
            }

            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        if event.sequence <= last_sequence {
                            continue;
                        }
                        last_sequence = event.sequence;
                        let is_terminal = matches!(
                            event.event,
                            ChatStreamEvent::Done { .. } | ChatStreamEvent::Error { .. }
                        );
                        if tx.send(Ok(event.event.to_sse_event())).await.is_err() {
                            return;
                        }
                        if is_terminal {
                            return;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => return,
                }
            }
        });

        ReceiverStream::new(rx)
    }
}

impl Default for ActiveChatHandle {
    fn default() -> Self {
        Self::new()
    }
}

fn cancelled_error() -> String {
    "cancelled".to_string()
}

async fn await_or_cancel<T, F>(active_chat: &ActiveChatHandle, future: F) -> Result<T, String>
where
    F: Future<Output = T>,
{
    active_chat.ensure_not_cancelled()?;
    let mut cancel_rx = active_chat.subscribe_cancel();
    tokio::pin!(future);

    tokio::select! {
        changed = cancel_rx.changed() => {
            if changed.is_ok() && *cancel_rx.borrow() {
                Err(cancelled_error())
            } else {
                Ok(future.await)
            }
        }
        result = &mut future => Ok(result),
    }
}

#[derive(Debug, Clone)]
pub(super) struct RetrievedChatSource {
    pub(super) source: ChatSource,
    pub(super) context_text: String,
}

#[derive(Debug, Clone)]
pub(super) struct AccumulatedSearchCandidate {
    pub(super) candidate: SearchCandidate,
    pub(super) keyword_score: f32,
    pub(super) semantic_score: f32,
    pub(super) retrieval_pass: usize,
}

impl AccumulatedSearchCandidate {
    pub(super) fn combined_score(&self) -> f32 {
        match (self.keyword_score > 0.0, self.semantic_score > 0.0) {
            (true, true) => self.keyword_score + self.semantic_score,
            (true, false) => self.keyword_score,
            (false, true) => self.semantic_score,
            (false, false) => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
struct RetrievalPassOutcome {
    sources: Vec<RetrievedChatSource>,
    assessment: CoverageAssessment,
}

#[derive(Clone, Copy)]
struct RetrievalPassRequest<'a> {
    conversation_id: &'a str,
    plan: &'a ChatRetrievalPlan,
    pass: usize,
    queries: &'a [String],
    channel_focus_ids: &'a [String],
    video_focus_ids: &'a [String],
    active_chat: &'a ActiveChatHandle,
}

struct ToolCallExecutionRequest<'a> {
    state: &'a AppState,
    call: PlannedChatToolCall,
    prompt_scope: &'a tools::MentionScope,
    rationale: Option<&'a str>,
    tool_outputs: &'a mut Vec<ToolEvidenceRecord>,
    gathered_sources: &'a mut Vec<RetrievedChatSource>,
    active_chat: &'a ActiveChatHandle,
}

struct RetrievalCandidateRequest<'a> {
    state: &'a AppState,
    queries: &'a [String],
    candidate_limit: usize,
    channel_focus_ids: &'a [String],
    video_focus_ids: &'a [String],
    source_kind: Option<crate::services::search::SearchSourceKind>,
    active_chat: &'a ActiveChatHandle,
}

#[derive(Debug, Clone)]
pub(super) struct CoverageAssessment {
    pub(super) needs_more: bool,
    pub(super) reason: Option<String>,
    pub(super) channel_focus_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct ChatRetrievalOutcome {
    plan: ChatRetrievalPlan,
    sources: Vec<RetrievedChatSource>,
}

#[derive(Debug, Clone)]
pub(super) struct VideoObservation {
    pub(super) video_title: String,
    pub(super) channel_name: String,
    pub(super) summary: String,
}

#[derive(Debug, Clone)]
pub(super) struct VideoObservationInput {
    pub(super) video_id: String,
    pub(super) video_title: String,
    pub(super) channel_name: String,
    pub(super) excerpts: Vec<RetrievedChatSource>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: Option<OllamaChatMessage>,
    done: bool,
    error: Option<String>,
    #[serde(default)]
    prompt_eval_count: Option<u64>,
    #[serde(default)]
    eval_count: Option<u64>,
    #[serde(default)]
    total_duration: Option<u64>,
}

#[derive(Debug, Clone)]
struct OllamaStreamStats {
    prompt_eval_count: Option<u64>,
    eval_count: Option<u64>,
    total_duration_ns: Option<u64>,
}

#[derive(Debug, Clone)]
pub(crate) struct GenerationMeta {
    pub(crate) model: String,
    pub(crate) prompt_tokens: Option<u64>,
    pub(crate) completion_tokens: Option<u64>,
    pub(crate) total_duration_ns: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatMessage {
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaRequestMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct OllamaRequestMessage {
    pub(super) role: String,
    pub(super) content: String,
}

/// Inputs for [`ChatService::spawn_reply`], grouped to stay within `clippy::too_many_arguments`.
pub struct SpawnReplyJob {
    pub state: AppState,
    pub conversation: ChatConversation,
    pub conversation_scope_id: String,
    pub prompt: String,
    pub should_auto_name: bool,
    pub deep_research: bool,
    pub reply_model: String,
    pub active_chat: ActiveChatHandle,
}

#[derive(Clone)]
pub struct ChatService {
    core: OllamaCore,
    multi_pass_enabled: bool,
}

impl ChatService {
    pub fn new(core: OllamaCore) -> Self {
        Self {
            core,
            multi_pass_enabled: true,
        }
    }

    pub fn with_multi_pass_enabled(mut self, enabled: bool) -> Self {
        self.multi_pass_enabled = enabled;
        self
    }

    pub fn model(&self) -> &str {
        self.core.model()
    }

    pub fn chat_client_config(&self) -> crate::models::ChatClientConfig {
        let default_model = cloud_models::default_chat_cloud_model_id(self.model());
        let models = cloud_models::CHAT_CLOUD_MODEL_CHOICES
            .iter()
            .map(|entry| crate::models::ChatModelOption {
                id: entry.id.to_string(),
                label: entry.label.to_string(),
            })
            .collect();
        crate::models::ChatClientConfig {
            default_model,
            models,
        }
    }

    pub async fn is_available(&self) -> bool {
        self.core.is_available().await
    }

    pub fn create_conversation(&self, title: Option<String>) -> ChatConversation {
        let now = Utc::now();
        ChatConversation {
            id: generate_chat_id("conv"),
            title: title.and_then(|value| trim_to_option(&value)),
            title_status: ChatTitleStatus::Idle,
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
        }
    }

    pub fn build_user_message(&self, content: &str) -> ChatMessage {
        ChatMessage {
            id: generate_chat_id("msg"),
            role: ChatRole::User,
            content: content.trim().to_string(),
            sources: Vec::new(),
            status: ChatMessageStatus::Completed,
            created_at: Utc::now(),
            model: None,
            prompt_tokens: None,
            completion_tokens: None,
            total_duration_ns: None,
        }
    }

    pub(crate) fn build_assistant_message(
        &self,
        content: String,
        sources: Vec<ChatSource>,
        status: ChatMessageStatus,
        generation: Option<GenerationMeta>,
    ) -> ChatMessage {
        ChatMessage {
            id: generate_chat_id("msg"),
            role: ChatRole::Assistant,
            content,
            sources,
            status,
            created_at: Utc::now(),
            model: generation.as_ref().map(|meta| meta.model.clone()),
            prompt_tokens: generation.as_ref().and_then(|meta| meta.prompt_tokens),
            completion_tokens: generation.as_ref().and_then(|meta| meta.completion_tokens),
            total_duration_ns: generation.as_ref().and_then(|meta| meta.total_duration_ns),
        }
    }

    fn assistant_generation_meta(
        &self,
        reply_model: &str,
        terminal: Option<OllamaStreamStats>,
    ) -> GenerationMeta {
        GenerationMeta {
            model: reply_model.to_string(),
            prompt_tokens: terminal.as_ref().and_then(|s| s.prompt_eval_count),
            completion_tokens: terminal.as_ref().and_then(|s| s.eval_count),
            total_duration_ns: terminal.as_ref().and_then(|s| s.total_duration_ns),
        }
    }

    pub fn build_provisional_title(&self, content: &str) -> Option<String> {
        trim_to_option(content).map(|value| limit_text(&value, CHAT_TITLE_MAX_CHARS))
    }

    pub fn spawn_reply(&self, job: SpawnReplyJob) {
        let service = self.clone();
        let SpawnReplyJob {
            state,
            conversation,
            conversation_scope_id,
            prompt,
            should_auto_name,
            deep_research,
            reply_model,
            active_chat,
        } = job;
        tokio::spawn(async move {
            if should_auto_name {
                let naming_service = service.clone();
                let naming_state = state.clone();
                let naming_conversation_scope_id = conversation_scope_id.clone();
                let naming_conversation_id = conversation.id.clone();
                let naming_prompt = prompt.clone();
                tokio::spawn(async move {
                    naming_service
                        .generate_and_store_title(
                            naming_state,
                            naming_conversation_scope_id,
                            naming_conversation_id,
                            naming_prompt,
                        )
                        .await;
                });
            }

            service
                .run_reply(
                    state,
                    conversation,
                    conversation_scope_id,
                    prompt,
                    deep_research,
                    reply_model,
                    active_chat,
                )
                .await;
        });
    }

    async fn run_reply(
        &self,
        state: AppState,
        conversation: ChatConversation,
        conversation_scope_id: String,
        prompt: String,
        deep_research: bool,
        reply_model: String,
        active_chat: ActiveChatHandle,
    ) {
        let conversation_id = conversation.id.clone();
        let span = logfire::span!(
            "chat.reply",
            conversation.id = conversation_id.clone(),
            query.chars = prompt.chars().count(),
        );

        async move {
            let reply_result = self
                .generate_reply(
                    &state,
                    &conversation,
                    &prompt,
                    deep_research,
                    &reply_model,
                    &active_chat,
                )
                .await;

            match reply_result {
                Ok(message) => {
                    if let Err(error) = persist_assistant_message(
                        &state,
                        &conversation_scope_id,
                        &conversation_id,
                        &message,
                    )
                    .await
                    {
                        tracing::error!(conversation_id = %conversation_id, error = %error, "failed to persist assistant message");
                        active_chat
                            .emit(ChatStreamEvent::Error {
                                message: "Failed to store chat response.".to_string(),
                            })
                            .await;
                    } else {
                        active_chat.emit(ChatStreamEvent::Done { message }).await;
                    }
                }
                Err(error) => {
                    if error == "cancelled" {
                        let message = self.build_assistant_message(
                            "Response cancelled.".to_string(),
                            Vec::new(),
                            ChatMessageStatus::Cancelled,
                            None,
                        );
                        let _ = persist_assistant_message(
                            &state,
                            &conversation_scope_id,
                            &conversation_id,
                            &message,
                        )
                        .await;
                        active_chat.emit(ChatStreamEvent::Done { message }).await;
                        let mut active_chats = state.active_chats.lock().await;
                        active_chats.remove(&conversation_id);
                        return;
                    }
                    tracing::error!(conversation_id = %conversation_id, error = %error, "chat reply failed");
                    let message = self.build_assistant_message(
                        "I ran into an error while generating that answer.".to_string(),
                        Vec::new(),
                        ChatMessageStatus::Failed,
                        None,
                    );
                    let _ = persist_assistant_message(
                        &state,
                        &conversation_scope_id,
                        &conversation_id,
                        &message,
                    )
                    .await;
                    active_chat
                        .emit(ChatStreamEvent::Error { message: error })
                        .await;
                }
            }

            let mut active_chats = state.active_chats.lock().await;
            active_chats.remove(&conversation_id);
        }
        .instrument(span)
        .await;
    }

    async fn generate_reply(
        &self,
        state: &AppState,
        conversation: &ChatConversation,
        prompt: &str,
        deep_research: bool,
        reply_model: &str,
        active_chat: &ActiveChatHandle,
    ) -> Result<ChatMessage, String> {
        active_chat.ensure_not_cancelled()?;
        if let Some(tool_outcome) = self
            .run_tool_loop(state, conversation, prompt, deep_research, active_chat)
            .await?
        {
            active_chat.ensure_not_cancelled()?;
            if tool_outcome.conversation_only {
                active_chat
                    .emit(ChatStreamEvent::Status {
                        status: ChatStatusPayload::new(
                            "generating",
                            "Answering from the conversation",
                        )
                        .with_detail("No tool call was needed for this turn.")
                        .with_decision(
                            tool_outcome.rationale.clone().unwrap_or_else(|| {
                                "The current conversation already contains enough context."
                                    .to_string()
                            }),
                        ),
                    })
                    .await;
                let grounding = build_conversation_only_grounding();
                let mut cancel_rx = active_chat.subscribe_cancel();
                let (content, terminal_stats) = self
                    .stream_ollama_reply(
                        conversation,
                        grounding,
                        active_chat,
                        &mut cancel_rx,
                        true,
                        reply_model,
                    )
                    .await?;
                return Ok(self.build_assistant_message(
                    content,
                    Vec::new(),
                    ChatMessageStatus::Completed,
                    Some(self.assistant_generation_meta(reply_model, terminal_stats)),
                ));
            }

            let sources = tool_outcome
                .sources
                .iter()
                .map(|source| source.source.clone())
                .collect::<Vec<_>>();
            let tool_outputs = tool_outcome
                .tool_outputs
                .iter()
                .map(|record| format!("{}:\n{}", record.summary, record.output))
                .collect::<Vec<_>>();

            active_chat
                .emit(ChatStreamEvent::Status {
                    status: ChatStatusPayload::new(
                        "generating",
                        "Answering from gathered evidence",
                    )
                    .with_detail(format!(
                        "Composing the final answer from {} tool result{} and {} excerpt{}.",
                        tool_outcome.tool_outputs.len(),
                        if tool_outcome.tool_outputs.len() == 1 {
                            ""
                        } else {
                            "s"
                        },
                        sources.len(),
                        if sources.len() == 1 { "" } else { "s" }
                    ))
                    .with_decision(
                        tool_outcome.rationale.clone().unwrap_or_else(|| {
                            "The tool loop gathered enough evidence to answer.".to_string()
                        }),
                    ),
                })
                .await;

            if !sources.is_empty() {
                active_chat
                    .emit(ChatStreamEvent::Sources {
                        sources: sources.clone(),
                    })
                    .await;
            }

            let grounding = build_tool_grounding_context(&tool_outputs, &tool_outcome.sources);
            let mut cancel_rx = active_chat.subscribe_cancel();
            let (content, terminal_stats) = self
                .stream_ollama_reply(
                    conversation,
                    grounding,
                    active_chat,
                    &mut cancel_rx,
                    false,
                    reply_model,
                )
                .await?;
            return Ok(self.build_assistant_message(
                content,
                sources,
                ChatMessageStatus::Completed,
                Some(self.assistant_generation_meta(reply_model, terminal_stats)),
            ));
        }

        let plan = self
            .plan_retrieval(
                state,
                conversation,
                &conversation.id,
                prompt,
                deep_research,
                active_chat,
            )
            .await?;

        if plan.skip_retrieval {
            active_chat
                .emit(ChatStreamEvent::Status {
                    status: ChatStatusPayload::new("generating", "Answering from the conversation")
                        .with_detail("No new library search for this turn."),
                })
                .await;
            let grounding = build_conversation_only_grounding();
            let mut cancel_rx = active_chat.subscribe_cancel();
            let (content, terminal_stats) = self
                .stream_ollama_reply(
                    conversation,
                    grounding,
                    active_chat,
                    &mut cancel_rx,
                    true,
                    reply_model,
                )
                .await?;
            return Ok(self.build_assistant_message(
                content,
                Vec::new(),
                ChatMessageStatus::Completed,
                Some(self.assistant_generation_meta(reply_model, terminal_stats)),
            ));
        }

        let retrieval_started = Instant::now();
        let retrieval = self
            .retrieve_sources_with_plan(state, &conversation.id, prompt, plan, active_chat)
            .await?;
        active_chat.ensure_not_cancelled()?;
        let retrieved_sources = retrieval.sources;
        tracing::info!(
            conversation_id = %conversation.id,
            query_chars = prompt.chars().count(),
            source_count = retrieved_sources.len(),
            retrieval_elapsed_ms = retrieval_started.elapsed().as_millis() as u64,
            "chat retrieval complete"
        );

        if retrieved_sources.is_empty() {
            return Ok(self.build_assistant_message(
                "I can’t answer that from the currently indexed transcripts and summaries."
                    .to_string(),
                Vec::new(),
                ChatMessageStatus::Rejected,
                None,
            ));
        }

        let mut grounding_context = self
            .build_answer_grounding_context(
                &conversation.id,
                prompt,
                &retrieval.plan,
                &retrieved_sources,
                active_chat,
            )
            .await?;
        active_chat.ensure_not_cancelled()?;
        if retrieval.plan.deep_research {
            grounding_context = format!(
                "The user enabled deep research: synthesize across as much of the grounded evidence below as is relevant. If the library still lacks coverage, say so clearly.\n\n{grounding_context}"
            );
        }
        let sources = retrieved_sources
            .iter()
            .map(|source| source.source.clone())
            .collect::<Vec<_>>();
        active_chat
            .emit(ChatStreamEvent::Status {
                status: ChatStatusPayload::new("generating", "Answering from the evidence")
                    .with_detail(format!(
                        "Composing the final answer from {} selected excerpts.",
                        sources.len()
                    )),
            })
            .await;
        active_chat
            .emit(ChatStreamEvent::Sources {
                sources: sources.clone(),
            })
            .await;

        let mut cancel_rx = active_chat.subscribe_cancel();
        let reply_started = Instant::now();
        let (content, terminal_stats) = self
            .stream_ollama_reply(
                conversation,
                grounding_context,
                active_chat,
                &mut cancel_rx,
                false,
                reply_model,
            )
            .await?;
        tracing::info!(
            conversation_id = %conversation.id,
            response_chars = content.chars().count(),
            response_elapsed_ms = reply_started.elapsed().as_millis() as u64,
            "chat response generated"
        );

        Ok(self.build_assistant_message(
            content,
            sources,
            ChatMessageStatus::Completed,
            Some(self.assistant_generation_meta(reply_model, terminal_stats)),
        ))
    }

    async fn run_tool_loop(
        &self,
        state: &AppState,
        conversation: &ChatConversation,
        prompt: &str,
        deep_research: bool,
        active_chat: &ActiveChatHandle,
    ) -> Result<Option<ToolLoopOutcome>, String> {
        let prompt_scope = tools::resolve_mention_scope(&state.db, prompt)
            .await
            .unwrap_or_else(|error| {
                tracing::warn!(error = %error, "failed to resolve tool-loop @mentions");
                tools::MentionScope {
                    cleaned_prompt: prompt.trim().to_string(),
                    ..tools::MentionScope::default()
                }
            });
        let mut tool_outputs = Vec::<ToolEvidenceRecord>::new();
        let mut gathered_sources = Vec::<RetrievedChatSource>::new();
        let max_steps = if deep_research {
            CHAT_TOOL_LOOP_MAX_STEPS_DEEP_RESEARCH
        } else {
            CHAT_TOOL_LOOP_MAX_STEPS
        };

        if let Some(call) = maybe_direct_recent_activity_tool_call(prompt, &prompt_scope) {
            self.execute_planned_tool_call(ToolCallExecutionRequest {
                state,
                call,
                prompt_scope: &prompt_scope,
                rationale: Some(
                    "This asks about what a scoped channel has been doing lately, so recent library activity was gathered first.",
                ),
                tool_outputs: &mut tool_outputs,
                gathered_sources: &mut gathered_sources,
                active_chat,
            })
            .await?;
        }

        for step in 1..=max_steps {
            active_chat.ensure_not_cancelled()?;
            active_chat
                .emit(ChatStreamEvent::Status {
                    status: ChatStatusPayload::new("tool_planning", "Planning next step")
                        .with_detail(format!(
                            "Choosing whether to answer now or call a tool (step {step}/{max_steps})."
                        )),
                })
                .await;

            let planner_prompt = prompt_scope.prompt_for_planner(prompt);
            let planner_input = format_tool_loop_input(
                conversation,
                &planner_prompt,
                &tool_outputs,
                &gathered_sources,
            );
            let planned = await_or_cancel(
                active_chat,
                timeout(
                    CHAT_CLASSIFY_TIMEOUT,
                    self.core.prompt_with_fallback(
                        "chat_tool_loop",
                        CHAT_TOOL_LOOP_PROMPT,
                        &planner_input,
                        crate::services::ollama::CooldownStatusPolicy::UseLocalFallback,
                    ),
                ),
            )
            .await?;

            let step_outcome = match planned {
                Ok(Ok((response, _))) => match parse_json_response::<ChatToolLoopResponse>(&response)
                {
                    Ok(payload) => payload.into_step_outcome().map_err(|error| {
                        tracing::warn!(error = %error, "chat tool loop returned invalid tool request");
                        error
                    })?,
                    Err(error) => {
                        tracing::warn!(error = %error, "chat tool loop returned unreadable JSON");
                        return Ok(None);
                    }
                },
                Ok(Err(error)) => {
                    tracing::warn!(error = ?error, "chat tool loop unavailable");
                    return Ok(None);
                }
                Err(_) => {
                    tracing::warn!("chat tool loop timed out");
                    return Ok(None);
                }
            };

            match step_outcome.action {
                ToolLoopAction::Respond => {
                    let conversation_only = tool_outputs.is_empty() && gathered_sources.is_empty();
                    return Ok(Some(ToolLoopOutcome {
                        conversation_only,
                        rationale: step_outcome.rationale,
                        tool_outputs,
                        sources: gathered_sources,
                    }));
                }
                ToolLoopAction::ToolCall(call) => {
                    self.execute_planned_tool_call(ToolCallExecutionRequest {
                        state,
                        call,
                        prompt_scope: &prompt_scope,
                        rationale: step_outcome.rationale.as_deref(),
                        tool_outputs: &mut tool_outputs,
                        gathered_sources: &mut gathered_sources,
                        active_chat,
                    })
                    .await?;
                }
            }
        }

        Ok(Some(ToolLoopOutcome {
            conversation_only: tool_outputs.is_empty() && gathered_sources.is_empty(),
            rationale: Some("Reached the tool-step limit for this turn.".to_string()),
            tool_outputs,
            sources: gathered_sources,
        }))
    }

    async fn execute_planned_tool_call(
        &self,
        request: ToolCallExecutionRequest<'_>,
    ) -> Result<(), String> {
        let ToolCallExecutionRequest {
            state,
            call,
            prompt_scope,
            rationale,
            tool_outputs,
            gathered_sources,
            active_chat,
        } = request;
        active_chat.ensure_not_cancelled()?;
        active_chat
            .emit(ChatStreamEvent::Status {
                status: ChatStatusPayload::new(
                    "tool",
                    format!("Running {}", call.label().to_ascii_lowercase()),
                )
                .with_detail(match &call {
                    PlannedChatToolCall::SearchLibrary(_) => {
                        "Running a grounded library search.".to_string()
                    }
                    PlannedChatToolCall::DbInspect(_) => {
                        "Running a read-only database query.".to_string()
                    }
                    PlannedChatToolCall::HighlightLookup(_) => {
                        "Looking up saved highlights.".to_string()
                    }
                    PlannedChatToolCall::RecentLibraryActivity(_) => {
                        "Reviewing recent processed videos for the scoped channel.".to_string()
                    }
                })
                .with_decision(
                    rationale.and_then(trim_to_option).unwrap_or_else(|| {
                        "This tool call is needed to gather evidence.".to_string()
                    }),
                )
                .with_tool(ChatToolStatusPayload::new(
                    call.tool_name(),
                    call.label(),
                    "running",
                    call.input_summary(),
                )),
            })
            .await;

        match &call {
            PlannedChatToolCall::DbInspect(query) => {
                let result = tools::execute_db_inspect_query(&state.db, *query)
                    .await
                    .map_err(|error| error.to_string())?;
                let output = result.output.clone();
                tool_outputs.push(ToolEvidenceRecord {
                    summary: result.summary.clone(),
                    output: output.clone(),
                });
                active_chat
                    .emit(ChatStreamEvent::Status {
                        status: ChatStatusPayload::new("tool_complete", "Database lookup complete")
                            .with_detail(output.clone())
                            .with_tool(
                                ChatToolStatusPayload::new(
                                    call.tool_name(),
                                    call.label(),
                                    "completed",
                                    result.summary,
                                )
                                .with_output(output),
                            ),
                    })
                    .await;
            }
            PlannedChatToolCall::SearchLibrary(query) => {
                let result = self
                    .execute_search_library_query(
                        state,
                        query.clone(),
                        Some(prompt_scope),
                        active_chat,
                    )
                    .await?;
                merge_retrieved_sources(gathered_sources, result.sources.iter().cloned());
                tool_outputs.push(ToolEvidenceRecord {
                    summary: result.summary.clone(),
                    output: result.output.clone(),
                });
                active_chat
                    .emit(ChatStreamEvent::Status {
                        status: ChatStatusPayload::new("tool_complete", "Library search complete")
                            .with_detail(result.output.clone())
                            .with_tool(
                                ChatToolStatusPayload::new(
                                    call.tool_name(),
                                    call.label(),
                                    "completed",
                                    result.summary,
                                )
                                .with_output(result.output),
                            ),
                    })
                    .await;
            }
            PlannedChatToolCall::HighlightLookup(query) => {
                let result = tools::execute_highlight_lookup_query(&state.db, query.clone())
                    .await
                    .map_err(|error| error.to_string())?;
                let output = result.output.clone();
                tool_outputs.push(ToolEvidenceRecord {
                    summary: result.summary.clone(),
                    output: output.clone(),
                });
                active_chat
                    .emit(ChatStreamEvent::Status {
                        status: ChatStatusPayload::new(
                            "tool_complete",
                            "Saved highlights lookup complete",
                        )
                        .with_detail(output.clone())
                        .with_tool(
                            ChatToolStatusPayload::new(
                                call.tool_name(),
                                call.label(),
                                "completed",
                                result.summary,
                            )
                            .with_output(output),
                        ),
                    })
                    .await;
            }
            PlannedChatToolCall::RecentLibraryActivity(query) => {
                let query = apply_recent_activity_scope(query.clone(), prompt_scope);
                let result = execute_recent_library_activity_query(&state.db, &query).await?;
                let output = result.output.clone();
                merge_retrieved_sources(
                    gathered_sources,
                    result
                        .materials
                        .into_iter()
                        .map(retrieved_source_from_search_material),
                );
                tool_outputs.push(ToolEvidenceRecord {
                    summary: result.summary.clone(),
                    output: output.clone(),
                });
                active_chat
                    .emit(ChatStreamEvent::Status {
                        status: ChatStatusPayload::new(
                            "tool_complete",
                            "Recent library activity complete",
                        )
                        .with_detail(output.clone())
                        .with_tool(
                            ChatToolStatusPayload::new(
                                call.tool_name(),
                                call.label(),
                                "completed",
                                result.summary,
                            )
                            .with_output(output),
                        ),
                    })
                    .await;
            }
        }

        Ok(())
    }

    async fn retrieve_sources_with_plan(
        &self,
        state: &AppState,
        conversation_id: &str,
        prompt: &str,
        plan: ChatRetrievalPlan,
        active_chat: &ActiveChatHandle,
    ) -> Result<ChatRetrievalOutcome, String> {
        let span = logfire::span!(
            "chat.retrieve",
            conversation.id = conversation_id.to_string(),
            query.chars = prompt.chars().count(),
        );

        async move {
            active_chat.ensure_not_cancelled()?;
            let mut pool = HashMap::<String, AccumulatedSearchCandidate>::new();

            let pass_one_queries = plan.queries_for_pass(1);
            let pass_one_channel_focus = plan.channel_focus_ids.clone();
            let pass_one_video_focus = plan.video_focus_ids.clone();
            let pass_one = self
                .run_retrieval_pass(
                    state,
                    &mut pool,
                    RetrievalPassRequest {
                        conversation_id,
                        plan: &plan,
                        pass: 1,
                        queries: &pass_one_queries,
                        channel_focus_ids: &pass_one_channel_focus,
                        video_focus_ids: &pass_one_video_focus,
                        active_chat,
                    },
                )
                .await?;
            let mut sources = pass_one.sources;
            let mut assessment = pass_one.assessment;
            let mut pass_count = 1;

            if CHAT_MAX_RETRIEVAL_PASSES > 1
                && self.multi_pass_enabled
                && assessment.needs_more
                && plan.supports_second_pass()
            {
                let mut status =
                    ChatStatusPayload::new("retrieving_pass_2", "Broadening the search")
                        .with_detail(format!(
                            "Pass 1 produced {} excerpts across {} videos.",
                            sources.len(),
                            count_unique_videos(&sources)
                        ));
                if let Some(reason) = &assessment.reason {
                    status = status.with_decision(reason.clone());
                }
                active_chat.emit(ChatStreamEvent::Status { status }).await;
                let pass_two_queries = plan.queries_for_pass(2);
                let pass_two_channel_focus =
                    merge_channel_focus_ids(&plan.channel_focus_ids, &assessment.channel_focus_ids);
                let pass_two_video_focus = plan.video_focus_ids.clone();
                let pass_two = self
                    .run_retrieval_pass(
                        state,
                        &mut pool,
                        RetrievalPassRequest {
                            conversation_id,
                            plan: &plan,
                            pass: 2,
                            queries: &pass_two_queries,
                            channel_focus_ids: &pass_two_channel_focus,
                            video_focus_ids: &pass_two_video_focus,
                            active_chat,
                        },
                    )
                    .await?;
                active_chat.ensure_not_cancelled()?;
                sources = pass_two.sources;
                assessment = pass_two.assessment;
                pass_count = 2;
            }

            if CHAT_MAX_RETRIEVAL_PASSES > 2
                && self.multi_pass_enabled
                && assessment.needs_more
                && plan.supports_third_pass()
            {
                let mut status = ChatStatusPayload::new("retrieving_pass_3", "Deepening evidence")
                    .with_detail(format!(
                        "After pass 2: {} excerpts across {} videos.",
                        sources.len(),
                        count_unique_videos(&sources)
                    ));
                if let Some(reason) = &assessment.reason {
                    status = status.with_decision(reason.clone());
                }
                active_chat.emit(ChatStreamEvent::Status { status }).await;
                let pass_three_queries = plan.queries_for_pass(3);
                let pass_three_channel_focus =
                    merge_channel_focus_ids(&plan.channel_focus_ids, &assessment.channel_focus_ids);
                let pass_three_video_focus = plan.video_focus_ids.clone();
                let pass_three = self
                    .run_retrieval_pass(
                        state,
                        &mut pool,
                        RetrievalPassRequest {
                            conversation_id,
                            plan: &plan,
                            pass: 3,
                            queries: &pass_three_queries,
                            channel_focus_ids: &pass_three_channel_focus,
                            video_focus_ids: &pass_three_video_focus,
                            active_chat,
                        },
                    )
                    .await?;
                active_chat.ensure_not_cancelled()?;
                sources = pass_three.sources;
                assessment = pass_three.assessment;
                pass_count = 3;
            }

            if let Some(reason) = assessment.reason {
                active_chat
                    .emit(ChatStreamEvent::Status {
                        status: ChatStatusPayload::new("retrieving_complete", "Search complete")
                            .with_detail(format!(
                                "Collected {} excerpts across {} videos.",
                                sources.len(),
                                count_unique_videos(&sources)
                            ))
                            .with_decision(reason),
                    })
                    .await;
            }

            tracing::info!(
                conversation_id = conversation_id,
                plan_label = %plan.label,
                source_count = sources.len(),
                unique_video_count = count_unique_videos(&sources),
                retrieval_passes = pass_count,
                "chat adaptive retrieval complete"
            );

            Ok(ChatRetrievalOutcome { plan, sources })
        }
        .instrument(span)
        .await
    }

    async fn plan_retrieval(
        &self,
        state: &AppState,
        conversation: &ChatConversation,
        conversation_id: &str,
        prompt: &str,
        deep_research: bool,
        active_chat: &ActiveChatHandle,
    ) -> Result<ChatRetrievalPlan, String> {
        let span = logfire::span!(
            "chat.plan",
            conversation.id = conversation_id.to_string(),
            query.chars = prompt.chars().count(),
            multi_pass_enabled = self.multi_pass_enabled,
        );

        async move {
            active_chat.ensure_not_cancelled()?;
            let prompt = prompt.trim();
            let scope = match tools::resolve_mention_scope(&state.db, prompt).await {
                Ok(scope) => scope,
                Err(error) => {
                    tracing::warn!(error = %error, "failed to resolve chat @mentions");
                    tools::MentionScope {
                        cleaned_prompt: prompt.to_string(),
                        ..tools::MentionScope::default()
                    }
                }
            };
            let retrieval_prompt = scope.prompt_for_retrieval(prompt);
            let planner_prompt = scope.prompt_for_planner(prompt);
            let planner_input = format_conversation_for_planner(conversation, &planner_prompt);

            active_chat
                .emit(ChatStreamEvent::Status {
                    status: ChatStatusPayload::new("classifying", "Planning search")
                        .with_detail(
                            "Deciding whether this needs a focused lookup, broader evidence, or only prior context.",
                        ),
                })
                .await;

            let planned = await_or_cancel(
                active_chat,
                timeout(
                    CHAT_CLASSIFY_TIMEOUT,
                    self.core.prompt_with_fallback(
                        "chat_query_plan",
                        CHAT_QUERY_PLAN_PROMPT,
                        &planner_input,
                        crate::services::ollama::CooldownStatusPolicy::UseLocalFallback,
                    ),
                ),
            )
            .await?;

            let mut plan = match planned {
                Ok(Ok((response, _))) => {
                    match parse_json_response::<ChatQueryPlanResponse>(&response) {
                        Ok(payload) => ChatRetrievalPlan::from_response(&retrieval_prompt, payload),
                        Err(error) => ChatRetrievalPlan::fallback(
                            &retrieval_prompt,
                            Some(format!(
                                "Planner returned unreadable JSON; falling back to synthesis ({error})."
                            )),
                        ),
                    }
                }
                Ok(Err(error)) => ChatRetrievalPlan::fallback(
                    &retrieval_prompt,
                    Some(format!(
                        "Planner unavailable; falling back to synthesis ({error:?})."
                    )),
                ),
                Err(_) => ChatRetrievalPlan::fallback(
                    &retrieval_prompt,
                    Some("Planner timed out; falling back to synthesis.".to_string()),
                ),
            };

            plan.apply_scope(&scope);

            if !self.multi_pass_enabled && !plan.skip_retrieval {
                let rationale = plan.rationale.clone().or(Some(
                    "Adaptive multi-pass retrieval is disabled; using a single direct search."
                        .to_string(),
                ));
                plan = ChatRetrievalPlan::fallback(&retrieval_prompt, rationale);
                plan.apply_scope(&scope);
            }

            if deep_research {
                plan.apply_deep_research(&retrieval_prompt);
                plan.apply_scope(&scope);
            }

            tracing::info!(
                conversation_id = conversation_id,
                intent = %plan.intent.label(),
                plan_label = %plan.label,
                budget = plan.budget,
                max_per_video = plan.max_per_video,
                query_count = plan.queries.len(),
                expansion_query_count = plan.expansion_queries.len(),
                attributed_preference = plan.attributed_preference,
                skip_retrieval = plan.skip_retrieval,
                deep_research = plan.deep_research,
                "chat retrieval plan resolved"
            );

            let mut status = ChatStatusPayload::new("classifying", "Search plan ready")
                .with_detail(format!(
                    "Using {} with up to {} excerpts and {} per video.",
                    plan.label.to_ascii_lowercase(),
                    plan.budget,
                    plan.max_per_video
                ))
                .with_plan(plan.visibility());
            if let Some(rationale) = &plan.rationale {
                status = status.with_decision(rationale.clone());
            }
            active_chat.emit(ChatStreamEvent::Status { status }).await;

            Ok(plan)
        }
        .instrument(span)
        .await
    }

    async fn run_retrieval_pass(
        &self,
        state: &AppState,
        pool: &mut HashMap<String, AccumulatedSearchCandidate>,
        request: RetrievalPassRequest<'_>,
    ) -> Result<RetrievalPassOutcome, String> {
        let RetrievalPassRequest {
            conversation_id,
            plan,
            pass,
            queries,
            channel_focus_ids,
            video_focus_ids,
            active_chat,
        } = request;
        let span = logfire::span!(
            "chat.retrieve.pass",
            conversation.id = conversation_id.to_string(),
            retrieval.pass = pass,
            query_count = queries.len(),
            channel_focus_count = channel_focus_ids.len(),
            video_focus_count = video_focus_ids.len(),
            plan.label = plan.label.clone(),
        );

        async move {
            active_chat.ensure_not_cancelled()?;
            if queries.is_empty() {
                let sources = rank_chat_sources(pool.values(), plan);
                return Ok(RetrievalPassOutcome {
                    assessment: assess_coverage(plan, &sources),
                    sources,
                });
            }

            let channel_scope_note = if channel_focus_ids.is_empty() {
                String::new()
            } else {
                format!(
                    " Balancing toward {} under-covered channel{}.",
                    channel_focus_ids.len(),
                    if channel_focus_ids.len() == 1 {
                        ""
                    } else {
                        "s"
                    }
                )
            };
            let mut status = ChatStatusPayload::new(
                format!("retrieving_pass_{pass}"),
                if pass == 1 {
                    "Searching the library".to_string()
                } else {
                    "Broadening the search".to_string()
                },
            )
            .with_detail(format!(
                "Running {} keyword + semantic search quer{}.{}",
                queries.len(),
                if queries.len() == 1 { "y" } else { "ies" },
                channel_scope_note
            ))
            .with_decision(format!("Queries: {}", queries.join(" · ")));
            if pass == 1 {
                status = status.with_plan(plan.visibility());
            }
            active_chat.emit(ChatStreamEvent::Status { status }).await;

            let candidate_limit = retrieval_candidate_limit(plan.budget, queries.len(), pass);
            let (keyword_batches, semantic_batches) = self
                .collect_retrieval_candidates(RetrievalCandidateRequest {
                    state,
                    queries,
                    candidate_limit,
                    channel_focus_ids,
                    video_focus_ids,
                    source_kind: None,
                    active_chat,
                })
                .await?;
            active_chat.ensure_not_cancelled()?;

            for batch in &keyword_batches {
                accumulate_ranked_candidates(pool, batch, false, pass);
            }
            for batch in &semantic_batches {
                accumulate_ranked_candidates(pool, batch, true, pass);
            }

            let sources = rank_chat_sources(pool.values(), plan);
            active_chat
                .emit(ChatStreamEvent::Sources {
                    sources: sources.iter().map(|source| source.source.clone()).collect(),
                })
                .await;

            let assessment = assess_coverage(plan, &sources);
            tracing::info!(
                conversation_id = conversation_id,
                pass = pass,
                query_count = queries.len(),
                candidate_limit = candidate_limit,
                keyword_batch_count = keyword_batches.len(),
                keyword_candidate_count = keyword_batches.iter().map(Vec::len).sum::<usize>(),
                semantic_batch_count = semantic_batches.len(),
                semantic_candidate_count = semantic_batches.iter().map(Vec::len).sum::<usize>(),
                selected_source_count = sources.len(),
                unique_video_count = count_unique_videos(&sources),
                needs_more = assessment.needs_more,
                "chat retrieval pass complete"
            );

            Ok(RetrievalPassOutcome {
                assessment,
                sources,
            })
        }
        .instrument(span)
        .await
    }

    async fn collect_retrieval_candidates(
        &self,
        request: RetrievalCandidateRequest<'_>,
    ) -> Result<(Vec<Vec<SearchCandidate>>, Vec<Vec<SearchCandidate>>), String> {
        let RetrievalCandidateRequest {
            state,
            queries,
            candidate_limit,
            channel_focus_ids,
            video_focus_ids,
            source_kind,
            active_chat,
        } = request;
        let conn = state.db.connect();
        let mut keyword_batches: Vec<Vec<SearchCandidate>> = Vec::new();
        let filters = if channel_focus_ids.is_empty() {
            vec![None]
        } else {
            channel_focus_ids
                .iter()
                .map(|value| Some(value.as_str()))
                .collect()
        };

        for query in queries {
            active_chat.ensure_not_cancelled()?;
            let query_tokens = crate::search_query::meaningful_search_terms(query);
            for channel_filter in &filters {
                active_chat.ensure_not_cancelled()?;
                let results = state
                    .fts
                    .search(query, source_kind, *channel_filter, candidate_limit)
                    .await;
                let candidates: Vec<SearchCandidate> = results
                    .into_iter()
                    .map(|r| {
                        let mut c: SearchCandidate = r.into();
                        if !query_tokens.is_empty() {
                            c.chunk_text = crate::services::search::extract_keyword_snippet(
                                &c.chunk_text,
                                &query_tokens,
                            );
                        }
                        c
                    })
                    .collect();
                keyword_batches.push(candidates);
            }
        }

        let semantic_batches = match state.search.model() {
            Some(model) if state.search.semantic_enabled() => {
                let embeddings =
                    match await_or_cancel(active_chat, state.search.embed_texts(queries)).await? {
                        Ok(embeddings) => embeddings,
                        Err(error) => {
                            tracing::warn!(error = %error, "chat semantic retrieval failed");
                            Vec::new()
                        }
                    };
                let mut semantic_batches = Vec::new();
                for embedding in &embeddings {
                    active_chat.ensure_not_cancelled()?;
                    let query_embedding = crate::services::search::vector_to_json(embedding);
                    for channel_filter in &filters {
                        active_chat.ensure_not_cancelled()?;
                        semantic_batches.push(
                            db::search_vector_candidates(
                                &conn,
                                &query_embedding,
                                model,
                                source_kind,
                                *channel_filter,
                                candidate_limit,
                            )
                            .await
                            .map_err(|error| error.to_string())?,
                        );
                    }
                }
                semantic_batches
            }
            _ => Vec::new(),
        };

        let keyword_batches = filter_batches_to_video_scope(keyword_batches, video_focus_ids);
        let semantic_batches = filter_batches_to_video_scope(semantic_batches, video_focus_ids);

        Ok((keyword_batches, semantic_batches))
    }

    async fn execute_search_library_query(
        &self,
        state: &AppState,
        query: tools::SearchLibraryQuery,
        prompt_scope: Option<&tools::MentionScope>,
        active_chat: &ActiveChatHandle,
    ) -> Result<SearchLibraryExecutionResult, String> {
        active_chat.ensure_not_cancelled()?;
        let candidate_limit = retrieval_candidate_limit(query.limit, 1, 1);
        let query_scope = tools::resolve_mention_scope(&state.db, &query.query)
            .await
            .unwrap_or_else(|error| {
                tracing::warn!(error = %error, "failed to resolve search_library @mentions");
                tools::MentionScope {
                    cleaned_prompt: query.query.clone(),
                    ..tools::MentionScope::default()
                }
            });
        let scope = merge_mention_scope(prompt_scope, &query_scope);
        let query_text = scope.scoped_query(&query.query);
        if let Some(video_id) = direct_video_lookup_target(&scope, &query) {
            active_chat.ensure_not_cancelled()?;
            let direct_sources =
                load_direct_video_sources(&state.db, video_id, query.source_kind).await?;
            let output = format_search_library_tool_output(&query, &direct_sources);
            return Ok(SearchLibraryExecutionResult {
                summary: describe_search_library_query(query),
                output,
                sources: direct_sources,
            });
        }
        let query_list = [query_text.clone()];
        let (keyword_batches, semantic_batches) = self
            .collect_retrieval_candidates(RetrievalCandidateRequest {
                state,
                queries: &query_list,
                candidate_limit,
                channel_focus_ids: &scope.channel_focus_ids,
                video_focus_ids: &scope.video_focus_ids,
                source_kind: query.source_kind,
                active_chat,
            })
            .await?;
        active_chat.ensure_not_cancelled()?;

        let mut pool = HashMap::<String, AccumulatedSearchCandidate>::new();
        for batch in &keyword_batches {
            accumulate_ranked_candidates(&mut pool, batch, false, 1);
        }
        for batch in &semantic_batches {
            accumulate_ranked_candidates(&mut pool, batch, true, 1);
        }

        let mut plan = ChatRetrievalPlan::fallback(&query.query, None);
        plan.budget = query.limit;
        plan.max_per_video = 3;
        plan.queries = vec![query_text.clone()];
        plan.expansion_queries.clear();
        plan.apply_scope(&scope);
        let sources = rank_chat_sources(pool.values(), &plan);
        let output = format_search_library_tool_output(&query, &sources);

        Ok(SearchLibraryExecutionResult {
            summary: describe_search_library_query(query),
            output,
            sources,
        })
    }

    async fn build_answer_grounding_context(
        &self,
        conversation_id: &str,
        prompt: &str,
        plan: &ChatRetrievalPlan,
        sources: &[RetrievedChatSource],
        active_chat: &ActiveChatHandle,
    ) -> Result<String, String> {
        let span = logfire::span!(
            "chat.synthesize",
            conversation.id = conversation_id.to_string(),
            plan.intent = plan.intent.label().to_string(),
            plan.label = plan.label.clone(),
            source_count = sources.len(),
            unique_video_count = count_unique_videos(sources),
        );

        async move {
            active_chat.ensure_not_cancelled()?;
            if !plan.intent.needs_synthesis_stage() {
                tracing::info!(
                    conversation_id = conversation_id,
                    source_count = sources.len(),
                    "chat grounding used direct excerpts"
                );
                return Ok(build_grounding_context(sources));
            }

            active_chat
                .emit(ChatStreamEvent::Status {
                    status: ChatStatusPayload::new("synthesizing", "Synthesizing evidence")
                        .with_detail(format!(
                            "Summarizing evidence across {} videos before the final answer.",
                            count_unique_videos(sources)
                        )),
                })
                .await;

            let observation_inputs =
                build_video_observation_inputs(sources, plan.synthesis_video_cap());
            let mut observations = Vec::new();
            for input in observation_inputs {
                active_chat.ensure_not_cancelled()?;
                match self
                    .generate_video_observation(conversation_id, prompt, &input, active_chat)
                    .await
                {
                    Ok(summary) if !summary.trim().is_empty() => {
                        observations.push(VideoObservation {
                            video_title: input.video_title,
                            channel_name: input.channel_name,
                            summary,
                        })
                    }
                    Ok(_) => continue,
                    Err(error) => {
                        tracing::warn!(
                            video_id = %input.video_id,
                            error = %error,
                            "chat video observation synthesis failed"
                        );
                    }
                }
            }

            if observations.is_empty() {
                tracing::info!(
                    conversation_id = conversation_id,
                    source_count = sources.len(),
                    "chat synthesis fell back to raw excerpts"
                );
                return Ok(build_grounding_context(sources));
            }

            tracing::info!(
                conversation_id = conversation_id,
                observation_count = observations.len(),
                unique_video_count = count_unique_videos(sources),
                "chat synthesis grounding ready"
            );

            Ok(build_synthesis_grounding_context(
                prompt,
                plan,
                sources,
                &observations,
                synthesis_raw_limit_for_plan(plan),
            ))
        }
        .instrument(span)
        .await
    }

    async fn generate_video_observation(
        &self,
        conversation_id: &str,
        prompt: &str,
        input: &VideoObservationInput,
        active_chat: &ActiveChatHandle,
    ) -> Result<String, String> {
        let span = logfire::span!(
            "chat.synthesize.observation",
            conversation.id = conversation_id.to_string(),
            video.id = input.video_id.clone(),
            excerpt_count = input.excerpts.len(),
        );

        async move {
            active_chat.ensure_not_cancelled()?;
            let mut evidence = String::new();
            for (index, excerpt) in input.excerpts.iter().enumerate() {
                let number = index + 1;
                evidence.push_str(&format!(
                    "[Excerpt {number}] Type: {}\n{}\n\n",
                    excerpt.source.source_kind.as_str(),
                    limit_text(
                        excerpt.context_text.trim(),
                        CHAT_SYNTHESIS_CONTEXT_MAX_CHARS
                    ),
                ));
            }

            let prompt = format!(
                "User question:\n{question}\n\nVideo: {video}\nChannel: {channel}\n\nGrounded excerpts:\n{evidence}",
                question = prompt.trim(),
                video = input.video_title,
                channel = input.channel_name,
                evidence = evidence.trim()
            );
            let (response, model_used) = await_or_cancel(
                active_chat,
                self.core.prompt_with_fallback(
                    "chat_video_observation",
                    CHAT_VIDEO_OBSERVATION_PROMPT,
                    &prompt,
                    crate::services::ollama::CooldownStatusPolicy::UseLocalFallback,
                ),
            )
            .await?
                .map_err(|error| format!("{error:?}"))?;
            let observation = trim_to_option(&response)
                .ok_or_else(|| "video observation was empty".to_string())?;
            tracing::info!(
                conversation_id = conversation_id,
                video_id = %input.video_id,
                model = %model_used,
                observation_chars = observation.chars().count(),
                "chat video observation generated"
            );
            Ok(observation)
        }
        .instrument(span)
        .await
    }

    async fn stream_ollama_reply(
        &self,
        conversation: &ChatConversation,
        grounding_context: String,
        active_chat: &ActiveChatHandle,
        cancel_rx: &mut watch::Receiver<bool>,
        conversation_only: bool,
        reply_model: &str,
    ) -> Result<(String, Option<OllamaStreamStats>), String> {
        // Cloud LLMs can take many minutes to stream a full response; override
        // the 20s default client timeout with one that covers the whole generation.
        const STREAM_TIMEOUT: Duration = Duration::from_secs(30 * 60);
        const MAX_ATTEMPTS: usize = 3;

        let reply_model = reply_model.to_string();

        let span = logfire::span!(
            "chat.generate",
            conversation.id = conversation.id.clone(),
            model = reply_model.clone(),
            history_count = conversation.messages.len().min(CHAT_HISTORY_LIMIT),
            grounding_chars = grounding_context.chars().count(),
        );

        async move {
            let messages = build_ollama_messages(conversation, grounding_context, conversation_only);
            let request = OllamaChatRequest {
                model: reply_model.clone(),
                messages,
                stream: true,
            };

            let _permit = self
                .core
                .acquire_local_permit(reply_model.as_str())
                .await
                .map_err(|error| error.to_string())?;

            let mut last_error = String::new();

            'retry: for attempt in 1..=MAX_ATTEMPTS {
                if attempt > 1 {
                    tracing::warn!(
                        conversation_id = %conversation.id,
                        model = %reply_model,
                        attempt,
                        error = %last_error,
                        "chat stream failed, retrying"
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }

                let response = match self
                    .core
                    .auth(
                        self.core
                            .client()
                            .post(format!("{}/api/chat", self.core.base_url()))
                            .timeout(STREAM_TIMEOUT)
                            .json(&request),
                    )
                    .send()
                    .await
                {
                    Ok(r) => r,
                    Err(error) => {
                        last_error = error.to_string();
                        continue 'retry;
                    }
                };

                if !response.status().is_success() {
                    let status = response.status();
                    let detail = response.text().await.unwrap_or_default();
                    last_error = format!("Ollama chat request failed ({status}): {detail}");
                    continue 'retry;
                }

                let mut response = response;
                let mut pending = String::new();
                let mut content = String::new();
                let mut token_event_count = 0usize;

                loop {
                    tokio::select! {
                        changed = cancel_rx.changed() => {
                            if changed.is_ok() && *cancel_rx.borrow() {
                                return Err("cancelled".to_string());
                            }
                        }
                        next_chunk = response.chunk() => {
                            let chunk = match next_chunk {
                                Ok(Some(c)) => c,
                                Ok(None) => break,
                                Err(error) => {
                                    last_error = error.to_string();
                                    // Only retry if no tokens were sent yet - we cannot
                                    // unsend SSE events that already reached the client.
                                    if content.is_empty() {
                                        continue 'retry;
                                    }
                                    return Err(last_error);
                                }
                            };
                            let chunk_text = std::str::from_utf8(&chunk).map_err(|error| error.to_string())?;
                            pending.push_str(chunk_text);

                            while let Some(newline_index) = pending.find('\n') {
                                let line = pending[..newline_index].trim().to_string();
                                pending.drain(..=newline_index);
                                if line.is_empty() {
                                    continue;
                                }

                                let payload = serde_json::from_str::<OllamaChatResponse>(&line)
                                    .map_err(|error| format!("Failed to parse Ollama chat stream: {error}"))?;
                                if let Some(error) = payload.error.filter(|value| !value.trim().is_empty()) {
                                    return Err(error);
                                }
                                if let Some(token) = payload
                                    .message
                                    .as_ref()
                                    .map(|message| message.content.as_str())
                                    .filter(|value| !value.is_empty())
                                {
                                    content.push_str(token);
                                    token_event_count += 1;
                                    active_chat
                                        .emit(ChatStreamEvent::Token {
                                            token: token.to_string(),
                                        })
                                        .await;
                                }
                                if payload.done {
                                    let content = content.trim().to_string();
                                    let stats = OllamaStreamStats {
                                        prompt_eval_count: payload.prompt_eval_count,
                                        eval_count: payload.eval_count,
                                        total_duration_ns: payload.total_duration,
                                    };
                                    tracing::info!(
                                        conversation_id = %conversation.id,
                                        model = %reply_model,
                                        response_chars = content.chars().count(),
                                        token_event_count,
                                        "chat streaming response complete"
                                    );
                                    return Ok((content, Some(stats)));
                                }
                            }
                        }
                    }
                }

                if !pending.trim().is_empty() {
                    let payload = serde_json::from_str::<OllamaChatResponse>(pending.trim())
                        .map_err(|error| format!("Failed to parse Ollama chat stream tail: {error}"))?;
                    if let Some(error) = payload.error.filter(|value| !value.trim().is_empty()) {
                        return Err(error);
                    }
                    if let Some(token) = payload
                        .message
                        .as_ref()
                        .map(|message| message.content.as_str())
                        .filter(|value| !value.is_empty())
                    {
                        content.push_str(token);
                        token_event_count += 1;
                        active_chat
                            .emit(ChatStreamEvent::Token {
                                token: token.to_string(),
                            })
                            .await;
                    }
                    let tail_stats = if payload.done {
                        Some(OllamaStreamStats {
                            prompt_eval_count: payload.prompt_eval_count,
                            eval_count: payload.eval_count,
                            total_duration_ns: payload.total_duration,
                        })
                    } else {
                        None
                    };
                    let content = content.trim().to_string();
                    tracing::info!(
                        conversation_id = %conversation.id,
                        model = %reply_model,
                        response_chars = content.chars().count(),
                        token_event_count,
                        "chat streaming response complete"
                    );
                    return Ok((content, tail_stats));
                }

                let content = content.trim().to_string();
                tracing::info!(
                    conversation_id = %conversation.id,
                    model = %reply_model,
                    response_chars = content.chars().count(),
                    token_event_count,
                    "chat streaming response complete"
                );
                return Ok((content, None));
            }

            Err(last_error)
        }
        .instrument(span)
        .await
    }

    async fn generate_and_store_title(
        &self,
        state: AppState,
        conversation_scope_id: String,
        conversation_id: String,
        prompt: String,
    ) {
        let span = logfire::span!(
            "chat.title",
            conversation.id = conversation_id.clone(),
            prompt.chars = prompt.chars().count(),
        );

        async move {
            let started = Instant::now();
            let (generated_title, model_used) = match self.generate_title(&prompt).await {
                Ok(title) => title,
                Err(error) => {
                    tracing::warn!(conversation_id = %conversation_id, error = %error, "chat title generation failed");
                    let _ = finalize_title_generation(
                        &state,
                        &conversation_scope_id,
                        &conversation_id,
                        None,
                    )
                    .await;
                    return;
                }
            };

            if let Err(error) = finalize_title_generation(
                &state,
                &conversation_scope_id,
                &conversation_id,
                Some(generated_title.clone()),
            )
            .await
            {
                tracing::warn!(conversation_id = %conversation_id, error = %error, "failed to persist generated title");
                return;
            }

            tracing::info!(
                conversation_id = %conversation_id,
                model = %model_used,
                elapsed_ms = started.elapsed().as_millis() as u64,
                "chat title generated"
            );
        }
        .instrument(span)
        .await;
    }

    async fn generate_title(&self, prompt: &str) -> Result<(String, String), String> {
        let (response, model_used) = self
            .core
            .prompt_with_fallback(
                "chat_title",
                "Generate a short conversation title in 3 to 5 words. Return only the title text without quotes or punctuation at the end.",
                prompt.trim(),
                crate::services::ollama::CooldownStatusPolicy::UseLocalFallback,
            )
            .await
            .map_err(|error| format!("{error:?}"))?;
        let title = Some(sanitize_generated_title(&response))
            .filter(|title| !title.is_empty())
            .ok_or_else(|| "Ollama title response was empty".to_string())?;
        Ok((title, model_used))
    }
}

async fn finalize_title_generation(
    state: &AppState,
    conversation_scope_id: &str,
    conversation_id: &str,
    title: Option<String>,
) -> Result<(), String> {
    let _lock = state.chat_store_lock.lock().await;
    let conn = state.db.connect();
    let Some(mut conversation) =
        db::get_conversation_for_scope(&conn, conversation_scope_id, conversation_id)
        .await
        .map_err(|error| error.to_string())?
    else {
        return Ok(());
    };

    if conversation.title_status != ChatTitleStatus::Generating {
        return Ok(());
    }

    if let Some(title) = title.and_then(|value| trim_to_option(&value)) {
        conversation.title = Some(limit_text(&title, CHAT_TITLE_MAX_CHARS));
        conversation.title_status = ChatTitleStatus::Ready;
    } else {
        conversation.title_status = if conversation.title.is_some() {
            ChatTitleStatus::Ready
        } else {
            ChatTitleStatus::Idle
        };
    }
    conversation.updated_at = Utc::now();
    db::upsert_conversation_for_scope(&conn, conversation_scope_id, &conversation)
        .await
        .map_err(|error| error.to_string())
}

async fn persist_assistant_message(
    state: &AppState,
    conversation_scope_id: &str,
    conversation_id: &str,
    message: &ChatMessage,
) -> Result<(), String> {
    let _lock = state.chat_store_lock.lock().await;
    let conn = state.db.connect();
    let Some(mut conversation) =
        db::get_conversation_for_scope(&conn, conversation_scope_id, conversation_id)
        .await
        .map_err(|error| error.to_string())?
    else {
        return Err("Conversation not found".to_string());
    };

    if message.status == ChatMessageStatus::Failed
        && conversation
            .messages
            .iter()
            .any(|candidate| candidate.id == message.id)
    {
        return Ok(());
    }

    conversation.messages.push(message.clone());
    conversation.updated_at = Utc::now();
    db::upsert_conversation_for_scope(&conn, conversation_scope_id, &conversation)
        .await
        .map_err(|error| error.to_string())
}

fn parse_json_response<T: for<'de> Deserialize<'de>>(response: &str) -> Result<T, String> {
    serde_json::from_str(response)
        .or_else(|_| {
            let start = response
                .find('{')
                .ok_or_else(|| "missing JSON object".to_string())?;
            let end = response
                .rfind('}')
                .ok_or_else(|| "missing JSON object end".to_string())?;
            serde_json::from_str(&response[start..=end]).map_err(|error| error.to_string())
        })
        .map_err(|error| error.to_string())
}

fn sanitize_generated_title(input: &str) -> String {
    limit_text(
        input
            .trim()
            .trim_matches('"')
            .trim_matches('“')
            .trim_matches('”')
            .trim_matches('.')
            .trim(),
        CHAT_TITLE_MAX_CHARS,
    )
}

fn trim_to_option(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn generate_chat_id(prefix: &str) -> String {
    format!(
        "{prefix}_{}_{}",
        Utc::now().timestamp_millis(),
        NEXT_CHAT_ID.fetch_add(1, Ordering::Relaxed)
    )
}

fn format_conversation_for_planner(
    conversation: &ChatConversation,
    current_prompt: &str,
) -> String {
    let mut lines = Vec::new();
    for message in conversation.messages.iter() {
        let label = match message.role {
            ChatRole::User => "User",
            ChatRole::Assistant => "Assistant",
            ChatRole::System => continue,
        };
        lines.push(format!("{label}: {}", message.content.trim()));
    }
    let transcript = lines.join("\n");
    let capped = limit_text(&transcript, CHAT_PLANNER_CONVERSATION_MAX_CHARS);
    format!(
        "RECENT CONVERSATION:\n{capped}\n\nCURRENT USER MESSAGE:\n{}",
        current_prompt.trim()
    )
}

fn format_tool_loop_input(
    conversation: &ChatConversation,
    current_prompt: &str,
    tool_outputs: &[ToolEvidenceRecord],
    gathered_sources: &[RetrievedChatSource],
) -> String {
    let mut tool_lines = Vec::new();
    for (index, record) in tool_outputs.iter().enumerate() {
        tool_lines.push(format!(
            "Tool {}: {}\n{}",
            index + 1,
            record.summary,
            record.output.trim()
        ));
    }
    if !gathered_sources.is_empty() {
        tool_lines.push(format!(
            "Retrieved excerpt count: {}",
            gathered_sources.len()
        ));
        for source in gathered_sources.iter().take(6) {
            tool_lines.push(format!(
                "- [{}] {} / {} / {}",
                source.source.source_kind.as_str(),
                source.source.channel_name,
                source.source.video_title,
                source.source.snippet
            ));
        }
    }

    let tool_results = if tool_lines.is_empty() {
        "none".to_string()
    } else {
        tool_lines.join("\n\n")
    };

    format!(
        "{}\n\nTOOL RESULTS FROM THIS TURN:\n{}",
        format_conversation_for_planner(conversation, current_prompt),
        limit_text(&tool_results, CHAT_PLANNER_CONVERSATION_MAX_CHARS)
    )
}

fn merge_channel_focus_ids(primary: &[String], secondary: &[String]) -> Vec<String> {
    let mut merged = primary.to_vec();
    for channel_id in secondary {
        if !merged.iter().any(|existing| existing == channel_id) {
            merged.push(channel_id.clone());
        }
    }
    merged
}

fn merge_mention_scope(
    primary: Option<&tools::MentionScope>,
    secondary: &tools::MentionScope,
) -> tools::MentionScope {
    let mut merged = primary.cloned().unwrap_or_default();
    if merged.cleaned_prompt.is_empty() {
        merged.cleaned_prompt = secondary.cleaned_prompt.clone();
    }
    merged.channel_focus_ids =
        merge_channel_focus_ids(&merged.channel_focus_ids, &secondary.channel_focus_ids);
    merged.video_focus_ids =
        merge_channel_focus_ids(&merged.video_focus_ids, &secondary.video_focus_ids);
    merged.channel_names = merge_channel_focus_ids(&merged.channel_names, &secondary.channel_names);
    merged.video_titles = merge_channel_focus_ids(&merged.video_titles, &secondary.video_titles);
    merged
}

fn filter_batches_to_video_scope(
    mut batches: Vec<Vec<SearchCandidate>>,
    video_focus_ids: &[String],
) -> Vec<Vec<SearchCandidate>> {
    if video_focus_ids.is_empty() {
        return batches;
    }

    let focus = video_focus_ids.iter().collect::<HashSet<_>>();
    let has_scoped_match = batches.iter().any(|batch| {
        batch
            .iter()
            .any(|candidate| focus.contains(&candidate.video_id))
    });
    if !has_scoped_match {
        return batches;
    }

    for batch in &mut batches {
        batch.retain(|candidate| focus.contains(&candidate.video_id));
    }
    batches
}

fn direct_video_lookup_target<'a>(
    scope: &'a tools::MentionScope,
    query: &tools::SearchLibraryQuery,
) -> Option<&'a str> {
    let [video_id] = scope.video_focus_ids.as_slice() else {
        return None;
    };
    if !is_direct_video_lookup_request(&scope.cleaned_prompt, &query.query) {
        return None;
    }
    Some(video_id.as_str())
}

fn is_direct_video_lookup_request(cleaned_prompt: &str, search_query: &str) -> bool {
    let meaningful_terms = crate::search_query::meaningful_search_terms(cleaned_prompt);
    if meaningful_terms.is_empty() {
        return true;
    }

    let generic_terms = HashSet::from([
        "answer",
        "explain",
        "give",
        "me",
        "read",
        "recap",
        "show",
        "summarize",
        "summary",
        "tell",
        "transcript",
        "video",
        "watch",
    ]);
    let title_terms = crate::search_query::meaningful_search_terms(search_query)
        .into_iter()
        .collect::<HashSet<_>>();

    meaningful_terms
        .into_iter()
        .all(|term| generic_terms.contains(term.as_str()) || title_terms.contains(&term))
}

fn maybe_direct_recent_activity_tool_call(
    prompt: &str,
    scope: &tools::MentionScope,
) -> Option<PlannedChatToolCall> {
    if !is_recent_activity_query(prompt) || is_explicit_realtime_status_query(prompt) {
        return None;
    }
    if !scope.video_focus_ids.is_empty() {
        return None;
    }
    let channel_id = scope.channel_focus_ids.first()?.clone();
    Some(PlannedChatToolCall::RecentLibraryActivity(
        tools::RecentLibraryActivityQuery {
            scope: tools::RecentLibraryActivityScope::Channel,
            channel_id: Some(channel_id),
            video_id: None,
            limit_videos: CHAT_RECENT_ACTIVITY_VIDEO_LIMIT,
            include_summaries: true,
            include_transcripts: true,
        },
    ))
}

fn apply_recent_activity_scope(
    mut query: tools::RecentLibraryActivityQuery,
    scope: &tools::MentionScope,
) -> tools::RecentLibraryActivityQuery {
    if query.channel_id.is_none()
        && matches!(query.scope, tools::RecentLibraryActivityScope::Channel)
    {
        query.channel_id = scope.channel_focus_ids.first().cloned();
    }
    if query.video_id.is_none() {
        query.video_id = scope.video_focus_ids.first().cloned();
    }
    query
}

async fn load_direct_video_sources(
    store: &db::Store,
    video_id: &str,
    source_kind: Option<crate::services::search::SearchSourceKind>,
) -> Result<Vec<RetrievedChatSource>, String> {
    let kinds = match source_kind {
        Some(kind) => vec![kind],
        None => vec![
            crate::services::search::SearchSourceKind::Summary,
            crate::services::search::SearchSourceKind::Transcript,
        ],
    };

    let mut sources = Vec::new();
    for kind in kinds {
        let Some(material) = db::load_search_material(store, video_id, kind)
            .await
            .map_err(|error| error.to_string())?
        else {
            continue;
        };
        sources.push(retrieved_source_from_search_material(material));
    }

    Ok(sources)
}

fn retrieved_source_from_search_material(material: db::SearchMaterial) -> RetrievedChatSource {
    let section_title = match material.source_kind {
        crate::services::search::SearchSourceKind::Summary => Some("Full summary".to_string()),
        crate::services::search::SearchSourceKind::Transcript => {
            Some("Full transcript".to_string())
        }
    };
    let source_kind = material.source_kind;
    let video_id = material.video_id;

    RetrievedChatSource {
        source: ChatSource {
            chunk_id: format!("{video_id}_direct_{}", source_kind.as_str()),
            video_id,
            channel_id: material.channel_id,
            channel_name: material.channel_name,
            video_title: material.video_title,
            source_kind,
            section_title,
            snippet: crate::services::search::truncate_chunk_for_display(&material.content),
            score: 1.0,
            retrieval_pass: Some(1),
        },
        context_text: limit_text(material.content.trim(), CHAT_CONTEXT_MAX_CHARS),
    }
}

fn describe_search_library_query(query: tools::SearchLibraryQuery) -> String {
    let source = match query.source_kind {
        Some(crate::services::search::SearchSourceKind::Summary) => Some("summaries"),
        Some(crate::services::search::SearchSourceKind::Transcript) => Some("transcripts"),
        None => None,
    };
    let source_label = source.unwrap_or("all sources");
    format!(
        "Search the library for \"{}\" in {} (limit {})",
        query.query, source_label, query.limit
    )
}

fn describe_highlight_lookup_query(query: tools::HighlightLookupQuery) -> String {
    match (&query.query, &query.video_title) {
        (Some(query_text), Some(video_title)) => format!(
            "Look up saved highlights for query \"{}\" in videos matching \"{}\" (limit {})",
            query_text, video_title, query.limit
        ),
        (Some(query_text), None) => format!(
            "Look up saved highlights for query \"{}\" (limit {})",
            query_text, query.limit
        ),
        (None, Some(video_title)) => format!(
            "Look up saved highlights in videos matching \"{}\" (limit {})",
            video_title, query.limit
        ),
        (None, None) => format!("Look up saved highlights (limit {})", query.limit),
    }
}

fn describe_recent_library_activity_query(query: tools::RecentLibraryActivityQuery) -> String {
    match query.scope {
        tools::RecentLibraryActivityScope::Channel => format!(
            "Review recent library activity for a scoped channel (latest {} videos)",
            query.limit_videos
        ),
        tools::RecentLibraryActivityScope::Video => format!(
            "Review recent library activity around a scoped video (latest {} videos)",
            query.limit_videos
        ),
        tools::RecentLibraryActivityScope::Library => format!(
            "Review recent library activity across the library (latest {} videos)",
            query.limit_videos
        ),
    }
}

fn format_search_library_tool_output(
    query: &tools::SearchLibraryQuery,
    sources: &[RetrievedChatSource],
) -> String {
    if sources.is_empty() {
        return format!("No grounded excerpts found for \"{}\".", query.query);
    }

    let rows = sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            format!(
                "{}. {} / {} / {} - {}",
                index + 1,
                source.source.channel_name,
                source.source.video_title,
                source.source.source_kind.as_str(),
                source.source.snippet
            )
        })
        .collect::<Vec<_>>();

    format!(
        "Found {} excerpt{} for \"{}\":\n{}",
        sources.len(),
        if sources.len() == 1 { "" } else { "s" },
        query.query,
        rows.join("\n")
    )
}

fn merge_retrieved_sources(
    existing: &mut Vec<RetrievedChatSource>,
    new_sources: impl IntoIterator<Item = RetrievedChatSource>,
) {
    let mut seen = existing
        .iter()
        .map(|source| source.source.chunk_id.clone())
        .collect::<HashSet<_>>();

    for source in new_sources {
        if seen.insert(source.source.chunk_id.clone()) {
            existing.push(source);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::chat_heuristics::{
        collect_focus_terms, is_attributed_preference_query, recommendation_query_variants,
    };
    use super::{
        ActiveChatHandle, CHAT_CLASSIFY_TIMEOUT, CHAT_RECENT_ACTIVITY_SOURCE_LIMIT,
        ChatQueryIntent, ChatQueryPlanResponse, ChatRetrievalPlan, ChatService,
        ChatToolLoopResponse, PlannedChatToolCall, RetrievedChatSource, ToolLoopAction,
        is_direct_video_lookup_request, maybe_direct_recent_activity_tool_call, tools,
    };
    use crate::models::ChatSource;
    use crate::services::chat::tools::{
        DbInspectOperation, DbInspectTarget, DbInspectToolInput, SearchLibraryToolInput,
    };
    use crate::services::ollama::{CLOUD_PROMPT_TIMEOUT_SECS, OllamaCore};
    use crate::services::search::SearchSourceKind;

    #[test]
    fn attributed_preference_queries_are_detected() {
        assert!(is_attributed_preference_query(
            "what is the best database according to theo?"
        ));
        assert!(is_attributed_preference_query(
            "what does theo recommend for databases"
        ));
        assert!(!is_attributed_preference_query(
            "when did theo publish his database video?"
        ));
    }

    #[test]
    fn direct_video_lookup_detects_generic_read_requests() {
        assert!(is_direct_video_lookup_request(
            "read for me",
            "3 Things Every Relationship Has"
        ));
        assert!(is_direct_video_lookup_request(
            "read 3 Things Every Relationship Has for me",
            "3 Things Every Relationship Has"
        ));
        assert!(is_direct_video_lookup_request(
            "summarize +{3 Things Every Relationship Has}",
            "3 Things Every Relationship Has"
        ));
    }

    #[test]
    fn direct_video_lookup_keeps_real_search_queries_as_search() {
        assert!(!is_direct_video_lookup_request(
            "what does it say about dopamine",
            "3 Things Every Relationship Has"
        ));
        assert!(!is_direct_video_lookup_request(
            "find the section about trust",
            "3 Things Every Relationship Has"
        ));
    }

    #[test]
    fn retrieval_plan_upgrades_fact_to_recommendation_lookup() {
        let plan = ChatRetrievalPlan::from_response(
            "what is the best database according to theo?",
            ChatQueryPlanResponse {
                needs_retrieval: Some(true),
                intent: Some("fact".to_string()),
                rationale: Some("single fact lookup".to_string()),
                sub_queries: Some(vec!["best database according to Theo".to_string()]),
                expansion_queries: Some(Vec::new()),
            },
        );

        assert_eq!(plan.intent, ChatQueryIntent::Synthesis);
        assert_eq!(plan.label, "Recommendation lookup");
        assert!(plan.budget >= 12);
        assert!(plan.supports_second_pass());
        assert!(plan.queries.len() > 1);
    }

    #[test]
    fn retrieval_plan_recognizes_recent_activity_queries() {
        let plan = ChatRetrievalPlan::fallback("What is HealthyGamerGG doing lately?", None);
        assert_eq!(plan.intent, ChatQueryIntent::RecentActivity);
        assert_eq!(plan.label, "Recent activity scan");
        assert_eq!(plan.budget, CHAT_RECENT_ACTIVITY_SOURCE_LIMIT);
    }

    #[test]
    fn recommendation_query_variants_keep_subject_and_topic() {
        let queries = recommendation_query_variants("what is the best database according to theo?");
        assert!(
            queries
                .iter()
                .any(|query| query.contains("theo database recommendation"))
        );
        assert!(
            queries
                .iter()
                .any(|query| query.contains("theo favorite database"))
        );
    }

    #[test]
    fn focus_terms_strip_boilerplate_but_keep_topic() {
        let focus_terms = collect_focus_terms("what is the best database according to theo?");
        assert!(focus_terms.contains(&"database".to_string()));
        assert!(!focus_terms.contains(&"theo".to_string()));
        assert!(!focus_terms.contains(&"best".to_string()));
    }

    #[test]
    fn planner_timeout_allows_slow_cloud_classification() {
        assert!(CHAT_CLASSIFY_TIMEOUT.as_secs() >= 15);
        assert!(CHAT_CLASSIFY_TIMEOUT.as_secs() < CLOUD_PROMPT_TIMEOUT_SECS);
    }

    #[test]
    fn deep_research_plan_raises_budget_and_forces_retrieval() {
        let mut plan = ChatRetrievalPlan::fallback("climate policy in my library", None);
        plan.skip_retrieval = true;
        plan.apply_deep_research("climate policy in my library");
        assert!(!plan.skip_retrieval);
        assert!(plan.deep_research);
        assert_eq!(plan.budget, 48);
        assert_eq!(plan.label, "Deep research");
        assert_eq!(plan.intent, ChatQueryIntent::Pattern);
        assert!(!plan.queries.is_empty());
    }

    #[test]
    fn tool_loop_builds_db_inspect_query_when_requested() {
        let execution = ChatToolLoopResponse {
            action: Some("tool_call".to_string()),
            rationale: Some("The user is asking about stored summary records.".to_string()),
            tool_name: Some("db_inspect".to_string()),
            search_library_input: None,
            highlight_lookup_input: None,
            db_inspect_input: Some(DbInspectToolInput {
                operation: Some("count".to_string()),
                resource: Some("summaries".to_string()),
                limit: None,
                group_by: None,
            }),
            recent_library_activity_input: None,
        }
        .into_step_outcome()
        .expect("valid tool plan");

        let ToolLoopAction::ToolCall(PlannedChatToolCall::DbInspect(query)) = execution.action
        else {
            panic!("expected db inspect tool call");
        };

        assert_eq!(query.operation, DbInspectOperation::Count);
        assert_eq!(query.target, DbInspectTarget::Summaries);
        assert_eq!(
            execution.rationale.as_deref(),
            Some("The user is asking about stored summary records.")
        );
    }

    #[test]
    fn tool_loop_builds_search_library_query_when_requested() {
        let execution = ChatToolLoopResponse {
            action: Some("tool_call".to_string()),
            rationale: Some("The user asked about transcript evidence.".to_string()),
            tool_name: Some("search_library".to_string()),
            search_library_input: Some(SearchLibraryToolInput {
                query: Some("ownership model".to_string()),
                source: Some("transcript".to_string()),
                limit: Some(3),
            }),
            highlight_lookup_input: None,
            db_inspect_input: None,
            recent_library_activity_input: None,
        }
        .into_step_outcome()
        .expect("valid tool plan");

        let ToolLoopAction::ToolCall(PlannedChatToolCall::SearchLibrary(query)) = execution.action
        else {
            panic!("expected search tool call");
        };

        assert_eq!(query.query, "ownership model");
        assert_eq!(query.source_kind, Some(SearchSourceKind::Transcript));
        assert_eq!(query.limit, 3);
    }

    #[test]
    fn tool_loop_accepts_direct_search_library_action() {
        let execution = ChatToolLoopResponse {
            action: Some("search_library".to_string()),
            rationale: Some("Need library evidence.".to_string()),
            tool_name: None,
            search_library_input: Some(SearchLibraryToolInput {
                query: Some("saved highlights".to_string()),
                source: Some("all".to_string()),
                limit: Some(4),
            }),
            highlight_lookup_input: None,
            db_inspect_input: None,
            recent_library_activity_input: None,
        }
        .into_step_outcome()
        .expect("direct search_library action should parse");

        let ToolLoopAction::ToolCall(PlannedChatToolCall::SearchLibrary(query)) = execution.action
        else {
            panic!("expected search tool call");
        };

        assert_eq!(query.query, "saved highlights");
        assert_eq!(query.source_kind, None);
        assert_eq!(query.limit, 4);
    }

    #[test]
    fn tool_loop_accepts_direct_highlight_lookup_action() {
        let execution = ChatToolLoopResponse {
            action: Some("highlight_lookup".to_string()),
            rationale: Some("Need saved highlights.".to_string()),
            tool_name: None,
            search_library_input: None,
            highlight_lookup_input: Some(tools::HighlightLookupToolInput {
                query: Some("prototype-first".to_string()),
                video_title: None,
                limit: Some(3),
            }),
            db_inspect_input: None,
            recent_library_activity_input: None,
        }
        .into_step_outcome()
        .expect("direct highlight_lookup action should parse");

        let ToolLoopAction::ToolCall(PlannedChatToolCall::HighlightLookup(query)) =
            execution.action
        else {
            panic!("expected highlight lookup tool call");
        };

        assert_eq!(query.query.as_deref(), Some("prototype-first"));
        assert_eq!(query.video_title, None);
        assert_eq!(query.limit, 3);
    }

    #[test]
    fn tool_loop_accepts_direct_recent_library_activity_action() {
        let execution = ChatToolLoopResponse {
            action: Some("recent_library_activity".to_string()),
            rationale: Some("Need recent channel activity.".to_string()),
            tool_name: None,
            search_library_input: None,
            highlight_lookup_input: None,
            db_inspect_input: None,
            recent_library_activity_input: Some(tools::RecentLibraryActivityToolInput {
                scope: Some("channel".to_string()),
                channel_id: None,
                video_id: None,
                limit_videos: Some(6),
                include_summaries: Some(true),
                include_transcripts: Some(true),
            }),
        }
        .into_step_outcome()
        .expect("direct recent_library_activity action should parse");

        let ToolLoopAction::ToolCall(PlannedChatToolCall::RecentLibraryActivity(query)) =
            execution.action
        else {
            panic!("expected recent library activity tool call");
        };

        assert_eq!(query.scope, tools::RecentLibraryActivityScope::Channel);
        assert_eq!(query.limit_videos, 6);
        assert!(query.include_summaries);
        assert!(query.include_transcripts);
    }

    #[test]
    fn tool_loop_rejects_invalid_db_resource() {
        let error = ChatToolLoopResponse {
            action: Some("tool_call".to_string()),
            rationale: None,
            tool_name: Some("db_inspect".to_string()),
            search_library_input: None,
            highlight_lookup_input: None,
            db_inspect_input: Some(DbInspectToolInput {
                operation: Some("count".to_string()),
                resource: Some("search_sources".to_string()),
                limit: None,
                group_by: None,
            }),
            recent_library_activity_input: None,
        }
        .into_step_outcome()
        .expect_err("invalid db resource should fail");

        assert!(error.contains("unsupported db_inspect resource"));
    }

    #[test]
    fn tool_loop_can_choose_to_respond_without_tool() {
        let outcome = ChatToolLoopResponse {
            action: Some("respond".to_string()),
            rationale: Some("This is just a greeting.".to_string()),
            tool_name: None,
            search_library_input: None,
            highlight_lookup_input: None,
            db_inspect_input: None,
            recent_library_activity_input: None,
        }
        .into_step_outcome()
        .expect("respond action should be accepted");

        assert!(matches!(outcome.action, ToolLoopAction::Respond));
        assert_eq!(
            outcome.rationale.as_deref(),
            Some("This is just a greeting.")
        );
    }

    #[test]
    fn direct_recent_tool_call_prefers_channel_scope_only() {
        let scope = tools::MentionScope {
            cleaned_prompt: "What is HealthyGamerGG doing lately?".to_string(),
            channel_focus_ids: vec!["chan_1".to_string()],
            video_focus_ids: Vec::new(),
            channel_names: vec!["HealthyGamerGG".to_string()],
            video_titles: Vec::new(),
        };

        let Some(PlannedChatToolCall::RecentLibraryActivity(query)) =
            maybe_direct_recent_activity_tool_call("What is HealthyGamerGG doing lately?", &scope)
        else {
            panic!("expected recent activity tool call");
        };

        assert_eq!(query.channel_id.as_deref(), Some("chan_1"));
    }

    #[tokio::test]
    async fn build_answer_grounding_context_returns_cancelled_when_cancelled_before_synthesis() {
        let service = ChatService::new(OllamaCore::new("://invalid-url", "qwen3:8b"));
        let active_chat = ActiveChatHandle::new();
        active_chat.cancel();

        let mut plan = ChatRetrievalPlan::fallback("compare the channels", None);
        plan.intent = ChatQueryIntent::Pattern;

        let result = service
            .build_answer_grounding_context(
                "conv_cancelled",
                "compare the channels",
                &plan,
                &[RetrievedChatSource {
                    source: ChatSource {
                        video_id: "vid_1".to_string(),
                        channel_id: "chan_1".to_string(),
                        channel_name: "Channel One".to_string(),
                        video_title: "Video One".to_string(),
                        source_kind: SearchSourceKind::Transcript,
                        section_title: Some("Intro".to_string()),
                        snippet: "Important supporting excerpt".to_string(),
                        score: 1.0,
                        chunk_id: "chunk_1".to_string(),
                        retrieval_pass: Some(1),
                    },
                    context_text: "Important supporting excerpt with extra context.".to_string(),
                }],
                &active_chat,
            )
            .await;

        assert_eq!(result, Err("cancelled".to_string()));
    }

    #[test]
    fn cancel_marks_handle_cancelled_without_existing_receivers() {
        let active_chat = ActiveChatHandle::new();
        active_chat.cancel();

        assert!(active_chat.is_cancelled());
    }
}
