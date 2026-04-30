use crate::models::VideoInfo;

use super::video_info_missing_channel_identity;

fn build_video_info(channel_id: Option<&str>) -> VideoInfo {
    VideoInfo {
        video_id: "video-123".to_string(),
        watch_url: "https://www.youtube.com/watch?v=video-123".to_string(),
        title: "Video".to_string(),
        description: None,
        thumbnail_url: None,
        channel_name: None,
        channel_id: channel_id.map(str::to_string),
        published_at: None,
        duration_iso8601: None,
        duration_seconds: None,
        view_count: None,
    }
}

#[test]
fn missing_channel_identity_detects_absent_or_blank_channel_ids() {
    assert!(video_info_missing_channel_identity(&build_video_info(None)));
    assert!(video_info_missing_channel_identity(&build_video_info(
        Some("   ")
    )));
    assert!(!video_info_missing_channel_identity(&build_video_info(
        Some("UC1234567890123456789012")
    )));
}
