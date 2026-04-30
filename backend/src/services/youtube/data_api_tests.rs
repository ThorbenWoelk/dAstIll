use super::{DataApiVideoItem, LiveBroadcastState};

fn item_from_json(json: &str) -> DataApiVideoItem {
    serde_json::from_str(json).expect("valid Data API video item")
}

#[test]
fn live_state_treats_regular_video_as_vod() {
    let item = item_from_json(
        r#"{
            "id": "vod12345678",
            "snippet": {
                "title": "Regular VOD",
                "liveBroadcastContent": "none"
            }
        }"#,
    );

    assert_eq!(item.live_broadcast_state(), LiveBroadcastState::Vod);
    assert!(item.live_broadcast_state().is_ingestable());
}

#[test]
fn live_state_rejects_upcoming_livestream() {
    let item = item_from_json(
        r#"{
            "id": "live1234567",
            "snippet": {
                "title": "Upcoming stream",
                "liveBroadcastContent": "upcoming"
            },
            "liveStreamingDetails": {
                "scheduledStartTime": "2026-04-19T18:00:00Z"
            }
        }"#,
    );

    assert_eq!(item.live_broadcast_state(), LiveBroadcastState::Upcoming);
    assert!(!item.live_broadcast_state().is_ingestable());
}

#[test]
fn live_state_rejects_active_livestream() {
    let item = item_from_json(
        r#"{
            "id": "live1234567",
            "snippet": {
                "title": "Live now",
                "liveBroadcastContent": "live"
            },
            "liveStreamingDetails": {
                "actualStartTime": "2026-04-19T18:00:00Z"
            }
        }"#,
    );

    assert_eq!(item.live_broadcast_state(), LiveBroadcastState::Live);
    assert!(!item.live_broadcast_state().is_ingestable());
}

#[test]
fn live_state_allows_completed_livestream() {
    let item = item_from_json(
        r#"{
            "id": "done1234567",
            "snippet": {
                "title": "Completed stream",
                "liveBroadcastContent": "none"
            },
            "liveStreamingDetails": {
                "actualStartTime": "2026-04-19T18:00:00Z",
                "actualEndTime": "2026-04-19T19:30:00Z"
            }
        }"#,
    );

    assert_eq!(
        item.live_broadcast_state(),
        LiveBroadcastState::CompletedLive
    );
    assert!(item.live_broadcast_state().is_ingestable());
}
