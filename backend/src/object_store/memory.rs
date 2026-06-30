use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use super::{ObjectStore, ObjectStoreError};

#[derive(Debug, Default, Clone)]
pub(crate) struct MemoryObjectStore {
    objects: Arc<RwLock<BTreeMap<String, Vec<u8>>>>,
}

impl MemoryObjectStore {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl ObjectStore for MemoryObjectStore {
    async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, ObjectStoreError> {
        Ok(self.objects.read().await.get(key).cloned())
    }

    async fn put_bytes(
        &self,
        key: &str,
        bytes: &[u8],
        _content_type: &str,
    ) -> Result<(), ObjectStoreError> {
        self.objects
            .write()
            .await
            .insert(key.to_string(), bytes.to_vec());
        Ok(())
    }

    async fn delete_key(&self, key: &str) -> Result<(), ObjectStoreError> {
        self.objects.write().await.remove(key);
        Ok(())
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, ObjectStoreError> {
        Ok(self
            .objects
            .read()
            .await
            .keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect())
    }

    async fn key_exists(&self, key: &str) -> Result<bool, ObjectStoreError> {
        Ok(self.objects.read().await.contains_key(key))
    }
}
