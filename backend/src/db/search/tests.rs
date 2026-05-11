use super::*;
use crate::models::{CanonicalChannelRecord, Channel};

fn state(
    content_hash: &str,
    index_status: &str,
    embedding_model: Option<&str>,
) -> SearchSourceState {
    SearchSourceState {
        id: 1,
        source_generation: 1,
        video_id: "video-1".to_string(),
        source_kind: SearchSourceKind::Transcript,
        content_hash: content_hash.to_string(),
        embedding_model: embedding_model.map(ToOwned::to_owned),
        index_status: index_status.to_string(),
        last_indexed_at: None,
        last_error: None,
    }
}

#[test]
fn refresh_required_when_source_missing() {
    assert!(should_refresh_search_source(
        None,
        "hash-a",
        true,
        Some("embed-a")
    ));
}

#[test]
fn ready_source_with_same_hash_and_model_does_not_refresh() {
    let current = state("hash-a", "ready", Some("embed-a"));

    assert!(!should_refresh_search_source(
        Some(&current),
        "hash-a",
        true,
        Some("embed-a")
    ));
}

#[test]
fn failed_source_refreshes_even_when_hash_matches() {
    let current = state("hash-a", "failed", Some("embed-a"));

    assert!(should_refresh_search_source(
        Some(&current),
        "hash-a",
        true,
        Some("embed-a")
    ));
}

#[test]
fn embedding_model_drift_refreshes_only_when_semantic_is_enabled() {
    let current = state("hash-a", "ready", Some("embed-a"));

    assert!(should_refresh_search_source(
        Some(&current),
        "hash-a",
        true,
        Some("embed-b")
    ));
    assert!(!should_refresh_search_source(
        Some(&current),
        "hash-a",
        false,
        Some("embed-b")
    ));
}

#[test]
fn canonical_channel_json_uses_legacy_channel_defaults_when_read_as_full_channel() {
    let json = r#"{"id":"channel-1","handle":"demo","name":"Demo","thumbnail_url":null}"#;

    let record: CanonicalChannelRecord =
        serde_json::from_str(json).expect("canonical channel record should deserialize");
    assert_eq!(record.name, "Demo");

    let channel: Channel =
        serde_json::from_str(json).expect("full channel should deserialize with legacy defaults");
    assert_eq!(channel.id, "channel-1");
    assert_eq!(channel.name, "Demo");
    assert_eq!(
        channel.added_at,
        chrono::DateTime::<chrono::Utc>::UNIX_EPOCH
    );
}
