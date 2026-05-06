mod scoped_views;

use std::collections::HashSet;

use crate::models::{
    CanonicalVideoRecord, Channel, ContentStatus, OTHERS_CHANNEL_ID, OTHERS_CHANNEL_NAME, Summary,
    Video,
};
use crate::read_cache::SuggestedVideo;

use super::{
    ChannelSnapshotData, ChannelVideoPageData, QueueFilter, Store, StoreError, VideoInsertOutcome,
    WorkspaceBootstrapData,
};
pub use scoped_views::{
    get_user_scoped_video, list_user_scoped_videos_by_channel, load_channel_snapshot_data,
    load_workspace_bootstrap_data,
};

const VIDEO_SUGGESTION_WINDOW_BATCH_SIZE: usize = 200;

fn canonical_video_key(video_id: &str) -> String {
    format!("videos/{video_id}.json")
}

fn canonical_video_from_video(video: &Video) -> CanonicalVideoRecord {
    CanonicalVideoRecord {
        id: video.id.clone(),
        channel_id: video.channel_id.clone(),
        title: video.title.clone(),
        thumbnail_url: video.thumbnail_url.clone(),
        published_at: video.published_at,
        is_short: video.is_short,
        transcript_status: video.transcript_status,
        summary_status: video.summary_status,
        retry_count: video.retry_count,
        quality_score: video.quality_score,
    }
}

fn video_from_canonical(record: CanonicalVideoRecord) -> Video {
    Video {
        id: record.id,
        channel_id: record.channel_id,
        title: record.title,
        thumbnail_url: record.thumbnail_url,
        published_at: record.published_at,
        is_short: record.is_short,
        transcript_status: record.transcript_status,
        summary_status: record.summary_status,
        acknowledged: false,
        retry_count: record.retry_count,
        quality_score: record.quality_score,
    }
}

fn summary_needs_evaluation_filter(summary: &Summary) -> bool {
    super::content::summary_needs_quality_eval(summary)
}

fn upsert_video_delta(record: CanonicalVideoRecord) -> super::LibsqlSnapshotDeltaOperation {
    super::LibsqlSnapshotDeltaOperation::UpsertVideo { record }
}

async fn mirror_video_snapshot(store: &Store, video_id: &str) -> Result<(), StoreError> {
    let Some(video) = super::sql_videos::sql_get_video(store, video_id, false).await? else {
        return Ok(());
    };
    let record = canonical_video_from_video(&video);
    store
        .put_json(&canonical_video_key(video_id), &record)
        .await?;
    store
        .record_libsql_snapshot_delta(vec![upsert_video_delta(record)])
        .await
}

async fn mirror_video_snapshots(store: &Store, video_ids: &[String]) -> Result<(), StoreError> {
    if video_ids.is_empty() {
        return Ok(());
    }

    let videos = super::sql_videos::sql_get_videos(store, video_ids, false).await?;
    let mut operations = Vec::with_capacity(video_ids.len());
    for video_id in video_ids {
        let Some(video) = videos.get(video_id) else {
            continue;
        };
        let record = canonical_video_from_video(video);
        store
            .put_json(&canonical_video_key(video_id), &record)
            .await?;
        operations.push(upsert_video_delta(record));
    }

    if !operations.is_empty() {
        store.record_libsql_snapshot_delta(operations).await?;
    }

    Ok(())
}

pub async fn sql_video_count(store: &Store) -> Result<usize, StoreError> {
    super::sql_videos::sql_count_videos(store).await
}

pub async fn snapshot_video_count(store: &Store) -> Result<usize, StoreError> {
    Ok(store.list_keys("videos/").await?.len())
}

pub async fn bootstrap_sql_videos_from_store(store: &Store) -> Result<usize, StoreError> {
    let records: Vec<CanonicalVideoRecord> = store.load_all("videos/").await?;
    if records.is_empty() {
        return Ok(0);
    }

    let videos = records
        .into_iter()
        .map(video_from_canonical)
        .collect::<Vec<_>>();
    super::sql_videos::sql_bulk_insert_videos(store, videos).await
}

pub async fn export_sql_videos_to_store(store: &Store) -> Result<usize, StoreError> {
    let videos = super::sql_videos::sql_load_all_videos(store).await?;
    let mut operations = Vec::with_capacity(videos.len());
    for video in &videos {
        let record = canonical_video_from_video(video);
        store
            .put_json(&canonical_video_key(&video.id), &record)
            .await?;
        operations.push(upsert_video_delta(record));
    }
    if !operations.is_empty() {
        store.record_libsql_snapshot_delta(operations).await?;
    }
    Ok(videos.len())
}

pub async fn insert_video(store: &Store, video: &Video) -> Result<VideoInsertOutcome, StoreError> {
    let outcome = super::sql_videos::sql_insert_video(store, video).await?;
    mirror_video_snapshot(store, &video.id).await?;
    if outcome == VideoInsertOutcome::Inserted {
        store.read_cache.evict_channel(&video.channel_id).await;
    }
    // Skip cache eviction for Existing — nothing changed.
    Ok(outcome)
}

