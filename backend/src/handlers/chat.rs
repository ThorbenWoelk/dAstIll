use std::convert::Infallible;

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Sse, sse::Event},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    audit, db,
    models::{
        ChatConversation, ChatMessage, ChatRole, ChatTitleStatus, CreateConversationRequest,
        EphemeralChatMessageRequest, SendChatMessageRequest, UpdateConversationRequest,
    },
    read_cache::SuggestedVideo,
    security::{AccessContext, AuthState},
    services::{
        CHAT_INPUT_BLOCK_MESSAGE, ReplyWorkflowRequest,
        chat::{
            default_chat_cloud_model_id, enforce_chat_conversation_storage_limits,
            is_chat_cloud_model_choice, validate_chat_conversation_bounds, validate_chat_prompt,
            validate_chat_title_length,
        },
    },
    state::{ActiveChatKey, AppState},
};

use super::{map_db_err, require_present, validate_nonempty};

const CHAT_SUGGESTION_LIMIT_DEFAULT: usize = 8;
const CHAT_SUGGESTION_LIMIT_MAX: usize = 12;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ChatSuggestionQuery {
    #[serde(default)]
    q: String,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ChatSuggestionItem {
    kind: &'static str,
    id: String,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<String>,
}

fn conversation_store_scope_id(access_context: &AccessContext) -> &str {
    access_context.user_id.as_deref().unwrap_or("anonymous")
}

fn active_reply_scope_key(access_context: &AccessContext) -> String {
    access_context.cache_scope_key()
}

fn video_suggestion_scope_key(access_context: &AccessContext) -> String {
    format!(
        "video-suggestions:{}",
        active_reply_scope_key(access_context)
    )
}

fn active_reply_key(access_context: &AccessContext, conversation_id: &str) -> ActiveChatKey {
    ActiveChatKey::new(active_reply_scope_key(access_context), conversation_id)
}

async fn load_video_suggestion_catalog(
    state: &AppState,
    access_context: &AccessContext,
) -> Result<Vec<SuggestedVideo>, (StatusCode, String)> {
    let scope_key = video_suggestion_scope_key(access_context);
    db::load_scoped_video_suggestions(
        &state.db,
        &scope_key,
        &access_context.allowed_channel_ids,
        &access_context.allowed_other_video_ids,
    )
    .await
    .map_err(map_db_err)
}

async fn lookup_active_reply(
    active_replies: &tokio::sync::Mutex<
        std::collections::HashMap<ActiveChatKey, crate::services::ActiveChatHandle>,
    >,
    access_context: &AccessContext,
    conversation_id: &str,
) -> Result<crate::services::ActiveChatHandle, (StatusCode, String)> {
    let runtime_key = active_reply_key(access_context, conversation_id);
    active_replies
        .lock()
        .await
        .get(&runtime_key)
        .cloned()
        .ok_or((StatusCode::NOT_FOUND, "Active reply not found".to_string()))
}

fn take_active_replies_for_scope(
    active_replies: &mut std::collections::HashMap<
        ActiveChatKey,
        crate::services::ActiveChatHandle,
    >,
    scope_key: &str,
) -> Vec<crate::services::ActiveChatHandle> {
    let keys = active_replies
        .keys()
        .filter(|key| key.scope_key == scope_key)
        .cloned()
        .collect::<Vec<_>>();
    let mut handles = Vec::with_capacity(keys.len());
    for key in keys {
        if let Some(active_reply) = active_replies.remove(&key) {
            handles.push(active_reply);
        }
    }
    handles
}

fn validate_ephemeral_conversation(
    conversation: &ChatConversation,
) -> Result<(), (StatusCode, String)> {
    validate_chat_conversation_bounds(conversation)
        .map_err(|message| (StatusCode::BAD_REQUEST, message.to_string()))
}

fn require_authenticated_persistent_chat(
    access_context: &AccessContext,
) -> Result<&str, (StatusCode, String)> {
    let Some(user_id) = access_context.user_id.as_deref() else {
        return Err((
            StatusCode::FORBIDDEN,
            "Sign-in required for persistent chat. Signed-out chat stays ephemeral.".to_string(),
        ));
    };
    if access_context.auth_state != AuthState::Authenticated {
        return Err((
            StatusCode::FORBIDDEN,
            "Sign-in required for persistent chat. Signed-out chat stays ephemeral.".to_string(),
        ));
    }
    Ok(user_id)
}

#[utoipa::path(
    get,
    path = "/api/chat/suggestions/channels",
    params(ChatSuggestionQuery),
    responses(
        (status = 200, description = "Channel suggestions", body = [ChatSuggestionItem]),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn suggest_channels(
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

#[utoipa::path(
    get,
    path = "/api/chat/suggestions/videos",
    params(ChatSuggestionQuery),
    responses(
        (status = 200, description = "Video suggestions", body = [ChatSuggestionItem]),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn suggest_videos(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Query(query): Query<ChatSuggestionQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let videos = load_video_suggestion_catalog(&state, &access_context).await?;
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

#[utoipa::path(
    get,
    path = "/api/chat/config",
    responses(
        (status = 200, description = "Chat client configuration", body = crate::models::ChatClientConfig)
    )
)]
pub async fn get_client_config(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.chat.chat_client_config())
}

#[utoipa::path(
    get,
    path = "/api/chat/conversations",
    responses(
        (status = 200, description = "Conversation summaries", body = [crate::models::ChatConversationSummary]),
        (status = 403, description = "Sign-in required", body = String)
    )
)]
pub async fn list_conversations(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_authenticated_persistent_chat(&access_context)?;
    let conversations =
        db::list_conversations_for_scope(&state.db, conversation_store_scope_id(&access_context))
            .await
            .map_err(map_db_err)?;
    Ok(Json(conversations))
}

