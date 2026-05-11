use super::{canonical_to_channel, default_earliest_sync_date_floor};
use crate::models::CanonicalChannelRecord;
use chrono::TimeZone;
use chrono::Utc;

#[test]
fn default_floor_is_start_of_utc_month() {
    let t = Utc.with_ymd_and_hms(2026, 3, 28, 15, 30, 0).unwrap();
    let floor = default_earliest_sync_date_floor(t);
    assert_eq!(floor, Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap());
}

#[test]
fn canonical_channel_gets_default_sync_floor_when_read_without_subscription() {
    let channel = canonical_to_channel(CanonicalChannelRecord {
        id: "podcast:series:show".to_string(),
        handle: None,
        name: "Show".to_string(),
        thumbnail_url: None,
    });

    assert!(channel.earliest_sync_date.is_some());
    assert!(!channel.earliest_sync_date_user_set);
}
