use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;

use aws_sdk_s3::primitives::ByteStream;
use chrono::{SecondsFormat, Utc};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use tokio::sync::{Mutex, Notify};

use crate::models::{CanonicalVideoRecord, ContentStatus, UserPreferences};

use super::{StoreError, format_aws_error};

const MANIFEST_KEY: &str = "runtime-cache/libsql/current.json";
const SOURCE_GENERATION_KEY: &str = "runtime-cache/libsql/source-generation.json";
const DELTA_PREFIX: &str = "runtime-cache/libsql/deltas";
const SNAPSHOT_PREFIX: &str = "runtime-cache/libsql/snapshots";
const SNAPSHOT_SCHEMA_VERSION: u32 = 2;
const DELTA_SCHEMA_VERSION: u32 = 1;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LibsqlSnapshotSourceState {
    pub generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixState {
    pub key_count: usize,
    pub latest_modified_epoch_ms: Option<u64>,
    pub fingerprint_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SnapshotGenerationWire {
    generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum SnapshotSourceStateWire {
    Generation(SnapshotGenerationWire),
    Legacy {
        videos: PrefixState,
        preferences: PrefixState,
        tts_stats: PrefixState,
    },
}

impl<'de> Deserialize<'de> for LibsqlSnapshotSourceState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match SnapshotSourceStateWire::deserialize(deserializer)? {
            SnapshotSourceStateWire::Generation(state) => Ok(Self {
                generation: state.generation,
            }),
            SnapshotSourceStateWire::Legacy { .. } => Ok(Self { generation: 0 }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LibsqlSourceGenerationMarker {
    schema_version: u32,
    generation: u64,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibsqlSnapshotDeltaRecord {
    pub schema_version: u32,
    pub generation: u64,
    pub created_at: String,
    pub operations: Vec<LibsqlSnapshotDeltaOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LibsqlSnapshotDeltaOperation {
    UpsertVideo {
        record: CanonicalVideoRecord,
    },
    DeleteVideo {
        video_id: String,
    },
    PutPreferences {
        user_id: String,
        data: UserPreferences,
    },
    PutTtsStats {
        stats: super::TtsGenerationStats,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibsqlSnapshotRestore {
    Restored {
        base_generation: u64,
        target_generation: u64,
    },
    Missing,
    Failed,
}

impl LibsqlSnapshotRestore {
    pub fn restored(self) -> bool {
        matches!(self, Self::Restored { .. })
    }

    pub fn replay_range(self) -> Option<(u64, u64)> {
        match self {
            Self::Restored {
                base_generation,
                target_generation,
            } if target_generation > base_generation => Some((base_generation, target_generation)),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct LibsqlSourceGenerationTracker {
    inner: Arc<LibsqlSourceGenerationTrackerInner>,
}

struct LibsqlSourceGenerationTrackerInner {
    s3: aws_sdk_s3::Client,
    bucket: String,
    current_generation: AtomicU64,
    lock: Mutex<()>,
}

impl LibsqlSourceGenerationTracker {
    pub async fn new(
        s3: aws_sdk_s3::Client,
        bucket: impl Into<String>,
    ) -> Result<Self, StoreError> {
        let bucket = bucket.into();
        let current_generation = current_generation_value(&s3, &bucket).await?;
        Ok(Self {
            inner: Arc::new(LibsqlSourceGenerationTrackerInner {
                s3,
                bucket,
                current_generation: AtomicU64::new(current_generation),
                lock: Mutex::new(()),
            }),
        })
    }

    pub async fn append_delta(
        &self,
        operations: Vec<LibsqlSnapshotDeltaOperation>,
    ) -> Result<u64, StoreError> {
        let _guard = self.inner.lock.lock().await;
        let current = self.inner.current_generation.load(Ordering::Acquire);
        let next = current.saturating_add(1);
        let delta = LibsqlSnapshotDeltaRecord {
            schema_version: DELTA_SCHEMA_VERSION,
            generation: next,
            created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            operations,
        };
        put_json(&self.inner.s3, &self.inner.bucket, &delta_key(next), &delta).await?;
        store_source_generation_marker(&self.inner.s3, &self.inner.bucket, next).await?;
        self.inner.current_generation.store(next, Ordering::Release);
        Ok(next)
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
        Ok(Some((base_generation, target_generation))) => LibsqlSnapshotRestore::Restored {
            base_generation,
            target_generation,
        },
        Ok(None) => LibsqlSnapshotRestore::Missing,
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
) -> Result<Option<(u64, u64)>, StoreError> {
    let Some(manifest) = get_json::<LibsqlSnapshotManifest>(s3, bucket, MANIFEST_KEY).await? else {
        tracing::info!("libSQL snapshot manifest not found - using S3 rebuild path");
        return Ok(None);
    };

    if manifest.schema_version != SNAPSHOT_SCHEMA_VERSION {
        tracing::warn!(
            manifest_schema_version = manifest.schema_version,
            expected_schema_version = SNAPSHOT_SCHEMA_VERSION,
            "libSQL snapshot schema version mismatch - using S3 rebuild path"
        );
        return Ok(None);
    }

    let current_source_state = load_source_state(s3, bucket).await?;
    if current_source_state.generation < manifest.source_state.generation {
        tracing::warn!(
            manifest_generation = manifest.source_state.generation,
            current_generation = current_source_state.generation,
            "libSQL source generation regressed behind snapshot manifest - using S3 rebuild path"
        );
        return Ok(None);
    }

    let Some(compressed) = get_bytes(s3, bucket, &manifest.snapshot_key).await? else {
        tracing::warn!(
            snapshot_key = %manifest.snapshot_key,
            "libSQL snapshot object missing - using S3 rebuild path"
        );
        return Ok(None);
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
        base_generation = manifest.source_state.generation,
        target_generation = current_source_state.generation,
        byte_size = manifest.byte_size,
        compressed_byte_size = manifest.compressed_byte_size,
        "libSQL snapshot restored"
    );
    Ok(Some((
        manifest.source_state.generation,
        current_source_state.generation,
    )))
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
    if let Err(err) =
        prune_delta_log_up_to_generation(s3, bucket, manifest.source_state.generation).await
    {
        tracing::warn!(
            error = %err,
            up_to_generation = manifest.source_state.generation,
            "failed to prune libSQL snapshot delta log"
        );
    }

    tracing::info!(
        snapshot_key = %manifest.snapshot_key,
        source_generation = manifest.source_state.generation,
        byte_size = manifest.byte_size,
        compressed_byte_size = manifest.compressed_byte_size,
        "libSQL snapshot published"
    );

    Ok(manifest)
}

pub async fn replay_libsql_snapshot_deltas(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    conn: &libsql::Connection,
    from_generation: u64,
    to_generation: u64,
) -> Result<usize, StoreError> {
    if to_generation <= from_generation {
        return Ok(0);
    }

    let mut applied_generations = 0usize;
    for generation in (from_generation + 1)..=to_generation {
        let delta_key = delta_key(generation);
        let Some(delta) = get_json::<LibsqlSnapshotDeltaRecord>(s3, bucket, &delta_key).await?
        else {
            return Err(StoreError::Other(format!(
                "missing libSQL delta record for generation {generation}"
            )));
        };
        if delta.schema_version != DELTA_SCHEMA_VERSION {
            return Err(StoreError::Other(format!(
                "unsupported libSQL delta schema version {} for generation {}",
                delta.schema_version, generation
            )));
        }
        if delta.generation != generation {
            return Err(StoreError::Other(format!(
                "libSQL delta generation mismatch: key expected {generation}, payload had {}",
                delta.generation
            )));
        }
        apply_delta_record(conn, &delta).await?;
        applied_generations += 1;
    }

    Ok(applied_generations)
}

pub async fn reset_local_libsql_cache(conn: &libsql::Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        r#"
        DELETE FROM videos;
        DELETE FROM preferences;
        DELETE FROM tts_stats;
        "#,
    )
    .await
    .map_err(|err| StoreError::Other(format!("failed to reset local libSQL cache: {err}")))?;
    Ok(())
}

fn delta_key(generation: u64) -> String {
    format!("{DELTA_PREFIX}/{generation:020}.json")
}

async fn apply_delta_record(
    conn: &libsql::Connection,
    delta: &LibsqlSnapshotDeltaRecord,
) -> Result<(), StoreError> {
    for operation in &delta.operations {
        match operation {
            LibsqlSnapshotDeltaOperation::UpsertVideo { record } => {
                apply_upsert_video_delta(conn, record).await?;
            }
            LibsqlSnapshotDeltaOperation::DeleteVideo { video_id } => {
                conn.execute(
                    "DELETE FROM videos WHERE id = ?1",
                    libsql::params![video_id.clone()],
                )
                .await
                .map_err(|err| {
                    StoreError::Other(format!(
                        "failed to apply delete_video delta for {}: {err}",
                        video_id
                    ))
                })?;
            }
            LibsqlSnapshotDeltaOperation::PutPreferences { user_id, data } => {
                let json = serde_json::to_string(data)?;
                conn.execute(
                    "INSERT INTO preferences (user_id, data) VALUES (?1, ?2) ON CONFLICT(user_id) DO UPDATE SET data = excluded.data",
                    libsql::params![user_id.clone(), json],
                )
                .await
                .map_err(|err| {
                    StoreError::Other(format!(
                        "failed to apply put_preferences delta for {}: {err}",
                        user_id
                    ))
                })?;
            }
            LibsqlSnapshotDeltaOperation::PutTtsStats { stats } => {
                conn.execute(
                    r#"INSERT INTO tts_stats (id, sample_count, total_words, total_duration_secs)
                       VALUES (?1, ?2, ?3, ?4)
                       ON CONFLICT(id) DO UPDATE SET
                         sample_count = excluded.sample_count,
                         total_words = excluded.total_words,
                         total_duration_secs = excluded.total_duration_secs"#,
                    libsql::params![
                        "global",
                        stats.sample_count as i64,
                        stats.total_words as i64,
                        stats.total_duration_secs
                    ],
                )
                .await
                .map_err(|err| {
                    StoreError::Other(format!("failed to apply put_tts_stats delta: {err}"))
                })?;
            }
        }
    }

    Ok(())
}

pub(crate) async fn apply_upsert_video_delta(
    conn: &libsql::Connection,
    record: &CanonicalVideoRecord,
) -> Result<(), StoreError> {
    conn.execute(
        r#"INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, retry_count, quality_score)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
           ON CONFLICT(id) DO UPDATE SET
             channel_id = excluded.channel_id,
             title = excluded.title,
             thumbnail_url = excluded.thumbnail_url,
             published_at = excluded.published_at,
             is_short = excluded.is_short,
             transcript_status = excluded.transcript_status,
             summary_status = excluded.summary_status,
             retry_count = excluded.retry_count,
             quality_score = excluded.quality_score"#,
        libsql::params![
            record.id.clone(),
            record.channel_id.clone(),
            record.title.clone(),
            record.thumbnail_url.clone(),
            record.published_at.to_rfc3339(),
            record.is_short as i64,
            content_status_to_str(record.transcript_status),
            content_status_to_str(record.summary_status),
            record.retry_count as i64,
            record.quality_score.map(|value| value as i64),
        ],
    )
    .await
    .map_err(|err| {
        StoreError::Other(format!(
            "failed to apply upsert_video delta for {}: {err}",
            record.id
        ))
    })?;
    Ok(())
}

fn content_status_to_str(status: ContentStatus) -> &'static str {
    match status {
        ContentStatus::Pending => "pending",
        ContentStatus::Loading => "loading",
        ContentStatus::Ready => "ready",
        ContentStatus::Failed => "failed",
    }
}

async fn prune_delta_log_up_to_generation(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    up_to_generation: u64,
) -> Result<(), StoreError> {
    let mut continuation_token: Option<String> = None;

    loop {
        let mut request = s3.list_objects_v2().bucket(bucket).prefix(DELTA_PREFIX);
        if let Some(token) = continuation_token.take() {
            request = request.continuation_token(token);
        }

        let output = request
            .send()
            .await
            .map_err(|err| StoreError::S3(format_aws_error(&err)))?;

        if let Some(contents) = output.contents {
            for object in contents {
                let Some(key) = object.key else {
                    continue;
                };
                let Some(generation) = delta_generation_from_key(&key) else {
                    continue;
                };
                if generation > up_to_generation {
                    continue;
                }
                s3.delete_object()
                    .bucket(bucket)
                    .key(key)
                    .send()
                    .await
                    .map_err(|err| StoreError::S3(format_aws_error(&err)))?;
            }
        }

        if output.is_truncated == Some(true) {
            continuation_token = output.next_continuation_token;
        } else {
            break;
        }
    }

    Ok(())
}

fn delta_generation_from_key(key: &str) -> Option<u64> {
    let suffix = key.strip_prefix(&format!("{DELTA_PREFIX}/"))?;
    let generation = suffix.strip_suffix(".json")?;
    generation.parse().ok()
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
        generation: current_generation_value(s3, bucket).await?,
    })
}

async fn current_generation_value(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
) -> Result<u64, StoreError> {
    let Some(marker) =
        get_json::<LibsqlSourceGenerationMarker>(s3, bucket, SOURCE_GENERATION_KEY).await?
    else {
        return Ok(0);
    };

    if marker.schema_version != 1 {
        tracing::warn!(
            marker_schema_version = marker.schema_version,
            expected_schema_version = 1,
            "libSQL source generation marker schema mismatch - treating generation as 0"
        );
        return Ok(0);
    }

    Ok(marker.generation)
}

async fn store_source_generation_marker(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    generation: u64,
) -> Result<(), StoreError> {
    let marker = LibsqlSourceGenerationMarker {
        schema_version: 1,
        generation,
        updated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    };
    put_json(s3, bucket, SOURCE_GENERATION_KEY, &marker).await
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
#[path = "tests.rs"]
mod tests;
