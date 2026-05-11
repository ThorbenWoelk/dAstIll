use super::*;

fn plural_suffix(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

fn related_video_search_query(query: &str) -> bool {
    let normalized = query.to_ascii_lowercase();
    [
        "related video",
        "same topic",
        "different angle",
        "different angles",
        "compare",
        "comparison",
        "closest in theme",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn inherited_search_scope(
    prompt_scope: Option<&tools::MentionScope>,
    query: &tools::SearchLibraryQuery,
    query_scope: &tools::MentionScope,
) -> Option<tools::MentionScope> {
    let Some(prompt_scope) = prompt_scope else {
        return None;
    };

    if !prompt_scope.video_focus_ids.is_empty()
        && query_scope.video_focus_ids.is_empty()
        && !is_direct_video_lookup_request(&prompt_scope.cleaned_prompt, &query.query)
        && related_video_search_query(&query.query)
    {
        let mut relaxed = prompt_scope.clone();
        relaxed.video_focus_ids.clear();
        relaxed.video_titles.clear();
        relaxed.channel_focus_ids.clear();
        relaxed.channel_names.clear();
        return Some(relaxed);
    }

    Some(prompt_scope.clone())
}

fn scope_resolution_detail(scope: &tools::MentionScope) -> String {
    let channel_count = scope.channel_focus_ids.len();
    let video_count = scope.video_focus_ids.len();
    match (channel_count, video_count) {
        (0, 0) => "Search scope resolved: full accessible library.".to_string(),
        (channels, 0) => format!(
            "Search scope resolved: {channels} channel{}.",
            plural_suffix(channels)
        ),
        (0, videos) => {
            format!(
                "Search scope resolved: {videos} item{}.",
                plural_suffix(videos)
            )
        }
        (channels, videos) => format!(
            "Search scope resolved: {channels} channel{} and {videos} item{}.",
            plural_suffix(channels),
            plural_suffix(videos)
        ),
    }
}

impl ChatService {
    pub(super) async fn retrieve_sources_with_plan(
        &self,
        state: &AppState,
        access_context: &crate::security::AccessContext,
        conversation_id: &str,
        prompt: &str,
        plan: ChatRetrievalPlan,
        active_chat: &ActiveChatHandle,
        turn: &mut ChatTurnState,
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
            if let Err(reason) = turn.consume_retrieval_pass(1) {
                emit_budget_exhausted(active_chat, turn, reason.clone()).await;
                return Err(reason);
            }
            let pass_one = self
                .run_retrieval_pass(
                    state,
                    &mut pool,
                    RetrievalPassRequest {
                        conversation_id,
                        plan: &plan,
                        access_context,
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
                if let Err(reason) = turn.consume_retrieval_pass(2) {
                    emit_budget_exhausted(active_chat, turn, reason).await;
                } else {
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
                    let pass_two_channel_focus = merge_channel_focus_ids(
                        &plan.channel_focus_ids,
                        &assessment.channel_focus_ids,
                    );
                    let pass_two_video_focus = plan.video_focus_ids.clone();
                    let pass_two = self
                        .run_retrieval_pass(
                            state,
                            &mut pool,
                            RetrievalPassRequest {
                                conversation_id,
                                plan: &plan,
                                access_context,
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
            }

            if CHAT_MAX_RETRIEVAL_PASSES > 2
                && self.multi_pass_enabled
                && assessment.needs_more
                && plan.supports_third_pass()
            {
                if let Err(reason) = turn.consume_retrieval_pass(3) {
                    emit_budget_exhausted(active_chat, turn, reason).await;
                } else {
                    let mut status =
                        ChatStatusPayload::new("retrieving_pass_3", "Deepening evidence")
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
                    let pass_three_channel_focus = merge_channel_focus_ids(
                        &plan.channel_focus_ids,
                        &assessment.channel_focus_ids,
                    );
                    let pass_three_video_focus = plan.video_focus_ids.clone();
                    let pass_three = self
                        .run_retrieval_pass(
                            state,
                            &mut pool,
                            RetrievalPassRequest {
                                conversation_id,
                                plan: &plan,
                                access_context,
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
            }

            if assessment.needs_more && pass_count >= CHAT_MAX_RETRIEVAL_PASSES {
                let reason = "Reached the retrieval-pass budget for this turn.";
                turn.mark_budget_exhausted(reason);
                emit_budget_exhausted(active_chat, turn, reason).await;
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

            let query_count = (1..=pass_count)
                .map(|pass| plan.queries_for_pass(pass).len())
                .sum();
            Ok(ChatRetrievalOutcome {
                plan,
                sources,
                pass_count,
                query_count,
            })
        }
        .instrument(span)
        .await
    }

    pub(super) async fn plan_retrieval(
        &self,
        state: &AppState,
        conversation: &ChatConversation,
        access_context: &crate::security::AccessContext,
        conversation_id: &str,
        prompt: &str,
        deep_research: bool,
        active_chat: &ActiveChatHandle,
        turn: &mut ChatTurnState,
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
            active_chat
                .emit(ChatStreamEvent::Status {
                    status: ChatStatusPayload::new("classifying", "Planning search")
                        .with_detail("Resolving scope of search."),
                })
                .await;
            let scope = match tools::resolve_mention_scope(&state.db, access_context, prompt).await
            {
                Ok(scope) => {
                    filter_mention_scope_for_access(&state.db, access_context, scope).await
                }
                Err(error) => {
                    tracing::warn!(error = %error, "failed to resolve chat @mentions");
                    tools::MentionScope {
                        cleaned_prompt: prompt.to_string(),
                        ..tools::MentionScope::default()
                    }
                }
            };
            active_chat
                .emit(ChatStreamEvent::Status {
                    status: ChatStatusPayload::new("classifying", "Planning search")
                        .with_detail(scope_resolution_detail(&scope)),
                })
                .await;
            let retrieval_prompt = scope.prompt_for_retrieval(prompt);
            let planner_prompt = scope.prompt_for_planner(prompt);
            let planner_input =
                format_conversation_for_planner(conversation, access_context, &planner_prompt);

            active_chat
                .emit(ChatStreamEvent::Status {
                    status: ChatStatusPayload::new("classifying", "Planning search")
                        .with_detail("Selecting search process."),
                })
                .await;

            if let Err(reason) = turn.consume_model_call("retrieval planning") {
                emit_budget_exhausted(active_chat, turn, reason.clone()).await;
                return Ok(ChatRetrievalPlan::fallback(&retrieval_prompt, Some(reason)));
            }
            let planned = await_or_cancel(
                active_chat,
                timeout(
                    CHAT_CLASSIFY_TIMEOUT,
                    self.core.prompt_json_schema::<ChatQueryPlanResponse>(
                        "chat_query_plan",
                        CHAT_QUERY_PLAN_PROMPT,
                        &planner_input,
                        &ChatQueryPlanResponse::json_schema(),
                        crate::services::ollama::CooldownStatusPolicy::UseLocalFallback,
                    ),
                ),
            )
            .await?;

            let mut plan = match planned {
                Ok(Ok((response, _))) => {
                    ChatRetrievalPlan::from_response(&retrieval_prompt, response)
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
                    "Selected \"{}\" search process: up to {} excerpts and {} per item.",
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
            access_context,
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
                    access_context,
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
            let mut completed_status = ChatStatusPayload::new(
                format!("retrieving_pass_{pass}_complete"),
                if pass == 1 {
                    "Library search complete".to_string()
                } else {
                    "Broadened search complete".to_string()
                },
            )
            .with_detail(format!(
                "Pass {pass} selected {} excerpts across {} videos.",
                sources.len(),
                count_unique_videos(&sources)
            ));
            if let Some(reason) = &assessment.reason {
                completed_status = completed_status.with_decision(reason.clone());
            }
            active_chat
                .emit(ChatStreamEvent::Status {
                    status: completed_status,
                })
                .await;
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
            access_context,
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
            let query_tokens = crate::search::query::meaningful_search_terms(query);
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
                            c.chunk_text = crate::search::extract_keyword_snippet(
                                &c.chunk_text,
                                &query_tokens,
                            );
                        }
                        c
                    })
                    .collect();
                keyword_batches.push(filter_search_candidates_for_access(
                    candidates,
                    access_context,
                ));
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
                    let query_embedding = crate::search::vector_to_json(embedding);
                    for channel_filter in &filters {
                        active_chat.ensure_not_cancelled()?;
                        semantic_batches.push(filter_search_candidates_for_access(
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
                            access_context,
                        ));
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

    pub(super) async fn execute_search_library_query(
        &self,
        state: &AppState,
        access_context: &crate::security::AccessContext,
        query: tools::SearchLibraryQuery,
        prompt_scope: Option<&tools::MentionScope>,
        active_chat: &ActiveChatHandle,
    ) -> Result<SearchLibraryExecutionResult, String> {
        active_chat.ensure_not_cancelled()?;
        let candidate_limit = retrieval_candidate_limit(query.limit, 1, 1);
        let query_scope = tools::resolve_mention_scope(&state.db, access_context, &query.query)
            .await
            .unwrap_or_else(|error| {
                tracing::warn!(error = %error, "failed to resolve search_library @mentions");
                tools::MentionScope {
                    cleaned_prompt: query.query.clone(),
                    ..tools::MentionScope::default()
                }
            });
        let inherited_scope = inherited_search_scope(prompt_scope, &query, &query_scope);
        let scope = filter_mention_scope_for_access(
            &state.db,
            access_context,
            merge_mention_scope(inherited_scope.as_ref(), &query_scope),
        )
        .await;
        let query_text = scope.scoped_query(&query.query);
        if let Some(video_id) = direct_video_lookup_target(&scope, &query) {
            active_chat.ensure_not_cancelled()?;
            let direct_sources = filter_retrieved_sources_for_access(
                load_direct_video_sources(&state.db, video_id, query.source_kind).await?,
                access_context,
            );
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
                access_context,
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
}
