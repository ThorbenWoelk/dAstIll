use crate::models::Channel;

use super::{Store, StoreError};

fn channel_key(id: &str) -> String {
    format!("channels/{id}.json")
}

pub async fn insert_channel(store: &Store, channel: &Channel) -> Result<(), StoreError> {
    store.put_json(&channel_key(&channel.id), channel).await
}

pub async fn get_channel(store: &Store, id: &str) -> Result<Option<Channel>, StoreError> {
    store.get_json(&channel_key(id)).await
}

pub async fn list_channels(store: &Store) -> Result<Vec<Channel>, StoreError> {
    let mut channels: Vec<Channel> = store.load_all("channels/").await?;
    channels.sort_by(|a, b| b.added_at.cmp(&a.added_at));
    Ok(channels)
}

pub async fn delete_channel(store: &Store, id: &str) -> Result<bool, StoreError> {
    let exists = store.key_exists(&channel_key(id)).await?;
    if !exists {
        return Ok(false);
    }

    let video_keys = store.list_keys("videos/").await?;
    for key in &video_keys {
        if let Some(video) = store.get_json::<crate::models::Video>(key).await? {
            if video.channel_id == id {
                super::highlights::delete_highlights_for_video(store, &video.id).await?;
                store
                    .delete_key(&format!("summaries/{}.json", video.id))
                    .await?;
                store
                    .delete_key(&format!("transcripts/{}.json", video.id))
                    .await?;
                store
                    .delete_key(&format!("video-info/{}.json", video.id))
                    .await?;
                store
                    .delete_prefix(&format!("search-sources/{}/", video.id))
                    .await?;
                super::search::delete_vectors_for_video(store, &video.id).await?;
                store.delete_key(key).await?;
            }
        }
    }

    store.delete_key(&channel_key(id)).await?;
    Ok(true)
}
