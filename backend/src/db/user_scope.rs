use std::collections::{HashMap, HashSet};

use crate::models::{
    CanonicalChannelRecord, Channel, UserChannelSubscription, UserVideoMembership, UserVideoState,
};

use super::{Store, StoreError};

fn user_channel_subscription_prefix(user_id: &str) -> String {
    format!("user-channel-subscriptions/{user_id}/")
}

fn user_channel_subscription_key(user_id: &str, channel_id: &str) -> String {
    format!("{}{}.json", user_channel_subscription_prefix(user_id), channel_id)
}

fn user_video_membership_prefix(user_id: &str) -> String {
    format!("user-video-memberships/{user_id}/")
}

fn user_video_membership_key(user_id: &str, video_id: &str) -> String {
    format!("{}{}.json", user_video_membership_prefix(user_id), video_id)
}

fn user_video_state_prefix(user_id: &str) -> String {
    format!("user-video-states/{user_id}/")
}

fn user_video_state_key(user_id: &str, video_id: &str) -> String {
    format!("{}{}.json", user_video_state_prefix(user_id), video_id)
}

pub async fn get_user_channel_subscription(
    store: &Store,
    user_id: &str,
    channel_id: &str,
) -> Result<Option<UserChannelSubscription>, StoreError> {
    store
        .get_json(&user_channel_subscription_key(user_id, channel_id))
        .await
}

pub async fn list_user_channel_subscriptions(
    store: &Store,
    user_id: &str,
) -> Result<Vec<UserChannelSubscription>, StoreError> {
    let mut subscriptions: Vec<UserChannelSubscription> = store
        .load_all(&user_channel_subscription_prefix(user_id))
        .await?;
    subscriptions.sort_by(|left, right| {
        right
            .added_at
            .cmp(&left.added_at)
            .then_with(|| left.channel_id.cmp(&right.channel_id))
    });
    Ok(subscriptions)
}

pub async fn put_user_channel_subscription(
    store: &Store,
    user_id: &str,
    subscription: &UserChannelSubscription,
) -> Result<(), StoreError> {
    store
        .put_json(
            &user_channel_subscription_key(user_id, &subscription.channel_id),
            subscription,
        )
        .await
}

pub async fn delete_user_channel_subscription(
    store: &Store,
    user_id: &str,
    channel_id: &str,
) -> Result<bool, StoreError> {
    let key = user_channel_subscription_key(user_id, channel_id);
    let exists = store.key_exists(&key).await?;
    if exists {
        store.delete_key(&key).await?;
    }
    Ok(exists)
}

pub async fn ensure_user_seeded_channel_subscription(
    store: &Store,
    user_id: &str,
    channel_id: &str,
) -> Result<(), StoreError> {
    if get_user_channel_subscription(store, user_id, channel_id)
        .await?
        .is_some()
    {
        return Ok(());
    }

    let subscription = UserChannelSubscription {
        channel_id: channel_id.to_string(),
        added_at: chrono::Utc::now(),
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    };
    put_user_channel_subscription(store, user_id, &subscription).await
}

pub async fn list_user_video_memberships(
    store: &Store,
    user_id: &str,
) -> Result<Vec<UserVideoMembership>, StoreError> {
    let mut memberships: Vec<UserVideoMembership> =
        store.load_all(&user_video_membership_prefix(user_id)).await?;
    memberships.sort_by(|left, right| {
        right
            .added_at
            .cmp(&left.added_at)
            .then_with(|| left.video_id.cmp(&right.video_id))
    });
    Ok(memberships)
}

pub async fn put_user_video_membership(
    store: &Store,
    user_id: &str,
    membership: &UserVideoMembership,
) -> Result<(), StoreError> {
    store
        .put_json(&user_video_membership_key(user_id, &membership.video_id), membership)
        .await
}

pub async fn delete_user_video_membership(
    store: &Store,
    user_id: &str,
    video_id: &str,
) -> Result<bool, StoreError> {
    let key = user_video_membership_key(user_id, video_id);
    let exists = store.key_exists(&key).await?;
    if exists {
        store.delete_key(&key).await?;
    }
    Ok(exists)
}

pub async fn list_user_video_membership_ids(
    store: &Store,
    user_id: &str,
) -> Result<HashSet<String>, StoreError> {
    Ok(list_user_video_memberships(store, user_id)
        .await?
        .into_iter()
        .map(|membership| membership.video_id)
        .collect())
}

pub async fn list_user_other_video_ids(
    store: &Store,
    user_id: &str,
    subscribed_channel_ids: &[String],
) -> Result<Vec<String>, StoreError> {
    let subscribed_channel_ids = subscribed_channel_ids
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let memberships = list_user_video_memberships(store, user_id).await?;
    let all_videos = super::videos::load_all_videos(store).await?;
    let channel_by_video_id = all_videos
        .into_iter()
        .map(|video| (video.id, video.channel_id))
        .collect::<HashMap<_, _>>();

    Ok(memberships
        .into_iter()
        .filter(|membership| {
            channel_by_video_id
                .get(&membership.video_id)
                .is_some_and(|channel_id| !subscribed_channel_ids.contains(channel_id))
        })
        .map(|membership| membership.video_id)
        .collect())
}

pub async fn get_user_video_state(
    store: &Store,
    user_id: &str,
    video_id: &str,
) -> Result<Option<UserVideoState>, StoreError> {
    store.get_json(&user_video_state_key(user_id, video_id)).await
}

pub async fn put_user_video_state(
    store: &Store,
    user_id: &str,
    state: &UserVideoState,
) -> Result<(), StoreError> {
    store
        .put_json(&user_video_state_key(user_id, &state.video_id), state)
        .await
}

pub async fn list_user_video_states(
    store: &Store,
    user_id: &str,
) -> Result<HashMap<String, UserVideoState>, StoreError> {
    Ok(store
        .load_all::<UserVideoState>(&user_video_state_prefix(user_id))
        .await?
        .into_iter()
        .map(|state| (state.video_id.clone(), state))
        .collect())
}

pub fn build_channel_from_records(
    canonical: &CanonicalChannelRecord,
    subscription: &UserChannelSubscription,
) -> Channel {
    Channel {
        id: canonical.id.clone(),
        handle: canonical.handle.clone(),
        name: canonical.name.clone(),
        thumbnail_url: canonical.thumbnail_url.clone(),
        added_at: subscription.added_at,
        earliest_sync_date: subscription.earliest_sync_date,
        earliest_sync_date_user_set: subscription.earliest_sync_date_user_set,
    }
}
