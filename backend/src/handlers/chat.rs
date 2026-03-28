use std::convert::Infallible;

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Sse, sse::Event},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    audit, db,
    models::{
        ChatConversation, ChatMessage, ChatRole, ChatTitleStatus, CreateConversationRequest,
        SendChatMessageRequest, UpdateConversationRequest,
    },
    security::{AccessContext, can_access_video},
    services::{
        SpawnReplyJob,
        chat::{default_chat_cloud_model_id, is_chat_cloud_model_choice},
    },
    state::AppState,
};

use super::{map_db_err, require_present, validate_nonempty};

const CHAT_SUGGESTION_LIMIT_DEFAULT: usize = 8;
const CHAT_SUGGESTION_LIMIT_MAX: usize = 12;

#[derive(Debug, Deserialize)]
pub struct ChatSuggestionQuery {
    #[serde(default)]
    q: String,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatSuggestionItem {
    kind: &'static str,
    id: String,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<String>,
}

fn conversation_scope_id(access_context: &AccessContext) -> &str {
    access_context.user_id.as_deref().unwrap_or("anonymous")
}

pub async fn channel_suggestions(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Query(query): Query<ChatSuggestionQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let channels = match access_context.user_id.as_deref() {
        Some(user_id) => db::list_user_channels_with_virtual_others(&state.db, user_id)
            .await
            .map_err(map_db_err)?,
        None => {
            let mut channels = Vec::new();
            for channel_id in &access_context.allowed_channel_ids {
                if let Some(channel) = db::get_channel(&state.db, channel_id)
                    .await
                    .map_err(map_db_err)?
                {
                    channels.push(channel);
                }
            }
            channels
        }
    };
    Ok(Json(rank_channel_suggestions(
        &channels,
        &query.q,
        query
            .limit
            .unwrap_or(CHAT_SUGGESTION_LIMIT_DEFAULT)
            .clamp(1, CHAT_SUGGESTION_LIMIT_MAX),
    )))
}

pub async fn video_suggestions(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Query(query): Query<ChatSuggestionQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let videos = db::load_all_videos(&state.db).await.map_err(map_db_err)?;
    let channels = match access_context.user_id.as_deref() {
        Some(user_id) => db::list_user_channels_with_virtual_others(&state.db, user_id)
            .await
            .map_err(map_db_err)?,
        None => {
            let mut channels = Vec::new();
            for channel_id in &access_context.allowed_channel_ids {
                if let Some(channel) = db::get_channel(&state.db, channel_id)
                    .await
                    .map_err(map_db_err)?
                {
                    channels.push(channel);
                }
            }
            channels
        }
    };
    let videos = videos
        .into_iter()
        .filter(|video| {
            can_access_video(&access_context, &video.id, &video.channel_id)
                || video.channel_id == crate::models::OTHERS_CHANNEL_ID
        })
        .collect::<Vec<_>>();
    Ok(Json(rank_video_suggestions(
        &videos,
        &channels,
        &query.q,
        query
            .limit
            .unwrap_or(CHAT_SUGGESTION_LIMIT_DEFAULT)
            .clamp(1, CHAT_SUGGESTION_LIMIT_MAX),
    )))
}

pub async fn chat_client_config(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.chat.chat_client_config())
}

pub async fn list_conversations(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conversations =
        db::list_conversations_for_scope(&state.db, conversation_scope_id(&access_context))
            .await
            .map_err(map_db_err)?;
    Ok(Json(conversations))
}

pub async fn create_conversation(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Json(payload): Json<CreateConversationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut conversation = state.chat.create_conversation(payload.title.clone());
    mark_manual_title_on_create(&mut conversation);

    let scope_id = conversation_scope_id(&access_context);
    let _lock = state.chat_store_lock.lock().await;
    db::upsert_conversation_for_scope(&state.db, scope_id, &conversation)
        .await
        .map_err(map_db_err)?;
    audit::log_chat_conversation_create(
        scope_id,
        &conversation.id,
        conversation.title.as_deref().map(str::len).unwrap_or(0),
    );
    Ok((StatusCode::CREATED, Json(conversation)))
}