#[utoipa::path(
    post,
    path = "/api/chat/conversations",
    request_body = CreateConversationRequest,
    responses(
        (status = 201, description = "Created conversation", body = ChatConversation),
        (status = 400, description = "Invalid title", body = String),
        (status = 403, description = "Sign-in required", body = String)
    )
)]
pub async fn create_conversation(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Json(payload): Json<CreateConversationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_authenticated_persistent_chat(&access_context)?;
    if let Some(title) = payload.title.as_deref() {
        validate_chat_title_length(title)
            .map_err(|message| (StatusCode::BAD_REQUEST, message.to_string()))?;
    }
    let mut conversation = state.chat.create_conversation(payload.title.clone());
    mark_manual_title_on_create(&mut conversation);

    let scope_id = conversation_store_scope_id(&access_context);
    let _lock = state.conversation_store_lock.lock().await;
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

#[utoipa::path(
    get,
    path = "/api/chat/conversations/{id}",
    params(
        ("id" = String, Path, description = "Conversation id")
    ),
    responses(
        (status = 200, description = "Conversation", body = ChatConversation),
        (status = 403, description = "Sign-in required", body = String),
        (status = 404, description = "Conversation not found", body = String)
    )
)]
pub async fn get_conversation(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(conversation_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_authenticated_persistent_chat(&access_context)?;
    let conversation = db::get_conversation_for_scope(
        &state.db,
        conversation_store_scope_id(&access_context),
        &conversation_id,
    )
    .await
    .map_err(map_db_err)
    .and_then(|opt| require_present(opt, "Conversation not found"))?;
    Ok(Json(conversation))
}

#[utoipa::path(
    put,
    path = "/api/chat/conversations/{id}",
    params(
        ("id" = String, Path, description = "Conversation id")
    ),
    request_body = UpdateConversationRequest,
    responses(
        (status = 200, description = "Updated conversation", body = ChatConversation),
        (status = 400, description = "Invalid title", body = String),
        (status = 403, description = "Sign-in required", body = String),
        (status = 404, description = "Conversation not found", body = String)
    )
)]
pub async fn update_conversation(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(conversation_id): Path<String>,
    Json(payload): Json<UpdateConversationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_authenticated_persistent_chat(&access_context)?;
    let title = validate_nonempty(&payload.title, "Conversation title must not be empty")?;
    validate_chat_title_length(title)
        .map_err(|message| (StatusCode::BAD_REQUEST, message.to_string()))?;
    let scope_id = conversation_store_scope_id(&access_context);

    let _lock = state.conversation_store_lock.lock().await;
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

#[utoipa::path(
    delete,
    path = "/api/chat/conversations/{id}",
    params(
        ("id" = String, Path, description = "Conversation id")
    ),
    responses(
        (status = 204, description = "Deleted conversation"),
        (status = 403, description = "Sign-in required", body = String)
    )
)]
pub async fn delete_conversation(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(conversation_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_authenticated_persistent_chat(&access_context)?;
    if let Some(active_reply) = state
        .active_replies
        .lock()
        .await
        .remove(&active_reply_key(&access_context, &conversation_id))
    {
        active_reply.cancel();
    }

    let scope_id = conversation_store_scope_id(&access_context);
    let _lock = state.conversation_store_lock.lock().await;
    db::delete_conversation_for_scope(&state.db, scope_id, &conversation_id)
        .await
        .map_err(map_db_err)?;
    audit::log_chat_conversation_delete(scope_id, &conversation_id);
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/chat/conversations",
    responses(
        (status = 204, description = "Deleted all conversations for the scope"),
        (status = 403, description = "Sign-in required", body = String)
    )
)]
pub async fn delete_all_conversations(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_authenticated_persistent_chat(&access_context)?;
    let scope_key = active_reply_scope_key(&access_context);
    let active_replies_to_cancel = {
        let mut active_replies = state.active_replies.lock().await;
        take_active_replies_for_scope(&mut active_replies, &scope_key)
    };
    for active_reply in active_replies_to_cancel {
        active_reply.cancel();
    }

    let scope_id = conversation_store_scope_id(&access_context);
    let _lock = state.conversation_store_lock.lock().await;
    db::delete_all_conversations_for_scope(&state.db, scope_id)
        .await
        .map_err(map_db_err)?;
    audit::log_chat_conversations_delete_all(scope_id);

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/chat/conversations/{id}/messages",
    params(
        ("id" = String, Path, description = "Conversation id")
    ),
    request_body = SendChatMessageRequest,
    responses(
        (status = 200, description = "Server-sent reply stream", body = String, content_type = "text/event-stream"),
        (status = 400, description = "Invalid prompt", body = String),
        (status = 403, description = "Sign-in required", body = String),
        (status = 404, description = "Conversation not found", body = String),
        (status = 409, description = "Conversation already has an active response", body = String)
    )
)]
pub async fn start_conversation_reply(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(conversation_id): Path<String>,
    Json(payload): Json<SendChatMessageRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)>
{
    require_authenticated_persistent_chat(&access_context)?;
    let prompt = payload.content.trim();
    if prompt.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Message content must not be empty".to_string(),
        ));
    }
    validate_chat_prompt(prompt)
        .map_err(|message| (StatusCode::BAD_REQUEST, message.to_string()))?;
    match state.input_guardrails.evaluate_blocking_input(prompt).await {
        Ok(verdict) if !verdict.allow => {
            return Err((StatusCode::FORBIDDEN, CHAT_INPUT_BLOCK_MESSAGE.to_string()));
        }
        Ok(_) => {}
        Err(error) => {
            tracing::error!(conversation_id = %conversation_id, error = %error, "chat blocking guardrail failed");
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "Chat safety checks are unavailable.".to_string(),
            ));
        }
    }

    let runtime_key = active_reply_key(&access_context, &conversation_id);
    let active_reply = {
        let mut active_replies = state.active_replies.lock().await;
        if active_replies.contains_key(&runtime_key) {
            return Err((
                StatusCode::CONFLICT,
                "Conversation already has an active response".to_string(),
            ));
        }
        let handle = crate::services::ActiveChatHandle::new();
        active_replies.insert(runtime_key.clone(), handle.clone());
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
            state.active_replies.lock().await.remove(&runtime_key);
            return Err((
                StatusCode::BAD_REQUEST,
                "Unknown chat model. Pick a cloud model from the selector.".to_string(),
            ));
        }
        None => default_chat_cloud_model_id(state.chat.model()),
    };

    let maybe_conversation =
        append_persistent_user_message(&state, &access_context, &conversation_id, prompt).await;
    let (conversation, should_auto_name) = match maybe_conversation {
        Ok(value) => value,
        Err(error) => {
            state.active_replies.lock().await.remove(&runtime_key);
            return Err(error);
        }
    };

    state.chat.start_reply_workflow(ReplyWorkflowRequest {
        state: state.clone(),
        conversation,
        access_context: access_context.clone(),
        conversation_scope_id: conversation_store_scope_id(&access_context).to_string(),
        active_reply_key: runtime_key,
        prompt: prompt.to_string(),
        should_auto_name,
        deep_research: payload.deep_research,
        reply_model,
        active_reply: active_reply.clone(),
        persist_to_store: true,
    });
    state.input_guardrails.spawn_nonblocking_monitor(
        conversation_id,
        prompt.to_string(),
        active_reply.clone(),
    );

    Ok(reply_sse_response(active_reply).await)
}

