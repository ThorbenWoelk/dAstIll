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
    /// When false, assistant output and titles are not persisted (ephemeral / client-local history).
    pub persist_to_store: bool,
}

#[derive(Clone)]
pub struct ChatService {
    core: OllamaCore,
    multi_pass_enabled: bool,
}
