use chrono::Utc;
use std::collections::HashMap;

use super::build_mini_summary_item;
use crate::models::{ContentStatus, Summary, Video, VideoInfo};

#[test]
fn build_mini_summary_item_falls_back_without_video_info() {
    let video = Video {
        id: "video-1".to_string(),
        channel_id: "channel-1".to_string(),
        title: "Example title".to_string(),
        thumbnail_url: Some("https://example.com/thumb.jpg".to_string()),
        published_at: Utc::now(),
        is_short: false,
        transcript_status: ContentStatus::Ready,
        summary_status: ContentStatus::Ready,
        acknowledged: true,
        retry_count: 0,
        quality_score: None,
    };
    let summary = Summary {
        video_id: video.id.clone(),
        content: "Summary content".to_string(),
        model_used: None,
        quality_score: None,
        quality_note: None,
        quality_model_used: None,
        summary_tags: Vec::new(),
        summary_tags_evaluated: false,
    };
    let channel_name_by_id =
        std::collections::HashMap::from([(video.channel_id.clone(), "HealthyGamerGG".to_string())]);

    let item = build_mini_summary_item(&video, &summary, None, &channel_name_by_id);

    assert_eq!(item.video_id, "video-1");
    assert_eq!(item.channel_name, "HealthyGamerGG");
    assert_eq!(item.watch_url, "https://www.youtube.com/watch?v=video-1");
    assert_eq!(item.summary_content, "Summary content");
    assert!(item.read);
}

#[test]
fn build_mini_summary_item_prefers_video_info_when_present() {
    let video = Video {
        id: "video-1".to_string(),
        channel_id: "channel-1".to_string(),
        title: "Video row title".to_string(),
        thumbnail_url: None,
        published_at: Utc::now(),
        is_short: false,
        transcript_status: ContentStatus::Ready,
        summary_status: ContentStatus::Ready,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    };
    let summary = Summary {
        video_id: video.id.clone(),
        content: "Summary content".to_string(),
        model_used: None,
        quality_score: None,
        quality_note: None,
        quality_model_used: None,
        summary_tags: Vec::new(),
        summary_tags_evaluated: false,
    };
    let info = VideoInfo {
        video_id: video.id.clone(),
        watch_url: "https://example.com/watch".to_string(),
        title: "Video info title".to_string(),
        description: None,
        thumbnail_url: Some("https://example.com/info-thumb.jpg".to_string()),
        channel_name: Some("Info channel".to_string()),
        channel_id: Some(video.channel_id.clone()),
        published_at: None,
        duration_iso8601: None,
        duration_seconds: None,
        view_count: None,
    };

    let item = build_mini_summary_item(&video, &summary, Some(&info), &HashMap::new());

    assert_eq!(item.channel_name, "Info channel");
    assert_eq!(item.watch_url, "https://example.com/watch");
    assert_eq!(
        item.thumbnail_url.as_deref(),
        Some("https://example.com/info-thumb.jpg")
    );
    assert!(!item.read);
}
