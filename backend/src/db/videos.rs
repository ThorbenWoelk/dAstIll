use crate::models::{Channel, ContentStatus, Video};

use super::{
    ChannelSnapshotData, QueueFilter, Store, StoreError, VideoInsertOutcome,
    WorkspaceBootstrapData,
};

fn video_key(id: &str) -> String {
    format!("videos/{id}.json")
}

pub async fn insert_video(store: &Store, video: &Video) -> Result<VideoInsertOutcome, StoreError> {
    let existing = store.get_json::<Video>(&video_key(&video.id)).await?;
    let already_exists = existing.is_some();

    let merged = if let Some(existing) = existing {
        Video {
            id: video.id.clone(),
            channel_id: video.channel_id.clone(),
            title: video.title.clone(),
            thumbnail_url: video.thumbnail_url.clone(),
            published_at: video.published_at,
            is_short: video.is_short,
            transcript_status: existing.transcript_status,
            summary_status: existing.summary_status,
            acknowledged: existing.acknowledged,
            retry_count: existing.retry_count,
            quality_score: existing.quality_score,
        }
    } else {
        video.clone()
    };

    store.put_json(&video_key(&video.id), &merged).await?;

    if already_exists {
        tracing::debug!(video_id = %video.id, title = %video.title, "found existing video");
        Ok(VideoInsertOutcome::Existing)
    } else {
        tracing::info!(video_id = %video.id, title = %video.title, "inserted new video");
        Ok(VideoInsertOutcome::Inserted)
    }
}

pub async fn bulk_insert_videos(
    store: &Store,
    videos: Vec<Video>,
) -> Result<usize, StoreError> {
    let mut inserted = 0;
    for video in &videos {
        if matches!(insert_video(store, video).await?, VideoInsertOutcome::Inserted) {
            inserted += 1;
        }
    }
    Ok(inserted)
}

pub async fn get_video(store: &Store, id: &str) -> Result<Option<Video>, StoreError> {
    let mut video: Option<Video> = store.get_json(&video_key(id)).await?;
    if let Some(ref mut v) = video {
        if let Some(summary) = store
            .get_json::<crate::models::Summary>(&format!("summaries/{id}.json"))
            .await?
        {
            v.quality_score = summary.quality_score;
        }
    }
    Ok(video)
}

async fn load_all_videos(store: &Store) -> Result<Vec<Video>, StoreError> {
    store.load_all("videos/").await
}

pub async fn list_videos_by_channel(
    store: &Store,
    channel_id: &str,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
) -> Result<Vec<Video>, StoreError> {
    let all = load_all_videos(store).await?;
    let mut filtered: Vec<Video> = all
        .into_iter()
        .filter(|v| v.channel_id == channel_id)
        .filter(|v| is_short.is_none_or(|s| v.is_short == s))
        .filter(|v| acknowledged.is_none_or(|a| v.acknowledged == a))
        .filter(|v| match queue_filter {
            Some(QueueFilter::AnyIncomplete) => {
                v.transcript_status != ContentStatus::Ready
                    || v.summary_status != ContentStatus::Ready
            }
            Some(QueueFilter::TranscriptsOnly) => {
                v.transcript_status != ContentStatus::Ready
            }
            Some(QueueFilter::SummariesOnly) => {
                v.transcript_status == ContentStatus::Ready
                    && v.summary_status != ContentStatus::Ready
            }
            Some(QueueFilter::EvaluationsOnly) => {
                v.transcript_status == ContentStatus::Ready
                    && v.summary_status == ContentStatus::Ready
            }
            None => true,
        })
        .collect();

    filtered.sort_by(|a, b| b.published_at.cmp(&a.published_at));

    // Attach quality scores for evaluation filter
    if queue_filter == Some(QueueFilter::EvaluationsOnly) {
        let mut result = Vec::new();
        for v in &filtered {
            let summary = store
                .get_json::<crate::models::Summary>(&format!("summaries/{}.json", v.id))
                .await?;
            if summary.is_some_and(|s| s.quality_score.is_none()) {
                result.push(v.clone());
            }
        }
        filtered = result;
    }

    Ok(filtered.into_iter().skip(offset).take(limit).collect())
}

pub async fn list_video_ids_by_channel(
    store: &Store,
    channel_id: &str,
) -> Result<Vec<String>, StoreError> {
    let all = load_all_videos(store).await?;
    let mut vids: Vec<_> = all.into_iter().filter(|v| v.channel_id == channel_id).collect();
    vids.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    Ok(vids.into_iter().map(|v| v.id).collect())
}

