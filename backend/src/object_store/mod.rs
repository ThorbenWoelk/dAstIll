mod gcs;
pub(crate) mod memory;

pub(crate) use gcs::GcsObjectStore;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct ObjectStoreError(String);

impl ObjectStoreError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

#[async_trait::async_trait]
pub trait ObjectStore: Send + Sync {
    async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, ObjectStoreError>;

    async fn put_bytes(
        &self,
        key: &str,
        bytes: &[u8],
        content_type: &str,
    ) -> Result<(), ObjectStoreError>;

    async fn delete_key(&self, key: &str) -> Result<(), ObjectStoreError>;

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, ObjectStoreError>;

    async fn key_exists(&self, key: &str) -> Result<bool, ObjectStoreError>;
}