pub async fn bulk_insert_videos(store: &Store, videos: Vec<Video>) -> Result<usize, StoreError> {
    let video_ids = videos
        .iter()
        .map(|video| video.id.clone())
        .collect::<Vec<_>>();
    let count = super::sql_videos::sql_bulk_insert_videos(store, videos).await?;
    mirror_video_snapshots(store, &video_ids).await?;
    if count > 0 {
        store.read_cache.evict_channel_list().await;
    }
    Ok(count)
}

pub async fn get_video(
    store: &Store,
    id: &str,
    include_summary: bool,
) -> Result<Option<Video>, StoreError> {
    super::sql_videos::sql_get_video(store, id, include_summary).await
}

pub async fn get_videos(
    store: &Store,
    ids: &[impl AsRef<str>],
    include_summary: bool,
) -> Result<std::collections::HashMap<String, Video>, StoreError> {
    super::sql_videos::sql_get_videos(store, ids, include_summary).await
}

pub async fn list_channel_videos_window(
    store: &Store,
    channel_id: &str,
    limit: usize,
    offset: usize,
    descending: bool,
) -> Result<Vec<Video>, StoreError> {
    super::sql_videos::sql_list_channel_videos_window(store, channel_id, limit, offset, descending)
        .await
}

pub async fn load_scoped_video_suggestions(
    store: &Store,
    scope_cache_key: &str,
    allowed_channel_ids: &[String],
    allowed_other_video_ids: &[String],
) -> Result<Vec<SuggestedVideo>, StoreError> {
    if let Some(videos) = store
        .read_cache
        .get_scoped_video_suggestions(scope_cache_key)
        .await
    {
        return Ok(videos);
    }

    let mut by_id = std::collections::HashMap::<String, SuggestedVideo>::new();

    for channel_id in allowed_channel_ids {
        let mut offset = 0usize;
        loop {
            let batch = list_channel_videos_window(
                store,
                channel_id,
                VIDEO_SUGGESTION_WINDOW_BATCH_SIZE,
                offset,
                true,
            )
            .await?;
            if batch.is_empty() {
                break;
            }

            let batch_len = batch.len();
            offset = offset.saturating_add(batch_len);
            for video in batch {
                by_id
                    .entry(video.id.clone())
                    .or_insert_with(|| SuggestedVideo {
                        id: video.id,
                        channel_id: video.channel_id,
                        title: video.title,
                        published_at: video.published_at,
                    });
            }

            if batch_len < VIDEO_SUGGESTION_WINDOW_BATCH_SIZE {
                break;
            }
        }
    }

    if !allowed_other_video_ids.is_empty() {
        let others = get_videos(store, allowed_other_video_ids, false).await?;
        for video in others.into_values() {
            by_id
                .entry(video.id.clone())
                .or_insert_with(|| SuggestedVideo {
                    id: video.id,
                    channel_id: video.channel_id,
                    title: video.title,
                    published_at: video.published_at,
                });
        }
    }

    let videos = by_id.into_values().collect::<Vec<_>>();
    store
        .read_cache
        .set_scoped_video_suggestions(scope_cache_key.to_string(), videos.clone())
        .await;
    Ok(videos)
}

/// Fetch every video row from local libSQL.
/// Filtering and sorting happen in-memory after this call.
pub async fn load_all_videos(store: &Store) -> Result<Vec<Video>, StoreError> {
    // 1. Try cache first (TTL-based)
    if let Some(videos) = store.read_cache.get_videos().await {
        return Ok(videos);
    }

    // 2. Cache miss: fetch from local libSQL
    let videos = super::sql_videos::sql_load_all_videos(store).await?;

    // 3. Populate cache
    store.read_cache.set_videos(videos.clone()).await;

    Ok(videos)
}

fn video_visible_in_list(video: &Video, queue_filter: Option<QueueFilter>) -> bool {
    video.transcript_status == ContentStatus::Ready
        || is_podcast_episode_video(video)
        || matches!(queue_filter, Some(QueueFilter::TranscriptsOnly))
}

