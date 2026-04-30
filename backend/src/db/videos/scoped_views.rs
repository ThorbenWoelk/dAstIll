use super::*;

const CHANNEL_WINDOW_BATCH_SIZE: usize = 200;

pub async fn load_channel_snapshot_data(
    store: &Store,
    channel_id: &str,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
) -> Result<Option<ChannelSnapshotData>, StoreError> {
    let stored_channels = crate::db::channels::list_channels(store).await?;
    let subscribed = subscribed_channel_ids(&stored_channels);
    let options = VideoListOptions {
        limit,
        offset,
        is_short,
        acknowledged,
        queue_filter,
        published_at_not_before: None,
    };

    if channel_id == OTHERS_CHANNEL_ID {
        if !has_unsubscribed_channel_videos(store).await? {
            return Ok(None);
        }

        return Ok(Some(
            build_channel_snapshot_data(
                store,
                build_virtual_others_channel(),
                &subscribed,
                options,
            )
            .await?,
        ));
    }

    let channel = stored_channels
        .into_iter()
        .find(|channel| channel.id == channel_id);
    match channel {
        Some(channel) => Ok(Some(
            build_channel_snapshot_data(store, channel, &subscribed, options).await?,
        )),
        None => Ok(None),
    }
}

pub async fn load_workspace_bootstrap_data(
    store: &Store,
    preferred_channel_id: Option<&str>,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
) -> Result<WorkspaceBootstrapData, StoreError> {
    let channels = list_channels_with_virtual_others(store).await?;
    let options = VideoListOptions {
        limit,
        offset,
        is_short,
        acknowledged,
        queue_filter,
        published_at_not_before: None,
    };
    let subscribed = subscribed_channel_ids(
        &channels
            .iter()
            .filter(|channel| channel.id != OTHERS_CHANNEL_ID)
            .cloned()
            .collect::<Vec<_>>(),
    );
    let selected_channel = preferred_channel_id
        .and_then(|id| channels.iter().find(|c| c.id == id))
        .cloned()
        .or_else(|| channels.first().cloned());
    let selected_channel_id = selected_channel.as_ref().map(|c| c.id.clone());
    let snapshot = match selected_channel {
        Some(channel) => {
            Some(build_channel_snapshot_data(store, channel, &subscribed, options).await?)
        }
        None => None,
    };
    Ok(WorkspaceBootstrapData {
        channels,
        selected_channel_id,
        snapshot,
    })
}

fn overlay_user_video_state(
    mut video: Video,
    user_video_states: &std::collections::HashMap<String, crate::models::UserVideoState>,
) -> Video {
    video.acknowledged = user_video_states
        .get(&video.id)
        .map(|state| state.acknowledged)
        .unwrap_or(false);
    video
}

fn user_scoped_evaluation_filter_allows(
    video: &Video,
    summary: Option<&crate::models::Summary>,
    queue_filter: Option<QueueFilter>,
) -> bool {
    if queue_filter != Some(QueueFilter::EvaluationsOnly) {
        return true;
    }
    if video.transcript_status != ContentStatus::Ready
        || video.summary_status != ContentStatus::Ready
    {
        return false;
    }
    summary.is_some_and(summary_needs_evaluation_filter)
}

async fn video_matches_user_scoped_evaluation_filter(
    store: &Store,
    video: &Video,
    queue_filter: Option<QueueFilter>,
) -> Result<bool, StoreError> {
    if queue_filter != Some(QueueFilter::EvaluationsOnly) {
        return Ok(true);
    }
    let summary = store
        .get_json::<crate::models::Summary>(&format!("summaries/{}.json", video.id))
        .await?;
    Ok(user_scoped_evaluation_filter_allows(
        video,
        summary.as_ref(),
        queue_filter,
    ))
}

pub async fn get_user_scoped_video(
    store: &Store,
    user_id: Option<&str>,
    allowed_channel_ids: &[String],
    allowed_other_video_ids: &[String],
    video_id: &str,
    include_summary: bool,
) -> Result<Option<Video>, StoreError> {
    let Some(video) = get_video(store, video_id, include_summary).await? else {
        return Ok(None);
    };

    if !allowed_channel_ids.iter().any(|id| id == &video.channel_id)
        && !allowed_other_video_ids.iter().any(|id| id == &video.id)
    {
        return Ok(None);
    }

    let user_states = match user_id {
        Some(user_id) => crate::db::list_user_video_states(store, user_id).await?,
        None => std::collections::HashMap::new(),
    };

    Ok(Some(overlay_user_video_state(video, &user_states)))
}

