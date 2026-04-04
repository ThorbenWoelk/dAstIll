use std::collections::{HashMap, HashSet};

use crate::models::{
    Channel, Highlight, HighlightChannelGroup, HighlightSource, HighlightVideoGroup, Video,
};

use super::{Store, StoreError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HighlightMigrationStats {
    pub scanned: usize,
    pub copied: usize,
    pub skipped_duplicates: usize,
    pub remapped_ids: usize,
}

fn highlight_prefix(user_id: &str) -> String {
    format!("user-highlights/{user_id}/")
}

fn highlight_key(user_id: &str, id: i64) -> String {
    format!("{}{id}.json", highlight_prefix(user_id))
}

fn generate_highlight_id() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    let random = (std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
        % 10000) as i64;
    millis * 10000 + random
}

fn normalize_highlight_text(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn clamp_highlight_context(input: &str) -> String {
    const MAX_CONTEXT_CHARS: usize = 160;
    input.chars().take(MAX_CONTEXT_CHARS).collect()
}

async fn list_user_highlights(store: &Store, user_id: &str) -> Result<Vec<Highlight>, StoreError> {
    store.load_all(&highlight_prefix(user_id)).await
}

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

pub async fn create_highlight(
    store: &Store,
    user_id: &str,
    video_id: &str,
    source: HighlightSource,
    text: &str,
    prefix_context: &str,
    suffix_context: &str,
) -> Result<Highlight, StoreError> {
    let normalized_text = normalize_highlight_text(text);
    let prefix_context = clamp_highlight_context(prefix_context);
    let suffix_context = clamp_highlight_context(suffix_context);

    let existing = list_video_highlights(store, user_id, video_id).await?;
    for highlight in &existing {
        if highlight.source == source
            && normalize_highlight_text(&highlight.text) == normalized_text
            && highlight.prefix_context == prefix_context
            && highlight.suffix_context == suffix_context
        {
            return Ok(highlight.clone());
        }
    }

    let id = generate_highlight_id();
    let highlight = Highlight {
        id,
        video_id: video_id.to_string(),
        source,
        text: text.to_string(),
        prefix_context,
        suffix_context,
        created_at: chrono::Utc::now(),
    };

    store
        .put_json(&highlight_key(user_id, id), &highlight)
        .await?;
    Ok(highlight)
}

pub async fn list_video_highlights(
    store: &Store,
    user_id: &str,
    video_id: &str,
) -> Result<Vec<Highlight>, StoreError> {
    let mut filtered = list_user_highlights(store, user_id)
        .await?
        .into_iter()
        .filter(|highlight| highlight.video_id == video_id)
        .collect::<Vec<_>>();
    filtered.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then(right.id.cmp(&left.id))
    });
    Ok(filtered)
}

pub async fn delete_highlight(
    store: &Store,
    user_id: &str,
    highlight_id: i64,
) -> Result<bool, StoreError> {
    let key = highlight_key(user_id, highlight_id);
    let exists = store.key_exists(&key).await?;
    if exists {
        store.delete_key(&key).await?;
    }
    Ok(exists)
}

pub async fn migrate_user_highlights(
    store: &Store,
    from_user_id: &str,
    to_user_id: &str,
    dry_run: bool,
) -> Result<HighlightMigrationStats, StoreError> {
    let source_highlights = list_user_highlights(store, from_user_id).await?;
    let mut target_highlights = list_user_highlights(store, to_user_id).await?;
    let mut occupied_ids = target_highlights
        .iter()
        .map(|highlight| highlight.id)
        .collect::<std::collections::HashSet<_>>();
    let mut stats = HighlightMigrationStats::default();

    for highlight in source_highlights {
        stats.scanned += 1;

        if target_highlights
            .iter()
            .any(|existing| highlights_are_equivalent(existing, &highlight))
        {
            stats.skipped_duplicates += 1;
            continue;
        }

        let mut migrated = highlight.clone();
        if occupied_ids.contains(&migrated.id) {
            migrated.id = next_available_highlight_id(&occupied_ids);
            stats.remapped_ids += 1;
        }

        if !dry_run {
            store
                .put_json(&highlight_key(to_user_id, migrated.id), &migrated)
                .await?;
        }

        occupied_ids.insert(migrated.id);
        target_highlights.push(migrated);
        stats.copied += 1;
    }

    Ok(stats)
}

