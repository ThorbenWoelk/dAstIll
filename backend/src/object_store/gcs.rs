use bytes::Bytes;
use google_cloud_gax::error::rpc::Code;
use google_cloud_storage::client::{Storage, StorageControl};

use super::{ObjectStore, ObjectStoreError};

#[derive(Clone)]
pub(crate) struct GcsObjectStore {
    storage: Storage,
    control: StorageControl,
    bucket_resource: String,
}

impl GcsObjectStore {
    pub async fn from_adc(bucket: impl Into<String>) -> Result<Self, ObjectStoreError> {
        let bucket = bucket.into();
        let storage = Storage::builder()
            .build()
            .await
            .map_err(|err| ObjectStoreError::new(format!("failed to build GCS client: {err}")))?;
        let control = StorageControl::builder().build().await.map_err(|err| {
            ObjectStoreError::new(format!("failed to build GCS control client: {err}"))
        })?;
        Ok(Self {
            storage,
            control,
            bucket_resource: format!("projects/_/buckets/{bucket}"),
        })
    }

    fn is_not_found(err: &google_cloud_storage::Error) -> bool {
        err.http_status_code() == Some(404)
            || err
                .status()
                .is_some_and(|status| status.code == Code::NotFound)
    }
}

#[async_trait::async_trait]
impl ObjectStore for GcsObjectStore {
    async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, ObjectStoreError> {
        let mut response = match self
            .storage
            .read_object(&self.bucket_resource, key)
            .send()
            .await
        {
            Ok(response) => response,
            Err(err) if Self::is_not_found(&err) => return Ok(None),
            Err(err) => {
                return Err(ObjectStoreError::new(format!(
                    "GCS read failed for {key}: {err}"
                )));
            }
        };

        let mut bytes = Vec::new();
        while let Some(chunk) = response.next().await {
            let chunk = chunk.map_err(|err| {
                ObjectStoreError::new(format!("GCS read stream failed for {key}: {err}"))
            })?;
            bytes.extend_from_slice(&chunk);
        }
        Ok(Some(bytes))
    }

    async fn put_bytes(
        &self,
        key: &str,
        bytes: &[u8],
        content_type: &str,
    ) -> Result<(), ObjectStoreError> {
        self.storage
            .write_object(&self.bucket_resource, key, Bytes::copy_from_slice(bytes))
            .set_content_type(content_type)
            .send_buffered()
            .await
            .map_err(|err| ObjectStoreError::new(format!("GCS write failed for {key}: {err}")))?;
        Ok(())
    }

    async fn delete_key(&self, key: &str) -> Result<(), ObjectStoreError> {
        match self
            .control
            .delete_object()
            .set_bucket(&self.bucket_resource)
            .set_object(key)
            .send()
            .await
        {
            Ok(()) => Ok(()),
            Err(err) if Self::is_not_found(&err) => Ok(()),
            Err(err) => Err(ObjectStoreError::new(format!(
                "GCS delete failed for {key}: {err}"
            ))),
        }
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, ObjectStoreError> {
        let mut keys = Vec::new();
        let mut page_token = String::new();

        loop {
            let mut request = self
                .control
                .list_objects()
                .set_parent(&self.bucket_resource)
                .set_prefix(prefix);
            if !page_token.is_empty() {
                request = request.set_page_token(page_token);
            }

            let response = request.send().await.map_err(|err| {
                ObjectStoreError::new(format!("GCS list failed for prefix {prefix}: {err}"))
            })?;
            keys.extend(response.objects.into_iter().map(|object| object.name));

            if response.next_page_token.is_empty() {
                break;
            }
            page_token = response.next_page_token;
        }

        Ok(keys)
    }

    async fn key_exists(&self, key: &str) -> Result<bool, ObjectStoreError> {
        match self
            .control
            .get_object()
            .set_bucket(&self.bucket_resource)
            .set_object(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) if Self::is_not_found(&err) => Ok(false),
            Err(err) => Err(ObjectStoreError::new(format!(
                "GCS metadata read failed for {key}: {err}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use google_cloud_gax::error::rpc::Status;

    use super::*;

    #[test]
    fn recognizes_rpc_not_found() {
        let error =
            google_cloud_storage::Error::service(Status::default().set_code(Code::NotFound));

        assert!(GcsObjectStore::is_not_found(&error));
    }

    #[test]
    fn rejects_other_rpc_errors_as_not_found() {
        let error = google_cloud_storage::Error::service(
            Status::default().set_code(Code::PermissionDenied),
        );

        assert!(!GcsObjectStore::is_not_found(&error));
    }
}
