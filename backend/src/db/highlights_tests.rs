use chrono::Utc;
use std::collections::HashMap;

use super::{generate_highlight_id, group_highlights_from_maps, normalize_highlight_text};
use crate::models::{Channel, ContentStatus, Highlight, HighlightSource, Video};

fn highlights_are_equivalent(left: &Highlight, right: &Highlight) -> bool {
    left.video_id == right.video_id
        && left.source == right.source
        && normalize_highlight_text(&left.text) == normalize_highlight_text(&right.text)
        && left.prefix_context == right.prefix_context
        && left.suffix_context == right.suffix_context
}

fn next_available_highlight_id(occupied_ids: &std::collections::HashSet<i64>) -> i64 {
    loop {
        let candidate = generate_highlight_id();
        if !occupied_ids.contains(&candidate) {
            return candidate;
        }
    }
}

fn sample_highlight(id: i64) -> Highlight {
    Highlight {
        id,
        video_id: "video-123".to_string(),
        source: HighlightSource::Summary,
        text: "Apple  Intelligence   runs  locally".to_string(),
        prefix_context: "prefix".to_string(),
        suffix_context: "suffix".to_string(),
        created_at: Utc::now(),
    }
}

fn sample_video(id: &str, channel_id: &str, title: &str) -> Video {
    Video {
        id: id.to_string(),
        channel_id: channel_id.to_string(),
        title: title.to_string(),
        thumbnail_url: None,
        published_at: Utc::now(),
        is_short: false,
        transcript_status: ContentStatus::Ready,
        summary_status: ContentStatus::Ready,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    }
}

fn sample_channel(id: &str, name: &str) -> Channel {
    Channel {
        id: id.to_string(),
        handle: None,
        name: name.to_string(),
        thumbnail_url: None,
        added_at: Utc::now(),
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    }
}

#[test]
fn equivalent_highlights_ignore_text_whitespace_differences() {
    let left = sample_highlight(1);
    let mut right = sample_highlight(2);
    right.text = "Apple Intelligence runs locally".to_string();

    assert!(highlights_are_equivalent(&left, &right));
}

#[test]
fn equivalent_highlights_require_matching_video_context() {
    let left = sample_highlight(1);
    let mut right = sample_highlight(2);
    right.video_id = "video-999".to_string();

    assert!(!highlights_are_equivalent(&left, &right));
}

#[test]
fn next_available_highlight_id_avoids_existing_ids() {
    let occupied = std::collections::HashSet::from([1_i64, 2_i64, 3_i64]);
    let next_id = next_available_highlight_id(&occupied);

    assert!(!occupied.contains(&next_id));
}

#[test]
fn group_highlights_from_maps_groups_related_rows_together() {
    let older = Highlight {
        id: 1,
        video_id: "video-123".to_string(),
        source: HighlightSource::Transcript,
        text: "older".to_string(),
        prefix_context: String::new(),
        suffix_context: String::new(),
        created_at: Utc::now() - chrono::Duration::minutes(5),
    };
    let newer = Highlight {
        id: 2,
        video_id: "video-123".to_string(),
        source: HighlightSource::Summary,
        text: "newer".to_string(),
        prefix_context: String::new(),
        suffix_context: String::new(),
        created_at: Utc::now(),
    };

    let video_map = HashMap::from([(
        "video-123".to_string(),
        sample_video("video-123", "channel-123", "Video Title"),
    )]);
    let channel_map = HashMap::from([(
        "channel-123".to_string(),
        sample_channel("channel-123", "Channel Name"),
    )]);

    let groups = group_highlights_from_maps(vec![older, newer.clone()], &video_map, &channel_map);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].channel_id, "channel-123");
    assert_eq!(groups[0].videos.len(), 1);
    assert_eq!(groups[0].videos[0].video_id, "video-123");
    assert_eq!(groups[0].videos[0].highlights.len(), 2);
    assert!(
        groups[0].videos[0]
            .highlights
            .iter()
            .any(|highlight| highlight.id == newer.id)
    );
}