pub async fn get_conversation(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(conversation_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conversation = db::get_conversation_for_scope(
        &state.db,
        conversation_scope_id(&access_context),
        &conversation_id,
    )
    .await
    .map_err(map_db_err)
    .and_then(|opt| require_present(opt, "Conversation not found"))?;
    Ok(Json(conversation))
}

pub async fn update_conversation(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(conversation_id): Path<String>,
    Json(payload): Json<UpdateConversationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let title = validate_nonempty(&payload.title, "Conversation title must not be empty")?;
    let scope_id = conversation_scope_id(&access_context);

    let _lock = state.chat_store_lock.lock().await;
    let Some(mut conversation) =
        db::get_conversation_for_scope(&state.db, scope_id, &conversation_id)
            .await
            .map_err(map_db_err)?
    else {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    };
    let old_title_len = conversation.title.as_deref().map(str::len).unwrap_or(0);
    let new_title_len = title.len();
    apply_manual_conversation_title(&mut conversation, title, Utc::now());
    db::upsert_conversation_for_scope(&state.db, scope_id, &conversation)
        .await
        .map_err(map_db_err)?;
    audit::log_chat_conversation_update(scope_id, &conversation_id, old_title_len, new_title_len);
    Ok(Json(conversation))
}

pub async fn delete_conversation(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(conversation_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(active_chat) = state.active_chats.lock().await.remove(&conversation_id) {
        active_chat.cancel();
    }

    let scope_id = conversation_scope_id(&access_context);
    let _lock = state.chat_store_lock.lock().await;
    db::delete_conversation_for_scope(&state.db, scope_id, &conversation_id)
        .await
        .map_err(map_db_err)?;
    audit::log_chat_conversation_delete(scope_id, &conversation_id);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_all_conversations(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    for (_, active_chat) in state.active_chats.lock().await.drain() {
        active_chat.cancel();
    }

    let scope_id = conversation_scope_id(&access_context);
    let _lock = state.chat_store_lock.lock().await;
    db::delete_all_conversations_for_scope(&state.db, scope_id)
        .await
        .map_err(map_db_err)?;
    audit::log_chat_conversations_delete_all(scope_id);

    for (_, active_chat) in state.active_chats.lock().await.drain() {
        active_chat.cancel();
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn send_message(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(conversation_id): Path<String>,
    Json(payload): Json<SendChatMessageRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)>
{
    let prompt = payload.content.trim();
    if prompt.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Message content must not be empty".to_string(),
        ));
    }

    let active_chat = {
        let mut active_chats = state.active_chats.lock().await;
        if active_chats.contains_key(&conversation_id) {
            return Err((
                StatusCode::CONFLICT,
                "Conversation already has an active response".to_string(),
            ));
        }
        let handle = crate::services::ActiveChatHandle::new();
        active_chats.insert(conversation_id.clone(), handle.clone());
        handle
    };

    let reply_model = match payload
        .model
        .as_deref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        Some(id) if is_chat_cloud_model_choice(id) => id.to_string(),
        Some(_) => {
            state.active_chats.lock().await.remove(&conversation_id);
            return Err((
                StatusCode::BAD_REQUEST,
                "Unknown chat model. Pick a cloud model from the selector.".to_string(),
            ));
        }
        None => default_chat_cloud_model_id(state.chat.model()),
    };

    let maybe_conversation =
        store_user_message(&state, &access_context, &conversation_id, prompt).await;
    let (conversation, should_auto_name) = match maybe_conversation {
        Ok(value) => value,
        Err(error) => {
            state.active_chats.lock().await.remove(&conversation_id);
            return Err(error);
        }
    };

    state.chat.spawn_reply(SpawnReplyJob {
        state: state.clone(),
        conversation,
        conversation_scope_id: conversation_scope_id(&access_context).to_string(),
        prompt: prompt.to_string(),
        should_auto_name,
        deep_research: payload.deep_research,
        reply_model,
        active_chat: active_chat.clone(),
    });

    Ok(sse_response(active_chat).await)
}

pub async fn reconnect_stream(
    State(state): State<AppState>,
    Extension(_access_context): Extension<AccessContext>,
    Path(conversation_id): Path<String>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)>
{
    let active_chat = state
        .active_chats
        .lock()
        .await
        .get(&conversation_id)
        .cloned()
        .ok_or((StatusCode::NOT_FOUND, "Active chat not found".to_string()))?;
    Ok(sse_response(active_chat).await)
}

pub async fn cancel_message(
    State(state): State<AppState>,
    Extension(_access_context): Extension<AccessContext>,
    Path(conversation_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let active_chat = state
        .active_chats
        .lock()
        .await
        .get(&conversation_id)
        .cloned()
        .ok_or((StatusCode::NOT_FOUND, "Active chat not found".to_string()))?;
    active_chat.cancel();
    Ok(StatusCode::ACCEPTED)
}

async fn sse_response(
    active_chat: crate::services::ActiveChatHandle,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    Sse::new(active_chat.into_sse_stream().await).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}

async fn store_user_message(
    state: &AppState,
    access_context: &AccessContext,
    conversation_id: &str,
    prompt: &str,
) -> Result<(ChatConversation, bool), (StatusCode, String)> {
    let scope_id = conversation_scope_id(access_context);
    let _lock = state.chat_store_lock.lock().await;
    let Some(mut conversation) =
        db::get_conversation_for_scope(&state.db, scope_id, conversation_id)
            .await
            .map_err(map_db_err)?
    else {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    };

    let user_message = state.chat.build_user_message(prompt);
    let provisional_title = state.chat.build_provisional_title(prompt);
    let should_auto_name = apply_user_message_to_conversation(
        &mut conversation,
        user_message,
        provisional_title,
        Utc::now(),
    );
    db::upsert_conversation_for_scope(&state.db, scope_id, &conversation)
        .await
        .map_err(map_db_err)?;
    Ok((conversation, should_auto_name))
}

fn apply_user_message_to_conversation(
    conversation: &mut ChatConversation,
    user_message: ChatMessage,
    provisional_title: Option<String>,
    updated_at: chrono::DateTime<Utc>,
) -> bool {
    conversation.messages.push(user_message);

    let user_message_count = conversation
        .messages
        .iter()
        .filter(|message| message.role == ChatRole::User)
        .count();
    let should_auto_name =
        user_message_count == 1 && conversation.title_status != ChatTitleStatus::Manual;

    if should_auto_name {
        if conversation.title.is_none() {
            conversation.title = provisional_title;
        }
        conversation.title_status = ChatTitleStatus::Generating;
    }

    conversation.updated_at = updated_at;
    should_auto_name
}

fn rank_channel_suggestions(
    channels: &[crate::models::Channel],
    query: &str,
    limit: usize,
) -> Vec<ChatSuggestionItem> {
    let needle = normalize_suggestion_query(query);
    let mut items = channels
        .iter()
        .filter_map(|channel| {
            let name_key = normalize_suggestion_query(&channel.name);
            let handle_key = channel
                .handle
                .as_deref()
                .map(|value| normalize_suggestion_query(value.trim_start_matches('@')));

            let score = score_channel_candidate(&needle, &name_key, handle_key.as_deref())?;
            Some((score, channel))
        })
        .collect::<Vec<_>>();

    items.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.name.cmp(&right.1.name))
            .then_with(|| left.1.id.cmp(&right.1.id))
    });

    items
        .into_iter()
        .take(limit)
        .map(|(_, channel)| ChatSuggestionItem {
            kind: "channel",
            id: channel.id.clone(),
            label: channel.name.clone(),
            subtitle: channel.handle.clone(),
        })
        .collect()
}

