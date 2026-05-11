use std::collections::HashSet;

use chrono::Utc;

use super::{
    VideoListParams, cached_video_info_needs_refresh, enrich_video_info,
    resolve_manual_video_target_channel_id,
};
use crate::db;
use crate::handlers::query::{QueueTab, VideoTypeFilter};
use crate::models::{ContentStatus, Video, VideoInfo};

fn build_video_info(duration_seconds: Option<u64>, duration_iso8601: Option<&str>) -> VideoInfo {
    VideoInfo {
        video_id: "video-123".to_string(),
        watch_url: "https://www.youtube.com/watch?v=video-123".to_string(),
        title: "Video".to_string(),
        description: None,
        thumbnail_url: None,
        channel_name: None,
        channel_id: None,
        published_at: None,
        duration_iso8601: duration_iso8601.map(str::to_string),
        duration_seconds,
        view_count: None,
    }
}

fn build_video() -> Video {
    Video {
        id: "video-123".to_string(),
        channel_id: "channel-123".to_string(),
        title: "Stored Video Title".to_string(),
        thumbnail_url: Some("https://example.com/thumb.jpg".to_string()),
        published_at: Utc::now(),
        is_short: false,
        transcript_status: ContentStatus::Ready,
        summary_status: ContentStatus::Ready,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    }
}

#[test]
fn cached_video_info_needs_refresh_when_duration_is_missing() {
    assert!(cached_video_info_needs_refresh(&build_video_info(
        None, None
    )));
    assert!(cached_video_info_needs_refresh(&build_video_info(
        None,
        Some(""),
    )));
}

#[test]
fn cached_video_info_with_known_duration_does_not_need_refresh() {
    assert!(!cached_video_info_needs_refresh(&build_video_info(
        Some(185),
        None,
    )));
    assert!(!cached_video_info_needs_refresh(&build_video_info(
        None,
        Some("PT3M5S"),
    )));
}

#[test]
fn cached_video_info_needs_refresh_when_description_is_placeholder() {
    let mut info = build_video_info(Some(185), Some("PT3M5S"));
    info.description = Some(
        "Auf YouTube findest du die angesagtesten Videos und Tracks. Außerdem kannst du eigene Inhalte hochladen und mit Freunden oder gleich der ganzen Welt teilen".to_string(),
    );
    assert!(cached_video_info_needs_refresh(&info));
}

#[test]
fn enrich_video_info_fills_missing_fields_from_video() {
    let video = build_video();
    let mut info = build_video_info(Some(185), Some("PT3M5S"));
    info.title.clear();

    enrich_video_info(&mut info, &video);

    assert_eq!(info.title, video.title);
    assert_eq!(info.thumbnail_url, video.thumbnail_url);
    assert_eq!(info.published_at, Some(video.published_at));
}

#[test]
fn enrich_video_info_preserves_fetched_fields_when_present() {
    let video = build_video();
    let published_at = Utc::now() - chrono::Duration::days(3);
    let mut info = VideoInfo {
        video_id: "video-123".to_string(),
        watch_url: "https://www.youtube.com/watch?v=video-123".to_string(),
        title: "Fetched Title".to_string(),
        description: None,
        thumbnail_url: Some("https://example.com/fetched-thumb.jpg".to_string()),
        channel_name: None,
        channel_id: None,
        published_at: Some(published_at),
        duration_iso8601: Some("PT3M5S".to_string()),
        duration_seconds: Some(185),
        view_count: None,
    };

    enrich_video_info(&mut info, &video);

    assert_eq!(info.title, "Fetched Title");
    assert_eq!(
        info.thumbnail_url,
        Some("https://example.com/fetched-thumb.jpg".to_string())
    );
    assert_eq!(info.published_at, Some(published_at));
}

#[test]
fn video_list_params_resolve_limits_and_filters() {
    let params = VideoListParams {
        limit: Some(500),
        offset: Some(7),
        include_shorts: Some(false),
        video_type: None,
        acknowledged: Some(true),
        queue_only: Some(true),
        queue_tab: None,
    };

    assert_eq!(params.limit_or_default(), 100);
    assert_eq!(params.offset_or_default(), 7);
    assert_eq!(params.is_short_filter(), Some(false));
    assert_eq!(params.acknowledged_filter(), Some(true));
    assert_eq!(params.queue_filter(), Some(db::QueueFilter::AnyIncomplete));
}

#[test]
fn video_list_params_prefer_explicit_queue_tab() {
    let params = VideoListParams {
        limit: None,
        offset: None,
        include_shorts: Some(true),
        video_type: Some(VideoTypeFilter::Short),
        acknowledged: None,
        queue_only: Some(true),
        queue_tab: Some(QueueTab::Summaries),
    };

    assert_eq!(params.limit_or_default(), 20);
    assert_eq!(params.offset_or_default(), 0);
    assert_eq!(params.is_short_filter(), Some(true));
    assert_eq!(params.queue_filter(), Some(db::QueueFilter::SummariesOnly));
}

#[test]
fn manual_video_targets_subscribed_channel_when_available() {
    let subscribed = HashSet::from(["UC_SUBSCRIBED".to_string()]);

    assert_eq!(
        resolve_manual_video_target_channel_id("UC_SUBSCRIBED", &subscribed),
        "UC_SUBSCRIBED"
    );
}

#[test]
fn manual_video_targets_others_when_channel_is_not_subscribed() {
    let subscribed = HashSet::from(["UC_SUBSCRIBED".to_string()]);

    assert_eq!(
        resolve_manual_video_target_channel_id("UC_OTHER", &subscribed),
        "__others__"
    );
}
