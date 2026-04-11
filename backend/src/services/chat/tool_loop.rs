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
        let prompt_scope = tools::resolve_mention_scope(&state.db, access_context, prompt)
            .await
            .unwrap_or_else(|error| {
                tracing::warn!(error = %error, "failed to resolve tool-loop @mentions");
                tools::MentionScope {
                    cleaned_prompt: prompt.trim().to_string(),
                    ..tools::MentionScope::default()
                }
            });
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
                        access_context,
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
