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
        ChatConversation, ChatMessage, ChatRole, ChatTitleStatus, CreateConversationRequest,
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
    mark_manual_title_on_create(&mut conversation);

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
    let title = validate_conversation_title(&payload.title)?;

    let _lock = state.chat_store_lock.lock().await;
    let conn = state.db.connect();
    let Some(mut conversation) = db::get_conversation(&conn, &conversation_id)
        .await
        .map_err(map_db_err)?
    else {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    };
    apply_manual_conversation_title(&mut conversation, title, Utc::now());
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
    let provisional_title = state.chat.build_provisional_title(prompt);
    let should_auto_name = apply_user_message_to_conversation(
        &mut conversation,
        user_message,
        provisional_title,
        Utc::now(),
    );
    db::upsert_conversation(&conn, &conversation)
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

fn mark_manual_title_on_create(conversation: &mut ChatConversation) {
    if conversation.title.is_some() {
        conversation.title_status = ChatTitleStatus::Manual;
    }
}

fn validate_conversation_title(title: &str) -> Result<&str, (StatusCode, String)> {
    let title = title.trim();
    if title.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Conversation title must not be empty".to_string(),
        ));
    }

    Ok(title)
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
        mark_manual_title_on_create, validate_conversation_title,
    };
    use crate::models::{
        ChatConversation, ChatMessage, ChatMessageStatus, ChatRole, ChatTitleStatus,
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
    fn validate_conversation_title_trims_and_rejects_blank_values() {
        assert_eq!(
            validate_conversation_title("  Useful title  ").unwrap(),
            "Useful title"
        );
        assert!(validate_conversation_title("   ").is_err());
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
}