/// Anonymous-only: runs one model turn without reading or writing persisted conversations.
#[utoipa::path(
    post,
    path = "/api/chat/ephemeral/messages",
    request_body = EphemeralChatMessageRequest,
    responses(
        (status = 200, description = "Server-sent reply stream", body = String, content_type = "text/event-stream"),
        (status = 400, description = "Invalid prompt or conversation payload", body = String),
        (status = 403, description = "Ephemeral chat is only for anonymous callers", body = String),
        (status = 409, description = "Conversation already has an active response", body = String)
    )
)]
pub async fn start_ephemeral_reply(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Json(payload): Json<EphemeralChatMessageRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)>
{
    if access_context.auth_state != AuthState::Anonymous {
        return Err((
            StatusCode::FORBIDDEN,
            "Ephemeral chat is only for signed-out visitors. Use the standard chat API when signed in."
                .to_string(),
        ));
    }

    let prompt = payload.content.trim();
    if prompt.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Message content must not be empty".to_string(),
        ));
    }
    validate_chat_prompt(prompt)
        .map_err(|message| (StatusCode::BAD_REQUEST, message.to_string()))?;
    match state.input_guardrails.evaluate_blocking_input(prompt).await {
        Ok(verdict) if !verdict.allow => {
            return Err((StatusCode::FORBIDDEN, CHAT_INPUT_BLOCK_MESSAGE.to_string()));
        }
        Ok(_) => {}
        Err(error) => {
            tracing::error!(conversation_id = %payload.conversation.id, error = %error, "ephemeral chat blocking guardrail failed");
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "Chat safety checks are unavailable.".to_string(),
            ));
        }
    }

    validate_ephemeral_conversation(&payload.conversation)?;

    let conversation_id = payload.conversation.id.clone();
    let runtime_key = active_reply_key(&access_context, &conversation_id);
    let active_reply = {
        let mut active_replies = state.active_replies.lock().await;
        if active_replies.contains_key(&runtime_key) {
            return Err((
                StatusCode::CONFLICT,
                "Conversation already has an active response".to_string(),
            ));
        }
        let handle = crate::services::ActiveChatHandle::new();
        active_replies.insert(runtime_key.clone(), handle.clone());
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
            state.active_replies.lock().await.remove(&runtime_key);
            return Err((
                StatusCode::BAD_REQUEST,
                "Unknown chat model. Pick a cloud model from the selector.".to_string(),
            ));
        }
        None => default_chat_cloud_model_id(state.chat.model()),
    };

    let mut conversation = payload.conversation;
    let user_message = state.chat.build_user_message(prompt);
    let provisional_title = state.chat.build_provisional_title(prompt);
    let should_auto_name = append_user_message_to_conversation(
        &mut conversation,
        user_message,
        provisional_title,
        Utc::now(),
    );

    state.chat.start_reply_workflow(ReplyWorkflowRequest {
        state: state.clone(),
        conversation,
        access_context: access_context.clone(),
        conversation_scope_id: String::new(),
        active_reply_key: runtime_key,
        prompt: prompt.to_string(),
        should_auto_name,
        deep_research: payload.deep_research,
        reply_model,
        active_reply: active_reply.clone(),
        persist_to_store: false,
    });
    state.input_guardrails.spawn_nonblocking_monitor(
        conversation_id,
        prompt.to_string(),
        active_reply.clone(),
    );

    Ok(reply_sse_response(active_reply).await)
}

