use aws_sdk_s3::primitives::ByteStream;
use serde::{Deserialize, Serialize};

use super::{Store, StoreError};

impl Store {
    pub(crate) async fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> Result<Option<T>, StoreError> {
        let result = self
            .s3
            .get_object()
            .bucket(&self.data_bucket)
            .key(key)
            .send()
            .await;

        match result {
            Ok(output) => {
                let bytes = output
                    .body
                    .collect()
                    .await
                    .map_err(|e| StoreError::S3(e.to_string()))?
                    .into_bytes();
                let value: T = serde_json::from_slice(&bytes)?;
                Ok(Some(value))
            }
            Err(err) => {
                if err.as_service_error().is_some_and(|e| e.is_no_such_key()) {
                    Ok(None)
                } else {
                    Err(StoreError::S3(format!("{err:#}")))
                }
            }
        }
    }

    pub(crate) async fn put_json<T: Serialize>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), StoreError> {
        let json = serde_json::to_vec(value)?;
        self.s3
            .put_object()
            .bucket(&self.data_bucket)
            .key(key)
            .body(ByteStream::from(json))
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| StoreError::S3(format!("{e:#}")))?;
        Ok(())
    }

    pub(crate) async fn delete_key(&self, key: &str) -> Result<(), StoreError> {
        self.s3
            .delete_object()
            .bucket(&self.data_bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StoreError::S3(format!("{e:#}")))?;
        Ok(())
    }

    pub(crate) async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        let mut keys = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut req = self
                .s3
                .list_objects_v2()
                .bucket(&self.data_bucket)
                .prefix(prefix);

            if let Some(token) = continuation_token.take() {
                req = req.continuation_token(token);
            }

            let output = req
                .send()
                .await
                .map_err(|e| StoreError::S3(format!("{e:#}")))?;

            if let Some(contents) = output.contents {
                for obj in contents {
                    if let Some(key) = obj.key {
                        keys.push(key);
                    }
                }
            }

            if output.is_truncated == Some(true) {
                continuation_token = output.next_continuation_token;
            } else {
                break;
            }
        }

        Ok(keys)
    }

    pub(crate) async fn load_all<T: for<'de> Deserialize<'de>>(
        &self,
        prefix: &str,
    ) -> Result<Vec<T>, StoreError> {
        let keys = self.list_keys(prefix).await?;
        let mut items = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some(item) = self.get_json::<T>(&key).await? {
                items.push(item);
            }
        }
        Ok(items)
    }

    pub(crate) async fn key_exists(&self, key: &str) -> Result<bool, StoreError> {
        let result = self
            .s3
            .head_object()
            .bucket(&self.data_bucket)
            .key(key)
            .send()
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(err) => {
                if err.as_service_error().is_some_and(|e| e.is_not_found()) {
                    Ok(false)
                } else {
                    Err(StoreError::S3(format!("{err:#}")))
                }
            }
        }
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
