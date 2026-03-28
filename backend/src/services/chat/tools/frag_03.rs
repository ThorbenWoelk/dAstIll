#[cfg(test)]
mod tests {
    use super::{
        DbGroupBy, DbInspectOperation, DbInspectQuery, DbInspectTarget, DbInspectToolInput,
        HighlightLookupQuery, HighlightLookupToolInput, RecentLibraryActivityScope,
        RecentLibraryActivityToolInput, SearchLibraryToolInput, build_db_inspect_query,
        build_highlight_lookup_query, build_recent_library_activity_query,
        build_search_library_query, describe_db_inspect_query, describe_highlight_lookup_query,
        resolve_mention_scope_from_catalog,
    };
    use crate::models::{Channel, ContentStatus, Video};
    use crate::services::search::SearchSourceKind;

    #[test]
    fn builds_count_query_from_valid_tool_request() {
        let query = build_db_inspect_query(
            Some("db_inspect"),
            Some(DbInspectToolInput {
                operation: Some("count".to_string()),
                resource: Some("summaries".to_string()),
                limit: None,
                group_by: None,
            }),
        )
        .expect("valid tool request")
        .expect("query should be built");

        assert_eq!(query.operation, DbInspectOperation::Count);
        assert_eq!(query.target, DbInspectTarget::Summaries);
        assert_eq!(query.limit, 5);
    }

    #[test]
    fn clamps_list_limit_from_tool_request() {
        let query = build_db_inspect_query(
            Some("db_inspect"),
            Some(DbInspectToolInput {
                operation: Some("list".to_string()),
                resource: Some("videos".to_string()),
                limit: Some(99),
                group_by: None,
            }),
        )
        .expect("valid tool request")
        .expect("query should be built");

        assert_eq!(query.operation, DbInspectOperation::List);
        assert_eq!(query.target, DbInspectTarget::Videos);
        assert_eq!(query.limit, 10);
    }

    #[test]
    fn rejects_unknown_tool_name() {
        let error = build_db_inspect_query(
            Some("search"),
            Some(DbInspectToolInput {
                operation: Some("count".to_string()),
                resource: Some("summaries".to_string()),
                limit: None,
                group_by: None,
            }),
        )
        .expect_err("unknown tool should be rejected");

        assert!(error.contains("unsupported tool"));
    }

    #[test]
    fn rejects_unknown_resource() {
        let error = build_db_inspect_query(
            Some("db_inspect"),
            Some(DbInspectToolInput {
                operation: Some("count".to_string()),
                resource: Some("search_sources".to_string()),
                limit: None,
                group_by: None,
            }),
        )
        .expect_err("unknown resource should be rejected");

        assert!(error.contains("unsupported db_inspect resource"));
    }

    #[test]
    fn describe_query_is_human_readable() {
        let description = describe_db_inspect_query(DbInspectQuery {
            operation: DbInspectOperation::List,
            target: DbInspectTarget::Channels,
            limit: 5,
            group_by: None,
        });
        assert_eq!(description, "List up to 5 channels from the database");
    }

    #[test]
    fn builds_breakdown_query_from_valid_tool_request() {
        let query = build_db_inspect_query(
            Some("db_inspect"),
            Some(DbInspectToolInput {
                operation: Some("breakdown".to_string()),
                resource: Some("summaries".to_string()),
                limit: None,
                group_by: Some("channel".to_string()),
            }),
        )
        .expect("valid tool request")
        .expect("query should be built");

        assert_eq!(query.operation, DbInspectOperation::Breakdown);
        assert_eq!(query.target, DbInspectTarget::Summaries);
        assert_eq!(query.group_by, Some(DbGroupBy::Channel));
    }

    #[test]
    fn rejects_breakdown_without_group_by() {
        let error = build_db_inspect_query(
            Some("db_inspect"),
            Some(DbInspectToolInput {
                operation: Some("breakdown".to_string()),
                resource: Some("summaries".to_string()),
                limit: None,
                group_by: None,
            }),
        )
        .expect_err("breakdown without group_by should be rejected");

        assert!(error.contains("requires group_by"));
    }

