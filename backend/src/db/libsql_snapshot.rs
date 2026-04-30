use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aws_sdk_s3::primitives::ByteStream;
use chrono::{SecondsFormat, Utc};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use tokio::sync::Notify;

use super::{StoreError, format_aws_error};

const MANIFEST_KEY: &str = "runtime-cache/libsql/current.json";
const SNAPSHOT_PREFIX: &str = "runtime-cache/libsql/snapshots";
const SNAPSHOT_SCHEMA_VERSION: u32 = 1;
const SNAPSHOT_PUBLISH_DEBOUNCE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibsqlSnapshotManifest {
    pub schema_version: u32,
    pub app_version: String,
    pub generation: String,
    pub snapshot_key: String,
    pub sha256: String,
    pub created_at: String,
    pub byte_size: u64,
    pub compressed_byte_size: u64,
    pub source_state: LibsqlSnapshotSourceState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibsqlSnapshotSourceState {
    pub videos: PrefixState,
    pub preferences: PrefixState,
    pub tts_stats: PrefixState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixState {
    pub key_count: usize,
    pub latest_modified_epoch_ms: Option<u64>,
    pub fingerprint_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibsqlSnapshotRestore {
    Restored,
    Missing,
    Failed,
}

impl LibsqlSnapshotRestore {
    pub fn restored(self) -> bool {
        self == Self::Restored
    }
}

#[derive(Clone)]
pub struct LibsqlSnapshotPublisher {
    inner: Arc<LibsqlSnapshotPublisherInner>,
}

struct LibsqlSnapshotPublisherInner {
    s3: aws_sdk_s3::Client,
    bucket: String,
    conn: libsql::Connection,
    db_path: PathBuf,
    debounce: Duration,
    generation: AtomicU64,
    notify: Notify,
}

impl LibsqlSnapshotPublisher {
    pub fn new(
        s3: aws_sdk_s3::Client,
        bucket: impl Into<String>,
        conn: libsql::Connection,
        db_path: PathBuf,
    ) -> Self {
        Self::new_with_debounce(s3, bucket, conn, db_path, SNAPSHOT_PUBLISH_DEBOUNCE)
    }

    fn new_with_debounce(
        s3: aws_sdk_s3::Client,
        bucket: impl Into<String>,
        conn: libsql::Connection,
        db_path: PathBuf,
        debounce: Duration,
    ) -> Self {
        let publisher = Self {
            inner: Arc::new(LibsqlSnapshotPublisherInner {
                s3,
                bucket: bucket.into(),
                conn,
                db_path,
                debounce,
                generation: AtomicU64::new(0),
                notify: Notify::new(),
            }),
        };
        publisher.spawn_worker();
        publisher
    }

    pub fn schedule(&self) {
        self.inner.generation.fetch_add(1, Ordering::AcqRel);
        self.inner.notify.notify_one();
    }

    fn spawn_worker(&self) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            run_snapshot_publish_worker(inner).await;
        });
    }
}

async fn run_snapshot_publish_worker(inner: Arc<LibsqlSnapshotPublisherInner>) {
    let mut published_generation = 0u64;

    loop {
        while inner.generation.load(Ordering::Acquire) == published_generation {
            inner.notify.notified().await;
        }

        let mut observed_generation = inner.generation.load(Ordering::Acquire);
        loop {
            tokio::time::sleep(inner.debounce).await;
            let current_generation = inner.generation.load(Ordering::Acquire);
            if current_generation == observed_generation {
                break;
            }
            observed_generation = current_generation;
        }

        match publish_libsql_snapshot(
            &inner.s3,
            &inner.bucket,
            &inner.conn,
            &inner.db_path,
            env!("CARGO_PKG_VERSION"),
        )
        .await
        {
            Ok(_) => {
                published_generation = observed_generation;
            }
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    path = %inner.db_path.display(),
                    "debounced libSQL snapshot publish failed"
                );
            }
        }
    }
}

pub async fn restore_libsql_snapshot(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    db_path: &Path,
) -> LibsqlSnapshotRestore {
    match try_restore_libsql_snapshot(s3, bucket, db_path).await {
        Ok(true) => LibsqlSnapshotRestore::Restored,
        Ok(false) => LibsqlSnapshotRestore::Missing,
        Err(err) => {
            tracing::warn!(error = %err, "libSQL snapshot restore failed - falling back to S3 rebuild");
            LibsqlSnapshotRestore::Failed
        }
    }
}

