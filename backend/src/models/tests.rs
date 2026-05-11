use chrono::TimeZone;

use super::Channel;

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
