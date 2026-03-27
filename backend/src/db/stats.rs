use std::collections::{HashMap, HashSet};

use crate::models::Channel;

use super::{Store, StoreError};

async fn count_prefix(store: &Store, prefix: &str) -> Result<usize, StoreError> {
    Ok(store.list_keys(prefix).await?.len())
}

pub async fn count_summaries(store: &Store) -> Result<usize, StoreError> {
    count_prefix(store, "summaries/").await
}

pub async fn count_transcripts(store: &Store) -> Result<usize, StoreError> {
    count_prefix(store, "transcripts/").await
}

pub async fn count_videos(store: &Store) -> Result<usize, StoreError> {
    count_prefix(store, "videos/").await
}

pub async fn count_channels(store: &Store) -> Result<usize, StoreError> {
    count_prefix(store, "channels/").await
}

pub async fn summaries_by_channel(store: &Store) -> Result<Vec<(String, usize)>, StoreError> {
    let keys = store.list_keys("summaries/").await?;
    let video_ids: HashSet<String> = keys
        .into_iter()
        .filter_map(|k| {
            k.strip_prefix("summaries/")
                .and_then(|s| s.strip_suffix(".json"))
                .map(str::to_string)
        })
        .collect();
    count_resource_by_channel(store, &video_ids).await
}

pub async fn transcripts_by_channel(store: &Store) -> Result<Vec<(String, usize)>, StoreError> {
    let keys = store.list_keys("transcripts/").await?;
    let video_ids: HashSet<String> = keys
        .into_iter()
        .filter_map(|k| {
            k.strip_prefix("transcripts/")
                .and_then(|s| s.strip_suffix(".json"))
                .map(str::to_string)
        })
        .collect();
    count_resource_by_channel(store, &video_ids).await
}

pub async fn videos_by_channel(store: &Store) -> Result<Vec<(String, usize)>, StoreError> {
    let all_videos = super::videos::load_all_videos(store).await?;
    let video_ids: HashSet<String> = all_videos.into_iter().map(|v| v.id).collect();
    count_resource_by_channel(store, &video_ids).await
}

async fn count_resource_by_channel(
    store: &Store,
    resource_video_ids: &HashSet<String>,
) -> Result<Vec<(String, usize)>, StoreError> {
    let all_videos = super::videos::load_all_videos(store).await?;
    let channels: Vec<Channel> = store.load_all("channels/").await?;
    let channel_names: HashMap<String, String> =
        channels.into_iter().map(|c| (c.id, c.name)).collect();

    let mut counts: HashMap<String, usize> = HashMap::new();
    for video in &all_videos {
        if resource_video_ids.contains(&video.id) {
            let name = channel_names
                .get(&video.channel_id)
                .cloned()
                .unwrap_or_else(|| video.channel_id.clone());
            *counts.entry(name).or_insert(0) += 1;
        }
    }

    let mut result: Vec<(String, usize)> = counts.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}