async fn try_restore_libsql_snapshot(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    db_path: &Path,
) -> Result<bool, StoreError> {
    let Some(manifest) = get_json::<LibsqlSnapshotManifest>(s3, bucket, MANIFEST_KEY).await? else {
        tracing::info!("libSQL snapshot manifest not found - using S3 rebuild path");
        return Ok(false);
    };

    if manifest.schema_version != SNAPSHOT_SCHEMA_VERSION {
        tracing::warn!(
            manifest_schema_version = manifest.schema_version,
            expected_schema_version = SNAPSHOT_SCHEMA_VERSION,
            "libSQL snapshot schema version mismatch - using S3 rebuild path"
        );
        return Ok(false);
    }

    let current_source_state = load_source_state(s3, bucket).await?;
    if current_source_state != manifest.source_state {
        tracing::info!("libSQL snapshot source state changed - using S3 rebuild path");
        return Ok(false);
    }

    let Some(compressed) = get_bytes(s3, bucket, &manifest.snapshot_key).await? else {
        tracing::warn!(
            snapshot_key = %manifest.snapshot_key,
            "libSQL snapshot object missing - using S3 rebuild path"
        );
        return Ok(false);
    };

    let actual_sha256 = sha256_hex(&compressed);
    if actual_sha256 != manifest.sha256 {
        return Err(StoreError::Other(format!(
            "libSQL snapshot checksum mismatch for {}",
            manifest.snapshot_key
        )));
    }

    let db_bytes = decompress_gzip(&compressed)?;
    if db_bytes.len() as u64 != manifest.byte_size {
        return Err(StoreError::Other(format!(
            "libSQL snapshot size mismatch for {}",
            manifest.snapshot_key
        )));
    }

    write_snapshot_db_file(db_path, &db_bytes).await?;

    tracing::info!(
        snapshot_key = %manifest.snapshot_key,
        generation = %manifest.generation,
        byte_size = manifest.byte_size,
        compressed_byte_size = manifest.compressed_byte_size,
        "libSQL snapshot restored"
    );
    Ok(true)
}

pub async fn publish_libsql_snapshot(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    conn: &libsql::Connection,
    db_path: &Path,
    app_version: &str,
) -> Result<LibsqlSnapshotManifest, StoreError> {
    let source_state = load_source_state(s3, bucket).await?;
    checkpoint_libsql_file(conn).await?;
    let db_bytes = tokio::fs::read(db_path)
        .await
        .map_err(|err| StoreError::Other(format!("failed to read libSQL snapshot file: {err}")))?;
    let compressed = compress_gzip(&db_bytes)?;
    let sha256 = sha256_hex(&compressed);
    ensure_source_state_stable(&source_state, &load_source_state(s3, bucket).await?)?;
    let generation = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let key_generation = generation.replace([':', '.'], "-");
    let snapshot_key = format!(
        "{SNAPSHOT_PREFIX}/search-fts-v{SNAPSHOT_SCHEMA_VERSION}-{key_generation}.sqlite.gz"
    );
    let manifest = LibsqlSnapshotManifest {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        app_version: app_version.to_string(),
        generation,
        snapshot_key: snapshot_key.clone(),
        sha256,
        created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        byte_size: db_bytes.len() as u64,
        compressed_byte_size: compressed.len() as u64,
        source_state,
    };

    put_bytes(s3, bucket, &snapshot_key, compressed, "application/gzip").await?;
    put_json(s3, bucket, MANIFEST_KEY, &manifest).await?;

    tracing::info!(
        snapshot_key = %manifest.snapshot_key,
        byte_size = manifest.byte_size,
        compressed_byte_size = manifest.compressed_byte_size,
        "libSQL snapshot published"
    );

    Ok(manifest)
}

async fn checkpoint_libsql_file(conn: &libsql::Connection) -> Result<(), StoreError> {
    let mut rows = conn
        .query("PRAGMA wal_checkpoint(TRUNCATE)", ())
        .await
        .map_err(|err| StoreError::Other(format!("failed to checkpoint snapshot DB: {err}")))?;
    while rows
        .next()
        .await
        .map_err(|err| StoreError::Other(format!("failed to checkpoint snapshot DB: {err}")))?
        .is_some()
    {}
    Ok(())
}

