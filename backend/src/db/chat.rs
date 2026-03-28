use crate::models::{ChatConversation, ChatConversationSummary};

use super::{Store, StoreError};

fn conversation_scope(scope_id: &str) -> String {
    if scope_id.trim().is_empty() {
        "anonymous".to_string()
    } else {
        scope_id.trim().to_string()
    }
}

fn chat_index_key(scope_id: &str) -> String {
    format!(
        "user-conversations/{}/index.json",
        conversation_scope(scope_id)
    )
}

fn conversation_key(scope_id: &str, conversation_id: &str) -> String {
    format!(
        "user-conversations/{}/{}.json",
        conversation_scope(scope_id),
        conversation_id
    )
}

async fn load_index(
    store: &Store,
    scope_id: &str,
) -> Result<Vec<ChatConversationSummary>, StoreError> {
    Ok(store
        .get_json::<Vec<ChatConversationSummary>>(&chat_index_key(scope_id))
        .await?
        .unwrap_or_default())
}

async fn store_index(
    store: &Store,
    scope_id: &str,
    conversations: &[ChatConversationSummary],
) -> Result<(), StoreError> {
    store.put_json(&chat_index_key(scope_id), conversations).await
}

fn sort_summaries(mut conversations: Vec<ChatConversationSummary>) -> Vec<ChatConversationSummary> {
    conversations.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| right.created_at.cmp(&left.created_at))
            .then_with(|| left.id.cmp(&right.id))
    });
    conversations
}

pub async fn list_conversations_for_scope(
    store: &Store,
    scope_id: &str,
) -> Result<Vec<ChatConversationSummary>, StoreError> {
    Ok(sort_summaries(load_index(store, scope_id).await?))
}

pub async fn get_conversation_for_scope(
    store: &Store,
    scope_id: &str,
    conversation_id: &str,
) -> Result<Option<ChatConversation>, StoreError> {
    store.get_json(&conversation_key(scope_id, conversation_id)).await
}

pub async fn upsert_conversation_for_scope(
    store: &Store,
    scope_id: &str,
    conversation: &ChatConversation,
) -> Result<(), StoreError> {
    store
        .put_json(&conversation_key(scope_id, &conversation.id), conversation)
        .await?;

    let mut index = load_index(store, scope_id).await?;
    let summary = ChatConversationSummary::from(conversation);
    match index
        .iter_mut()
        .find(|existing| existing.id == conversation.id)
    {
        Some(existing) => *existing = summary,
        None => index.push(summary),
    }
    store_index(store, scope_id, &sort_summaries(index)).await
}

pub async fn delete_conversation_for_scope(
    store: &Store,
    scope_id: &str,
    conversation_id: &str,
) -> Result<(), StoreError> {
    store
        .delete_key(&conversation_key(scope_id, conversation_id))
        .await?;
    let mut index = load_index(store, scope_id).await?;
    index.retain(|conversation| conversation.id != conversation_id);
    store_index(store, scope_id, &sort_summaries(index)).await
}

pub async fn delete_all_conversations_for_scope(
    store: &Store,
    scope_id: &str,
) -> Result<(), StoreError> {
    let conversations = load_index(store, scope_id).await?;
    for conversation in conversations {
        store
            .delete_key(&conversation_key(scope_id, &conversation.id))
            .await?;
    }
    store_index(store, scope_id, &[]).await
}
