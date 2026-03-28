use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use crate::models::{Channel, ContentStatus, OTHERS_CHANNEL_ID, OTHERS_CHANNEL_NAME, Video};

use super::{
    ChannelSnapshotData, ChannelVideoPageData, QueueFilter, Store, StoreError, VideoInsertOutcome,
    WorkspaceBootstrapData,
};

pub async fn insert_video(store: &Store, video: &Video) -> Result<VideoInsertOutcome, StoreError> {
    super::firestore_videos::fs_insert_video(store, video).await
}

pub async fn bulk_insert_videos(store: &Store, videos: Vec<Video>) -> Result<usize, StoreError> {
    super::firestore_videos::fs_bulk_insert_videos(store, videos).await
}

pub async fn get_video(
    store: &Store,
    id: &str,
    include_summary: bool,
) -> Result<Option<Video>, StoreError> {
    super::firestore_videos::fs_get_video(store, id, include_summary).await
}

pub(crate) async fn load_all_videos(store: &Store) -> Result<Vec<Video>, StoreError> {
    let videos: Vec<Video> = store
        .firestore
        .fluent()
        .select()
        .from(super::firestore_videos::COLLECTION)
        .obj()
        .query()
        .await
        .map_err(|e| StoreError::Other(format!("Firestore error: {e}")))?;
    Ok(videos)
}

fn video_visible_in_list(video: &Video, queue_filter: Option<QueueFilter>) -> bool {
    video.transcript_status == ContentStatus::Ready
        || matches!(queue_filter, Some(QueueFilter::TranscriptsOnly))
}

fn video_matches_channel_scope(
    video: &Video,
    channel_id: &str,
    subscribed_channel_ids: &HashSet<String>,
) -> bool {
    if channel_id == OTHERS_CHANNEL_ID {
        !subscribed_channel_ids.contains(&video.channel_id)
    } else {
        video.channel_id == channel_id
    }
}

fn build_virtual_others_channel() -> Channel {
    Channel {
        id: OTHERS_CHANNEL_ID.to_string(),
        handle: None,
        name: OTHERS_CHANNEL_NAME.to_string(),
        thumbnail_url: None,
        added_at: chrono::Utc::now(),
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    }
}

fn subscribed_channel_ids(channels: &[Channel]) -> HashSet<String> {
    channels.iter().map(|channel| channel.id.clone()).collect()
}

fn warn_missing_index_once(query_kind: &str, channel_id: &str, error: &StoreError) {
    static SEEN: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let key = format!("{query_kind}:{channel_id}");
    let seen = SEEN.get_or_init(|| Mutex::new(HashSet::new()));
    let mut guard = seen.lock().expect("missing index warning mutex poisoned");
    if !guard.insert(key) {
        return;
    }
    tracing::warn!(
        channel_id = %channel_id,
        error = %error,
        "Firestore {query_kind} missing index, falling back to full scan"
    );
}

pub async fn has_unsubscribed_channel_videos(store: &Store) -> Result<bool, StoreError> {
    let channels = super::channels::list_channels(store).await?;
    let subscribed = subscribed_channel_ids(&channels);
    let all_videos = load_all_videos(store).await?;
    Ok(all_videos
        .iter()
        .any(|video| video_matches_channel_scope(video, OTHERS_CHANNEL_ID, &subscribed)))
}

pub async fn list_channels_with_virtual_others(store: &Store) -> Result<Vec<Channel>, StoreError> {
    let mut channels = super::channels::list_channels(store).await?;
    if has_unsubscribed_channel_videos(store).await? {
        channels.push(build_virtual_others_channel());
    }
    Ok(channels)
}

/// Compute the oldest `published_at` date across fully-ready videos in a
/// channel, using an already-loaded slice — avoids an extra S3 round-trip
/// when the caller has already fetched the video list.
fn channel_sync_floor(channel: &Channel) -> Option<chrono::DateTime<chrono::Utc>> {
    if channel.earliest_sync_date_user_set {
        channel.earliest_sync_date
    } else {
        None
    }
}

