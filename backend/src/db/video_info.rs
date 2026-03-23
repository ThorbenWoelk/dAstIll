use crate::models::VideoInfo;
use crate::services::youtube::placeholder::is_site_wide_placeholder_description;

use super::{Store, StoreError};

fn video_info_key(video_id: &str) -> String {
    format!("video-info/{video_id}.json")
}

pub async fn upsert_video_info(store: &Store, info: &VideoInfo) -> Result<(), StoreError> {
    store.put_json(&video_info_key(&info.video_id), info).await
}

pub async fn get_video_info(
    store: &Store,
    video_id: &str,
) -> Result<Option<VideoInfo>, StoreError> {
    store.get_json(&video_info_key(video_id)).await
}

pub async fn list_video_ids_missing_info(
    store: &Store,
    limit: usize,
) -> Result<Vec<String>, StoreError> {
    let video_keys = store.list_keys("videos/").await?;
    let info_keys: std::collections::HashSet<String> =
        store.list_keys("video-info/").await?.into_iter().collect();

    let mut results = Vec::new();
    for key in video_keys {
        let video_id = key
            .strip_prefix("videos/")
            .and_then(|s| s.strip_suffix(".json"))
            .unwrap_or_default();
        if video_id.is_empty() {
            continue;
        }
        let info_key = format!("video-info/{video_id}.json");
        if !info_keys.contains(&info_key) {
            results.push(video_id.to_string());
            if results.len() >= limit {
                break;
            }
        }
    }
    Ok(results)
}

pub async fn list_video_ids_for_info_refresh(
    store: &Store,
    limit: usize,
) -> Result<Vec<String>, StoreError> {
    let all_videos: Vec<crate::models::Video> = store.load_all("videos/").await?;
    let mut sorted = all_videos;
    sorted.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    Ok(sorted.into_iter().take(limit).map(|v| v.id).collect())
}

/// Stored `video-info` rows whose description is the generic YouTube site blurb (bad scrape).
pub async fn list_video_ids_with_placeholder_descriptions(
    store: &Store,
    limit: usize,
) -> Result<Vec<String>, StoreError> {
    let keys = store.list_keys("video-info/").await?;
    let mut out = Vec::new();
    for key in keys {
        if out.len() >= limit {
            break;
        }
        let Some(video_id) = key
            .strip_prefix("video-info/")
            .and_then(|s| s.strip_suffix(".json"))
        else {
            continue;
        };
        if let Some(info) = get_video_info(store, video_id).await? {
            if info
                .description
                .as_deref()
                .is_some_and(is_site_wide_placeholder_description)
            {
                out.push(video_id.to_string());
            }
        }
    }
    Ok(out)
}
