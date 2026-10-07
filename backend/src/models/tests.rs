use chrono::TimeZone;

use super::{Channel, Highlight};

#[test]
fn channel_deserialization_defaults_legacy_added_at() {
    let channel: Channel = serde_json::from_str(
        r#"{
            "id":"chan-1",
            "handle":"@legacy",
            "name":"Legacy Channel",
            "thumbnail_url":null,
            "earliest_sync_date":null,
            "earliest_sync_date_user_set":false
        }"#,
    )
    .expect("legacy channel JSON should deserialize");

    assert_eq!(
        channel.added_at,
        chrono::Utc
            .timestamp_opt(0, 0)
            .single()
            .expect("unix epoch should be representable")
    );
}

#[test]
fn highlight_ids_are_sent_as_strings_and_read_from_either_form() {
    let stored_as_number = r#"{"id":17590000000001234,"video_id":"v","source":"summary","text":"t","prefix_context":"","suffix_context":"","created_at":"2026-10-07T00:00:00Z"}"#;
    let highlight: Highlight = serde_json::from_str(stored_as_number).unwrap();
    assert_eq!(highlight.id, 17_590_000_000_001_234);

    let json = serde_json::to_value(&highlight).unwrap();
    assert_eq!(json["id"], "17590000000001234");

    let round_trip: Highlight = serde_json::from_value(json).unwrap();
    assert_eq!(round_trip.id, highlight.id);
}
