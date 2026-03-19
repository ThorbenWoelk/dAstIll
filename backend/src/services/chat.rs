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
use crate::services::search::{SEARCH_RRF_K, SearchCandidate, truncate_chunk_for_display};
use crate::services::text::limit_text;
use crate::state::AppState;

const CHAT_SOURCE_LIMIT: usize = 6;
const CHAT_SYNTHESIS_SOURCE_LIMIT: usize = 12;
const CHAT_RECOMMENDATION_SOURCE_LIMIT: usize = 14;
const CHAT_PATTERN_SOURCE_LIMIT: usize = 24;
const CHAT_COMPARISON_SOURCE_LIMIT: usize = 20;
const CHAT_HISTORY_LIMIT: usize = 12;
const CHAT_CONTEXT_MAX_CHARS: usize = 1_400;
const CHAT_TITLE_MAX_CHARS: usize = 80;
const CHAT_LOG_PREVIEW_MAX_CHARS: usize = 120;
const CHAT_CLASSIFY_TIMEOUT: Duration = Duration::from_millis(3_000);
const CHAT_MAX_RETRIEVAL_PASSES: usize = 2;
const CHAT_DIVERSITY_PENALTY: f32 = 0.3;
const CHAT_SOURCE_KIND_DIVERSITY_BONUS: f32 = 1.08;
const CHAT_QUERY_LIMIT_PER_PASS: usize = 3;
const CHAT_QUERY_LIMIT_TOTAL: usize = 5;
const CHAT_RETRIEVAL_CANDIDATE_LIMIT_MIN: usize = 8;
const CHAT_RETRIEVAL_CANDIDATE_LIMIT_MAX: usize = 24;
const CHAT_SYNTHESIS_VIDEO_LIMIT: usize = 6;
const CHAT_SYNTHESIS_SOURCES_PER_VIDEO: usize = 3;
const CHAT_SYNTHESIS_CONTEXT_MAX_CHARS: usize = 1_200;
const CHAT_SYNTHESIS_RAW_SOURCE_LIMIT: usize = 8;

static NEXT_CHAT_ID: AtomicU64 = AtomicU64::new(1);

const CHAT_SYSTEM_PROMPT: &str = "You are the dAstIll assistant. Answer only from the provided ground-truth excerpts and the visible conversation history. If the excerpts are missing, incomplete, or not directly relevant, say that you cannot answer from the current library. Do not use outside knowledge. Do not invent facts, citations, or timestamps. Be concise but useful. When helpful, mention the relevant video title or channel name from the provided excerpts.";

const CHAT_QUERY_PLAN_PROMPT: &str = "Classify the user's grounded library question for retrieval. Return valid JSON only with this shape: {\"intent\":\"fact|synthesis|pattern|comparison\",\"rationale\":\"short explanation\",\"sub_queries\":[\"...\"],\"expansion_queries\":[\"...\"]}. Use the user's wording where possible. Keep each query short. fact: 1 direct query, no expansion. synthesis: 1-2 queries, optional expansion. pattern/comparison: 2-3 initial queries plus 1-2 expansion queries for broader coverage. No markdown or code fences.";

const CHAT_VIDEO_OBSERVATION_PROMPT: &str = "You are distilling grounded evidence for a later answer. Use only the supplied excerpts. Return exactly two concise bullet points describing observations relevant to the user's question. If the excerpts are weak, say that the evidence from this video is limited.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ChatQueryIntent {
    Fact,
    Synthesis,
    Pattern,
    Comparison,
}

