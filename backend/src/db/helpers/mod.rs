use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use super::{Store, StoreError};

impl Store {
    pub(crate) async fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, StoreError> {
        let Some(bytes) = self.get_bytes(key).await? else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_slice(&bytes)?))
    }

    pub(crate) async fn put_json<T: Serialize + ?Sized>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), StoreError> {
        let json = serde_json::to_vec(value)?;
        self.put_bytes(key, &json, "application/json").await
    }

    pub(crate) async fn get_json_gz<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, StoreError> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let Some(bytes) = self.get_bytes(key).await? else {
            return Ok(None);
        };
        let mut decoder = GzDecoder::new(&bytes[..]);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| StoreError::Other(format!("gzip decompression failed: {e}")))?;
        let value: T = serde_json::from_slice(&decompressed)?;
        Ok(Some(value))
    }

    pub(crate) async fn put_json_gz<T: Serialize + ?Sized>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), StoreError> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let json = serde_json::to_vec(value)?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&json)
            .map_err(|e| StoreError::Other(format!("gzip compression failed: {e}")))?;
        let compressed = encoder
            .finish()
            .map_err(|e| StoreError::Other(format!("gzip compression finish failed: {e}")))?;

        self.put_bytes(key, &compressed, "application/x-gzip").await
    }

    pub(crate) async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.objects
            .get_bytes(key)
            .await
            .map_err(|err| StoreError::ObjectStore(err.to_string()))
    }

    pub(crate) async fn put_bytes(
        &self,
        key: &str,
        bytes: &[u8],
        content_type: &str,
    ) -> Result<(), StoreError> {
        self.objects
            .put_bytes(key, bytes, content_type)
            .await
            .map_err(|err| StoreError::ObjectStore(err.to_string()))
    }

    pub(crate) async fn delete_key(&self, key: &str) -> Result<(), StoreError> {
        self.objects
            .delete_key(key)
            .await
            .map_err(|err| StoreError::ObjectStore(err.to_string()))
    }

    pub(crate) async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        self.objects
            .list_keys(prefix)
            .await
            .map_err(|err| StoreError::ObjectStore(err.to_string()))
    }

    pub(crate) async fn load_all<T: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        prefix: &str,
    ) -> Result<Vec<T>, StoreError> {
        let keys = self.list_keys(prefix).await?;
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let semaphore = Arc::new(Semaphore::new(super::MAX_CONCURRENT_OBJECT_STORE_OPS));
        let mut join_set: JoinSet<Result<Option<T>, StoreError>> = JoinSet::new();

        for key in keys {
            let store = self.clone();
            let semaphore = Arc::clone(&semaphore);
            join_set.spawn(async move {
                let _permit = semaphore.acquire().await.expect("semaphore closed");
                store.get_json::<T>(&key).await
            });
        }

        let mut items = Vec::new();
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(Some(item))) => items.push(item),
                Ok(Ok(None)) => {}
                Ok(Err(e)) => return Err(e),
                Err(e) => {
                    return Err(StoreError::ObjectStore(format!(
                        "parallel fetch task error: {e}"
                    )));
                }
            }
        }
        Ok(items)
    }

    pub(crate) async fn key_exists(&self, key: &str) -> Result<bool, StoreError> {
        self.objects
            .key_exists(key)
            .await
            .map_err(|err| StoreError::ObjectStore(err.to_string()))
    }

    pub(crate) async fn delete_prefix(&self, prefix: &str) -> Result<usize, StoreError> {
        let keys = self.list_keys(prefix).await?;
        let count = keys.len();
        for key in keys {
            self.delete_key(&key).await?;
        }
        Ok(count)
    }
}
