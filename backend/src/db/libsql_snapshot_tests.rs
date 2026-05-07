use super::{
    DELTA_SCHEMA_VERSION, LibsqlSnapshotDeltaOperation, LibsqlSnapshotDeltaRecord,
    LibsqlSnapshotSourceState, PrefixState, apply_delta_record, checkpoint_libsql_file,
    compress_gzip, decompress_gzip, reset_local_libsql_cache, sha256_hex,
};
use crate::models::{CanonicalVideoRecord, ContentStatus, UserPreferences};
use chrono::TimeZone;
use tempfile::tempdir;

#[test]
fn publish_loads_source_state_before_reading_snapshot_file() {
    let source = include_str!("libsql_snapshot.rs");
    let publish_start = source
        .find("pub async fn publish_libsql_snapshot")
        .expect("publish function exists");
    let publish_source = &source[publish_start..];
    let source_state = publish_source
        .find("load_source_state")
        .expect("publish loads source state");
    let checkpoint = publish_source
        .find("checkpoint_libsql_file")
        .expect("publish checkpoints snapshot file");
    let file_read = publish_source
        .find("tokio::fs::read")
        .expect("publish reads snapshot file");

    assert!(
        source_state < checkpoint && checkpoint < file_read,
        "snapshot publish must capture source generation before checkpointing, then checkpoint before reading the DB file"
    );
}

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
fn source_state_deserializes_generation_marker_shape() {
    let state: LibsqlSnapshotSourceState =
        serde_json::from_str(r#"{"generation":42}"#).expect("deserialize generation state");
    assert_eq!(state.generation, 42);
}

#[test]
fn source_state_deserializes_legacy_prefix_shape_for_schema_migration() {
    let state: LibsqlSnapshotSourceState = serde_json::from_str(
        r#"{
            "videos":{"key_count":3,"latest_modified_epoch_ms":100,"fingerprint_sha256":"v"},
            "preferences":{"key_count":1,"latest_modified_epoch_ms":101,"fingerprint_sha256":"p"},
            "tts_stats":{"key_count":1,"latest_modified_epoch_ms":102,"fingerprint_sha256":"t"}
        }"#,
    )
    .expect("deserialize legacy state");
    assert_eq!(state.generation, 0);
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

#[tokio::test]
async fn reset_local_libsql_cache_clears_persisted_fts_rows() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("snapshot.db");
    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .expect("build db");
    let conn = db.connect().expect("connect db");
    crate::db::sql_schema::initialize_sql_schema(&conn)
        .await
        .expect("init schema");
    crate::services::FtsIndex::new_with_db(db.clone(), Some(db_path.clone()))
        .await
        .expect("init fts");
    conn.execute(
        "INSERT INTO videos (id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, retry_count, quality_score) VALUES (?1, ?2, ?3, NULL, ?4, 0, 'ready', 'ready', 0, NULL)",
        libsql::params!["video-1", "channel-1", "Video", "2026-05-01T00:00:00Z"],
    )
    .await
    .expect("insert video");
    conn.execute(
        r#"
        INSERT INTO fts_search (
            chunk_id, video_id, channel_id, source_kind, source_key, section_title,
            chunk_text, video_title, channel_name, published_at, start_sec
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
        libsql::params![
            "chunk-1",
            "video-1",
            "channel-1",
            "summary",
            "video-1_summary",
            "Section",
            "stale searchable text",
            "Video",
            "Channel",
            "2026-05-01T00:00:00Z",
            0i64,
        ],
    )
    .await
    .expect("insert fts row");

    reset_local_libsql_cache(&conn).await.expect("reset cache");
    let fts = crate::services::FtsIndex::new_with_db(db, Some(db_path))
        .await
        .expect("recreate fts");

    assert_eq!(fts.doc_count().await, 0);
}

#[tokio::test]
async fn apply_delta_record_updates_sql_cache_semantically() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("snapshot.db");
    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .expect("build db");
    let conn = db.connect().expect("connect db");
    crate::db::sql_schema::initialize_sql_schema(&conn)
        .await
        .expect("init schema");

    let delta = LibsqlSnapshotDeltaRecord {
        schema_version: DELTA_SCHEMA_VERSION,
        generation: 7,
        created_at: "2026-05-01T00:00:00Z".to_string(),
        operations: vec![
            LibsqlSnapshotDeltaOperation::UpsertVideo {
                record: CanonicalVideoRecord {
                    id: "video-1".to_string(),
                    channel_id: "channel-1".to_string(),
                    title: "Delta video".to_string(),
                    thumbnail_url: Some("https://example.com/thumb.jpg".to_string()),
                    published_at: chrono::Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
                    is_short: false,
                    transcript_status: ContentStatus::Ready,
                    summary_status: ContentStatus::Pending,
                    retry_count: 2,
                    quality_score: Some(88),
                },
            },
            LibsqlSnapshotDeltaOperation::PutPreferences {
                user_id: "user-1".to_string(),
                data: UserPreferences {
                    channel_order: vec!["channel-1".to_string()],
                    channel_sort_mode: "custom".to_string(),
                    vocabulary_replacements: vec![],
                },
            },
            LibsqlSnapshotDeltaOperation::PutTtsStats {
                stats: super::super::TtsGenerationStats {
                    sample_count: 3,
                    total_words: 120,
                    total_duration_secs: 12.5,
                },
            },
            LibsqlSnapshotDeltaOperation::DeleteVideo {
                video_id: "video-1".to_string(),
            },
        ],
    };

    apply_delta_record(&conn, &delta)
        .await
        .expect("apply delta");

    let mut video_rows = conn
        .query(
            "SELECT COUNT(*) FROM videos WHERE id = ?1",
            libsql::params!["video-1"],
        )
        .await
        .expect("query videos");
    let video_count: i64 = video_rows
        .next()
        .await
        .expect("next video row")
        .expect("video count row")
        .get(0)
        .expect("video count");
    assert_eq!(video_count, 0);

    let mut pref_rows = conn
        .query(
            "SELECT data FROM preferences WHERE user_id = ?1",
            libsql::params!["user-1"],
        )
        .await
        .expect("query preferences");
    let pref_json: String = pref_rows
        .next()
        .await
        .expect("next pref row")
        .expect("pref row")
        .get(0)
        .expect("pref json");
    assert!(pref_json.contains("\"channel-1\""));

    let mut tts_rows = conn
        .query(
            "SELECT sample_count, total_words, total_duration_secs FROM tts_stats WHERE id = ?1",
            libsql::params!["global"],
        )
        .await
        .expect("query tts");
    let tts_row = tts_rows
        .next()
        .await
        .expect("next tts row")
        .expect("tts row");
    let sample_count: i64 = tts_row.get(0).expect("sample_count");
    let total_words: i64 = tts_row.get(1).expect("total_words");
    let total_duration: f64 = tts_row.get(2).expect("total_duration");
    assert_eq!(sample_count, 3);
    assert_eq!(total_words, 120);
    assert_eq!(total_duration, 12.5);
}