fn ensure_source_state_stable(
    before: &LibsqlSnapshotSourceState,
    after: &LibsqlSnapshotSourceState,
) -> Result<(), StoreError> {
    if before == after {
        Ok(())
    } else {
        Err(StoreError::Other(
            "libSQL snapshot source state changed while publishing".to_string(),
        ))
    }
}

async fn write_snapshot_db_file(db_path: &Path, db_bytes: &[u8]) -> Result<(), StoreError> {
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| StoreError::Other(format!("failed to create libSQL dir: {err}")))?;
    }
    remove_if_exists(db_path).await?;
    remove_if_exists(&db_path.with_extension("db-wal")).await?;
    remove_if_exists(&db_path.with_extension("db-shm")).await?;

    let temp_path = db_path.with_extension("db.restore-tmp");
    tokio::fs::write(&temp_path, db_bytes)
        .await
        .map_err(|err| StoreError::Other(format!("failed to write temp libSQL snapshot: {err}")))?;
    tokio::fs::rename(&temp_path, db_path)
        .await
        .map_err(|err| StoreError::Other(format!("failed to install libSQL snapshot: {err}")))?;
    Ok(())
}

async fn remove_if_exists(path: &Path) -> Result<(), StoreError> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(StoreError::Other(format!(
            "failed to remove {}: {err}",
            path.display()
        ))),
    }
}

async fn get_json<T: for<'de> Deserialize<'de>>(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
) -> Result<Option<T>, StoreError> {
    let Some(bytes) = get_bytes(s3, bucket, key).await? else {
        return Ok(None);
    };
    Ok(Some(serde_json::from_slice(&bytes)?))
}

async fn load_source_state(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
) -> Result<LibsqlSnapshotSourceState, StoreError> {
    Ok(LibsqlSnapshotSourceState {
        videos: prefix_state(s3, bucket, "videos/").await?,
        preferences: prefix_state(s3, bucket, "user-preferences/").await?,
        tts_stats: prefix_state(s3, bucket, "tts-stats/").await?,
    })
}

async fn prefix_state(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    prefix: &str,
) -> Result<PrefixState, StoreError> {
    let mut continuation_token: Option<String> = None;
    let mut key_count = 0usize;
    let mut latest_modified_epoch_ms: Option<u64> = None;
    let mut object_fingerprints = Vec::new();

    loop {
        let mut req = s3.list_objects_v2().bucket(bucket).prefix(prefix);
        if let Some(token) = continuation_token.take() {
            req = req.continuation_token(token);
        }

        let output = req
            .send()
            .await
            .map_err(|err| StoreError::S3(format_aws_error(&err)))?;

        if let Some(contents) = output.contents {
            for object in contents {
                key_count += 1;
                let key = object.key.unwrap_or_default();
                let etag = object.e_tag.unwrap_or_default();
                let size = object.size.unwrap_or_default();
                if let Some(last_modified) = object.last_modified {
                    if let Some(epoch_ms) = aws_datetime_epoch_ms(last_modified) {
                        latest_modified_epoch_ms = Some(
                            latest_modified_epoch_ms
                                .map(|current| current.max(epoch_ms))
                                .unwrap_or(epoch_ms),
                        );
                        object_fingerprints.push(format!("{key}\0{etag}\0{size}\0{epoch_ms}"));
                    } else {
                        object_fingerprints.push(format!("{key}\0{etag}\0{size}\0"));
                    }
                } else {
                    object_fingerprints.push(format!("{key}\0{etag}\0{size}\0"));
                }
            }
        }

        if output.is_truncated == Some(true) {
            continuation_token = output.next_continuation_token;
        } else {
            break;
        }
    }

    object_fingerprints.sort();
    let fingerprint_sha256 = sha256_hex(object_fingerprints.join("\n").as_bytes());

    Ok(PrefixState {
        key_count,
        latest_modified_epoch_ms,
        fingerprint_sha256,
    })
}

async fn put_json<T: Serialize + ?Sized>(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    value: &T,
) -> Result<(), StoreError> {
    let json = serde_json::to_vec(value)?;
    put_bytes(s3, bucket, key, json, "application/json").await
}

