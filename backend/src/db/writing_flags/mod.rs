use crate::models::WritingFlag;

use super::{Store, StoreError};

const WRITING_FLAG_PREFIX: &str = "writing-flags/";
const MAX_CONTEXT_CHARS: usize = 160;

fn user_prefix(user_id: &str) -> String {
    format!("{WRITING_FLAG_PREFIX}{user_id}/")
}

fn writing_flag_key(user_id: &str, id: i64) -> String {
    format!("{}{id}.json", user_prefix(user_id))
}

fn newest_first(flags: &mut [WritingFlag]) {
    flags.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then(right.id.cmp(&left.id))
    });
}

pub async fn list_user_writing_flags(
    store: &Store,
    user_id: &str,
) -> Result<Vec<WritingFlag>, StoreError> {
    let mut flags: Vec<WritingFlag> = store.load_all(&user_prefix(user_id)).await?;
    newest_first(&mut flags);
    Ok(flags)
}

pub async fn list_video_writing_flags(
    store: &Store,
    user_id: &str,
    video_id: &str,
) -> Result<Vec<WritingFlag>, StoreError> {
    let mut flags = list_user_writing_flags(store, user_id).await?;
    flags.retain(|flag| flag.video_id == video_id);
    Ok(flags)
}

pub async fn create_writing_flag(
    store: &Store,
    user_id: &str,
    video_id: &str,
    text: &str,
    comment: &str,
    prefix_context: &str,
    suffix_context: &str,
) -> Result<WritingFlag, StoreError> {
    let id = super::highlights::generate_highlight_id();
    let flag = WritingFlag {
        id,
        video_id: video_id.to_string(),
        text: text.to_string(),
        comment: comment.to_string(),
        prefix_context: prefix_context.chars().take(MAX_CONTEXT_CHARS).collect(),
        suffix_context: suffix_context.chars().take(MAX_CONTEXT_CHARS).collect(),
        created_at: chrono::Utc::now(),
    };
    store
        .put_json(&writing_flag_key(user_id, id), &flag)
        .await?;
    Ok(flag)
}

pub async fn delete_writing_flag(
    store: &Store,
    user_id: &str,
    flag_id: i64,
) -> Result<bool, StoreError> {
    let key = writing_flag_key(user_id, flag_id);
    let exists = store.key_exists(&key).await?;
    if exists {
        store.delete_key(&key).await?;
    }
    Ok(exists)
}

/// Every flag from every reader, newest first, for the summary writer.
pub async fn list_all_writing_flags(store: &Store) -> Result<Vec<WritingFlag>, StoreError> {
    let mut flags: Vec<WritingFlag> = store.load_all(WRITING_FLAG_PREFIX).await?;
    newest_first(&mut flags);
    Ok(flags)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