fn oldest_ready_video_published_at_from_slice(
    videos: &[Video],
    channel_id: &str,
    published_at_not_before: Option<chrono::DateTime<chrono::Utc>>,
) -> Option<chrono::DateTime<chrono::Utc>> {
    videos
        .iter()
        .filter(|v| {
            v.channel_id == channel_id
                && v.transcript_status == ContentStatus::Ready
                && v.summary_status == ContentStatus::Ready
        })
        .filter(|v| published_at_not_before.is_none_or(|floor| v.published_at >= floor))
        .map(|v| v.published_at)
        .min()
}

fn oldest_ready_video_published_at_for_scope(
    videos: &[Video],
    channel_id: &str,
    subscribed_channel_ids: &HashSet<String>,
    published_at_not_before: Option<chrono::DateTime<chrono::Utc>>,
) -> Option<chrono::DateTime<chrono::Utc>> {
    videos
        .iter()
        .filter(|v| video_matches_channel_scope(v, channel_id, subscribed_channel_ids))
        .filter(|v| published_at_not_before.is_none_or(|floor| v.published_at >= floor))
        .filter(|v| {
            v.transcript_status == ContentStatus::Ready && v.summary_status == ContentStatus::Ready
        })
        .map(|v| v.published_at)
        .min()
}

#[derive(Clone, Copy)]
struct VideoListOptions {
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
    /// When the user set a sync floor, hide videos published before it (matches backfill `until`).
    published_at_not_before: Option<chrono::DateTime<chrono::Utc>>,
}

