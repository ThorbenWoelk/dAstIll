use std::convert::Infallible;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Sse, sse::Event},
};
use chrono::Utc;

use crate::{
    db,
    models::{
        ChatConversation, ChatRole, ChatTitleStatus, CreateConversationRequest,
        SendChatMessageRequest, UpdateConversationRequest,
    },
    state::AppState,
};

use super::map_db_err;

pub async fn list_conversations(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.db.connect();
    let conversations = db::list_conversations(&conn).await.map_err(map_db_err)?;
    Ok(Json(conversations))
}

pub async fn create_conversation(
    State(state): State<AppState>,
    Json(payload): Json<CreateConversationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut conversation = state.chat.create_conversation(payload.title.clone());
    if conversation.title.is_some() {
        conversation.title_status = ChatTitleStatus::Manual;
    }

    let _lock = state.chat_store_lock.lock().await;
    let conn = state.db.connect();
    db::upsert_conversation(&conn, &conversation)
        .await
        .map_err(map_db_err)?;
    Ok((StatusCode::CREATED, Json(conversation)))
}

pub async fn get_conversation(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.db.connect();
    let conversation = db::get_conversation(&conn, &conversation_id)
        .await
        .map_err(map_db_err)?
        .ok_or((StatusCode::NOT_FOUND, "Conversation not found".to_string()))?;
    Ok(Json(conversation))
}

pub async fn update_conversation(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Json(payload): Json<UpdateConversationRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let title = payload.title.trim();
    if title.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Conversation title must not be empty".to_string(),
        ));
    }

    let _lock = state.chat_store_lock.lock().await;
    let conn = state.db.connect();
    let Some(mut conversation) = db::get_conversation(&conn, &conversation_id)
        .await
        .map_err(map_db_err)?
    else {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    };
    conversation.title = Some(title.to_string());
    conversation.title_status = ChatTitleStatus::Manual;
    conversation.updated_at = Utc::now();
    db::upsert_conversation(&conn, &conversation)
        .await
        .map_err(map_db_err)?;
    Ok(Json(conversation))
}

pub async fn delete_conversation(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(active_chat) = state.active_chats.lock().await.remove(&conversation_id) {
        active_chat.cancel();
    }

    let _lock = state.chat_store_lock.lock().await;
    let conn = state.db.connect();
    db::delete_conversation(&conn, &conversation_id)
        .await
        .map_err(map_db_err)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn send_message(
    State(state): State<AppState>,
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

    let maybe_conversation = store_user_message(&state, &conversation_id, prompt).await;
    let (conversation, should_auto_name) = match maybe_conversation {
        Ok(value) => value,
        Err(error) => {
            state.active_chats.lock().await.remove(&conversation_id);
            return Err(error);
        }
    };

    state.chat.spawn_reply(
        state.clone(),
        conversation,
        prompt.to_string(),
        should_auto_name,
        active_chat.clone(),
    );

    Ok(sse_response(active_chat).await)
}

pub async fn reconnect_stream(
    State(state): State<AppState>,
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
    conversation_id: &str,
    prompt: &str,
) -> Result<(ChatConversation, bool), (StatusCode, String)> {
    let _lock = state.chat_store_lock.lock().await;
    let conn = state.db.connect();
    let Some(mut conversation) = db::get_conversation(&conn, conversation_id)
        .await
        .map_err(map_db_err)?
    else {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    };

    let user_message = state.chat.build_user_message(prompt);
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
            conversation.title = state.chat.build_provisional_title(prompt);
        }
        conversation.title_status = ChatTitleStatus::Generating;
    }

    conversation.updated_at = Utc::now();
    db::upsert_conversation(&conn, &conversation)
        .await
        .map_err(map_db_err)?;
    Ok((conversation, should_auto_name))
}
