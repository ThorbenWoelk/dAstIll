impl ChatService {
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
}
