async fn finalize_title_generation(
    state: &AppState,
    conversation_scope_id: &str,
    conversation_id: &str,
    title: Option<String>,
) -> Result<(), String> {
    let _lock = state.chat_store_lock.lock().await;
    let conn = state.db.connect();
    let Some(mut conversation) =
        db::get_conversation_for_scope(&conn, conversation_scope_id, conversation_id)
            .await
            .map_err(|error| error.to_string())?
    else {
        return Ok(());
    };

    if conversation.title_status != ChatTitleStatus::Generating {
        return Ok(());
    }

    if let Some(title) = title.and_then(|value| trim_to_option(&value)) {
        conversation.title = Some(limit_text(&title, CHAT_TITLE_MAX_CHARS));
        conversation.title_status = ChatTitleStatus::Ready;
    } else {
        conversation.title_status = if conversation.title.is_some() {
            ChatTitleStatus::Ready
        } else {
            ChatTitleStatus::Idle
        };
    }
    conversation.updated_at = Utc::now();
    db::upsert_conversation_for_scope(&conn, conversation_scope_id, &conversation)
        .await
        .map_err(|error| error.to_string())
}

async fn persist_assistant_message(
    state: &AppState,
    conversation_scope_id: &str,
    conversation_id: &str,
    message: &ChatMessage,
) -> Result<(), String> {
    let _lock = state.chat_store_lock.lock().await;
    let conn = state.db.connect();
    let Some(mut conversation) =
        db::get_conversation_for_scope(&conn, conversation_scope_id, conversation_id)
            .await
            .map_err(|error| error.to_string())?
    else {
        return Err("Conversation not found".to_string());
    };

    if message.status == ChatMessageStatus::Failed
        && conversation
            .messages
            .iter()
            .any(|candidate| candidate.id == message.id)
    {
        return Ok(());
    }

    conversation.messages.push(message.clone());
    conversation.updated_at = Utc::now();
    db::upsert_conversation_for_scope(&conn, conversation_scope_id, &conversation)
        .await
        .map_err(|error| error.to_string())
}

fn parse_json_response<T: for<'de> Deserialize<'de>>(response: &str) -> Result<T, String> {
    serde_json::from_str(response)
        .or_else(|_| {
            let start = response
                .find('{')
                .ok_or_else(|| "missing JSON object".to_string())?;
            let end = response
                .rfind('}')
                .ok_or_else(|| "missing JSON object end".to_string())?;
            serde_json::from_str(&response[start..=end]).map_err(|error| error.to_string())
        })
        .map_err(|error| error.to_string())
}

fn sanitize_generated_title(input: &str) -> String {
    limit_text(
        input
            .trim()
            .trim_matches('"')
            .trim_matches('“')
            .trim_matches('”')
            .trim_matches('.')
            .trim(),
        CHAT_TITLE_MAX_CHARS,
    )
}