pub async fn list_user_scoped_videos_by_channel(
    store: &Store,
    user_id: Option<&str>,
    channel_id: &str,
    allowed_channel_ids: &[String],
    allowed_other_video_ids: &[String],
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
) -> Result<Option<ChannelVideoPageData>, StoreError> {
    if channel_id != OTHERS_CHANNEL_ID && !allowed_channel_ids.iter().any(|id| id == channel_id) {
        return Ok(None);
    }

    let user_states = match user_id {
        Some(user_id) => crate::db::list_user_video_states(store, user_id).await?,
        None => std::collections::HashMap::new(),
    };
    let matches_filters = |video: &Video| {
        is_short.is_none_or(|value| video.is_short == value)
            && acknowledged.is_none_or(|value| video.acknowledged == value)
            && video_visible_in_list(video, queue_filter)
            && match queue_filter {
                Some(QueueFilter::AnyIncomplete) => {
                    video.transcript_status != ContentStatus::Ready
                        || video.summary_status != ContentStatus::Ready
                }
                Some(QueueFilter::TranscriptsOnly) => {
                    video.transcript_status != ContentStatus::Ready
                }
                Some(QueueFilter::SummariesOnly) => {
                    video.transcript_status == ContentStatus::Ready
                        && video.summary_status != ContentStatus::Ready
                }
                Some(QueueFilter::EvaluationsOnly) => {
                    video.transcript_status == ContentStatus::Ready
                        && video.summary_status == ContentStatus::Ready
                }
                None => true,
            }
    };

    if channel_id == OTHERS_CHANNEL_ID {
        let allowed_other_video_ids = allowed_other_video_ids
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let subscribed_channel_ids = allowed_channel_ids.iter().cloned().collect::<HashSet<_>>();
        let mut filtered = Vec::new();
        for video in get_videos(
            store,
            &allowed_other_video_ids.iter().collect::<Vec<_>>(),
            false,
        )
        .await?
        .into_values()
        {
            let video = overlay_user_video_state(video, &user_states);
            if !allowed_other_video_ids.contains(&video.id)
                || subscribed_channel_ids.contains(&video.channel_id)
                || !matches_filters(&video)
                || !video_matches_user_scoped_evaluation_filter(store, &video, queue_filter).await?
            {
                continue;
            }
            filtered.push(video);
        }

        filtered.sort_by(|left, right| right.published_at.cmp(&left.published_at));
        let total_len = filtered.len();
        let videos = filtered
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();
        let next_offset = offset + videos.len();

        return Ok(Some(ChannelVideoPageData {
            videos,
            has_more: total_len > next_offset,
            next_offset: (total_len > next_offset).then_some(next_offset),
        }));
    }

    let target_len = offset.saturating_add(limit).saturating_add(1);
    let mut matched = Vec::new();
    let mut scanned = 0usize;

    loop {
        let batch =
            list_channel_videos_window(store, channel_id, CHANNEL_WINDOW_BATCH_SIZE, scanned, true)
                .await?;
        if batch.is_empty() {
            break;
        }
        let batch_len = batch.len();
        scanned = scanned.saturating_add(batch_len);

        for video in batch {
            let video = overlay_user_video_state(video, &user_states);
            if !matches_filters(&video)
                || !video_matches_user_scoped_evaluation_filter(store, &video, queue_filter).await?
            {
                continue;
            }
            matched.push(video);
            if matched.len() >= target_len {
                break;
            }
        }

        if matched.len() >= target_len || batch_len < CHANNEL_WINDOW_BATCH_SIZE {
            break;
        }
    }

    let has_more = matched.len() > offset.saturating_add(limit);
    let videos = matched
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    let next_offset = offset + videos.len();

    Ok(Some(ChannelVideoPageData {
        videos,
        has_more,
        next_offset: has_more.then_some(next_offset),
    }))
}

#[cfg(test)]
#[path = "scoped_views_tests.rs"]
mod scoped_views_tests;
