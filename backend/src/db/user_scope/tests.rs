use chrono::{TimeZone, Utc};

use super::build_channel_from_records;
use crate::models::{CanonicalChannelRecord, UserChannelSubscription};

#[test]
fn legacy_subscription_without_sync_floor_gets_added_month_floor() {
    let channel = build_channel_from_records(
        &CanonicalChannelRecord {
            id: "podcast:series:show".to_string(),
            handle: None,
            name: "Show".to_string(),
            thumbnail_url: None,
        },
        &UserChannelSubscription {
            channel_id: "podcast:series:show".to_string(),
            added_at: Utc.with_ymd_and_hms(2026, 4, 20, 12, 30, 0).unwrap(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        },
    );

    assert_eq!(
        channel.earliest_sync_date,
        Some(Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap())
    );
    assert!(!channel.earliest_sync_date_user_set);
}

#[test]
fn manual_subscription_without_sync_floor_stays_empty() {
    let channel = build_channel_from_records(
        &CanonicalChannelRecord {
            id: "podcast:series:show".to_string(),
            handle: None,
            name: "Show".to_string(),
            thumbnail_url: None,
        },
        &UserChannelSubscription {
            channel_id: "podcast:series:show".to_string(),
            added_at: Utc.with_ymd_and_hms(2026, 4, 20, 12, 30, 0).unwrap(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: true,
        },
    );

    assert_eq!(channel.earliest_sync_date, None);
    assert!(channel.earliest_sync_date_user_set);
}