fn trim_to_option(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn generate_chat_id(prefix: &str) -> String {
    format!(
        "{prefix}_{}_{}",
        Utc::now().timestamp_millis(),
        NEXT_CHAT_ID.fetch_add(1, Ordering::Relaxed)
    )
}

fn format_conversation_for_planner(
    conversation: &ChatConversation,
    current_prompt: &str,
) -> String {
    let mut lines = Vec::new();
    for message in conversation.messages.iter() {
        let label = match message.role {
            ChatRole::User => "User",
            ChatRole::Assistant => "Assistant",
            ChatRole::System => continue,
        };
        lines.push(format!("{label}: {}", message.content.trim()));
    }
    let transcript = lines.join("\n");
    let capped = limit_text(&transcript, CHAT_PLANNER_CONVERSATION_MAX_CHARS);
    format!(
        "RECENT CONVERSATION:\n{capped}\n\nCURRENT USER MESSAGE:\n{}",
        current_prompt.trim()
    )
}

fn format_tool_loop_input(
    conversation: &ChatConversation,
    current_prompt: &str,
    tool_outputs: &[ToolEvidenceRecord],
    gathered_sources: &[RetrievedChatSource],
) -> String {
    let mut tool_lines = Vec::new();
    for (index, record) in tool_outputs.iter().enumerate() {
        tool_lines.push(format!(
            "Tool {}: {}\n{}",
            index + 1,
            record.summary,
            record.output.trim()
        ));
    }
    if !gathered_sources.is_empty() {
        tool_lines.push(format!(
            "Retrieved excerpt count: {}",
            gathered_sources.len()
        ));
        for source in gathered_sources.iter().take(6) {
            tool_lines.push(format!(
                "- [{}] {} / {} / {}",
                source.source.source_kind.as_str(),
                source.source.channel_name,
                source.source.video_title,
                source.source.snippet
            ));
        }
    }

    let tool_results = if tool_lines.is_empty() {
        "none".to_string()
    } else {
        tool_lines.join("\n\n")
    };

    format!(
        "{}\n\nTOOL RESULTS FROM THIS TURN:\n{}",
        format_conversation_for_planner(conversation, current_prompt),
        limit_text(&tool_results, CHAT_PLANNER_CONVERSATION_MAX_CHARS)
    )
}

fn merge_channel_focus_ids(primary: &[String], secondary: &[String]) -> Vec<String> {
    let mut merged = primary.to_vec();
    for channel_id in secondary {
        if !merged.iter().any(|existing| existing == channel_id) {
            merged.push(channel_id.clone());
        }
    }
    merged
}

fn merge_mention_scope(
    primary: Option<&tools::MentionScope>,
    secondary: &tools::MentionScope,
) -> tools::MentionScope {
    let mut merged = primary.cloned().unwrap_or_default();
    if merged.cleaned_prompt.is_empty() {
        merged.cleaned_prompt = secondary.cleaned_prompt.clone();
    }
    merged.channel_focus_ids =
        merge_channel_focus_ids(&merged.channel_focus_ids, &secondary.channel_focus_ids);
    merged.video_focus_ids =
        merge_channel_focus_ids(&merged.video_focus_ids, &secondary.video_focus_ids);
    merged.channel_names = merge_channel_focus_ids(&merged.channel_names, &secondary.channel_names);
    merged.video_titles = merge_channel_focus_ids(&merged.video_titles, &secondary.video_titles);
    merged
}

fn filter_batches_to_video_scope(
    mut batches: Vec<Vec<SearchCandidate>>,
    video_focus_ids: &[String],
) -> Vec<Vec<SearchCandidate>> {
    if video_focus_ids.is_empty() {
        return batches;
    }

    let focus = video_focus_ids.iter().collect::<HashSet<_>>();
    let has_scoped_match = batches.iter().any(|batch| {
        batch
            .iter()
            .any(|candidate| focus.contains(&candidate.video_id))
    });
    if !has_scoped_match {
        return batches;
    }

    for batch in &mut batches {
        batch.retain(|candidate| focus.contains(&candidate.video_id));
    }
    batches
}

fn direct_video_lookup_target<'a>(
    scope: &'a tools::MentionScope,
    query: &tools::SearchLibraryQuery,
) -> Option<&'a str> {
    let [video_id] = scope.video_focus_ids.as_slice() else {
        return None;
    };
    if !is_direct_video_lookup_request(&scope.cleaned_prompt, &query.query) {
        return None;
    }
    Some(video_id.as_str())
}

fn is_direct_video_lookup_request(cleaned_prompt: &str, search_query: &str) -> bool {
    let meaningful_terms = crate::search_query::meaningful_search_terms(cleaned_prompt);
    if meaningful_terms.is_empty() {
        return true;
    }

    let generic_terms = HashSet::from([
        "answer",
        "explain",
        "give",
        "me",
        "read",
        "recap",
        "show",
        "summarize",
        "summary",
        "tell",
        "transcript",
        "video",
        "watch",
    ]);
    let title_terms = crate::search_query::meaningful_search_terms(search_query)
        .into_iter()
        .collect::<HashSet<_>>();

    meaningful_terms
        .into_iter()
        .all(|term| generic_terms.contains(term.as_str()) || title_terms.contains(&term))
}

fn maybe_direct_recent_activity_tool_call(
    prompt: &str,
    scope: &tools::MentionScope,
) -> Option<PlannedChatToolCall> {
    if !is_recent_activity_query(prompt) || is_explicit_realtime_status_query(prompt) {
        return None;
    }
    if !scope.video_focus_ids.is_empty() {
        return None;
    }
    let channel_id = scope.channel_focus_ids.first()?.clone();
    Some(PlannedChatToolCall::RecentLibraryActivity(
        tools::RecentLibraryActivityQuery {
            scope: tools::RecentLibraryActivityScope::Channel,
            channel_id: Some(channel_id),
            video_id: None,
            limit_videos: CHAT_RECENT_ACTIVITY_VIDEO_LIMIT,
            include_summaries: true,
            include_transcripts: true,
        },
    ))
}

