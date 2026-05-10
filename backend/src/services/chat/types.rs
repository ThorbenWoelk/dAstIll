use super::*;

#[derive(Debug, Clone)]
pub(super) struct ToolEvidenceRecord {
    pub(super) summary: String,
    pub(super) output: String,
}

#[derive(Debug, Clone)]
pub(super) struct ToolLoopOutcome {
    pub(super) conversation_only: bool,
    pub(super) rationale: Option<String>,
    pub(super) tool_outputs: Vec<ToolEvidenceRecord>,
    pub(super) sources: Vec<RetrievedChatSource>,
}

#[derive(Debug, Clone)]
pub(super) struct SearchLibraryExecutionResult {
    pub(super) summary: String,
    pub(super) output: String,
    pub(super) sources: Vec<RetrievedChatSource>,
}

impl ChatToolLoopResponse {
    pub(super) fn into_step_outcome(self) -> Result<ToolLoopStepOutcome, String> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CancelledChatOutcome {
    pub(super) status: ChatMessageStatus,
    pub(super) message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ChatCancellationState {
    pub(super) cancelled: bool,
    pub(super) outcome: Option<CancelledChatOutcome>,
}

#[derive(Debug)]
struct ActiveChatState {
    next_sequence: AtomicU64,
    cancel_tx: watch::Sender<ChatCancellationState>,
    events_tx: broadcast::Sender<SequencedChatEvent>,
    buffered_events: Mutex<Vec<SequencedChatEvent>>,
}

#[derive(Debug, Clone)]
pub struct ActiveChatHandle {
    inner: Arc<ActiveChatState>,
}

impl ActiveChatHandle {
    pub fn new() -> Self {
        let (cancel_tx, _) = watch::channel(ChatCancellationState::default());
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
        self.request_cancellation(None);
    }

    pub fn reject(&self, status: ChatMessageStatus, message: impl Into<String>) {
        self.request_cancellation(Some(CancelledChatOutcome {
            status,
            message: message.into(),
        }));
    }

    pub(super) fn is_cancelled(&self) -> bool {
        self.inner.cancel_tx.borrow().cancelled
    }

    pub(super) fn ensure_not_cancelled(&self) -> Result<(), String> {
        if self.is_cancelled() {
            Err(cancelled_error())
        } else {
            Ok(())
        }
    }

    pub(super) fn subscribe_cancel(&self) -> watch::Receiver<ChatCancellationState> {
        self.inner.cancel_tx.subscribe()
    }

    pub(super) fn cancelled_outcome(&self) -> Option<(ChatMessageStatus, String)> {
        self.inner
            .cancel_tx
            .borrow()
            .outcome
            .as_ref()
            .map(|outcome| (outcome.status, outcome.message.clone()))
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

    fn request_cancellation(&self, outcome: Option<CancelledChatOutcome>) {
        if self.inner.cancel_tx.borrow().cancelled {
            return;
        }

        self.inner.cancel_tx.send_replace(ChatCancellationState {
            cancelled: true,
            outcome,
        });
    }
}

impl Default for ActiveChatHandle {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn cancelled_error() -> String {
    "cancelled".to_string()
}

pub(super) async fn await_or_cancel<T, F>(
    active_chat: &ActiveChatHandle,
    future: F,
) -> Result<T, String>
where
    F: Future<Output = T>,
{
    active_chat.ensure_not_cancelled()?;
    let mut cancel_rx = active_chat.subscribe_cancel();
    tokio::pin!(future);

    tokio::select! {
        changed = cancel_rx.changed() => {
            if changed.is_ok() && cancel_rx.borrow().cancelled {
                Err(cancelled_error())
            } else {
                Ok(future.await)
            }
        }
        result = &mut future => Ok(result),
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RetrievedChatSource {
    pub(crate) source: ChatSource,
    pub(crate) context_text: String,
}

#[derive(Debug, Clone)]
pub(crate) struct AccumulatedSearchCandidate {
    pub(crate) candidate: SearchCandidate,
    pub(crate) keyword_score: f32,
    pub(crate) semantic_score: f32,
    pub(crate) retrieval_pass: usize,
}

impl AccumulatedSearchCandidate {
    pub(crate) fn combined_score(&self) -> f32 {
        match (self.keyword_score > 0.0, self.semantic_score > 0.0) {
            (true, true) => self.keyword_score + self.semantic_score,
            (true, false) => self.keyword_score,
            (false, true) => self.semantic_score,
            (false, false) => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct RetrievalPassOutcome {
    pub(super) sources: Vec<RetrievedChatSource>,
    pub(super) assessment: CoverageAssessment,
}

#[derive(Clone, Copy)]
pub(super) struct RetrievalPassRequest<'a> {
    pub(super) conversation_id: &'a str,
    pub(super) plan: &'a ChatRetrievalPlan,
    pub(super) access_context: &'a crate::security::AccessContext,
    pub(super) pass: usize,
    pub(super) queries: &'a [String],
    pub(super) channel_focus_ids: &'a [String],
    pub(super) video_focus_ids: &'a [String],
    pub(super) active_chat: &'a ActiveChatHandle,
}

pub(super) struct ToolCallExecutionRequest<'a> {
    pub(super) state: &'a AppState,
    pub(super) call: PlannedChatToolCall,
    pub(super) access_context: &'a crate::security::AccessContext,
    pub(super) prompt_scope: &'a tools::MentionScope,
    pub(super) rationale: Option<&'a str>,
    pub(super) tool_outputs: &'a mut Vec<ToolEvidenceRecord>,
    pub(super) gathered_sources: &'a mut Vec<RetrievedChatSource>,
    pub(super) active_chat: &'a ActiveChatHandle,
    pub(super) turn: &'a mut ChatTurnState,
}

pub(super) struct RetrievalCandidateRequest<'a> {
    pub(super) state: &'a AppState,
    pub(super) access_context: &'a crate::security::AccessContext,
    pub(super) queries: &'a [String],
    pub(super) candidate_limit: usize,
    pub(super) channel_focus_ids: &'a [String],
    pub(super) video_focus_ids: &'a [String],
    pub(super) source_kind: Option<crate::search::SearchSourceKind>,
    pub(super) active_chat: &'a ActiveChatHandle,
}

#[derive(Debug, Clone)]
pub(crate) struct CoverageAssessment {
    pub(crate) needs_more: bool,
    pub(crate) reason: Option<String>,
    pub(crate) channel_focus_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub(super) struct ChatRetrievalOutcome {
    pub(super) plan: ChatRetrievalPlan,
    pub(super) sources: Vec<RetrievedChatSource>,
    pub(super) pass_count: usize,
    pub(super) query_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct VideoObservation {
    pub(crate) video_title: String,
    pub(crate) channel_name: String,
    pub(crate) summary: String,
}

#[derive(Debug, Clone)]
pub(crate) struct VideoObservationInput {
    pub(crate) video_id: String,
    pub(crate) video_title: String,
    pub(crate) channel_name: String,
    pub(crate) excerpts: Vec<RetrievedChatSource>,
}

#[derive(Debug, Deserialize)]
pub(super) struct OllamaChatResponse {
    pub(super) message: Option<OllamaChatMessage>,
    pub(super) done: bool,
    pub(super) error: Option<String>,
    #[serde(default)]
    pub(super) prompt_eval_count: Option<u64>,
    #[serde(default)]
    pub(super) eval_count: Option<u64>,
    #[serde(default)]
    pub(super) total_duration: Option<u64>,
}

#[derive(Debug, Clone)]
pub(super) struct OllamaStreamStats {
    pub(super) prompt_eval_count: Option<u64>,
    pub(super) eval_count: Option<u64>,
    pub(super) total_duration_ns: Option<u64>,
}

#[derive(Debug, Clone)]
pub(crate) struct GenerationMeta {
    pub(crate) model: String,
    pub(crate) prompt_tokens: Option<u64>,
    pub(crate) completion_tokens: Option<u64>,
    pub(crate) total_duration_ns: Option<u64>,
}

#[derive(Debug, Clone)]
pub(super) struct ChatTurnState {
    strategy: String,
    plan_intent: Option<String>,
    plan_label: Option<String>,
    tool_calls: Vec<crate::models::ChatTurnToolTrace>,
    retrieval: Option<crate::models::ChatTurnRetrievalTrace>,
    budget: ChatTurnBudget,
}

impl ChatTurnState {
    pub(super) fn new(deep_research: bool) -> Self {
        let max_model_calls = if deep_research {
            CHAT_TURN_MODEL_CALL_LIMIT_DEEP_RESEARCH
        } else {
            CHAT_TURN_MODEL_CALL_LIMIT
        };
        let max_tool_calls = if deep_research {
            CHAT_TOOL_LOOP_MAX_STEPS_DEEP_RESEARCH
        } else {
            CHAT_TOOL_LOOP_MAX_STEPS
        };
        Self {
            strategy: "unresolved".to_string(),
            plan_intent: None,
            plan_label: None,
            tool_calls: Vec::new(),
            retrieval: None,
            budget: ChatTurnBudget {
                max_model_calls,
                model_calls: 0,
                max_tool_calls,
                tool_calls: 0,
                max_retrieval_passes: CHAT_MAX_RETRIEVAL_PASSES,
                retrieval_passes: 0,
                exhaustion_reason: None,
            },
        }
    }

    pub(super) fn set_strategy(&mut self, strategy: impl Into<String>) {
        self.strategy = strategy.into();
    }

    pub(super) fn set_plan(&mut self, plan: &ChatRetrievalPlan) {
        self.plan_intent = Some(plan.intent.label().to_string());
        self.plan_label = Some(plan.label.clone());
    }

    pub(super) fn record_retrieval(
        &mut self,
        plan: &ChatRetrievalPlan,
        pass_count: usize,
        query_count: usize,
        selected_source_count: usize,
        unique_video_count: usize,
    ) {
        self.set_plan(plan);
        self.retrieval = Some(crate::models::ChatTurnRetrievalTrace {
            pass_count,
            query_count,
            selected_source_count,
            unique_video_count,
            deep_research: plan.deep_research,
        });
    }

    pub(super) fn record_tool_call(&mut self, name: impl Into<String>, state: impl Into<String>) {
        self.tool_calls.push(crate::models::ChatTurnToolTrace {
            name: name.into(),
            state: state.into(),
        });
    }

    pub(super) fn update_last_tool_state(&mut self, state: impl Into<String>) {
        if let Some(last) = self.tool_calls.last_mut() {
            last.state = state.into();
        }
    }

    pub(super) fn consume_model_call(&mut self, label: &str) -> Result<(), String> {
        if self.budget.model_calls >= self.budget.max_model_calls {
            let reason = format!("Reached the model-call budget before {label}.");
            self.mark_budget_exhausted(reason.clone());
            return Err(reason);
        }
        self.budget.model_calls += 1;
        Ok(())
    }

    pub(super) fn consume_tool_call(&mut self, label: &str) -> Result<(), String> {
        if self.budget.tool_calls >= self.budget.max_tool_calls {
            let reason = format!("Reached the tool-call budget before {label}.");
            self.mark_budget_exhausted(reason.clone());
            return Err(reason);
        }
        self.budget.tool_calls += 1;
        Ok(())
    }

    pub(super) fn consume_retrieval_pass(&mut self, pass: usize) -> Result<(), String> {
        if self.budget.retrieval_passes >= self.budget.max_retrieval_passes {
            let reason = format!("Reached the retrieval-pass budget before pass {pass}.");
            self.mark_budget_exhausted(reason.clone());
            return Err(reason);
        }
        self.budget.retrieval_passes += 1;
        Ok(())
    }

    pub(super) fn mark_budget_exhausted(&mut self, reason: impl Into<String>) {
        if self.budget.exhaustion_reason.is_none() {
            self.budget.exhaustion_reason = Some(reason.into());
        }
    }

    pub(super) fn budget_exhausted(&self) -> bool {
        self.budget.exhaustion_reason.is_some()
    }

    pub(super) fn budget_snapshot(&self) -> crate::models::ChatTurnBudgetSnapshot {
        self.budget.snapshot()
    }

    pub(super) fn finish(self) -> crate::models::ChatTurnTrace {
        let budget = self.budget.snapshot();
        crate::models::ChatTurnTrace {
            strategy: self.strategy,
            plan_intent: self.plan_intent,
            plan_label: self.plan_label,
            tool_calls: self.tool_calls,
            retrieval: self.retrieval,
            budget,
        }
    }
}

#[derive(Debug, Clone)]
struct ChatTurnBudget {
    max_model_calls: usize,
    model_calls: usize,
    max_tool_calls: usize,
    tool_calls: usize,
    max_retrieval_passes: usize,
    retrieval_passes: usize,
    exhaustion_reason: Option<String>,
}

impl ChatTurnBudget {
    fn snapshot(&self) -> crate::models::ChatTurnBudgetSnapshot {
        crate::models::ChatTurnBudgetSnapshot {
            max_model_calls: self.max_model_calls,
            model_calls: self.model_calls,
            max_tool_calls: self.max_tool_calls,
            tool_calls: self.tool_calls,
            max_retrieval_passes: self.max_retrieval_passes,
            retrieval_passes: self.retrieval_passes,
            exhausted: self.exhaustion_reason.is_some(),
            exhaustion_reason: self.exhaustion_reason.clone(),
        }
    }
}

pub(super) async fn emit_budget_exhausted(
    active_chat: &ActiveChatHandle,
    turn: &ChatTurnState,
    reason: impl Into<String>,
) {
    let reason = reason.into();
    active_chat
        .emit(ChatStreamEvent::Status {
            status: ChatStatusPayload::new("budget_exhausted", "Turn budget reached")
                .with_detail(reason.clone())
                .with_decision(reason)
                .with_budget(turn.budget_snapshot()),
        })
        .await;
}

#[derive(Debug, Deserialize)]
pub(super) struct OllamaChatMessage {
    pub(super) content: String,
}

#[derive(Debug, Serialize)]
pub(super) struct OllamaChatRequest {
    pub(super) model: String,
    pub(super) messages: Vec<OllamaRequestMessage>,
    pub(super) stream: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct OllamaRequestMessage {
    pub(crate) role: String,
    pub(crate) content: String,
}

/// Inputs for [`ChatService::start_reply_workflow`], grouped to stay within `clippy::too_many_arguments`.
pub struct ReplyWorkflowRequest {
    pub state: AppState,
    pub conversation: ChatConversation,
    pub access_context: crate::security::AccessContext,
    pub conversation_scope_id: String,
    pub active_reply_key: ActiveChatKey,
    pub prompt: String,
    pub should_auto_name: bool,
    pub deep_research: bool,
    pub reply_model: String,
    pub active_reply: ActiveChatHandle,
    /// When false, assistant output and titles are not persisted (ephemeral / client-local history).
    pub persist_to_store: bool,
}

#[derive(Clone)]
pub struct ChatService {
    pub(super) core: OllamaCore,
    pub(super) multi_pass_enabled: bool,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