    #[test]
    fn builds_search_library_query_from_valid_tool_request() {
        let query = build_search_library_query(
            Some("search_library"),
            Some(SearchLibraryToolInput {
                query: Some("ownership model".to_string()),
                source: Some("summary".to_string()),
                limit: Some(4),
            }),
        )
        .expect("valid tool request")
        .expect("query should be built");

        assert_eq!(query.query, "ownership model");
        assert_eq!(query.source_kind, Some(SearchSourceKind::Summary));
        assert_eq!(query.limit, 4);
    }

    #[test]
    fn search_library_defaults_to_all_sources_and_clamps_limit() {
        let query = build_search_library_query(
            Some("search_library"),
            Some(SearchLibraryToolInput {
                query: Some("rust vector search".to_string()),
                source: None,
                limit: Some(99),
            }),
        )
        .expect("valid tool request")
        .expect("query should be built");

        assert_eq!(query.source_kind, None);
        assert_eq!(query.limit, 24);
    }

    #[test]
    fn search_library_rejects_unknown_source() {
        let error = build_search_library_query(
            Some("search_library"),
            Some(SearchLibraryToolInput {
                query: Some("rust vector search".to_string()),
                source: Some("video".to_string()),
                limit: None,
            }),
        )
        .expect_err("invalid source should be rejected");

        assert!(error.contains("unsupported search_library source"));
    }

    #[test]
    fn builds_highlight_lookup_query_from_valid_request() {
        let query = build_highlight_lookup_query(
            Some("highlight_lookup"),
            Some(HighlightLookupToolInput {
                query: Some("prototype-first".to_string()),
                video_title: Some("Theo".to_string()),
                limit: Some(4),
            }),
        )
        .expect("valid request")
        .expect("query should be built");

        assert_eq!(query.query.as_deref(), Some("prototype-first"));
        assert_eq!(query.video_title.as_deref(), Some("Theo"));
        assert_eq!(query.limit, 4);
    }

    #[test]
    fn highlight_lookup_requires_query_or_video_title() {
        let error = build_highlight_lookup_query(
            Some("highlight_lookup"),
            Some(HighlightLookupToolInput {
                query: Some("   ".to_string()),
                video_title: None,
                limit: None,
            }),
        )
        .expect_err("empty highlight lookup request should fail");

        assert!(error.contains("requires at least one of query or video_title"));
    }

    #[test]
    fn highlight_lookup_description_is_human_readable() {
        let description = describe_highlight_lookup_query(&HighlightLookupQuery {
            query: Some("agent".to_string()),
            video_title: None,
            limit: 5,
        });

        assert_eq!(description, "Look up saved highlights for query \"agent\"");
    }

    #[test]
    fn resolves_bare_channel_mentions_into_scope() {
        let channels = vec![sample_channel("chan_1", "Theo", Some("@theo"))];
        let videos = vec![sample_video("vid_1", "chan_1", "Vector Search Guide")];

        let scope = resolve_mention_scope_from_catalog(
            "What does @theo recommend for databases?",
            &channels,
            &videos,
        );

        assert_eq!(scope.channel_focus_ids, vec!["chan_1".to_string()]);
        assert_eq!(scope.channel_names, vec!["Theo".to_string()]);
        assert_eq!(scope.cleaned_prompt, "What does recommend for databases?");
    }