/// Apply channel-scoped filtering, sorting, and pagination to a pre-loaded
/// video slice.  The caller is responsible for loading the full video list
/// (via `load_all_videos`) before calling this function so multiple callers
/// can share a single S3 round-trip.
async fn apply_channel_video_filters(
    store: &Store,
    all_videos: &[Video],
    channel_id: &str,
    subscribed_channel_ids: &HashSet<String>,
    options: VideoListOptions,
) -> Result<ChannelVideoPageData, StoreError> {
    let mut filtered: Vec<Video> = all_videos
        .iter()
        .filter(|v| video_matches_channel_scope(v, channel_id, subscribed_channel_ids))
        .filter(|v| {
            options
                .published_at_not_before
                .is_none_or(|floor| v.published_at >= floor)
        })
        .filter(|v| options.is_short.is_none_or(|s| v.is_short == s))
        .filter(|v| options.acknowledged.is_none_or(|a| v.acknowledged == a))
        .filter(|v| video_visible_in_list(v, options.queue_filter))
        .filter(|v| match options.queue_filter {
            Some(QueueFilter::AnyIncomplete) => {
                v.transcript_status != ContentStatus::Ready
                    || v.summary_status != ContentStatus::Ready
            }
            Some(QueueFilter::TranscriptsOnly) => v.transcript_status != ContentStatus::Ready,
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
        .cloned()
        .collect();

    filtered.sort_by(|a, b| b.published_at.cmp(&a.published_at));

    // Attach quality scores for evaluation filter
    if options.queue_filter == Some(QueueFilter::EvaluationsOnly) {
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

    let total_len = filtered.len();
    let page_videos: Vec<Video> = filtered
        .into_iter()
        .skip(options.offset)
        .take(options.limit)
        .collect();
    let next_offset = options.offset + page_videos.len();
    let has_more = total_len > next_offset;

    Ok(ChannelVideoPageData {
        videos: page_videos,
        has_more,
        next_offset: has_more.then_some(next_offset),
    })
}

pub async fn list_videos_by_channel(
    store: &Store,
    channel_id: &str,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
) -> Result<ChannelVideoPageData, StoreError> {
    if channel_id != OTHERS_CHANNEL_ID && queue_filter.is_none() {
        let channels = super::channels::list_channels(store).await?;
        let published_at_not_before = channels
            .iter()
            .find(|c| c.id == channel_id)
            .and_then(channel_sync_floor);
        let fetched = match super::firestore_videos::fs_list_videos_by_channel(
            store,
            channel_id,
            limit + 1,
            offset,
            is_short,
            acknowledged,
            published_at_not_before,
        )
        .await
        {
            Ok(videos) => videos,
            Err(error) if super::firestore_videos::is_missing_index_error(&error) => {
                warn_missing_index_once("channel video query", channel_id, &error);
                let all = load_all_videos(store).await?;
                let subscribed = subscribed_channel_ids(&channels);
                let options = VideoListOptions {
                    limit,
                    offset,
                    is_short,
                    acknowledged,
                    queue_filter: None,
                    published_at_not_before,
                };
                return apply_channel_video_filters(store, &all, channel_id, &subscribed, options)
                    .await;
            }
            Err(error) => return Err(error),
        };
        let has_more = fetched.len() > limit;
        let videos = fetched.into_iter().take(limit).collect::<Vec<_>>();
        let next_offset = offset + videos.len();
        return Ok(ChannelVideoPageData {
            videos,
            has_more,
            next_offset: has_more.then_some(next_offset),
        });
    }

    let all = load_all_videos(store).await?;
    let channels = super::channels::list_channels(store).await?;
    let subscribed = subscribed_channel_ids(&channels);
    let published_at_not_before = channels
        .iter()
        .find(|c| c.id == channel_id)
        .and_then(channel_sync_floor);
    let options = VideoListOptions {
        limit,
        offset,
        is_short,
        acknowledged,
        queue_filter,
        published_at_not_before,
    };
    apply_channel_video_filters(store, &all, channel_id, &subscribed, options).await
}

pub async fn list_video_ids_by_channel(
    store: &Store,
    channel_id: &str,
) -> Result<Vec<String>, StoreError> {
    let all = load_all_videos(store).await?;
    let channels = super::channels::list_channels(store).await?;
    let subscribed = subscribed_channel_ids(&channels);
    let mut vids: Vec<_> = all
        .into_iter()
        .filter(|v| video_matches_channel_scope(v, channel_id, &subscribed))
        .collect();
    vids.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    Ok(vids.into_iter().map(|v| v.id).collect())
}

pub async fn get_oldest_ready_video_published_at(
    store: &Store,
    channel: &Channel,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, StoreError> {
    let floor = channel_sync_floor(channel);
    if channel.id != OTHERS_CHANNEL_ID {
        match super::firestore_videos::fs_get_oldest_fully_ready_video_published_at_by_channel(
            store,
            &channel.id,
            floor,
        )
        .await
        {
            Ok(value) => return Ok(value),
            Err(error) if super::firestore_videos::is_missing_index_error(&error) => {
                warn_missing_index_once("oldest-ready query", &channel.id, &error);
            }
            Err(error) => return Err(error),
        }
    }

    let all = load_all_videos(store).await?;
    Ok(oldest_ready_video_published_at_from_slice(
        &all,
        &channel.id,
        floor,
    ))
}

pub async fn list_videos_for_queue_processing(
    store: &Store,
    limit: usize,
    max_retries: u8,
) -> Result<Vec<Video>, StoreError> {
    super::firestore_videos::fs_list_videos_for_queue_processing(store, limit, max_retries).await
}

pub async fn update_video_transcript_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    super::firestore_videos::fs_update_video_transcript_status(store, video_id, status).await
}

pub async fn update_video_summary_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    super::firestore_videos::fs_update_video_summary_status(store, video_id, status).await
}

pub async fn update_video_acknowledged(
    store: &Store,
    video_id: &str,
    acknowledged: bool,
) -> Result<(), StoreError> {
    super::firestore_videos::fs_update_video_acknowledged(store, video_id, acknowledged).await
}

pub async fn increment_video_retry_count(store: &Store, video_id: &str) -> Result<(), StoreError> {
    super::firestore_videos::fs_increment_video_retry_count(store, video_id).await
}

pub async fn reset_video_retry_count(store: &Store, video_id: &str) -> Result<(), StoreError> {
    super::firestore_videos::fs_reset_video_retry_count(store, video_id).await
}

/// Repair stale `loading` rows and re-queue videos that hit `max_retries` (excluded from
/// [`list_videos_for_queue_processing`]). Used once at worker startup after fixing async
/// status races so existing S3 objects recover without manual edits.
pub(crate) fn apply_heal_queue_video_fields(video: &mut Video, max_retries: u8) -> bool {
    if video.transcript_status == ContentStatus::Ready
        && video.summary_status == ContentStatus::Ready
    {
        return false;
    }
    if video.retry_count < max_retries {
        return false;
    }
    if video.transcript_status == ContentStatus::Loading {
        video.transcript_status = ContentStatus::Failed;
    }
    if video.transcript_status == ContentStatus::Ready
        && video.summary_status == ContentStatus::Loading
    {
        video.summary_status = ContentStatus::Failed;
    }
    video.retry_count = 0;
    true
}

pub async fn heal_queue_videos(store: &Store, max_retries: u8) -> Result<usize, StoreError> {
    super::firestore_videos::fs_heal_queue_videos(store, max_retries).await
}

/// Build a channel snapshot, loading video data from S3 **exactly once** and
/// deriving both the oldest-ready date and the filtered/sorted video list
/// from the same in-memory slice.
async fn build_channel_snapshot_data(
    store: &Store,
    channel: Channel,
    subscribed_channel_ids: &HashSet<String>,
    mut options: VideoListOptions,
) -> Result<ChannelSnapshotData, StoreError> {
    let sync_floor = channel_sync_floor(&channel);
    options.published_at_not_before = sync_floor;

    if channel.id != OTHERS_CHANNEL_ID && options.queue_filter.is_none() {
        let fetched = match super::firestore_videos::fs_list_videos_by_channel(
            store,
            &channel.id,
            options.limit + 1,
            options.offset,
            options.is_short,
            options.acknowledged,
            sync_floor,
        )
        .await
        {
            Ok(videos) => videos,
            Err(error) if super::firestore_videos::is_missing_index_error(&error) => {
                warn_missing_index_once("channel snapshot query", &channel.id, &error);
                let all_videos = load_all_videos(store).await?;
                let derived_earliest_ready_date = oldest_ready_video_published_at_for_scope(
                    &all_videos,
                    &channel.id,
                    subscribed_channel_ids,
                    sync_floor,
                );
                let channel_video_count = all_videos
                    .iter()
                    .filter(|v| video_matches_channel_scope(v, &channel.id, subscribed_channel_ids))
                    .filter(|v| sync_floor.is_none_or(|floor| v.published_at >= floor))
                    .count();
                let page = apply_channel_video_filters(
                    store,
                    &all_videos,
                    &channel.id,
                    subscribed_channel_ids,
                    options,
                )
                .await?;
                return Ok(ChannelSnapshotData {
                    channel,
                    derived_earliest_ready_date,
                    channel_video_count: Some(channel_video_count),
                    has_more: page.has_more,
                    next_offset: page.next_offset,
                    videos: page.videos,
                });
            }
            Err(error) => return Err(error),
        };
        let has_more = fetched.len() > options.limit;
        let videos = fetched.into_iter().take(options.limit).collect::<Vec<_>>();
        let next_offset = options.offset + videos.len();
        let derived_earliest_ready_date =
            match super::firestore_videos::fs_get_oldest_fully_ready_video_published_at_by_channel(
                store,
                &channel.id,
                sync_floor,
            )
            .await
            {
                Ok(value) => value,
                Err(error) if super::firestore_videos::is_missing_index_error(&error) => {
                    warn_missing_index_once("oldest-ready snapshot query", &channel.id, &error);
                    let all_videos = load_all_videos(store).await?;
                    oldest_ready_video_published_at_for_scope(
                        &all_videos,
                        &channel.id,
                        subscribed_channel_ids,
                        sync_floor,
                    )
                }
                Err(error) => return Err(error),
            };

        return Ok(ChannelSnapshotData {
            channel,
            derived_earliest_ready_date,
            channel_video_count: None,
            has_more,
            next_offset: has_more.then_some(next_offset),
            videos,
        });
    }

    // Load all videos for the whole store once — both derived values share this slice.
    let all_videos = load_all_videos(store).await?;

    let derived_earliest_ready_date = oldest_ready_video_published_at_for_scope(
        &all_videos,
        &channel.id,
        subscribed_channel_ids,
        sync_floor,
    );

    let channel_video_count = all_videos
        .iter()
        .filter(|v| video_matches_channel_scope(v, &channel.id, subscribed_channel_ids))
        .filter(|v| sync_floor.is_none_or(|floor| v.published_at >= floor))
        .count();

    let page = apply_channel_video_filters(
        store,
        &all_videos,
        &channel.id,
        subscribed_channel_ids,
        options,
    )
    .await?;

    Ok(ChannelSnapshotData {
        channel,
        derived_earliest_ready_date,
        channel_video_count: Some(channel_video_count),
        has_more: page.has_more,
        next_offset: page.next_offset,
        videos: page.videos,
    })
}


include!("frag_02.rs");
