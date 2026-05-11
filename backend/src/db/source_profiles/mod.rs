use serde::{Deserialize, Serialize};

use crate::models::{ContentSource, OpenAlexSavedSearchQuery, SubscriptionContainer};

use super::{Store, StoreError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceProfileRecord {
    pub source: ContentSource,
    pub container: SubscriptionContainer,
    #[serde(default)]
    pub openalex_query: Option<OpenAlexSavedSearchQuery>,
}

fn source_profile_key(source_id: &str) -> String {
    format!("source-profiles/{source_id}.json")
}

pub async fn get_source_profile(
    store: &Store,
    source_id: &str,
) -> Result<Option<SourceProfileRecord>, StoreError> {
    store.get_json(&source_profile_key(source_id)).await
}

pub async fn put_source_profile(
    store: &Store,
    profile: &SourceProfileRecord,
) -> Result<(), StoreError> {
    store
        .put_json(&source_profile_key(&profile.source.id), profile)
        .await
}

pub async fn delete_source_profile(store: &Store, source_id: &str) -> Result<bool, StoreError> {
    let key = source_profile_key(source_id);
    let exists = store.key_exists(&key).await?;
    if exists {
        store.delete_key(&key).await?;
    }
    Ok(exists)
}