fn apply_recent_activity_scope(
    mut query: tools::RecentLibraryActivityQuery,
    scope: &tools::MentionScope,
) -> tools::RecentLibraryActivityQuery {
    if query.channel_id.is_none()
        && matches!(query.scope, tools::RecentLibraryActivityScope::Channel)
    {
        query.channel_id = scope.channel_focus_ids.first().cloned();
    }
    if query.video_id.is_none() {
        query.video_id = scope.video_focus_ids.first().cloned();
    }
    query
}

async fn load_direct_video_sources(
    store: &db::Store,
    video_id: &str,
    source_kind: Option<crate::services::search::SearchSourceKind>,
) -> Result<Vec<RetrievedChatSource>, String> {
    let kinds = match source_kind {
        Some(kind) => vec![kind],
        None => vec![
            crate::services::search::SearchSourceKind::Summary,
            crate::services::search::SearchSourceKind::Transcript,
        ],
    };

    let mut sources = Vec::new();
    for kind in kinds {
        let Some(material) = db::load_search_material(store, video_id, kind)
            .await
            .map_err(|error| error.to_string())?
        else {
            continue;
        };
        sources.push(retrieved_source_from_search_material(material));
    }

    Ok(sources)
}

fn retrieved_source_from_search_material(material: db::SearchMaterial) -> RetrievedChatSource {
    let section_title = match material.source_kind {
        crate::services::search::SearchSourceKind::Summary => Some("Full summary".to_string()),
        crate::services::search::SearchSourceKind::Transcript => {
            Some("Full transcript".to_string())
        }
    };
    let source_kind = material.source_kind;
    let video_id = material.video_id;

    RetrievedChatSource {
        source: ChatSource {
            chunk_id: format!("{video_id}_direct_{}", source_kind.as_str()),
            video_id,
            channel_id: material.channel_id,
            channel_name: material.channel_name,
            video_title: material.video_title,
            source_kind,
            section_title,
            snippet: crate::services::search::truncate_chunk_for_display(&material.content),
            score: 1.0,
            retrieval_pass: Some(1),
        },
        context_text: limit_text(material.content.trim(), CHAT_CONTEXT_MAX_CHARS),
    }
}

fn describe_search_library_query(query: tools::SearchLibraryQuery) -> String {
    let source = match query.source_kind {
        Some(crate::services::search::SearchSourceKind::Summary) => Some("summaries"),
        Some(crate::services::search::SearchSourceKind::Transcript) => Some("transcripts"),
        None => None,
    };
    let source_label = source.unwrap_or("all sources");
    format!(
        "Search the library for \"{}\" in {} (limit {})",
        query.query, source_label, query.limit
    )
}

fn describe_highlight_lookup_query(query: tools::HighlightLookupQuery) -> String {
    match (&query.query, &query.video_title) {
        (Some(query_text), Some(video_title)) => format!(
            "Look up saved highlights for query \"{}\" in videos matching \"{}\" (limit {})",
            query_text, video_title, query.limit
        ),
        (Some(query_text), None) => format!(
            "Look up saved highlights for query \"{}\" (limit {})",
            query_text, query.limit
        ),
        (None, Some(video_title)) => format!(
            "Look up saved highlights in videos matching \"{}\" (limit {})",
            video_title, query.limit
        ),
        (None, None) => format!("Look up saved highlights (limit {})", query.limit),
    }
}

fn describe_recent_library_activity_query(query: tools::RecentLibraryActivityQuery) -> String {
    match query.scope {
        tools::RecentLibraryActivityScope::Channel => format!(
            "Review recent library activity for a scoped channel (latest {} videos)",
            query.limit_videos
        ),
        tools::RecentLibraryActivityScope::Video => format!(
            "Review recent library activity around a scoped video (latest {} videos)",
            query.limit_videos
        ),
        tools::RecentLibraryActivityScope::Library => format!(
            "Review recent library activity across the library (latest {} videos)",
            query.limit_videos
        ),
    }
}
