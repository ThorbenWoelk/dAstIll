use super::*;

impl ChatService {
    pub(super) async fn run_tool_loop(
        &self,
        state: &AppState,
        conversation: &ChatConversation,
        access_context: &crate::security::AccessContext,
        prompt: &str,
        deep_research: bool,
        active_chat: &ActiveChatHandle,
    ) -> Result<Option<ToolLoopOutcome>, String> {
        active_chat
            .emit(ChatStreamEvent::Status {
                status: ChatStatusPayload::new("tool_planning", "Preparing chat plan").with_detail(
                    "Resolving scope and deciding whether to answer directly or gather evidence.",
                ),
            })
            .await;

        let prompt_scope = match timeout(
            CHAT_MENTION_SCOPE_TIMEOUT,
            tools::resolve_mention_scope(&state.db, access_context, prompt),
        )
        .await
        {
            Ok(Ok(scope)) => scope,
            Ok(Err(error)) => {
                tracing::warn!(error = %error, "failed to resolve tool-loop @mentions");
                tools::MentionScope {
                    cleaned_prompt: prompt.trim().to_string(),
                    ..tools::MentionScope::default()
                }
            }
            Err(_) => {
                tracing::warn!(
                    timeout_secs = CHAT_MENTION_SCOPE_TIMEOUT.as_secs(),
                    "timed out resolving tool-loop mention scope"
                );
                tools::MentionScope {
                    cleaned_prompt: prompt.trim().to_string(),
                    ..tools::MentionScope::default()
                }
            }
        };
        let prompt_scope =
            filter_mention_scope_for_access(&state.db, access_context, prompt_scope).await;
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
                access_context,
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

        if comparison_related_video_prompt(prompt) && !prompt_scope.video_titles.is_empty() {
            self.execute_planned_tool_call(ToolCallExecutionRequest {
                state,
                call: PlannedChatToolCall::SearchLibrary(tools::SearchLibraryQuery {
                    query: prompt_scope.video_titles[0].clone(),
                    source_kind: None,
                    limit: 6,
                }),
                access_context,
                prompt_scope: &prompt_scope,
                rationale: Some(
                    "Load the currently scoped video before searching for a related comparison candidate.",
                ),
                tool_outputs: &mut tool_outputs,
                gathered_sources: &mut gathered_sources,
                active_chat,
            })
            .await?;

            if let Some(channel_id) =
                comparison_fallback_channel_id(&prompt_scope, &gathered_sources)
            {
                self.execute_planned_tool_call(ToolCallExecutionRequest {
                    state,
                    call: PlannedChatToolCall::RecentLibraryActivity(
                        tools::RecentLibraryActivityQuery {
                            scope: tools::RecentLibraryActivityScope::Channel,
                            channel_id: Some(channel_id),
                            video_id: None,
                            limit_videos: CHAT_RECENT_ACTIVITY_VIDEO_LIMIT,
                            include_summaries: true,
                            include_transcripts: true,
                        },
                    ),
                    access_context,
                    prompt_scope: &prompt_scope,
                    rationale: Some(
                        "Load recent channel videos so comparison prompts can find a nearby related video.",
                    ),
                    tool_outputs: &mut tool_outputs,
                    gathered_sources: &mut gathered_sources,
                    active_chat,
                })
                .await?;
            }
        }

        for step in 1..=max_steps {
            active_chat.ensure_not_cancelled()?;
            let comparison_channel_id =
                comparison_fallback_channel_id(&prompt_scope, &gathered_sources);
            if step == 2
                && comparison_related_video_prompt(prompt)
                && gathered_source_video_count(&gathered_sources) < 2
                && comparison_channel_id.is_some()
            {
                self.execute_planned_tool_call(ToolCallExecutionRequest {
                    state,
                    call: PlannedChatToolCall::RecentLibraryActivity(
                        tools::RecentLibraryActivityQuery {
                            scope: tools::RecentLibraryActivityScope::Channel,
                            channel_id: comparison_channel_id,
                            video_id: None,
                            limit_videos: CHAT_RECENT_ACTIVITY_VIDEO_LIMIT,
                            include_summaries: true,
                            include_transcripts: true,
                        },
                    ),
                    access_context,
                    prompt_scope: &prompt_scope,
                    rationale: Some(
                        "Need recent channel context to find another related video for comparison.",
                    ),
                    tool_outputs: &mut tool_outputs,
                    gathered_sources: &mut gathered_sources,
                    active_chat,
                })
                .await?;
            }

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
                access_context,
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
                        return Ok(fallback_tool_loop_outcome(
                            &tool_outputs,
                            &gathered_sources,
                            "Tool planner returned unreadable JSON after gathering evidence.",
                        ));
                    }
                },
                Ok(Err(error)) => {
                    tracing::warn!(error = ?error, "chat tool loop unavailable");
                    return Ok(fallback_tool_loop_outcome(
                        &tool_outputs,
                        &gathered_sources,
                        "Tool planner unavailable after gathering evidence.",
                    ));
                }
                Err(_) => {
                    tracing::warn!("chat tool loop timed out");
                    return Ok(fallback_tool_loop_outcome(
                        &tool_outputs,
                        &gathered_sources,
                        "Tool planner timed out after gathering evidence.",
                    ));
                }
            };

            match step_outcome.action {
                ToolLoopAction::Respond => {
                    let conversation_only = tool_outputs.is_empty() && gathered_sources.is_empty();
                    if conversation_only && analytical_prompt_needs_retrieval(prompt, &prompt_scope)
                    {
                        tracing::warn!(
                            "tool loop chose conversation-only answer for analytical prompt without evidence; falling back to retrieval planner"
                        );
                        return Ok(None);
                    }
                    return Ok(Some(ToolLoopOutcome {
                        conversation_only,
                        rationale: step_outcome.rationale,
                        tool_outputs,
                        sources: gathered_sources,
                    }));
                }
                ToolLoopAction::ToolCall(call) => {
                    let (call, rationale) = rewrite_subject_overlap_db_inspect(
                        call,
                        step_outcome.rationale,
                        prompt,
                        &prompt_scope,
                    );
                    self.execute_planned_tool_call(ToolCallExecutionRequest {
                        state,
                        call,
                        access_context,
                        prompt_scope: &prompt_scope,
                        rationale: rationale.as_deref(),
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
            access_context,
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
                let result = if crate::security::can_use_db_inspect(access_context) {
                    tools::execute_db_inspect_query(&state.db, access_context, *query)
                        .await
                        .map_err(|error| error.to_string())?
                } else {
                    tools::db_inspect_forbidden_result()
                };
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
                        access_context,
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
                let result = tools::execute_highlight_lookup_query(
                    &state.db,
                    access_context.user_id.as_deref(),
                    query.clone(),
                )
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
                let result =
                    execute_recent_library_activity_query(&state.db, access_context, &query)
                        .await?;
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
}

fn fallback_tool_loop_outcome(
    tool_outputs: &[ToolEvidenceRecord],
    gathered_sources: &[RetrievedChatSource],
    rationale: &str,
) -> Option<ToolLoopOutcome> {
    if tool_outputs.is_empty() && gathered_sources.is_empty() {
        None
    } else {
        Some(ToolLoopOutcome {
            conversation_only: false,
            rationale: Some(rationale.to_string()),
            tool_outputs: tool_outputs.to_vec(),
            sources: gathered_sources.to_vec(),
        })
    }
}

fn rewrite_subject_overlap_db_inspect(
    call: PlannedChatToolCall,
    rationale: Option<String>,
    prompt: &str,
    prompt_scope: &tools::MentionScope,
) -> (PlannedChatToolCall, Option<String>) {
    let PlannedChatToolCall::DbInspect(_query) = call else {
        return (call, rationale);
    };

    if !subject_overlap_prompt(prompt) {
        return (call, rationale);
    }

    let search_query = prompt_scope.prompt_for_retrieval(prompt);
    let rationale = Some(match rationale {
        Some(existing) if !existing.is_empty() => format!(
            "{existing} Topic-overlap questions need grounded content search, not database metadata."
        ),
        _ => "Topic-overlap questions need grounded content search, not database metadata."
            .to_string(),
    });

    (
        PlannedChatToolCall::SearchLibrary(tools::SearchLibraryQuery {
            query: search_query,
            source_kind: None,
            limit: CHAT_PATTERN_SOURCE_LIMIT,
        }),
        rationale,
    )
}

fn subject_overlap_prompt(prompt: &str) -> bool {
    let normalized = prompt.to_ascii_lowercase();
    [
        "same subjects",
        "same subject",
        "same topic",
        "same topics",
        "different angles",
        "different angle",
        "what topics",
        "what themes",
        "which channels talk about",
        "cover the same",
        "same subjects most often",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn analytical_prompt_needs_retrieval(prompt: &str, prompt_scope: &tools::MentionScope) -> bool {
    if prompt_scope.has_scope() {
        return false;
    }

    let normalized = prompt.to_ascii_lowercase();
    [
        "what does the speaker",
        "what is the speaker",
        "what parts of the discussion",
        "what are the strongest arguments",
        "core thesis",
        "key takeaways",
        "actionable ideas",
        "what problem is this",
        "clearest explanation",
        "most confusing",
        "audience already knows",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn comparison_related_video_prompt(prompt: &str) -> bool {
    let normalized = prompt.to_ascii_lowercase();
    normalized.contains("last related video")
        || normalized.contains("same topic")
        || normalized.contains("compare this video")
}

fn gathered_source_video_count(sources: &[RetrievedChatSource]) -> usize {
    sources
        .iter()
        .map(|source| source.source.video_id.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len()
}

fn comparison_fallback_channel_id(
    prompt_scope: &tools::MentionScope,
    gathered_sources: &[RetrievedChatSource],
) -> Option<String> {
    prompt_scope.channel_focus_ids.first().cloned().or_else(|| {
        gathered_sources
            .first()
            .map(|source| source.source.channel_id.clone())
    })
}
