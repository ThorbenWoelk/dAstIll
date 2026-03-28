use std::collections::HashMap;

use crate::models::{
    CanonicalChannelRecord, Channel, OTHERS_CHANNEL_ID, OTHERS_CHANNEL_NAME,
    UserChannelSubscription,
};

use super::{Store, StoreError, build_channel_from_records};

fn canonical_channel_key(id: &str) -> String {
    format!("channels/{id}.json")
}

fn canonical_to_channel(record: CanonicalChannelRecord) -> Channel {
    Channel {
        id: record.id,
        handle: record.handle,
        name: record.name,
        thumbnail_url: record.thumbnail_url,
        added_at: chrono::Utc::now(),
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    }
}

pub fn build_virtual_others_channel(added_at: chrono::DateTime<chrono::Utc>) -> Channel {
    Channel {
        id: OTHERS_CHANNEL_ID.to_string(),
        handle: None,
        name: OTHERS_CHANNEL_NAME.to_string(),
        thumbnail_url: None,
        added_at,
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    }
}

pub async fn insert_channel(store: &Store, channel: &Channel) -> Result<(), StoreError> {
    let record = CanonicalChannelRecord {
        id: channel.id.clone(),
        handle: channel.handle.clone(),
        name: channel.name.clone(),
        thumbnail_url: channel.thumbnail_url.clone(),
    };
    store
        .put_json(&canonical_channel_key(&channel.id), &record)
        .await
}

pub async fn get_canonical_channel(
    store: &Store,
    id: &str,
) -> Result<Option<CanonicalChannelRecord>, StoreError> {
    store.get_json(&canonical_channel_key(id)).await
}

pub async fn get_channel(store: &Store, id: &str) -> Result<Option<Channel>, StoreError> {
    Ok(get_canonical_channel(store, id)
        .await?
        .map(canonical_to_channel))
}

pub async fn list_canonical_channels(
    store: &Store,
) -> Result<Vec<CanonicalChannelRecord>, StoreError> {
    let mut channels: Vec<CanonicalChannelRecord> = store.load_all("channels/").await?;
    channels.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(channels)
}

pub async fn list_channels(store: &Store) -> Result<Vec<Channel>, StoreError> {
    Ok(list_canonical_channels(store)
        .await?
        .into_iter()
        .map(canonical_to_channel)
        .collect())
}

pub async fn list_user_channels(store: &Store, user_id: &str) -> Result<Vec<Channel>, StoreError> {
    let canonical_by_id = list_canonical_channels(store)
        .await?
        .into_iter()
        .map(|channel| (channel.id.clone(), channel))
        .collect::<HashMap<_, _>>();
    let subscriptions = super::list_user_channel_subscriptions(store, user_id).await?;

    let mut channels = subscriptions
        .into_iter()
        .filter_map(|subscription| {
            canonical_by_id
                .get(&subscription.channel_id)
                .map(|canonical| build_channel_from_records(canonical, &subscription))
        })
        .collect::<Vec<_>>();
    channels.sort_by(|left, right| {
        right
            .added_at
            .cmp(&left.added_at)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(channels)
}

pub async fn list_user_channel_ids(
    store: &Store,
    user_id: &str,
) -> Result<Vec<String>, StoreError> {
    Ok(super::list_user_channel_subscriptions(store, user_id)
        .await?
        .into_iter()
        .map(|subscription| subscription.channel_id)
        .collect())
}

pub async fn list_user_channels_with_virtual_others(
    store: &Store,
    user_id: &str,
) -> Result<Vec<Channel>, StoreError> {
    let mut channels = list_user_channels(store, user_id).await?;
    let subscribed_channel_ids = channels
        .iter()
        .map(|channel| channel.id.clone())
        .collect::<Vec<_>>();
    let memberships = super::list_user_video_memberships(store, user_id).await?;
    let other_video_ids =
        super::list_user_other_video_ids(store, user_id, &subscribed_channel_ids).await?;

    if !other_video_ids.is_empty() {
        let added_at = memberships
            .iter()
            .find(|membership| {
                other_video_ids
                    .iter()
                    .any(|video_id| video_id == &membership.video_id)
            })
            .map(|membership| membership.added_at)
            .unwrap_or_else(chrono::Utc::now);
        channels.push(build_virtual_others_channel(added_at));
    }

    Ok(channels)
}

pub async fn get_user_channel(
    store: &Store,
    user_id: &str,
    channel_id: &str,
) -> Result<Option<Channel>, StoreError> {
    let canonical = get_canonical_channel(store, channel_id).await?;
    let subscription = super::get_user_channel_subscription(store, user_id, channel_id).await?;
    Ok(match (canonical, subscription) {
        (Some(canonical), Some(subscription)) => {
            Some(build_channel_from_records(&canonical, &subscription))
        }
        _ => None,
    })
}

pub async fn save_user_channel(
    store: &Store,
    user_id: &str,
    channel: &Channel,
) -> Result<(), StoreError> {
    let subscription = UserChannelSubscription {
        channel_id: channel.id.clone(),
        added_at: channel.added_at,
        earliest_sync_date: channel.earliest_sync_date,
        earliest_sync_date_user_set: channel.earliest_sync_date_user_set,
    };
    super::put_user_channel_subscription(store, user_id, &subscription).await
}

pub async fn delete_channel(store: &Store, id: &str) -> Result<bool, StoreError> {
    let exists = store.key_exists(&canonical_channel_key(id)).await?;
    if !exists {
        return Ok(false);
    }

    let all_videos = super::videos::load_all_videos(store).await?;
    for video in all_videos {
        if video.channel_id != id {
            continue;
        }

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
        store
            .delete_key(&format!("videos/{}.json", video.id))
            .await?;
    }

    store.delete_key(&canonical_channel_key(id)).await?;
    Ok(true)
}
