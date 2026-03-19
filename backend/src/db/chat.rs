use crate::models::{ChatConversation, ChatConversationSummary};

use super::{Store, StoreError};

const CHAT_INDEX_KEY: &str = "conversations/index.json";

fn conversation_key(conversation_id: &str) -> String {
    format!("conversations/{conversation_id}.json")
}

async fn load_index(store: &Store) -> Result<Vec<ChatConversationSummary>, StoreError> {
    Ok(store
        .get_json::<Vec<ChatConversationSummary>>(CHAT_INDEX_KEY)
        .await?
        .unwrap_or_default())
}

async fn store_index(
    store: &Store,
    conversations: &[ChatConversationSummary],
) -> Result<(), StoreError> {
    store.put_json(CHAT_INDEX_KEY, conversations).await
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

pub async fn list_conversations(store: &Store) -> Result<Vec<ChatConversationSummary>, StoreError> {
    Ok(sort_summaries(load_index(store).await?))
}

pub async fn get_conversation(
    store: &Store,
    conversation_id: &str,
) -> Result<Option<ChatConversation>, StoreError> {
    store.get_json(&conversation_key(conversation_id)).await
}

pub async fn upsert_conversation(
    store: &Store,
    conversation: &ChatConversation,
) -> Result<(), StoreError> {
    store
        .put_json(&conversation_key(&conversation.id), conversation)
        .await?;

    let mut index = load_index(store).await?;
    let summary = ChatConversationSummary::from(conversation);
    match index
        .iter_mut()
        .find(|existing| existing.id == conversation.id)
    {
        Some(existing) => *existing = summary,
        None => index.push(summary),
    }
    store_index(store, &sort_summaries(index)).await
}

pub async fn delete_conversation(store: &Store, conversation_id: &str) -> Result<(), StoreError> {
    store.delete_key(&conversation_key(conversation_id)).await?;
    let mut index = load_index(store).await?;
    index.retain(|conversation| conversation.id != conversation_id);
    store_index(store, &sort_summaries(index)).await
}