pub(crate) async fn delete_highlights_for_video(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    let keys = store.list_keys("user-highlights/").await?;
    for key in keys {
        let Some(highlight) = store.get_json::<Highlight>(&key).await? else {
            continue;
        };
        if highlight.video_id == video_id {
            store.delete_key(&key).await?;
        }
    }
    Ok(())
}

pub async fn list_highlights_grouped(
    store: &Store,
) -> Result<Vec<HighlightChannelGroup>, StoreError> {
    let keys = store.list_keys("user-highlights/").await?;
    let mut all_highlights = Vec::new();
    for key in keys {
        if let Some(highlight) = store.get_json::<Highlight>(&key).await? {
            all_highlights.push(highlight);
        }
    }
    group_highlights(store, all_highlights).await
}

pub async fn list_highlights_grouped_for_user(
    store: &Store,
    user_id: &str,
) -> Result<Vec<HighlightChannelGroup>, StoreError> {
    group_highlights(store, list_user_highlights(store, user_id).await?).await
}

async fn group_highlights(
    store: &Store,
    all_highlights: Vec<Highlight>,
) -> Result<Vec<HighlightChannelGroup>, StoreError> {
    if all_highlights.is_empty() {
        return Ok(Vec::new());
    }

    let video_ids = all_highlights
        .iter()
        .map(|highlight| highlight.video_id.as_str())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let video_map = super::videos::get_videos(store, &video_ids, false).await?;

    let channel_ids = video_map
        .values()
        .map(|video| video.channel_id.clone())
        .collect::<HashSet<_>>();
    let mut channel_map = HashMap::<String, Channel>::new();
    for channel_id in channel_ids {
        if let Some(channel) = super::channels::get_channel(store, &channel_id).await? {
            channel_map.insert(channel.id.clone(), channel);
        }
    }

    let mut groups = group_highlights_from_maps(all_highlights, &video_map, &channel_map);

    for group in &mut groups {
        group
            .videos
            .sort_by(|left, right| right.published_at.cmp(&left.published_at));
        for video_group in &mut group.videos {
            video_group.highlights.sort_by(|left, right| {
                right
                    .created_at
                    .cmp(&left.created_at)
                    .then(right.id.cmp(&left.id))
            });
        }
    }

    Ok(groups)
}

fn group_highlights_from_maps(
    all_highlights: Vec<Highlight>,
    video_map: &HashMap<String, Video>,
    channel_map: &HashMap<String, Channel>,
) -> Vec<HighlightChannelGroup> {
    let mut groups: Vec<HighlightChannelGroup> = Vec::new();

    for highlight in all_highlights {
        let Some(video) = video_map.get(&highlight.video_id) else {
            continue;
        };
        let Some(channel) = channel_map.get(&video.channel_id) else {
            continue;
        };

        let channel_index = groups
            .iter()
            .position(|group| group.channel_id == channel.id)
            .unwrap_or_else(|| {
                groups.push(HighlightChannelGroup {
                    source_id: channel.id.clone(),
                    channel_id: channel.id.clone(),
                    provider: crate::models::infer_provider_kind_for_source_id(&channel.id),
                    source_kind: crate::models::infer_source_kind_for_source_id(&channel.id),
                    channel_name: channel.name.clone(),
                    channel_thumbnail_url: channel.thumbnail_url.clone(),
                    videos: Vec::new(),
                });
                groups.len() - 1
            });

        let video_index = groups[channel_index]
            .videos
            .iter()
            .position(|group| group.video_id == video.id)
            .unwrap_or_else(|| {
                groups[channel_index].videos.push(HighlightVideoGroup {
                    source_id: channel.id.clone(),
                    video_id: video.id.clone(),
                    item_id: video.id.clone(),
                    provider: crate::models::infer_provider_kind_for_source_id(&channel.id),
                    item_kind: crate::models::infer_item_kind_for_source_kind(
                        crate::models::infer_source_kind_for_source_id(&channel.id),
                    ),
                    title: video.title.clone(),
                    thumbnail_url: video.thumbnail_url.clone(),
                    published_at: video.published_at,
                    highlights: Vec::new(),
                });
                groups[channel_index].videos.len() - 1
            });

        groups[channel_index].videos[video_index]
            .highlights
            .push(highlight);
    }

    groups
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use std::collections::HashMap;

    use super::{
        group_highlights_from_maps, highlights_are_equivalent, next_available_highlight_id,
    };
    use crate::models::{Channel, ContentStatus, Highlight, HighlightSource, Video};

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

        let groups =
            group_highlights_from_maps(vec![older, newer.clone()], &video_map, &channel_map);
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
}
