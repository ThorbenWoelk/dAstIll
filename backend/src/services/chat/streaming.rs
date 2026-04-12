use super::*;

impl ChatService {
    pub(super) async fn build_answer_grounding_context(
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

    pub(super) async fn stream_ollama_reply(
        &self,
        conversation: &ChatConversation,
        grounding_context: String,
        active_chat: &ActiveChatHandle,
        cancel_rx: &mut watch::Receiver<ChatCancellationState>,
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
                            if changed.is_ok() && cancel_rx.borrow().cancelled {
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
}
