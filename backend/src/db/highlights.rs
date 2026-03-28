use crate::models::{
    Highlight, HighlightChannelGroup, HighlightSource, HighlightVideoGroup, Video,
};

use super::{Store, StoreError};

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

    let all_videos: Vec<Video> = super::videos::load_all_videos(store).await?;
    let all_channels = super::channels::list_canonical_channels(store).await?;

    let video_map = all_videos
        .iter()
        .map(|video| (video.id.as_str(), video))
        .collect::<std::collections::HashMap<_, _>>();
    let channel_map = all_channels
        .iter()
        .map(|channel| (channel.id.as_str(), channel))
        .collect::<std::collections::HashMap<_, _>>();

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
            .position(|group| group.channel_id == channel.id)
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
            .position(|group| group.video_id == video.id)
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
