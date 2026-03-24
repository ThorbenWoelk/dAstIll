use crate::models::{
    Highlight, HighlightChannelGroup, HighlightSource, HighlightVideoGroup, Video,
};

use super::{Store, StoreError};

fn highlight_key(id: i64) -> String {
    format!("highlights/{id}.json")
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

pub async fn create_highlight(
    store: &Store,
    video_id: &str,
    source: HighlightSource,
    text: &str,
    prefix_context: &str,
    suffix_context: &str,
) -> Result<Highlight, StoreError> {
    let normalized_text = normalize_highlight_text(text);
    let prefix_context = clamp_highlight_context(prefix_context);
    let suffix_context = clamp_highlight_context(suffix_context);

    // Check for duplicates
    let existing = list_video_highlights(store, video_id).await?;
    for h in &existing {
        if h.source == source
            && normalize_highlight_text(&h.text) == normalized_text
            && h.prefix_context == prefix_context
            && h.suffix_context == suffix_context
        {
            return Ok(h.clone());
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

    store.put_json(&highlight_key(id), &highlight).await?;
    Ok(highlight)
}

pub async fn list_video_highlights(
    store: &Store,
    video_id: &str,
) -> Result<Vec<Highlight>, StoreError> {
    let all: Vec<Highlight> = store.load_all("highlights/").await?;
    let mut filtered: Vec<Highlight> = all.into_iter().filter(|h| h.video_id == video_id).collect();
    filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));
    Ok(filtered)
}

pub async fn delete_highlight(store: &Store, highlight_id: i64) -> Result<bool, StoreError> {
    let key = highlight_key(highlight_id);
    let exists = store.key_exists(&key).await?;
    if exists {
        store.delete_key(&key).await?;
    }
    Ok(exists)
}

pub(crate) async fn delete_highlights_for_video(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    let highlights = list_video_highlights(store, video_id).await?;
    for h in highlights {
        store.delete_key(&highlight_key(h.id)).await?;
    }
    Ok(())
}

pub async fn list_highlights_grouped(
    store: &Store,
) -> Result<Vec<HighlightChannelGroup>, StoreError> {
    let all_highlights: Vec<Highlight> = store.load_all("highlights/").await?;
    if all_highlights.is_empty() {
        return Ok(Vec::new());
    }

    let all_videos: Vec<Video> = super::videos::load_all_videos(store).await?;
    let all_channels: Vec<crate::models::Channel> = super::channels::list_channels(store).await?;

    let video_map: std::collections::HashMap<&str, &Video> =
        all_videos.iter().map(|v| (v.id.as_str(), v)).collect();
    let channel_map: std::collections::HashMap<&str, &crate::models::Channel> =
        all_channels.iter().map(|c| (c.id.as_str(), c)).collect();

    let mut groups: Vec<HighlightChannelGroup> = Vec::new();

    for highlight in all_highlights {
        let Some(video) = video_map.get(highlight.video_id.as_str()) else {
            continue;
        };
        let Some(channel) = channel_map.get(video.channel_id.as_str()) else {
            continue;
        };

        let channel_index = groups
            .iter()
            .position(|g| g.channel_id == channel.id)
            .unwrap_or_else(|| {
                groups.push(HighlightChannelGroup {
                    channel_id: channel.id.clone(),
                    channel_name: channel.name.clone(),
                    channel_thumbnail_url: channel.thumbnail_url.clone(),
                    videos: Vec::new(),
                });
                groups.len() - 1
            });

        let video_index = groups[channel_index]
            .videos
            .iter()
            .position(|v| v.video_id == video.id)
            .unwrap_or_else(|| {
                groups[channel_index].videos.push(HighlightVideoGroup {
                    video_id: video.id.clone(),
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

    // Sort: videos by published_at DESC, highlights by created_at DESC
    for group in &mut groups {
        group
            .videos
            .sort_by(|a, b| b.published_at.cmp(&a.published_at));
        for vg in &mut group.videos {
            vg.highlights
                .sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));
        }
    }

    Ok(groups)
}