#[utoipa::path(
    get,
    path = "/api/chat/conversations/{id}/stream",
    params(
        ("id" = String, Path, description = "Conversation id")
    ),
    responses(
        (status = 200, description = "Server-sent reply stream", body = String, content_type = "text/event-stream"),
        (status = 403, description = "Sign-in required", body = String),
        (status = 404, description = "Active reply not found", body = String)
    )
)]
pub async fn resume_conversation_reply(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(conversation_id): Path<String>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)>
{
    require_authenticated_persistent_chat(&access_context)?;
    let active_reply =
        lookup_active_reply(&state.active_replies, &access_context, &conversation_id).await?;
    Ok(reply_sse_response(active_reply).await)
}

#[utoipa::path(
    post,
    path = "/api/chat/conversations/{id}/cancel",
    params(
        ("id" = String, Path, description = "Conversation id")
    ),
    responses(
        (status = 202, description = "Cancelled active reply"),
        (status = 403, description = "Sign-in required", body = String),
        (status = 404, description = "Active reply not found", body = String)
    )
)]
pub async fn cancel_conversation_reply(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(conversation_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_authenticated_persistent_chat(&access_context)?;
    let active_reply =
        lookup_active_reply(&state.active_replies, &access_context, &conversation_id).await?;
    active_reply.cancel();
    Ok(StatusCode::ACCEPTED)
}

async fn reply_sse_response(
    active_reply: crate::services::ActiveChatHandle,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    Sse::new(active_reply.into_sse_stream().await).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}

async fn append_persistent_user_message(
    state: &AppState,
    access_context: &AccessContext,
    conversation_id: &str,
    prompt: &str,
) -> Result<(ChatConversation, bool), (StatusCode, String)> {
    let scope_id = conversation_store_scope_id(access_context);
    let _lock = state.conversation_store_lock.lock().await;
    let Some(mut conversation) =
        db::get_conversation_for_scope(&state.db, scope_id, conversation_id)
            .await
            .map_err(map_db_err)?
    else {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    };

    let user_message = state.chat.build_user_message(prompt);
    let provisional_title = state.chat.build_provisional_title(prompt);
    let should_auto_name = append_user_message_to_conversation(
        &mut conversation,
        user_message,
        provisional_title,
        Utc::now(),
    );
    enforce_chat_conversation_storage_limits(&mut conversation);
    db::upsert_conversation_for_scope(&state.db, scope_id, &conversation)
        .await
        .map_err(map_db_err)?;
    Ok((conversation, should_auto_name))
}

fn append_user_message_to_conversation(
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
    videos: &[SuggestedVideo],
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
    use axum::http::StatusCode;
    use chrono::{Duration, Utc};

    use super::{
        active_reply_key, append_user_message_to_conversation, apply_manual_conversation_title,
        lookup_active_reply, mark_manual_title_on_create, rank_channel_suggestions,
        rank_video_suggestions, require_authenticated_persistent_chat,
        take_active_replies_for_scope,
    };
    use crate::handlers::validate_nonempty;
    use crate::models::{
        Channel, ChatConversation, ChatMessage, ChatMessageStatus, ChatRole, ChatTitleStatus,
        ContentStatus, Video,
    };
    use crate::read_cache::SuggestedVideo;
    use crate::security::{AccessContext, AccessRole, AuthState};
    use crate::services::ActiveChatHandle;

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
            turn_trace: None,
        }
    }

    #[test]
    fn first_user_message_sets_provisional_title_and_generating_status() {
        let mut conversation = sample_conversation(None, ChatTitleStatus::Idle);
        let updated_at = Utc::now();

        let should_auto_name = append_user_message_to_conversation(
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

        let should_auto_name = append_user_message_to_conversation(
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

        let should_auto_name = append_user_message_to_conversation(
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
    fn suggest_channels_prefers_handle_prefix_matches() {
        let channels = vec![
            sample_channel("chan-1", "HealthyGamerGG", Some("@healthygamergg")),
            sample_channel("chan-2", "Theo - t3.gg", Some("@t3dotgg")),
        ];

        let items = rank_channel_suggestions(&channels, "hea", 5);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "HealthyGamerGG");
    }

    #[test]
    fn suggest_videos_prefers_newer_titles_on_ties() {
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

    fn sample_video(id: &str, channel_id: &str, title: &str, age_days: i64) -> SuggestedVideo {
        let video = Video {
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
        };
        SuggestedVideo {
            id: video.id,
            channel_id: video.channel_id,
            title: video.title,
            published_at: video.published_at,
        }
    }

    fn auth_context(user_id: &str) -> AccessContext {
        AccessContext {
            user_id: Some(user_id.to_string()),
            auth_state: AuthState::Authenticated,
            access_role: AccessRole::User,
            allowed_channel_ids: Vec::new(),
            allowed_other_video_ids: Vec::new(),
        }
    }

    fn anonymous_context() -> AccessContext {
        AccessContext {
            user_id: None,
            auth_state: AuthState::Anonymous,
            access_role: AccessRole::Anonymous,
            allowed_channel_ids: Vec::new(),
            allowed_other_video_ids: Vec::new(),
        }
    }

    #[test]
    fn active_reply_key_separates_anonymous_and_authenticated_scope_for_same_id() {
        let authenticated = auth_context("anonymous");
        let anonymous = anonymous_context();

        assert_ne!(
            active_reply_key(&authenticated, "conv-shared"),
            active_reply_key(&anonymous, "conv-shared")
        );
    }

    #[test]
    fn persistent_chat_requires_authenticated_context() {
        let authenticated = auth_context("user-a");
        assert_eq!(
            require_authenticated_persistent_chat(&authenticated).unwrap(),
            "user-a"
        );

        let anonymous = anonymous_context();
        let error = require_authenticated_persistent_chat(&anonymous)
            .expect_err("anonymous access should be rejected");
        assert_eq!(error.0, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn resume_and_cancel_require_matching_scope_ownership() {
        let owner = auth_context("user-a");
        let foreign = auth_context("user-b");
        let conversation_id = "conv-shared".to_string();
        let owner_key = active_reply_key(&owner, &conversation_id);
        let active_replies = tokio::sync::Mutex::new(std::collections::HashMap::from([(
            owner_key.clone(),
            ActiveChatHandle::new(),
        )]));

        let reconnect_error = lookup_active_reply(&active_replies, &foreign, &conversation_id)
            .await
            .expect_err("foreign reconnect should fail");
        assert_eq!(reconnect_error.0, StatusCode::NOT_FOUND);

        let _ = lookup_active_reply(&active_replies, &owner, &conversation_id)
            .await
            .expect("owner reconnect should succeed");

        let cancel_error = lookup_active_reply(&active_replies, &foreign, &conversation_id)
            .await
            .expect_err("foreign cancel should fail");
        assert_eq!(cancel_error.0, StatusCode::NOT_FOUND);
        assert!(active_replies.lock().await.contains_key(&owner_key));

        let active_reply = lookup_active_reply(&active_replies, &owner, &conversation_id)
            .await
            .expect("owner cancel should succeed");
        active_reply.cancel();
        assert!(active_replies.lock().await.contains_key(&owner_key));
    }

    #[test]
    fn take_active_replies_for_scope_only_takes_matching_scope_entries() {
        let authenticated = auth_context("user-a");
        let anonymous = anonymous_context();
        let conversation_id = "conv-shared";

        let authenticated_key = active_reply_key(&authenticated, conversation_id);
        let anonymous_key = active_reply_key(&anonymous, conversation_id);
        let mut active_replies = std::collections::HashMap::new();
        active_replies.insert(authenticated_key.clone(), ActiveChatHandle::new());
        active_replies.insert(anonymous_key.clone(), ActiveChatHandle::new());

        let removed = take_active_replies_for_scope(&mut active_replies, "user:user-a");
        assert_eq!(removed.len(), 1);
        assert!(!active_replies.contains_key(&authenticated_key));
        assert!(active_replies.contains_key(&anonymous_key));
    }
}