impl ChatQueryIntent {
    fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "fact" => Some(Self::Fact),
            "synthesis" => Some(Self::Synthesis),
            "pattern" => Some(Self::Pattern),
            "comparison" => Some(Self::Comparison),
            _ => None,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Fact => "fact lookup",
            Self::Synthesis => "targeted synthesis",
            Self::Pattern => "broad pattern analysis",
            Self::Comparison => "comparison",
        }
    }

    fn needs_synthesis_stage(&self) -> bool {
        matches!(self, Self::Pattern | Self::Comparison)
    }
}

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
struct ChatRetrievalPlan {
    intent: ChatQueryIntent,
    label: String,
    budget: usize,
    max_per_video: usize,
    queries: Vec<String>,
    expansion_queries: Vec<String>,
    focus_terms: Vec<String>,
    attributed_preference: bool,
    rationale: Option<String>,
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
        }
    }

    fn queries_for_pass(&self, pass: usize) -> Vec<String> {
        match pass {
            1 => self.queries.clone(),
            2 => {
                let mut queries = self.expansion_queries.clone();
                if queries.is_empty() {
                    queries = heuristic_expansion_queries(self);
                }
                queries.truncate(CHAT_QUERY_LIMIT_PER_PASS);
                queries
            }
            _ => Vec::new(),
        }
    }

    fn supports_second_pass(&self) -> bool {
        !self.queries_for_pass(2).is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct ChatQueryPlanResponse {
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

#[derive(Debug, Clone)]
struct RetrievedChatSource {
    source: ChatSource,
    context_text: String,
}

#[derive(Debug, Clone)]
struct AccumulatedSearchCandidate {
    candidate: SearchCandidate,
    keyword_score: f32,
    semantic_score: f32,
    retrieval_pass: usize,
}

impl AccumulatedSearchCandidate {
    fn combined_score(&self) -> f32 {
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

#[derive(Debug, Clone)]
struct CoverageAssessment {
    needs_more: bool,
    reason: Option<String>,
    channel_focus_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct ChatRetrievalOutcome {
    plan: ChatRetrievalPlan,
    sources: Vec<RetrievedChatSource>,
}

#[derive(Debug, Clone)]
struct VideoObservation {
    video_title: String,
    channel_name: String,
    summary: String,
}

#[derive(Debug, Clone)]
struct VideoObservationInput {
    video_id: String,
    video_title: String,
    channel_name: String,
    excerpts: Vec<RetrievedChatSource>,
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
struct OllamaRequestMessage {
    role: String,
    content: String,
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
                .run_reply(state, conversation, prompt, active_chat)
                .await;
        });
    }

    async fn run_reply(
        &self,
        state: AppState,
        conversation: ChatConversation,
        prompt: String,
        active_chat: ActiveChatHandle,
    ) {
        let conversation_id = conversation.id.clone();
        let span = logfire::span!(
            "chat.reply",
            conversation.id = conversation_id.clone(),
            query.chars = prompt.chars().count(),
            query.preview = limit_text(prompt.trim(), CHAT_LOG_PREVIEW_MAX_CHARS),
        );

        async move {
            let reply_result = self
                .generate_reply(&state, &conversation, &prompt, &active_chat)
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
        active_chat: &ActiveChatHandle,
    ) -> Result<ChatMessage, String> {
        let retrieval_started = Instant::now();
        let retrieval = self
            .retrieve_sources_adaptive(state, &conversation.id, prompt, active_chat)
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

        let grounding_context = self
            .build_answer_grounding_context(
                &conversation.id,
                prompt,
                &retrieval.plan,
                &retrieved_sources,
                active_chat,
            )
            .await?;
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
            .stream_ollama_reply(conversation, grounding_context, active_chat, &mut cancel_rx)
            .await?;
        tracing::info!(
            conversation_id = %conversation.id,
            response_chars = content.chars().count(),
            response_elapsed_ms = reply_started.elapsed().as_millis() as u64,
            "chat response generated"
        );

        Ok(self.build_assistant_message(content, sources, ChatMessageStatus::Completed))
    }

    async fn retrieve_sources_adaptive(
        &self,
        state: &AppState,
        conversation_id: &str,
        prompt: &str,
        active_chat: &ActiveChatHandle,
    ) -> Result<ChatRetrievalOutcome, String> {
        let span = logfire::span!(
            "chat.retrieve",
            conversation.id = conversation_id.to_string(),
            query.chars = prompt.chars().count(),
            query.preview = limit_text(prompt.trim(), CHAT_LOG_PREVIEW_MAX_CHARS),
        );

        async move {
            let plan = self
                .plan_retrieval(conversation_id, prompt, active_chat)
                .await;
            let mut pool = HashMap::<String, AccumulatedSearchCandidate>::new();

            let pass_one_queries = plan.queries_for_pass(1);
            let pass_one = self
                .run_retrieval_pass(
                    state,
                    conversation_id,
                    &plan,
                    &mut pool,
                    1,
                    &pass_one_queries,
                    &[],
                    active_chat,
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
                        conversation_id,
                        &plan,
                        &mut pool,
                        2,
                        &pass_two_queries,
                        &assessment.channel_focus_ids,
                        active_chat,
                    )
                    .await?;
                sources = pass_two.sources;
                assessment = pass_two.assessment;
                pass_count = 2;
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
        conversation_id: &str,
        prompt: &str,
        active_chat: &ActiveChatHandle,
    ) -> ChatRetrievalPlan {
        let span = logfire::span!(
            "chat.plan",
            conversation.id = conversation_id.to_string(),
            query.chars = prompt.chars().count(),
            query.preview = limit_text(prompt.trim(), CHAT_LOG_PREVIEW_MAX_CHARS),
            multi_pass_enabled = self.multi_pass_enabled,
        );

        async move {
            let prompt = prompt.trim();
            if !self.multi_pass_enabled {
                let plan = ChatRetrievalPlan::fallback(
                    prompt,
                    Some("Adaptive retrieval disabled; using direct synthesis.".to_string()),
                );
                tracing::info!(
                    conversation_id = conversation_id,
                    intent = %plan.intent.label(),
                    plan_label = %plan.label,
                    budget = plan.budget,
                    query_count = plan.queries.len(),
                    expansion_query_count = plan.expansion_queries.len(),
                    "chat retrieval planning bypassed"
                );
                active_chat
                    .emit(ChatStreamEvent::Status {
                        status: ChatStatusPayload::new("classifying", "Using direct search")
                            .with_detail("Adaptive planning is disabled for this runtime.")
                            .with_plan(plan.visibility()),
                    })
                    .await;
                return plan;
            }

            active_chat
                .emit(ChatStreamEvent::Status {
                    status: ChatStatusPayload::new("classifying", "Planning search")
                        .with_detail(
                            "Deciding whether this needs a focused lookup or broader evidence gathering.",
                        ),
                })
                .await;

            let planned = timeout(
                CHAT_CLASSIFY_TIMEOUT,
                self.core.prompt_with_fallback(
                    "chat_query_plan",
                    CHAT_QUERY_PLAN_PROMPT,
                    prompt,
                    crate::services::ollama::CooldownStatusPolicy::UseLocalFallback,
                ),
            )
            .await;

            let plan = match planned {
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

            tracing::info!(
                conversation_id = conversation_id,
                intent = %plan.intent.label(),
                plan_label = %plan.label,
                budget = plan.budget,
                max_per_video = plan.max_per_video,
                query_count = plan.queries.len(),
                expansion_query_count = plan.expansion_queries.len(),
                attributed_preference = plan.attributed_preference,
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
        conversation_id: &str,
        plan: &ChatRetrievalPlan,
        pool: &mut HashMap<String, AccumulatedSearchCandidate>,
        pass: usize,
        queries: &[String],
        channel_focus_ids: &[String],
        active_chat: &ActiveChatHandle,
    ) -> Result<RetrievalPassOutcome, String> {
        let span = logfire::span!(
            "chat.retrieve.pass",
            conversation.id = conversation_id.to_string(),
            retrieval.pass = pass,
            query_count = queries.len(),
            queries.preview = limit_text(&queries.join(" · "), CHAT_LOG_PREVIEW_MAX_CHARS),
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

            let observation_inputs = build_video_observation_inputs(sources);
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
            question.preview = limit_text(prompt.trim(), CHAT_LOG_PREVIEW_MAX_CHARS),
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
            let messages = build_ollama_messages(conversation, grounding_context);
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
            prompt.preview = limit_text(prompt.trim(), CHAT_LOG_PREVIEW_MAX_CHARS),
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
                title = %generated_title,
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

fn build_ollama_messages(
    conversation: &ChatConversation,
    grounding_context: String,
) -> Vec<OllamaRequestMessage> {
    let mut messages = vec![
        OllamaRequestMessage {
            role: "system".to_string(),
            content: CHAT_SYSTEM_PROMPT.to_string(),
        },
        OllamaRequestMessage {
            role: "system".to_string(),
            content: grounding_context,
        },
    ];

    let history = conversation
        .messages
        .iter()
        .rev()
        .take(CHAT_HISTORY_LIMIT)
        .cloned()
        .collect::<Vec<_>>();

    for message in history.into_iter().rev() {
        messages.push(OllamaRequestMessage {
            role: match message.role {
                ChatRole::System => "system",
                ChatRole::User => "user",
                ChatRole::Assistant => "assistant",
            }
            .to_string(),
            content: message.content,
        });
    }

    messages
}

fn build_grounding_context(retrieved_sources: &[RetrievedChatSource]) -> String {
    let mut context = String::from("Ground-truth excerpts for the next answer only:\n\n");
    for (index, source) in retrieved_sources.iter().enumerate() {
        let source_number = index + 1;
        context.push_str(&format!(
            "[Source {source_number}] Video: {}\nChannel: {}\nType: {}\n",
            source.source.video_title,
            source.source.channel_name,
            source.source.source_kind.as_str(),
        ));
        if let Some(section_title) = &source.source.section_title {
            context.push_str(&format!("Section: {section_title}\n"));
        }
        context.push_str(&format!("Excerpt:\n{}\n\n", source.context_text));
    }
    context.push_str("If these excerpts are not enough, explicitly say so.");
    context
}

fn build_synthesis_grounding_context(
    prompt: &str,
    plan: &ChatRetrievalPlan,
    retrieved_sources: &[RetrievedChatSource],
    observations: &[VideoObservation],
) -> String {
    let mut context = format!(
        "Question type: {}\nRetrieval budget: {} excerpts (max {} per video)\nOriginal question: {}\n\n",
        plan.intent.label(),
        plan.budget,
        plan.max_per_video,
        prompt.trim(),
    );
    context.push_str(
        "Intermediate synthesis notes derived only from the raw excerpts below. Treat the raw excerpts as the source of truth.\n\n",
    );

    for (index, observation) in observations.iter().enumerate() {
        let number = index + 1;
        context.push_str(&format!(
            "[Video note {number}] Video: {}\nChannel: {}\n{}\n\n",
            observation.video_title,
            observation.channel_name,
            observation.summary.trim(),
        ));
    }

    context.push_str("Supporting raw excerpts:\n\n");
    for (index, source) in retrieved_sources
        .iter()
        .take(CHAT_SYNTHESIS_RAW_SOURCE_LIMIT)
        .enumerate()
    {
        let source_number = index + 1;
        context.push_str(&format!(
            "[Source {source_number}] Video: {}\nChannel: {}\nType: {}\n",
            source.source.video_title,
            source.source.channel_name,
            source.source.source_kind.as_str(),
        ));
        if let Some(section_title) = &source.source.section_title {
            context.push_str(&format!("Section: {section_title}\n"));
        }
        context.push_str(&format!("Excerpt:\n{}\n\n", source.context_text));
    }

    context.push_str(
        "If the notes and excerpts do not fully support an answer, explain the limitation explicitly.",
    );
    context
}

fn rank_chat_sources(
    candidates: impl IntoIterator<Item = impl std::borrow::Borrow<AccumulatedSearchCandidate>>,
    plan: &ChatRetrievalPlan,
) -> Vec<RetrievedChatSource> {
    let mut remaining = candidates
        .into_iter()
        .map(|candidate| candidate.borrow().clone())
        .filter(|candidate| candidate.combined_score() > 0.0)
        .collect::<Vec<_>>();
    let mut selected = Vec::new();
    let mut video_counts = HashMap::<String, usize>::new();
    let mut kind_counts = HashMap::<crate::services::search::SearchSourceKind, usize>::new();

    while selected.len() < plan.budget && !remaining.is_empty() {
        let best_index = remaining
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| {
                selection_score(left, &video_counts, &kind_counts, plan)
                    .total_cmp(&selection_score(right, &video_counts, &kind_counts, plan))
            })
            .map(|(index, _)| index)
            .expect("remaining candidates should not be empty");
        let candidate = remaining.swap_remove(best_index);
        let score = selection_score(&candidate, &video_counts, &kind_counts, plan);
        *video_counts
            .entry(candidate.candidate.video_id.clone())
            .or_insert(0) += 1;
        *kind_counts
            .entry(candidate.candidate.source_kind)
            .or_insert(0) += 1;
        selected.push(RetrievedChatSource {
            source: ChatSource {
                video_id: candidate.candidate.video_id.clone(),
                channel_id: candidate.candidate.channel_id.clone(),
                channel_name: candidate.candidate.channel_name.clone(),
                video_title: candidate.candidate.video_title.clone(),
                source_kind: candidate.candidate.source_kind,
                section_title: candidate.candidate.section_title.clone(),
                snippet: truncate_chunk_for_display(&candidate.candidate.chunk_text),
                score,
                retrieval_pass: Some(candidate.retrieval_pass),
            },
            context_text: limit_text(
                candidate.candidate.chunk_text.trim(),
                CHAT_CONTEXT_MAX_CHARS,
            ),
        });
    }

    selected
}

fn selection_score(
    candidate: &AccumulatedSearchCandidate,
    video_counts: &HashMap<String, usize>,
    kind_counts: &HashMap<crate::services::search::SearchSourceKind, usize>,
    plan: &ChatRetrievalPlan,
) -> f32 {
    let mut score = candidate.combined_score();
    let video_count = video_counts
        .get(&candidate.candidate.video_id)
        .copied()
        .unwrap_or(0);
    if video_count >= plan.max_per_video {
        score *= CHAT_DIVERSITY_PENALTY.powi((video_count + 1 - plan.max_per_video) as i32);
    }

    let transcript_count = kind_counts
        .get(&crate::services::search::SearchSourceKind::Transcript)
        .copied()
        .unwrap_or(0);
    let summary_count = kind_counts
        .get(&crate::services::search::SearchSourceKind::Summary)
        .copied()
        .unwrap_or(0);
    if transcript_count > 0
        && summary_count == 0
        && candidate.candidate.source_kind == crate::services::search::SearchSourceKind::Summary
    {
        score *= CHAT_SOURCE_KIND_DIVERSITY_BONUS;
    } else if summary_count > 0
        && transcript_count == 0
        && candidate.candidate.source_kind == crate::services::search::SearchSourceKind::Transcript
    {
        score *= CHAT_SOURCE_KIND_DIVERSITY_BONUS;
    }

    if plan.attributed_preference {
        let preference_score =
            preference_signal_score(&candidate.candidate.chunk_text, &plan.focus_terms);
        if preference_score > 0.0 {
            score *= 1.0 + preference_score;
        }
    }

    score
}

fn retrieval_candidate_limit(budget: usize, query_count: usize, pass: usize) -> usize {
    let query_count = query_count.max(1);
    let base = ((budget * 2) / query_count).max(CHAT_RETRIEVAL_CANDIDATE_LIMIT_MIN);
    let boosted = if pass > 1 { base + 4 } else { base };
    boosted.clamp(
        CHAT_RETRIEVAL_CANDIDATE_LIMIT_MIN,
        CHAT_RETRIEVAL_CANDIDATE_LIMIT_MAX,
    )
}

fn accumulate_ranked_candidates(
    pool: &mut HashMap<String, AccumulatedSearchCandidate>,
    candidates: &[SearchCandidate],
    semantic: bool,
    pass: usize,
) {
    for (index, candidate) in candidates.iter().enumerate() {
        let rank = index + 1;
        let score = 1.0 / (SEARCH_RRF_K + rank as f32);
        let entry =
            pool.entry(candidate.chunk_id.clone())
                .or_insert_with(|| AccumulatedSearchCandidate {
                    candidate: candidate.clone(),
                    keyword_score: 0.0,
                    semantic_score: 0.0,
                    retrieval_pass: pass,
                });
        if semantic {
            entry.semantic_score += score;
        } else {
            entry.keyword_score += score;
        }
        entry.retrieval_pass = entry.retrieval_pass.min(pass);
        entry.candidate = candidate.clone();
    }
}

fn assess_coverage(
    plan: &ChatRetrievalPlan,
    sources: &[RetrievedChatSource],
) -> CoverageAssessment {
    if sources.is_empty() {
        return CoverageAssessment {
            needs_more: false,
            reason: Some("No grounded excerpts were found.".to_string()),
            channel_focus_ids: Vec::new(),
        };
    }

    let unique_video_count = count_unique_videos(sources);
    let mut video_counts = HashMap::<String, usize>::new();
    let mut channel_counts = HashMap::<String, usize>::new();
    for source in sources {
        *video_counts
            .entry(source.source.video_id.clone())
            .or_insert(0) += 1;
        *channel_counts
            .entry(source.source.channel_id.clone())
            .or_insert(0) += 1;
    }
    let dominant_video_count = video_counts.values().copied().max().unwrap_or(0);
    let unique_channel_count = channel_counts.len();

    if plan.attributed_preference {
        let direct_evidence_count = count_direct_preference_evidence(sources, &plan.focus_terms);
        let needs_more = plan.supports_second_pass()
            && (direct_evidence_count == 0 || (direct_evidence_count < 2 && sources.len() < 8));
        return CoverageAssessment {
            needs_more,
            reason: Some(if direct_evidence_count == 0 {
                if needs_more {
                    "The current excerpts mention the topic, but they do not yet contain enough direct recommendation-style language, so the search should broaden.".to_string()
                } else {
                    "The current excerpts mention the topic, but they still do not contain a direct recommendation or preference statement.".to_string()
                }
            } else if needs_more {
                format!(
                    "Found {direct_evidence_count} excerpts with direct preference language, but the evidence is still thin enough to justify a broader pass."
                )
            } else {
                format!(
                    "Found {direct_evidence_count} excerpts with direct preference language across {unique_video_count} videos."
                )
            }),
            channel_focus_ids: Vec::new(),
        };
    }

    match plan.intent {
        ChatQueryIntent::Fact => CoverageAssessment {
            needs_more: false,
            reason: Some("Fact lookup stayed focused on the strongest direct matches.".to_string()),
            channel_focus_ids: Vec::new(),
        },
        ChatQueryIntent::Synthesis => {
            let needs_more = sources.len() < 6 && plan.supports_second_pass();
            CoverageAssessment {
                needs_more,
                reason: Some(if needs_more {
                    format!(
                        "Pass 1 found only {} strong excerpts, so broader synthesis coverage is useful.",
                        sources.len()
                    )
                } else {
                    format!(
                        "Pass 1 gathered {} excerpts across {} videos.",
                        sources.len(),
                        unique_video_count
                    )
                }),
                channel_focus_ids: Vec::new(),
            }
        }
        ChatQueryIntent::Pattern => {
            let needs_more = plan.supports_second_pass()
                && (unique_video_count < 4 || dominant_video_count > plan.max_per_video + 1);
            CoverageAssessment {
                needs_more,
                reason: Some(if needs_more {
                    format!(
                        "Pass 1 covered {} videos with heavy concentration in one video, so a broader pass should reduce bias.",
                        unique_video_count
                    )
                } else {
                    format!(
                        "Coverage reached {} videos with a balanced spread for pattern analysis.",
                        unique_video_count
                    )
                }),
                channel_focus_ids: Vec::new(),
            }
        }
        ChatQueryIntent::Comparison => {
            let dominant_channel_count = channel_counts.values().copied().max().unwrap_or(0);
            let channel_focus_ids = if unique_channel_count > 1 {
                channel_counts
                    .iter()
                    .filter(|(_, count)| **count < dominant_channel_count)
                    .map(|(channel_id, _)| channel_id.clone())
                    .take(2)
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let needs_more = plan.supports_second_pass()
                && (unique_channel_count < 2
                    || dominant_channel_count
                        > (sources.len().saturating_sub(dominant_channel_count) + 2));
            CoverageAssessment {
                needs_more,
                reason: Some(if needs_more {
                    if unique_channel_count < 2 {
                        "Pass 1 did not surface enough distinct channels for a fair comparison."
                            .to_string()
                    } else {
                        "Pass 1 leaned too heavily toward one channel, so the next pass rebalances evidence.".to_string()
                    }
                } else {
                    format!(
                        "Comparison coverage spans {} channels with enough balance to synthesize.",
                        unique_channel_count
                    )
                }),
                channel_focus_ids,
            }
        }
    }
}

fn build_video_observation_inputs(sources: &[RetrievedChatSource]) -> Vec<VideoObservationInput> {
    let mut groups = Vec::<VideoObservationInput>::new();
    let mut group_indexes = HashMap::<String, usize>::new();

    for source in sources {
        if let Some(index) = group_indexes.get(&source.source.video_id).copied() {
            if groups[index].excerpts.len() < CHAT_SYNTHESIS_SOURCES_PER_VIDEO {
                groups[index].excerpts.push(source.clone());
            }
            continue;
        }

        if groups.len() >= CHAT_SYNTHESIS_VIDEO_LIMIT {
            continue;
        }

        group_indexes.insert(source.source.video_id.clone(), groups.len());
        groups.push(VideoObservationInput {
            video_id: source.source.video_id.clone(),
            video_title: source.source.video_title.clone(),
            channel_name: source.source.channel_name.clone(),
            excerpts: vec![source.clone()],
        });
    }

    groups
}

fn count_unique_videos(sources: &[RetrievedChatSource]) -> usize {
    sources
        .iter()
        .map(|source| source.source.video_id.as_str())
        .collect::<HashSet<_>>()
        .len()
}

fn count_direct_preference_evidence(
    sources: &[RetrievedChatSource],
    focus_terms: &[String],
) -> usize {
    sources
        .iter()
        .filter(|source| preference_signal_score(&source.context_text, focus_terms) >= 0.14)
        .count()
}

fn preference_signal_score(text: &str, focus_terms: &[String]) -> f32 {
    let normalized = normalize_for_matching(text);
    let preference_hits = preference_signal_terms()
        .iter()
        .filter(|term| normalized.contains(**term))
        .count();
    if preference_hits == 0 {
        return 0.0;
    }

    let focus_hits = focus_terms
        .iter()
        .filter(|term| normalized.contains(term.as_str()))
        .count();
    if !focus_terms.is_empty() && focus_hits == 0 {
        return 0.0;
    }

    0.12 + (preference_hits.min(2) as f32 * 0.07) + (focus_hits.min(2) as f32 * 0.04)
}

fn build_plan_label(intent: ChatQueryIntent, attributed_preference: bool) -> &'static str {
    if attributed_preference {
        "Recommendation lookup"
    } else {
        match intent {
            ChatQueryIntent::Fact => "Focused lookup",
            ChatQueryIntent::Synthesis => "Broader synthesis",
            ChatQueryIntent::Pattern => "Pattern scan",
            ChatQueryIntent::Comparison => "Comparison scan",
        }
    }
}

fn is_attributed_preference_query(prompt: &str) -> bool {
    let normalized = normalize_for_matching(prompt);
    let has_attribution = normalized.contains("according to ")
        || normalized.starts_with("what does ")
        || normalized.starts_with("what do ")
        || normalized.contains(" does ")
            && (normalized.contains(" think ")
                || normalized.contains(" use ")
                || normalized.contains(" prefer ")
                || normalized.contains(" recommend "));
    let has_preference = preference_signal_terms()
        .iter()
        .any(|term| normalized.contains(term));
    has_attribution && has_preference
}

fn recommendation_query_variants(prompt: &str) -> Vec<String> {
    let subject = extract_subject_phrase(prompt);
    let focus = collect_focus_terms(prompt).join(" ");
    let mut queries = Vec::new();

    if !subject.is_empty() && !focus.is_empty() {
        queries.push(format!("{subject} {focus} recommendation"));
        queries.push(format!("{subject} favorite {focus}"));
        queries.push(format!("{subject} preferred {focus}"));
        queries.push(format!("{subject} {focus} opinion"));
        queries.push(format!("{subject} {focus} use"));
    } else if !focus.is_empty() {
        queries.push(format!("{focus} recommendation"));
        queries.push(format!("best {focus}"));
        queries.push(format!("favorite {focus}"));
    }

    queries
}

fn extract_subject_phrase(prompt: &str) -> String {
    let tokens = tokenize_for_matching(prompt);

    if let Some(index) = tokens.iter().position(|token| token == "according")
        && tokens.get(index + 1).is_some_and(|token| token == "to")
    {
        let subject = tokens
            .iter()
            .skip(index + 2)
            .take_while(|token| !is_boundary_token(token))
            .take(4)
            .cloned()
            .collect::<Vec<_>>();
        if !subject.is_empty() {
            return subject.join(" ");
        }
    }

    if tokens.starts_with(&["what".to_string(), "does".to_string()]) {
        let subject = tokens
            .iter()
            .skip(2)
            .take_while(|token| {
                !matches!(
                    token.as_str(),
                    "think" | "recommend" | "prefer" | "use" | "say"
                )
            })
            .take(4)
            .cloned()
            .collect::<Vec<_>>();
        if !subject.is_empty() {
            return subject.join(" ");
        }
    }

    String::new()
}

fn collect_focus_terms(prompt: &str) -> Vec<String> {
    let subject_terms = tokenize_for_matching(&extract_subject_phrase(prompt));
    tokenize_for_matching(prompt)
        .into_iter()
        .filter(|token| token.len() > 2)
        .filter(|token| !is_query_stopword(token))
        .filter(|token| !subject_terms.contains(token))
        .take(4)
        .collect()
}

fn tokenize_for_matching(input: &str) -> Vec<String> {
    normalize_for_matching(input)
        .split_whitespace()
        .map(ToString::to_string)
        .collect()
}

fn normalize_for_matching(input: &str) -> String {
    input
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() || char.is_whitespace() {
                char.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect()
}

fn is_boundary_token(token: &str) -> bool {
    matches!(
        token,
        "about" | "for" | "on" | "in" | "with" | "and" | "or" | "vs" | "versus"
    )
}

fn is_query_stopword(token: &str) -> bool {
    matches!(
        token,
        "what"
            | "which"
            | "who"
            | "does"
            | "do"
            | "did"
            | "is"
            | "are"
            | "the"
            | "a"
            | "an"
            | "according"
            | "to"
            | "think"
            | "opinion"
            | "best"
            | "favorite"
            | "favourite"
            | "prefer"
            | "preferred"
            | "recommend"
            | "recommendation"
            | "use"
            | "uses"
            | "should"
            | "i"
            | "we"
            | "me"
    )
}

fn preference_signal_terms() -> &'static [&'static str] {
    &[
        " best ",
        " favorite ",
        " favourite ",
        " prefer ",
        " preferred ",
        " recommend ",
        " recommendation ",
        " would use ",
        " i use ",
        " i choose ",
        " go with ",
    ]
}

fn sanitize_queries(queries: Vec<String>) -> Vec<String> {
    let mut sanitized = Vec::new();
    for query in queries {
        push_unique_query(&mut sanitized, query);
        if sanitized.len() >= CHAT_QUERY_LIMIT_TOTAL {
            break;
        }
    }
    sanitized
}

fn push_unique_query(queries: &mut Vec<String>, query: impl Into<String>) {
    let query = query.into();
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return;
    }
    if queries
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(trimmed))
    {
        return;
    }
    queries.push(trimmed.to_string());
}

fn heuristic_query_variants(prompt: &str, intent: ChatQueryIntent) -> Vec<String> {
    let prompt = prompt.trim();
    match intent {
        ChatQueryIntent::Fact => Vec::new(),
        ChatQueryIntent::Synthesis => vec![format!("{prompt} overview")],
        ChatQueryIntent::Pattern => vec![
            format!("{prompt} speaking style"),
            format!("{prompt} rhetoric examples"),
            format!("{prompt} tone and phrasing"),
        ],
        ChatQueryIntent::Comparison => vec![
            format!("{prompt} differences"),
            format!("{prompt} similarities"),
            format!("{prompt} contrasting viewpoints"),
        ],
    }
}

fn heuristic_expansion_queries(plan: &ChatRetrievalPlan) -> Vec<String> {
    let base_queries = if plan.attributed_preference {
        recommendation_query_variants(plan.queries.first().map(String::as_str).unwrap_or_default())
    } else {
        heuristic_query_variants(
            plan.queries.first().map(String::as_str).unwrap_or_default(),
            plan.intent,
        )
    };

    base_queries
        .into_iter()
        .filter(|query| {
            !plan
                .queries
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(query))
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::{
        ChatQueryIntent, ChatQueryPlanResponse, ChatRetrievalPlan, collect_focus_terms,
        is_attributed_preference_query, recommendation_query_variants,
    };

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
}