fn rank_video_suggestions(
    videos: &[crate::models::Video],
    channels: &[crate::models::Channel],
    query: &str,
    limit: usize,
) -> Vec<ChatSuggestionItem> {
    let needle = normalize_suggestion_query(query);
    let channel_names = channels
        .iter()
        .map(|channel| (channel.id.as_str(), channel.name.as_str()))
        .collect::<std::collections::HashMap<_, _>>();

    let mut items = videos
        .iter()
        .filter_map(|video| {
            let title_key = normalize_suggestion_query(&video.title);
            let score = score_text_candidate(&needle, &title_key)?;
            Some((score, video))
        })
        .collect::<Vec<_>>();

    items.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.published_at.cmp(&left.1.published_at))
            .then_with(|| left.1.title.cmp(&right.1.title))
            .then_with(|| left.1.id.cmp(&right.1.id))
    });

    items
        .into_iter()
        .take(limit)
        .map(|(_, video)| ChatSuggestionItem {
            kind: "video",
            id: video.id.clone(),
            label: video.title.clone(),
            subtitle: channel_names
                .get(video.channel_id.as_str())
                .map(|value| (*value).to_string()),
        })
        .collect()
}

fn score_channel_candidate(needle: &str, name_key: &str, handle_key: Option<&str>) -> Option<u8> {
    if needle.is_empty() {
        return Some(1);
    }
    let handle_score = handle_key.and_then(|value| score_text_candidate(needle, value));
    let name_score = score_text_candidate(needle, name_key);
    match (handle_score, name_score) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(score), None) | (None, Some(score)) => Some(score),
        (None, None) => None,
    }
}