pub async fn get_oldest_ready_video_published_at(
    store: &Store,
    channel_id: &str,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, StoreError> {
    let all = load_all_videos(store).await?;
    Ok(all
        .iter()
        .filter(|v| {
            v.channel_id == channel_id
                && v.transcript_status == ContentStatus::Ready
                && v.summary_status == ContentStatus::Ready
        })
        .map(|v| v.published_at)
        .min())
}

pub async fn list_videos_for_queue_processing(
    store: &Store,
    limit: usize,
    max_retries: u8,
) -> Result<Vec<Video>, StoreError> {
    let all = load_all_videos(store).await?;
    let mut filtered: Vec<Video> = all
        .into_iter()
        .filter(|v| {
            (matches!(
                v.transcript_status,
                ContentStatus::Pending | ContentStatus::Loading | ContentStatus::Failed
            ) || (v.transcript_status == ContentStatus::Ready
                && matches!(
                    v.summary_status,
                    ContentStatus::Pending | ContentStatus::Loading | ContentStatus::Failed
                )))
                && v.retry_count < max_retries
        })
        .collect();
    filtered.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    filtered.truncate(limit);
    Ok(filtered)
}

pub async fn update_video_transcript_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    let key = video_key(video_id);
    if let Some(mut video) = store.get_json::<Video>(&key).await? {
        video.transcript_status = status;
        store.put_json(&key, &video).await?;
    }
    Ok(())
}

pub async fn update_video_summary_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    let key = video_key(video_id);
    if let Some(mut video) = store.get_json::<Video>(&key).await? {
        video.summary_status = status;
        store.put_json(&key, &video).await?;
    }
    Ok(())
}

pub async fn update_video_acknowledged(
    store: &Store,
    video_id: &str,
    acknowledged: bool,
) -> Result<(), StoreError> {
    let key = video_key(video_id);
    if let Some(mut video) = store.get_json::<Video>(&key).await? {
        video.acknowledged = acknowledged;
        store.put_json(&key, &video).await?;
    }
    Ok(())
}

pub async fn increment_video_retry_count(
    store: &Store,
    video_id: &str,
) -> Result<(), StoreError> {
    let key = video_key(video_id);
    if let Some(mut video) = store.get_json::<Video>(&key).await? {
        video.retry_count = video.retry_count.saturating_add(1);
        store.put_json(&key, &video).await?;
    }
    Ok(())
}

pub async fn reset_video_retry_count(store: &Store, video_id: &str) -> Result<(), StoreError> {
    let key = video_key(video_id);
    if let Some(mut video) = store.get_json::<Video>(&key).await? {
        video.retry_count = 0;
        store.put_json(&key, &video).await?;
    }
    Ok(())
}

async fn build_channel_snapshot_data(
    store: &Store,
    channel: Channel,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
) -> Result<ChannelSnapshotData, StoreError> {
    let derived_earliest_ready_date =
        get_oldest_ready_video_published_at(store, &channel.id).await?;
    let videos =
        list_videos_by_channel(store, &channel.id, limit, offset, is_short, acknowledged, queue_filter)
            .await?;
    Ok(ChannelSnapshotData {
        channel,
        derived_earliest_ready_date,
        videos,
    })
}

pub async fn load_channel_snapshot_data(
    store: &Store,
    channel_id: &str,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
) -> Result<Option<ChannelSnapshotData>, StoreError> {
    let channel = super::channels::get_channel(store, channel_id).await?;
    match channel {
        Some(channel) => Ok(Some(
            build_channel_snapshot_data(store, channel, limit, offset, is_short, acknowledged, queue_filter)
                .await?,
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
    let channels = super::channels::list_channels(store).await?;
    let selected_channel = preferred_channel_id
        .and_then(|id| channels.iter().find(|c| c.id == id))
        .cloned()
        .or_else(|| channels.first().cloned());
    let selected_channel_id = selected_channel.as_ref().map(|c| c.id.clone());
    let snapshot = match selected_channel {
        Some(channel) => Some(
            build_channel_snapshot_data(store, channel, limit, offset, is_short, acknowledged, queue_filter)
                .await?,
        ),
        None => None,
    };
    Ok(WorkspaceBootstrapData {
        channels,
        selected_channel_id,
        snapshot,
    })
}