    #[test]
    fn resolves_quoted_video_mentions_into_scope() {
        let channels = vec![sample_channel("chan_1", "Theo", Some("@theo"))];
        let videos = vec![sample_video("vid_1", "chan_1", "Rust Search Deep Dive")];

        let scope = resolve_mention_scope_from_catalog(
            "Summarize @\"Rust Search Deep Dive\" in three bullets",
            &channels,
            &videos,
        );

        assert_eq!(scope.video_focus_ids, vec!["vid_1".to_string()]);
        assert_eq!(scope.channel_focus_ids, vec!["chan_1".to_string()]);
        assert_eq!(
            scope.video_titles,
            vec!["Rust Search Deep Dive".to_string()]
        );
        assert_eq!(
            scope.prompt_for_retrieval("Summarize @\"Rust Search Deep Dive\" in three bullets"),
            "Summarize in three bullets \"Rust Search Deep Dive\""
        );
    }

    #[test]
    fn plus_mentions_scope_videos_only() {
        let channels = vec![sample_channel(
            "chan_1",
            "HealthyGamerGG",
            Some("@healthygamergg"),
        )];
        let videos = vec![sample_video(
            "vid_1",
            "chan_1",
            "Why Effort Alone Doesn’t Lead to Change",
        )];

        let scope = resolve_mention_scope_from_catalog(
            "Summarize +{Why Effort Alone Doesn’t Lead to Change}",
            &channels,
            &videos,
        );

        assert_eq!(scope.video_focus_ids, vec!["vid_1".to_string()]);
        assert!(scope.channel_names.is_empty());
        assert_eq!(
            scope.prompt_for_retrieval("Summarize +{Why Effort Alone Doesn’t Lead to Change}"),
            "Summarize \"Why Effort Alone Doesn’t Lead to Change\""
        );
    }

    #[test]
    fn plain_channel_reference_resolves_scope_when_unambiguous() {
        let channels = vec![
            sample_channel("chan_1", "HealthyGamerGG", Some("@healthygamergg")),
            sample_channel("chan_2", "Theo", Some("@theo")),
        ];
        let scope = resolve_mention_scope_from_catalog(
            "What is HealthyGamerGG doing lately?",
            &channels,
            &[],
        );

        assert_eq!(scope.channel_focus_ids, vec!["chan_1".to_string()]);
        assert_eq!(scope.channel_names, vec!["HealthyGamerGG".to_string()]);
        assert_eq!(
            scope.cleaned_prompt,
            "What is HealthyGamerGG doing lately?".to_string()
        );
    }

    #[test]
    fn plain_video_reference_resolves_scope_when_unambiguous() {
        let videos = vec![sample_video(
            "vid_1",
            "chan_1",
            "Why Effort Alone Doesn’t Lead to Change",
        )];
        let scope = resolve_mention_scope_from_catalog(
            "Summarize Why Effort Alone Doesn’t Lead to Change",
            &[],
            &videos,
        );

        assert_eq!(scope.video_focus_ids, vec!["vid_1".to_string()]);
        assert_eq!(
            scope.video_titles,
            vec!["Why Effort Alone Doesn’t Lead to Change".to_string()]
        );
    }

    #[test]
    fn builds_recent_library_activity_query_with_defaults() {
        let query = build_recent_library_activity_query(
            Some("recent_library_activity"),
            Some(RecentLibraryActivityToolInput {
                scope: Some("channel".to_string()),
                channel_id: None,
                video_id: None,
                limit_videos: None,
                include_summaries: None,
                include_transcripts: None,
            }),
        )
        .expect("valid tool request")
        .expect("query should be built");

        assert_eq!(query.scope, RecentLibraryActivityScope::Channel);
        assert_eq!(query.limit_videos, 6);
        assert!(query.include_summaries);
        assert!(query.include_transcripts);
    }

    fn sample_channel(id: &str, name: &str, handle: Option<&str>) -> Channel {
        Channel {
            id: id.to_string(),
            handle: handle.map(str::to_string),
            name: name.to_string(),
            thumbnail_url: None,
            added_at: chrono::Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        }
    }

    fn sample_video(id: &str, channel_id: &str, title: &str) -> Video {
        Video {
            id: id.to_string(),
            channel_id: channel_id.to_string(),
            title: title.to_string(),
            thumbnail_url: None,
            published_at: chrono::Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        }
    }
}