async fn get_bytes(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
) -> Result<Option<Vec<u8>>, StoreError> {
    let result = s3.get_object().bucket(bucket).key(key).send().await;
    match result {
        Ok(output) => {
            let bytes = output
                .body
                .collect()
                .await
                .map_err(|err| StoreError::S3(err.to_string()))?
                .into_bytes();
            Ok(Some(bytes.to_vec()))
        }
        Err(err) => {
            if err
                .as_service_error()
                .is_some_and(|err| err.is_no_such_key())
            {
                Ok(None)
            } else {
                Err(StoreError::S3(format_aws_error(&err)))
            }
        }
    }
}

async fn put_bytes(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    bytes: Vec<u8>,
    content_type: &str,
) -> Result<(), StoreError> {
    s3.put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(bytes))
        .content_type(content_type)
        .send()
        .await
        .map_err(|err| StoreError::S3(format_aws_error(&err)))?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn aws_datetime_epoch_ms(value: aws_smithy_types::DateTime) -> Option<u64> {
    let system_time = SystemTime::try_from(value).ok()?;
    let duration = system_time.duration_since(UNIX_EPOCH).ok()?;
    u64::try_from(duration.as_millis()).ok()
}

fn compress_gzip(bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(bytes)
        .map_err(|err| StoreError::Other(format!("gzip compression failed: {err}")))?;
    encoder
        .finish()
        .map_err(|err| StoreError::Other(format!("gzip compression finish failed: {err}")))
}

fn decompress_gzip(bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
    let mut decoder = GzDecoder::new(bytes);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|err| StoreError::Other(format!("gzip decompression failed: {err}")))?;
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::{
        LibsqlSnapshotSourceState, PrefixState, checkpoint_libsql_file, compress_gzip,
        decompress_gzip, ensure_source_state_stable, sha256_hex,
    };
    use tempfile::tempdir;

    #[test]
    fn gzip_round_trip_preserves_snapshot_bytes() {
        let original = b"sqlite bytes";
        let compressed = compress_gzip(original).expect("compress");
        assert_ne!(compressed, original);
        assert_eq!(decompress_gzip(&compressed).expect("decompress"), original);
    }

    #[test]
    fn sha256_hex_is_stable() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn prefix_state_detects_count_or_timestamp_changes() {
        let original = PrefixState {
            key_count: 3,
            latest_modified_epoch_ms: Some(100),
            fingerprint_sha256: "a".to_string(),
        };
        assert_eq!(original, original.clone());
        assert_ne!(
            original,
            PrefixState {
                key_count: 4,
                latest_modified_epoch_ms: Some(100),
                fingerprint_sha256: "a".to_string(),
            }
        );
        assert_ne!(
            original,
            PrefixState {
                key_count: 3,
                latest_modified_epoch_ms: Some(101),
                fingerprint_sha256: "a".to_string(),
            }
        );
        assert_ne!(
            original,
            PrefixState {
                key_count: 3,
                latest_modified_epoch_ms: Some(100),
                fingerprint_sha256: "b".to_string(),
            }
        );
    }

    #[test]
    fn source_state_guard_rejects_publish_when_sources_change() {
        let original = LibsqlSnapshotSourceState {
            videos: PrefixState {
                key_count: 3,
                latest_modified_epoch_ms: Some(100),
                fingerprint_sha256: "videos-a".to_string(),
            },
            preferences: PrefixState {
                key_count: 1,
                latest_modified_epoch_ms: Some(100),
                fingerprint_sha256: "prefs-a".to_string(),
            },
            tts_stats: PrefixState {
                key_count: 1,
                latest_modified_epoch_ms: Some(100),
                fingerprint_sha256: "tts-a".to_string(),
            },
        };
        let mut changed = original.clone();
        changed.videos.fingerprint_sha256 = "videos-b".to_string();

        assert!(ensure_source_state_stable(&original, &original).is_ok());
        assert!(ensure_source_state_stable(&original, &changed).is_err());
    }

    #[tokio::test]
    async fn checkpoint_libsql_file_accepts_wal_checkpoint_rows() {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("snapshot.db");
        let db = libsql::Builder::new_local(&db_path)
            .build()
            .await
            .expect("build db");
        let conn = db.connect().expect("connect db");
        conn.execute_batch(
            r#"
            CREATE TABLE test_rows (id INTEGER PRIMARY KEY, value TEXT);
            INSERT INTO test_rows (value) VALUES ('hello');
            "#,
        )
        .await
        .expect("seed db");

        checkpoint_libsql_file(&conn)
            .await
            .expect("checkpoint should succeed");
    }
}
