mod constants;
mod intent;

pub(crate) use constants::*;
pub use intent::ChatQueryIntent;

use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
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

use super::chat_heuristics::{
    build_plan_label, collect_focus_terms, heuristic_expansion_queries, heuristic_query_variants,
    is_attributed_preference_query, push_unique_query, recommendation_query_variants,
    sanitize_queries,
};
use super::chat_prompt::{
    build_conversation_only_grounding, build_grounding_context, build_ollama_messages,
    build_synthesis_grounding_context, synthesis_raw_limit_for_plan,
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
}

impl ChatStatusPayload {
    fn new(stage: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            stage: stage.into(),
            label: Some(label.into()),
            detail: None,
            decision: None,
            plan: None,
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
        Self {
            intent: ChatQueryIntent::Synthesis,
            label: build_plan_label(ChatQueryIntent::Synthesis, attributed_preference).to_string(),
            budget: if attributed_preference {
                CHAT_RECOMMENDATION_SOURCE_LIMIT
            } else {
                CHAT_SYNTHESIS_SOURCE_LIMIT
            },
            max_per_video: if attributed_preference { 3 } else { 4 },
            queries: vec![prompt.trim().to_string()],
            expansion_queries: Vec::new(),
            focus_terms: collect_focus_terms(prompt),
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
        if attributed_preference && matches!(intent, ChatQueryIntent::Fact) {
            intent = ChatQueryIntent::Synthesis;
        }

        let skip_retrieval = response.needs_retrieval == Some(false);

        let (mut budget, mut max_per_video) = match intent {
            ChatQueryIntent::Fact => (CHAT_SOURCE_LIMIT, 3),
            ChatQueryIntent::Synthesis => (CHAT_SYNTHESIS_SOURCE_LIMIT, 4),
            ChatQueryIntent::Pattern => (CHAT_PATTERN_SOURCE_LIMIT, 3),
            ChatQueryIntent::Comparison => (CHAT_COMPARISON_SOURCE_LIMIT, 5),
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
        let _ = self.inner.cancel_tx.send(true);
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
        }
    }

    pub fn build_assistant_message(
        &self,
        content: String,
        sources: Vec<ChatSource>,
        status: ChatMessageStatus,
    ) -> ChatMessage {
        ChatMessage {
            id: generate_chat_id("msg"),
            role: ChatRole::Assistant,
            content,
            sources,
            status,
            created_at: Utc::now(),
        }
    }

    pub fn build_provisional_title(&self, content: &str) -> Option<String> {
        trim_to_option(content).map(|value| limit_text(&value, CHAT_TITLE_MAX_CHARS))
    }

    pub fn spawn_reply(
        &self,
        state: AppState,
        conversation: ChatConversation,
        prompt: String,
        should_auto_name: bool,
        deep_research: bool,
        active_chat: ActiveChatHandle,
    ) {
        let service = self.clone();
        tokio::spawn(async move {
            if should_auto_name {
                let naming_service = service.clone();
                let naming_state = state.clone();
                let naming_conversation_id = conversation.id.clone();
                let naming_prompt = prompt.clone();
                tokio::spawn(async move {
                    naming_service
                        .generate_and_store_title(
                            naming_state,
                            naming_conversation_id,
                            naming_prompt,
                        )
                        .await;
                });
            }

            service
                .run_reply(state, conversation, prompt, deep_research, active_chat)
                .await;
        });
    }

    async fn run_reply(
        &self,
        state: AppState,
        conversation: ChatConversation,
        prompt: String,
        deep_research: bool,
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
                .generate_reply(&state, &conversation, &prompt, deep_research, &active_chat)
                .await;

            match reply_result {
                Ok(message) => {
                    if let Err(error) =
                        persist_assistant_message(&state, &conversation_id, &message).await
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
                        );
                        let _ = persist_assistant_message(&state, &conversation_id, &message).await;
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
                    );
                    let _ = persist_assistant_message(&state, &conversation_id, &message).await;
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
        active_chat: &ActiveChatHandle,
    ) -> Result<ChatMessage, String> {
        let mut plan = self
            .plan_retrieval(
                conversation,
                &conversation.id,
                prompt,
                deep_research,
                active_chat,
            )
            .await;

        if plan.skip_retrieval && !conversation_has_prior_assistant(conversation) {
            tracing::warn!(
                conversation_id = %conversation.id,
                "planner requested conversation-only turn without prior assistant context; running retrieval"
            );
            plan.skip_retrieval = false;
        }

        if plan.skip_retrieval {
            active_chat
                .emit(ChatStreamEvent::Status {
                    status: ChatStatusPayload::new("generating", "Answering from the conversation")
                        .with_detail("No new library search for this turn."),
                })
                .await;
            let grounding = build_conversation_only_grounding();
            let mut cancel_rx = active_chat.subscribe_cancel();
            let content = self
                .stream_ollama_reply(conversation, grounding, active_chat, &mut cancel_rx, true)
                .await?;
            return Ok(self.build_assistant_message(
                content,
                Vec::new(),
                ChatMessageStatus::Completed,
            ));
        }

        let retrieval_started = Instant::now();
        let retrieval = self
            .retrieve_sources_with_plan(state, &conversation.id, prompt, plan, active_chat)
            .await?;
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
        let content = self
            .stream_ollama_reply(
                conversation,
                grounding_context,
                active_chat,
                &mut cancel_rx,
                false,
            )
            .await?;
        tracing::info!(
            conversation_id = %conversation.id,
            response_chars = content.chars().count(),
            response_elapsed_ms = reply_started.elapsed().as_millis() as u64,
            "chat response generated"
        );

        Ok(self.build_assistant_message(content, sources, ChatMessageStatus::Completed))
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
            let mut pool = HashMap::<String, AccumulatedSearchCandidate>::new();

            let pass_one_queries = plan.queries_for_pass(1);
            let pass_one = self
                .run_retrieval_pass(
                    state,
                    &mut pool,
                    RetrievalPassRequest {
                        conversation_id,
                        plan: &plan,
                        pass: 1,
                        queries: &pass_one_queries,
                        channel_focus_ids: &[],
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
                let pass_two = self
                    .run_retrieval_pass(
                        state,
                        &mut pool,
                        RetrievalPassRequest {
                            conversation_id,
                            plan: &plan,
                            pass: 2,
                            queries: &pass_two_queries,
                            channel_focus_ids: &assessment.channel_focus_ids,
                            active_chat,
                        },
                    )
                    .await?;
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
                let pass_three = self
                    .run_retrieval_pass(
                        state,
                        &mut pool,
                        RetrievalPassRequest {
                            conversation_id,
                            plan: &plan,
                            pass: 3,
                            queries: &pass_three_queries,
                            channel_focus_ids: &assessment.channel_focus_ids,
                            active_chat,
                        },
                    )
                    .await?;
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
        conversation: &ChatConversation,
        conversation_id: &str,
        prompt: &str,
        deep_research: bool,
        active_chat: &ActiveChatHandle,
    ) -> ChatRetrievalPlan {
        let span = logfire::span!(
            "chat.plan",
            conversation.id = conversation_id.to_string(),
            query.chars = prompt.chars().count(),
            multi_pass_enabled = self.multi_pass_enabled,
        );

        async move {
            let prompt = prompt.trim();
            let planner_input = format_conversation_for_planner(conversation, prompt);

            active_chat
                .emit(ChatStreamEvent::Status {
                    status: ChatStatusPayload::new("classifying", "Planning search")
                        .with_detail(
                            "Deciding whether this needs a focused lookup, broader evidence, or only prior context.",
                        ),
                })
                .await;

            let planned = timeout(
                CHAT_CLASSIFY_TIMEOUT,
                self.core.prompt_with_fallback(
                    "chat_query_plan",
                    CHAT_QUERY_PLAN_PROMPT,
                    &planner_input,
                    crate::services::ollama::CooldownStatusPolicy::UseLocalFallback,
                ),
            )
            .await;

            let mut plan = match planned {
                Ok(Ok((response, _))) => {
                    match parse_json_response::<ChatQueryPlanResponse>(&response) {
                        Ok(payload) => ChatRetrievalPlan::from_response(prompt, payload),
                        Err(error) => ChatRetrievalPlan::fallback(
                            prompt,
                            Some(format!(
                                "Planner returned unreadable JSON; falling back to synthesis ({error})."
                            )),
                        ),
                    }
                }
                Ok(Err(error)) => ChatRetrievalPlan::fallback(
                    prompt,
                    Some(format!(
                        "Planner unavailable; falling back to synthesis ({error:?})."
                    )),
                ),
                Err(_) => ChatRetrievalPlan::fallback(
                    prompt,
                    Some("Planner timed out; falling back to synthesis.".to_string()),
                ),
            };

            if !self.multi_pass_enabled && !plan.skip_retrieval {
                let rationale = plan.rationale.clone().or(Some(
                    "Adaptive multi-pass retrieval is disabled; using a single direct search."
                        .to_string(),
                ));
                plan = ChatRetrievalPlan::fallback(prompt, rationale);
            }

            if deep_research {
                plan.apply_deep_research(prompt);
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

            plan
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
            active_chat,
        } = request;
        let span = logfire::span!(
            "chat.retrieve.pass",
            conversation.id = conversation_id.to_string(),
            retrieval.pass = pass,
            query_count = queries.len(),
            channel_focus_count = channel_focus_ids.len(),
            plan.label = plan.label.clone(),
        );

        async move {
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
                .collect_retrieval_candidates(state, queries, candidate_limit, channel_focus_ids)
                .await?;

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
        state: &AppState,
        queries: &[String],
        candidate_limit: usize,
        channel_focus_ids: &[String],
    ) -> Result<(Vec<Vec<SearchCandidate>>, Vec<Vec<SearchCandidate>>), String> {
        let conn = state.db.connect();
        let mut keyword_batches = Vec::new();
        let filters = if channel_focus_ids.is_empty() {
            vec![None]
        } else {
            channel_focus_ids
                .iter()
                .map(|value| Some(value.as_str()))
                .collect()
        };

        for query in queries {
            for channel_filter in &filters {
                keyword_batches.push(
                    db::search_fts_candidates(
                        &conn,
                        query,
                        None,
                        None,
                        *channel_filter,
                        candidate_limit,
                    )
                    .await
                    .map_err(|error| error.to_string())?,
                );
            }
        }

        let semantic_batches = match state.search.model() {
            Some(model) if state.search.semantic_enabled() => {
                let embeddings = match state.search.embed_texts(queries).await {
                    Ok(embeddings) => embeddings,
                    Err(error) => {
                        tracing::warn!(error = %error, "chat semantic retrieval failed");
                        Vec::new()
                    }
                };
                let mut semantic_batches = Vec::new();
                for embedding in &embeddings {
                    let query_embedding = crate::services::search::vector_to_json(embedding);
                    for channel_filter in &filters {
                        semantic_batches.push(
                            db::search_vector_candidates(
                                &conn,
                                &query_embedding,
                                model,
                                None,
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

        Ok((keyword_batches, semantic_batches))
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
                match self
                    .generate_video_observation(conversation_id, prompt, &input)
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
    ) -> Result<String, String> {
        let span = logfire::span!(
            "chat.synthesize.observation",
            conversation.id = conversation_id.to_string(),
            video.id = input.video_id.clone(),
            excerpt_count = input.excerpts.len(),
        );

        async move {
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
            let (response, model_used) = self
                .core
                .prompt_with_fallback(
                    "chat_video_observation",
                    CHAT_VIDEO_OBSERVATION_PROMPT,
                    &prompt,
                    crate::services::ollama::CooldownStatusPolicy::UseLocalFallback,
                )
                .await
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
    ) -> Result<String, String> {
        // Cloud LLMs can take many minutes to stream a full response; override
        // the 20s default client timeout with one that covers the whole generation.
        const STREAM_TIMEOUT: Duration = Duration::from_secs(30 * 60);
        const MAX_ATTEMPTS: usize = 3;

        let span = logfire::span!(
            "chat.generate",
            conversation.id = conversation.id.clone(),
            model = self.model().to_string(),
            history_count = conversation.messages.len().min(CHAT_HISTORY_LIMIT),
            grounding_chars = grounding_context.chars().count(),
        );

        async move {
            let messages = build_ollama_messages(conversation, grounding_context, conversation_only);
            let request = OllamaChatRequest {
                model: self.model().to_string(),
                messages,
                stream: true,
            };

            let _permit = self
                .core
                .acquire_local_permit(self.model())
                .await
                .map_err(|error| error.to_string())?;

            let mut last_error = String::new();

            'retry: for attempt in 1..=MAX_ATTEMPTS {
                if attempt > 1 {
                    tracing::warn!(
                        conversation_id = %conversation.id,
                        model = %self.model(),
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
                                    tracing::info!(
                                        conversation_id = %conversation.id,
                                        model = %self.model(),
                                        response_chars = content.chars().count(),
                                        token_event_count,
                                        "chat streaming response complete"
                                    );
                                    return Ok(content);
                                }
                            }
                        }
                    }
                }

                if !pending.trim().is_empty() {
                    let payload = serde_json::from_str::<OllamaChatResponse>(pending.trim())
                        .map_err(|error| format!("Failed to parse Ollama chat stream tail: {error}"))?;
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
                }

                let content = content.trim().to_string();
                tracing::info!(
                    conversation_id = %conversation.id,
                    model = %self.model(),
                    response_chars = content.chars().count(),
                    token_event_count,
                    "chat streaming response complete"
                );
                return Ok(content);
            }

            Err(last_error)
        }
        .instrument(span)
        .await
    }

    async fn generate_and_store_title(
        &self,
        state: AppState,
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
                    let _ = finalize_title_generation(&state, &conversation_id, None).await;
                    return;
                }
            };

            if let Err(error) =
                finalize_title_generation(&state, &conversation_id, Some(generated_title.clone())).await
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
    conversation_id: &str,
    title: Option<String>,
) -> Result<(), String> {
    let _lock = state.chat_store_lock.lock().await;
    let conn = state.db.connect();
    let Some(mut conversation) = db::get_conversation(&conn, conversation_id)
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
    db::upsert_conversation(&conn, &conversation)
        .await
        .map_err(|error| error.to_string())
}

async fn persist_assistant_message(
    state: &AppState,
    conversation_id: &str,
    message: &ChatMessage,
) -> Result<(), String> {
    let _lock = state.chat_store_lock.lock().await;
    let conn = state.db.connect();
    let Some(mut conversation) = db::get_conversation(&conn, conversation_id)
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
    db::upsert_conversation(&conn, &conversation)
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

fn conversation_has_prior_assistant(conversation: &ChatConversation) -> bool {
    conversation
        .messages
        .iter()
        .any(|message| message.role == ChatRole::Assistant)
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

#[cfg(test)]
mod tests {
    use super::super::chat_heuristics::{
        collect_focus_terms, is_attributed_preference_query, recommendation_query_variants,
    };
    use super::{CHAT_CLASSIFY_TIMEOUT, ChatQueryIntent, ChatQueryPlanResponse, ChatRetrievalPlan};
    use crate::services::ollama::CLOUD_PROMPT_TIMEOUT_SECS;

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
}
