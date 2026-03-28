
fn format_search_library_tool_output(
    query: &tools::SearchLibraryQuery,
    sources: &[RetrievedChatSource],
) -> String {
    if sources.is_empty() {
        return format!("No grounded excerpts found for \"{}\".", query.query);
    }

    let rows = sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            format!(
                "{}. {} / {} / {} - {}",
                index + 1,
                source.source.channel_name,
                source.source.video_title,
                source.source.source_kind.as_str(),
                source.source.snippet
            )
        })
        .collect::<Vec<_>>();

    format!(
        "Found {} excerpt{} for \"{}\":\n{}",
        sources.len(),
        if sources.len() == 1 { "" } else { "s" },
        query.query,
        rows.join("\n")
    )
}

fn merge_retrieved_sources(
    existing: &mut Vec<RetrievedChatSource>,
    new_sources: impl IntoIterator<Item = RetrievedChatSource>,
) {
    let mut seen = existing
        .iter()
        .map(|source| source.source.chunk_id.clone())
        .collect::<HashSet<_>>();

    for source in new_sources {
        if seen.insert(source.source.chunk_id.clone()) {
            existing.push(source);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::chat_heuristics::{
        collect_focus_terms, is_attributed_preference_query, recommendation_query_variants,
    };
    use super::{
        ActiveChatHandle, CHAT_CLASSIFY_TIMEOUT, CHAT_RECENT_ACTIVITY_SOURCE_LIMIT,
        ChatQueryIntent, ChatQueryPlanResponse, ChatRetrievalPlan, ChatService,
        ChatToolLoopResponse, PlannedChatToolCall, RetrievedChatSource, ToolLoopAction,
        is_direct_video_lookup_request, maybe_direct_recent_activity_tool_call, tools,
    };
    use crate::models::ChatSource;
    use crate::services::chat::tools::{
        DbInspectOperation, DbInspectTarget, DbInspectToolInput, SearchLibraryToolInput,
    };
    use crate::services::ollama::{CLOUD_PROMPT_TIMEOUT_SECS, OllamaCore};
    use crate::services::search::SearchSourceKind;

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
    fn direct_video_lookup_detects_generic_read_requests() {
        assert!(is_direct_video_lookup_request(
            "read for me",
            "3 Things Every Relationship Has"
        ));
        assert!(is_direct_video_lookup_request(
            "read 3 Things Every Relationship Has for me",
            "3 Things Every Relationship Has"
        ));
        assert!(is_direct_video_lookup_request(
            "summarize +{3 Things Every Relationship Has}",
            "3 Things Every Relationship Has"
        ));
    }

    #[test]
    fn direct_video_lookup_keeps_real_search_queries_as_search() {
        assert!(!is_direct_video_lookup_request(
            "what does it say about dopamine",
            "3 Things Every Relationship Has"
        ));
        assert!(!is_direct_video_lookup_request(
            "find the section about trust",
            "3 Things Every Relationship Has"
        ));
    }

    #[test]
    fn retrieval_plan_upgrades_fact_to_recommendation_lookup() {
        let plan = ChatRetrievalPlan::from_response(
            "what is the best database according to theo?",
            ChatQueryPlanResponse {
                needs_retrieval: Some(true),
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
    fn retrieval_plan_recognizes_recent_activity_queries() {
        let plan = ChatRetrievalPlan::fallback("What is HealthyGamerGG doing lately?", None);
        assert_eq!(plan.intent, ChatQueryIntent::RecentActivity);
        assert_eq!(plan.label, "Recent activity scan");
        assert_eq!(plan.budget, CHAT_RECENT_ACTIVITY_SOURCE_LIMIT);
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

    #[test]
    fn planner_timeout_allows_slow_cloud_classification() {
        assert!(CHAT_CLASSIFY_TIMEOUT.as_secs() >= 15);
        assert!(CHAT_CLASSIFY_TIMEOUT.as_secs() < CLOUD_PROMPT_TIMEOUT_SECS);
    }

    #[test]
    fn deep_research_plan_raises_budget_and_forces_retrieval() {
        let mut plan = ChatRetrievalPlan::fallback("climate policy in my library", None);
        plan.skip_retrieval = true;
        plan.apply_deep_research("climate policy in my library");
        assert!(!plan.skip_retrieval);
        assert!(plan.deep_research);
        assert_eq!(plan.budget, 48);
        assert_eq!(plan.label, "Deep research");
        assert_eq!(plan.intent, ChatQueryIntent::Pattern);
        assert!(!plan.queries.is_empty());
    }

    #[test]
    fn tool_loop_builds_db_inspect_query_when_requested() {
        let execution = ChatToolLoopResponse {
            action: Some("tool_call".to_string()),
            rationale: Some("The user is asking about stored summary records.".to_string()),
            tool_name: Some("db_inspect".to_string()),
            search_library_input: None,
            highlight_lookup_input: None,
            db_inspect_input: Some(DbInspectToolInput {
                operation: Some("count".to_string()),
                resource: Some("summaries".to_string()),
                limit: None,
                group_by: None,
            }),
            recent_library_activity_input: None,
        }
        .into_step_outcome()
        .expect("valid tool plan");

        let ToolLoopAction::ToolCall(PlannedChatToolCall::DbInspect(query)) = execution.action
        else {
            panic!("expected db inspect tool call");
        };

        assert_eq!(query.operation, DbInspectOperation::Count);
        assert_eq!(query.target, DbInspectTarget::Summaries);
        assert_eq!(
            execution.rationale.as_deref(),
            Some("The user is asking about stored summary records.")
        );
    }

    #[test]
    fn tool_loop_builds_search_library_query_when_requested() {
        let execution = ChatToolLoopResponse {
            action: Some("tool_call".to_string()),
            rationale: Some("The user asked about transcript evidence.".to_string()),
            tool_name: Some("search_library".to_string()),
            search_library_input: Some(SearchLibraryToolInput {
                query: Some("ownership model".to_string()),
                source: Some("transcript".to_string()),
                limit: Some(3),
            }),
            highlight_lookup_input: None,
            db_inspect_input: None,
            recent_library_activity_input: None,
        }
        .into_step_outcome()
        .expect("valid tool plan");

        let ToolLoopAction::ToolCall(PlannedChatToolCall::SearchLibrary(query)) = execution.action
        else {
            panic!("expected search tool call");
        };

        assert_eq!(query.query, "ownership model");
        assert_eq!(query.source_kind, Some(SearchSourceKind::Transcript));
        assert_eq!(query.limit, 3);
    }

    #[test]
    fn tool_loop_accepts_direct_search_library_action() {
        let execution = ChatToolLoopResponse {
            action: Some("search_library".to_string()),
            rationale: Some("Need library evidence.".to_string()),
            tool_name: None,
            search_library_input: Some(SearchLibraryToolInput {
                query: Some("saved highlights".to_string()),
                source: Some("all".to_string()),
                limit: Some(4),
            }),
            highlight_lookup_input: None,
            db_inspect_input: None,
            recent_library_activity_input: None,
        }
        .into_step_outcome()
        .expect("direct search_library action should parse");

        let ToolLoopAction::ToolCall(PlannedChatToolCall::SearchLibrary(query)) = execution.action
        else {
            panic!("expected search tool call");
        };

        assert_eq!(query.query, "saved highlights");
        assert_eq!(query.source_kind, None);
        assert_eq!(query.limit, 4);
    }

    #[test]
    fn tool_loop_accepts_direct_highlight_lookup_action() {
        let execution = ChatToolLoopResponse {
            action: Some("highlight_lookup".to_string()),
            rationale: Some("Need saved highlights.".to_string()),
            tool_name: None,
            search_library_input: None,
            highlight_lookup_input: Some(tools::HighlightLookupToolInput {
                query: Some("prototype-first".to_string()),
                video_title: None,
                limit: Some(3),
            }),
            db_inspect_input: None,
            recent_library_activity_input: None,
        }
        .into_step_outcome()
        .expect("direct highlight_lookup action should parse");

        let ToolLoopAction::ToolCall(PlannedChatToolCall::HighlightLookup(query)) =
            execution.action
        else {
            panic!("expected highlight lookup tool call");
        };

        assert_eq!(query.query.as_deref(), Some("prototype-first"));
        assert_eq!(query.video_title, None);
        assert_eq!(query.limit, 3);
    }

    #[test]
    fn tool_loop_accepts_direct_recent_library_activity_action() {
        let execution = ChatToolLoopResponse {
            action: Some("recent_library_activity".to_string()),
            rationale: Some("Need recent channel activity.".to_string()),
            tool_name: None,
            search_library_input: None,
            highlight_lookup_input: None,
            db_inspect_input: None,
            recent_library_activity_input: Some(tools::RecentLibraryActivityToolInput {
                scope: Some("channel".to_string()),
                channel_id: None,
                video_id: None,
                limit_videos: Some(6),
                include_summaries: Some(true),
                include_transcripts: Some(true),
            }),
        }
        .into_step_outcome()
        .expect("direct recent_library_activity action should parse");

        let ToolLoopAction::ToolCall(PlannedChatToolCall::RecentLibraryActivity(query)) =
            execution.action
        else {
            panic!("expected recent library activity tool call");
        };

        assert_eq!(query.scope, tools::RecentLibraryActivityScope::Channel);
        assert_eq!(query.limit_videos, 6);
        assert!(query.include_summaries);
        assert!(query.include_transcripts);
    }

    #[test]
    fn tool_loop_rejects_invalid_db_resource() {
        let error = ChatToolLoopResponse {
            action: Some("tool_call".to_string()),
            rationale: None,
            tool_name: Some("db_inspect".to_string()),
            search_library_input: None,
            highlight_lookup_input: None,
            db_inspect_input: Some(DbInspectToolInput {
                operation: Some("count".to_string()),
                resource: Some("search_sources".to_string()),
                limit: None,
                group_by: None,
            }),
            recent_library_activity_input: None,
        }
        .into_step_outcome()
        .expect_err("invalid db resource should fail");

        assert!(error.contains("unsupported db_inspect resource"));
    }

    #[test]
    fn tool_loop_can_choose_to_respond_without_tool() {
        let outcome = ChatToolLoopResponse {
            action: Some("respond".to_string()),
            rationale: Some("This is just a greeting.".to_string()),
            tool_name: None,
            search_library_input: None,
            highlight_lookup_input: None,
            db_inspect_input: None,
            recent_library_activity_input: None,
        }
        .into_step_outcome()
        .expect("respond action should be accepted");

        assert!(matches!(outcome.action, ToolLoopAction::Respond));
        assert_eq!(
            outcome.rationale.as_deref(),
            Some("This is just a greeting.")
        );
    }

    #[test]
    fn direct_recent_tool_call_prefers_channel_scope_only() {
        let scope = tools::MentionScope {
            cleaned_prompt: "What is HealthyGamerGG doing lately?".to_string(),
            channel_focus_ids: vec!["chan_1".to_string()],
            video_focus_ids: Vec::new(),
            channel_names: vec!["HealthyGamerGG".to_string()],
            video_titles: Vec::new(),
        };

        let Some(PlannedChatToolCall::RecentLibraryActivity(query)) =
            maybe_direct_recent_activity_tool_call("What is HealthyGamerGG doing lately?", &scope)
        else {
            panic!("expected recent activity tool call");
        };

        assert_eq!(query.channel_id.as_deref(), Some("chan_1"));
    }

    #[tokio::test]
    async fn build_answer_grounding_context_returns_cancelled_when_cancelled_before_synthesis() {
        let service = ChatService::new(OllamaCore::new("://invalid-url", "qwen3:8b"));
        let active_chat = ActiveChatHandle::new();
        active_chat.cancel();

        let mut plan = ChatRetrievalPlan::fallback("compare the channels", None);
        plan.intent = ChatQueryIntent::Pattern;

        let result = service
            .build_answer_grounding_context(
                "conv_cancelled",
                "compare the channels",
                &plan,
                &[RetrievedChatSource {
                    source: ChatSource {
                        video_id: "vid_1".to_string(),
                        channel_id: "chan_1".to_string(),
                        channel_name: "Channel One".to_string(),
                        video_title: "Video One".to_string(),
                        source_kind: SearchSourceKind::Transcript,
                        section_title: Some("Intro".to_string()),
                        snippet: "Important supporting excerpt".to_string(),
                        score: 1.0,
                        chunk_id: "chunk_1".to_string(),
                        retrieval_pass: Some(1),
                    },
                    context_text: "Important supporting excerpt with extra context.".to_string(),
                }],
                &active_chat,
            )
            .await;

        assert_eq!(result, Err("cancelled".to_string()));
    }

    #[test]
    fn cancel_marks_handle_cancelled_without_existing_receivers() {
        let active_chat = ActiveChatHandle::new();
        active_chat.cancel();

        assert!(active_chat.is_cancelled());
    }
}
