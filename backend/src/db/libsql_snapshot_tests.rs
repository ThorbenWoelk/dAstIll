use super::{PrefixState, checkpoint_libsql_file, compress_gzip, decompress_gzip, sha256_hex};
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