fn is_podcast_episode_video(video: &Video) -> bool {
    video.id.starts_with("podcast:episode:") || video.channel_id.starts_with("podcast:rss:")
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
            if summary.is_some_and(|s| summary_needs_evaluation_filter(&s)) {
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
    let subscribed = subscribed_channel_ids(&super::channels::list_channels(store).await?);
    let options = VideoListOptions {
        limit,
        offset: 0,
        is_short,
        acknowledged,
        queue_filter,
        published_at_not_before: None,
    };
    let mut matched = Vec::new();
    let target_len = offset.saturating_add(limit).saturating_add(1);
    let mut scanned = 0usize;

    loop {
        let batch = list_channel_videos_window(store, channel_id, 200, scanned, true).await?;
        if batch.is_empty() {
            break;
        }
        scanned = scanned.saturating_add(batch.len());

        let page =
            apply_channel_video_filters(store, &batch, channel_id, &subscribed, options).await?;
        matched.extend(page.videos);
        if matched.len() >= target_len || batch.len() < 200 {
            break;
        }
    }

    let has_more = matched.len() > offset.saturating_add(limit);
    let videos: Vec<Video> = matched.into_iter().skip(offset).take(limit).collect();
    let next_offset = offset + videos.len();

    Ok(ChannelVideoPageData {
        videos,
        has_more,
        next_offset: has_more.then_some(next_offset),
    })
}

pub async fn list_video_ids_by_channel(
    store: &Store,
    channel_id: &str,
) -> Result<Vec<String>, StoreError> {
    if channel_id == OTHERS_CHANNEL_ID {
        let all = load_all_videos(store).await?;
        let channels = super::channels::list_channels(store).await?;
        let subscribed = subscribed_channel_ids(&channels);
        let mut vids: Vec<_> = all
            .into_iter()
            .filter(|v| video_matches_channel_scope(v, channel_id, &subscribed))
            .collect();
        vids.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        return Ok(vids.into_iter().map(|v| v.id).collect());
    }

    let mut ids = Vec::new();
    let mut scanned = 0usize;
    loop {
        let batch = list_channel_videos_window(store, channel_id, 200, scanned, true).await?;
        if batch.is_empty() {
            break;
        }
        let batch_len = batch.len();
        scanned = scanned.saturating_add(batch.len());
        ids.extend(batch.into_iter().map(|video| video.id));
        if batch_len < 200 {
            break;
        }
    }
    Ok(ids)
}

pub async fn get_oldest_ready_video_published_at(
    store: &Store,
    channel: &Channel,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, StoreError> {
    let floor = channel_sync_floor(channel);
    let mut scanned = 0usize;
    loop {
        let batch = list_channel_videos_window(store, &channel.id, 100, scanned, false).await?;
        if batch.is_empty() {
            return Ok(None);
        }
        let batch_len = batch.len();
        scanned = scanned.saturating_add(batch_len);
        if let Some(published_at) =
            oldest_ready_video_published_at_from_slice(&batch, &channel.id, floor)
        {
            return Ok(Some(published_at));
        }
        if batch_len < 100 {
            return Ok(None);
        }
    }
}

pub async fn list_videos_for_queue_processing(
    store: &Store,
    limit: usize,
    max_retries: u8,
) -> Result<Vec<Video>, StoreError> {
    // Use the cached full-video list instead of 6 separate Firestore queries.
    let all = load_all_videos(store).await?;
    let mut candidates: Vec<Video> = all
        .into_iter()
        .filter(|v| v.retry_count < max_retries)
        .filter(|v| {
            matches!(
                v.transcript_status,
                ContentStatus::Pending | ContentStatus::Loading | ContentStatus::Failed
            ) || (v.transcript_status == ContentStatus::Ready
                && matches!(
                    v.summary_status,
                    ContentStatus::Pending | ContentStatus::Loading | ContentStatus::Failed
                ))
        })
        .collect();
    candidates.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    candidates.truncate(limit);
    Ok(candidates)
}

pub async fn update_video_transcript_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    super::sql_videos::sql_update_video_transcript_status(store, video_id, status).await?;
    mirror_video_snapshot(store, video_id).await?;
    store.read_cache.evict_videos().await;
    Ok(())
}

pub async fn update_video_summary_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    super::sql_videos::sql_update_video_summary_status(store, video_id, status).await?;
    mirror_video_snapshot(store, video_id).await?;
    store.read_cache.evict_videos().await;
    Ok(())
}

pub async fn increment_video_retry_count(store: &Store, video_id: &str) -> Result<(), StoreError> {
    super::sql_videos::sql_increment_video_retry_count(store, video_id).await?;
    mirror_video_snapshot(store, video_id).await?;
    Ok(())
}

pub async fn reset_video_retry_count(store: &Store, video_id: &str) -> Result<(), StoreError> {
    super::sql_videos::sql_reset_video_retry_count(store, video_id).await?;
    mirror_video_snapshot(store, video_id).await?;
    Ok(())
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
    let updated_video_ids = super::sql_videos::sql_heal_queue_videos(store, max_retries).await?;
    mirror_video_snapshots(store, &updated_video_ids).await?;
    Ok(updated_video_ids.len())
}

pub async fn delete_videos(store: &Store, video_ids: &[String]) -> Result<(), StoreError> {
    super::sql_videos::sql_delete_videos(store, video_ids).await?;
    store.read_cache.evict_videos().await;
    Ok(())
}

/// Build a channel snapshot. Loads the full video list once and derives both
/// the oldest-ready date and the filtered/sorted video page from the same slice.
async fn build_channel_snapshot_data(
    store: &Store,
    channel: Channel,
    subscribed_channel_ids: &HashSet<String>,
    mut options: VideoListOptions,
) -> Result<ChannelSnapshotData, StoreError> {
    let sync_floor = channel_sync_floor(&channel);
    options.published_at_not_before = sync_floor;

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