fn score_text_candidate(needle: &str, haystack: &str) -> Option<u8> {
    if needle.is_empty() {
        return Some(1);
    }
    if haystack == needle {
        return Some(5);
    }
    if haystack.starts_with(needle) {
        return Some(4);
    }
    if haystack
        .split_whitespace()
        .any(|word| word.starts_with(needle))
    {
        return Some(3);
    }
    if haystack.contains(needle) {
        return Some(2);
    }
    None
}

fn normalize_suggestion_query(input: &str) -> String {
    input
        .trim()
        .trim_start_matches('@')
        .trim_start_matches('+')
        .trim_matches('"')
        .trim_matches('{')
        .trim_matches('}')
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn mark_manual_title_on_create(conversation: &mut ChatConversation) {
    if conversation.title.is_some() {
        conversation.title_status = ChatTitleStatus::Manual;
    }
}

fn apply_manual_conversation_title(
    conversation: &mut ChatConversation,
    title: &str,
    updated_at: chrono::DateTime<Utc>,
) {
    conversation.title = Some(title.to_string());
    conversation.title_status = ChatTitleStatus::Manual;
    conversation.updated_at = updated_at;
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::{
        apply_manual_conversation_title, apply_user_message_to_conversation,
        mark_manual_title_on_create, rank_channel_suggestions, rank_video_suggestions,
    };
    use crate::handlers::validate_nonempty;
    use crate::models::{
        Channel, ChatConversation, ChatMessage, ChatMessageStatus, ChatRole, ChatTitleStatus,
        ContentStatus, Video,
    };

    fn sample_conversation(title: Option<&str>, title_status: ChatTitleStatus) -> ChatConversation {
        let created_at = Utc::now() - Duration::minutes(5);
        ChatConversation {
            id: "conv-123".to_string(),
            title: title.map(str::to_string),
            title_status,
            created_at,
            updated_at: created_at,
            messages: Vec::new(),
        }
    }

    fn sample_message(role: ChatRole, content: &str) -> ChatMessage {
        ChatMessage {
            id: format!("msg-{content}"),
            role,
            content: content.to_string(),
            sources: Vec::new(),
            status: ChatMessageStatus::Completed,
            created_at: Utc::now(),
            model: None,
            prompt_tokens: None,
            completion_tokens: None,
            total_duration_ns: None,
        }
    }

    #[test]
    fn first_user_message_sets_provisional_title_and_generating_status() {
        let mut conversation = sample_conversation(None, ChatTitleStatus::Idle);
        let updated_at = Utc::now();

        let should_auto_name = apply_user_message_to_conversation(
            &mut conversation,
            sample_message(ChatRole::User, "Find the best Rust video"),
            Some("Find the best Rust video".to_string()),
            updated_at,
        );

        assert!(should_auto_name);
        assert_eq!(
            conversation.title.as_deref(),
            Some("Find the best Rust video")
        );
        assert_eq!(conversation.title_status, ChatTitleStatus::Generating);
        assert_eq!(conversation.messages.len(), 1);
        assert_eq!(conversation.updated_at, updated_at);
    }

    #[test]
    fn first_user_message_keeps_manual_title_without_triggering_auto_name() {
        let mut conversation =
            sample_conversation(Some("My chosen title"), ChatTitleStatus::Manual);
        let updated_at = Utc::now();

        let should_auto_name = apply_user_message_to_conversation(
            &mut conversation,
            sample_message(ChatRole::User, "Summarize this channel"),
            Some("Summarize this channel".to_string()),
            updated_at,
        );

        assert!(!should_auto_name);
        assert_eq!(conversation.title.as_deref(), Some("My chosen title"));
        assert_eq!(conversation.title_status, ChatTitleStatus::Manual);
        assert_eq!(conversation.messages.len(), 1);
        assert_eq!(conversation.updated_at, updated_at);
    }

    #[test]
    fn follow_up_user_message_does_not_retrigger_auto_naming() {
        let mut conversation = sample_conversation(Some("Existing title"), ChatTitleStatus::Ready);
        conversation
            .messages
            .push(sample_message(ChatRole::User, "First question"));
        conversation
            .messages
            .push(sample_message(ChatRole::Assistant, "First answer"));
        let updated_at = Utc::now();

        let should_auto_name = apply_user_message_to_conversation(
            &mut conversation,
            sample_message(ChatRole::User, "Follow-up question"),
            Some("Follow-up question".to_string()),
            updated_at,
        );

        assert!(!should_auto_name);
        assert_eq!(conversation.title.as_deref(), Some("Existing title"));
        assert_eq!(conversation.title_status, ChatTitleStatus::Ready);
        assert_eq!(conversation.messages.len(), 3);
        assert_eq!(conversation.updated_at, updated_at);
    }

    #[test]
    fn mark_manual_title_on_create_only_changes_conversations_with_titles() {
        let mut untitled = sample_conversation(None, ChatTitleStatus::Idle);
        mark_manual_title_on_create(&mut untitled);
        assert_eq!(untitled.title_status, ChatTitleStatus::Idle);

        let mut titled = sample_conversation(Some("Pinned title"), ChatTitleStatus::Idle);
        mark_manual_title_on_create(&mut titled);
        assert_eq!(titled.title_status, ChatTitleStatus::Manual);
    }

    #[test]
    fn validate_nonempty_trims_and_rejects_blank_values() {
        assert_eq!(
            validate_nonempty("  Useful title  ", "must not be empty").unwrap(),
            "Useful title"
        );
        assert!(validate_nonempty("   ", "must not be empty").is_err());
    }

    #[test]
    fn apply_manual_conversation_title_updates_state() {
        let mut conversation = sample_conversation(Some("Old"), ChatTitleStatus::Generating);
        let updated_at = Utc::now();

        apply_manual_conversation_title(&mut conversation, "New title", updated_at);

        assert_eq!(conversation.title.as_deref(), Some("New title"));
        assert_eq!(conversation.title_status, ChatTitleStatus::Manual);
        assert_eq!(conversation.updated_at, updated_at);
    }

    #[test]
    fn channel_suggestions_prefer_handle_prefix_matches() {
        let channels = vec![
            sample_channel("chan-1", "HealthyGamerGG", Some("@healthygamergg")),
            sample_channel("chan-2", "Theo - t3.gg", Some("@t3dotgg")),
        ];

        let items = rank_channel_suggestions(&channels, "hea", 5);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "HealthyGamerGG");
    }

    #[test]
    fn video_suggestions_prefer_newer_titles_on_ties() {
        let older = sample_video("vid-old", "chan-1", "Effort and Change", 10);
        let newer = sample_video("vid-new", "chan-1", "Effort and Change Again", 1);
        let channels = vec![sample_channel(
            "chan-1",
            "HealthyGamerGG",
            Some("@healthygamergg"),
        )];

        let items = rank_video_suggestions(&[older, newer], &channels, "eff", 5);

        assert_eq!(items[0].id, "vid-new");
    }

    fn sample_channel(id: &str, name: &str, handle: Option<&str>) -> Channel {
        Channel {
            id: id.to_string(),
            handle: handle.map(str::to_string),
            name: name.to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        }
    }

    fn sample_video(id: &str, channel_id: &str, title: &str, age_days: i64) -> Video {
        Video {
            id: id.to_string(),
            channel_id: channel_id.to_string(),
            title: title.to_string(),
            thumbnail_url: None,
            published_at: Utc::now() - Duration::days(age_days),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        }
    }
}
