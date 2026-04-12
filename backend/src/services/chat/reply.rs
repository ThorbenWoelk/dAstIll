use super::*;

impl ChatService {
    pub(super) async fn run_reply_workflow(
        &self,
        state: AppState,
        conversation: ChatConversation,
        access_context: crate::security::AccessContext,
        conversation_scope_id: String,
        active_reply_key: ActiveChatKey,
        prompt: String,
        deep_research: bool,
        reply_model: String,
        active_reply: ActiveChatHandle,
        persist_to_store: bool,
    ) {
        let conversation_id = conversation.id.clone();
        let span = logfire::span!(
            "chat.reply",
            conversation.id = conversation_id.clone(),
            query.chars = prompt.chars().count(),
        );

        async move {
            tracing::info!(
                conversation_id = %conversation_id,
                reply_model = %reply_model,
                deep_research,
                persist_to_store,
                "chat reply started"
            );
            let reply_result = self
                .generate_reply(
                    &state,
                    &conversation,
                    &access_context,
                    &prompt,
                    deep_research,
                    &reply_model,
                    &active_reply,
                )
                .await;

            match reply_result {
                Ok(message) => {
                    tracing::info!(
                        conversation_id = %conversation_id,
                        model = message.model.as_deref().unwrap_or("-"),
                        response_chars = message.content.chars().count(),
                        source_count = message.sources.len(),
                        "chat reply completed"
                    );
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
                            active_reply
                                .emit(ChatStreamEvent::Error {
                                    message: "Failed to store chat response.".to_string(),
                                })
                                .await;
                        } else {
                            active_reply.emit(ChatStreamEvent::Done { message }).await;
                        }
                    } else {
                        active_reply.emit(ChatStreamEvent::Done { message }).await;
                    }
                }
                Err(error) => {
                    if error == "cancelled" {
                        tracing::info!(conversation_id = %conversation_id, "chat reply cancelled");
                        let (status, content) = active_reply
                            .cancelled_outcome()
                            .unwrap_or((
                                ChatMessageStatus::Cancelled,
                                "Response cancelled.".to_string(),
                            ));
                        let message =
                            self.build_assistant_message(content, Vec::new(), status, None);
                        if persist_to_store {
                            let _ = persist_assistant_message(
                                &state,
                                &conversation_scope_id,
                                &conversation_id,
                                &message,
                            )
                            .await;
                        }
                        active_reply.emit(ChatStreamEvent::Done { message }).await;
                        let mut active_replies = state.active_replies.lock().await;
                        active_replies.remove(&active_reply_key);
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
                    active_reply
                        .emit(ChatStreamEvent::Error { message: error })
                        .await;
                }
            }

            let mut active_replies = state.active_replies.lock().await;
            active_replies.remove(&active_reply_key);
        }
        .instrument(span)
        .await;
    }

    async fn generate_reply(
        &self,
        state: &AppState,
        conversation: &ChatConversation,
        access_context: &crate::security::AccessContext,
        prompt: &str,
        deep_research: bool,
        reply_model: &str,
        active_chat: &ActiveChatHandle,
    ) -> Result<ChatMessage, String> {
        active_chat.ensure_not_cancelled()?;
        if let Some(tool_outcome) = self
            .run_tool_loop(
                state,
                conversation,
                access_context,
                prompt,
                deep_research,
                active_chat,
            )
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

            let grounding =
                build_tool_grounding_context(prompt, &tool_outputs, &tool_outcome.sources);
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
                access_context,
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
            .retrieve_sources_with_plan(
                state,
                access_context,
                &conversation.id,
                prompt,
                plan,
                active_chat,
            )
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
}
