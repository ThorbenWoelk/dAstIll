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
            persist_to_store,
        } = job;
        tokio::spawn(async move {
            if persist_to_store && should_auto_name {
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
                    persist_to_store,
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
        persist_to_store: bool,
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
                    if persist_to_store {
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
                        if persist_to_store {
                            let _ = persist_assistant_message(
                                &state,
                                &conversation_scope_id,
                                &conversation_id,
                                &message,
                            )
                            .await;
                        }
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
                    if persist_to_store {
                        let _ = persist_assistant_message(
                            &state,
                            &conversation_scope_id,
                            &conversation_id,
                            &message,
                        )
                        .await;
                    }
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
