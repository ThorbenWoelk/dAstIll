use super::{Store, StoreError};

async fn count_prefix(store: &Store, prefix: &str) -> Result<usize, StoreError> {
    Ok(store.list_keys(prefix).await?.len())
}

pub async fn count_summaries(store: &Store) -> Result<usize, StoreError> {
    count_prefix(store, "summaries/").await
}

pub async fn count_transcripts(store: &Store) -> Result<usize, StoreError> {
    count_prefix(store, "transcripts/").await
}

pub async fn count_videos(store: &Store) -> Result<usize, StoreError> {
    count_prefix(store, "videos/").await
}

pub async fn count_channels(store: &Store) -> Result<usize, StoreError> {
    count_prefix(store, "channels/").await
}
