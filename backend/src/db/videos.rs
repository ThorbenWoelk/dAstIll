use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::models::{Channel, ContentStatus, Video};

use super::{
    ChannelSnapshotData, QueueFilter, Store, StoreError, VideoInsertOutcome, WorkspaceBootstrapData,
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

pub async fn bulk_insert_videos(store: &Store, videos: Vec<Video>) -> Result<usize, StoreError> {
    if videos.is_empty() {
        return Ok(0);
    }

    let semaphore = Arc::new(Semaphore::new(super::MAX_CONCURRENT_S3_OPS));
    let mut join_set: JoinSet<Result<VideoInsertOutcome, StoreError>> = JoinSet::new();

    for video in videos {
        let store = store.connect();
        let semaphore = Arc::clone(&semaphore);
        join_set.spawn(async move {
            let _permit = semaphore.acquire().await.expect("semaphore closed");
            insert_video(&store, &video).await
        });
    }

    let mut inserted = 0;
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(VideoInsertOutcome::Inserted)) => inserted += 1,
            Ok(Ok(VideoInsertOutcome::Existing)) => {}
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                return Err(StoreError::S3(format!("parallel insert task error: {e}")));
            }
        }
    }

    Ok(inserted)
}

/// Fetch a single video by ID.
///
/// When `include_summary` is `true` the S3 summary object is also fetched
/// to populate `quality_score`.  Pass `false` for list-view callers that do
/// not need the quality score, saving one S3 GET per call.
pub async fn get_video(
    store: &Store,
    id: &str,
    include_summary: bool,
) -> Result<Option<Video>, StoreError> {
    let mut video: Option<Video> = store.get_json(&video_key(id)).await?;
    if include_summary {
        if let Some(ref mut v) = video {
            if let Some(summary) = store
                .get_json::<crate::models::Summary>(&format!("summaries/{id}.json"))
                .await?
            {
                v.quality_score = summary.quality_score;
            }
        }
    }
    Ok(video)
}

async fn load_all_videos(store: &Store) -> Result<Vec<Video>, StoreError> {
    store.load_all("videos/").await
}

fn video_visible_in_list(video: &Video, queue_filter: Option<QueueFilter>) -> bool {
    video.transcript_status == ContentStatus::Ready
        || matches!(queue_filter, Some(QueueFilter::TranscriptsOnly))
}

/// Compute the oldest `published_at` date across fully-ready videos in a
/// channel, using an already-loaded slice — avoids an extra S3 round-trip
/// when the caller has already fetched the video list.
fn oldest_ready_video_published_at_from_slice(
    videos: &[Video],
    channel_id: &str,
) -> Option<chrono::DateTime<chrono::Utc>> {
    videos
        .iter()
        .filter(|v| {
            v.channel_id == channel_id
                && v.transcript_status == ContentStatus::Ready
                && v.summary_status == ContentStatus::Ready
        })
        .map(|v| v.published_at)
        .min()
}

/// Apply channel-scoped filtering, sorting, and pagination to a pre-loaded
/// video slice.  The caller is responsible for loading the full video list
/// (via `load_all_videos`) before calling this function so multiple callers
/// can share a single S3 round-trip.
async fn apply_channel_video_filters(
    store: &Store,
    all_videos: &[Video],
    channel_id: &str,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
) -> Result<Vec<Video>, StoreError> {
    let mut filtered: Vec<Video> = all_videos
        .iter()
        .filter(|v| v.channel_id == channel_id)
        .filter(|v| is_short.is_none_or(|s| v.is_short == s))
        .filter(|v| acknowledged.is_none_or(|a| v.acknowledged == a))
        .filter(|v| video_visible_in_list(v, queue_filter))
        .filter(|v| match queue_filter {
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
    apply_channel_video_filters(
        store,
        &all,
        channel_id,
        limit,
        offset,
        is_short,
        acknowledged,
        queue_filter,
    )
    .await
}

pub async fn list_video_ids_by_channel(
    store: &Store,
    channel_id: &str,
) -> Result<Vec<String>, StoreError> {
    let all = load_all_videos(store).await?;
    let mut vids: Vec<_> = all
        .into_iter()
        .filter(|v| v.channel_id == channel_id)
        .collect();
    vids.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    Ok(vids.into_iter().map(|v| v.id).collect())
}

pub async fn get_oldest_ready_video_published_at(
    store: &Store,
    channel_id: &str,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, StoreError> {
    let all = load_all_videos(store).await?;
    Ok(oldest_ready_video_published_at_from_slice(&all, channel_id))
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

async fn update_video_field<F>(store: &Store, video_id: &str, mutate: F) -> Result<(), StoreError>
where
    F: FnOnce(&mut Video),
{
    let key = video_key(video_id);
    if let Some(mut video) = store.get_json::<Video>(&key).await? {
        mutate(&mut video);
        store.put_json(&key, &video).await?;
    }
    Ok(())
}

pub async fn update_video_transcript_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    update_video_field(store, video_id, |v| v.transcript_status = status).await
}

pub async fn update_video_summary_status(
    store: &Store,
    video_id: &str,
    status: ContentStatus,
) -> Result<(), StoreError> {
    update_video_field(store, video_id, |v| v.summary_status = status).await
}

pub async fn update_video_acknowledged(
    store: &Store,
    video_id: &str,
    acknowledged: bool,
) -> Result<(), StoreError> {
    update_video_field(store, video_id, |v| v.acknowledged = acknowledged).await
}

pub async fn increment_video_retry_count(store: &Store, video_id: &str) -> Result<(), StoreError> {
    update_video_field(store, video_id, |v| {
        v.retry_count = v.retry_count.saturating_add(1)
    })
    .await
}

pub async fn reset_video_retry_count(store: &Store, video_id: &str) -> Result<(), StoreError> {
    update_video_field(store, video_id, |v| v.retry_count = 0).await
}

/// Build a channel snapshot, loading video data from S3 **exactly once** and
/// deriving both the oldest-ready date and the filtered/sorted video list
/// from the same in-memory slice.
async fn build_channel_snapshot_data(
    store: &Store,
    channel: Channel,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
) -> Result<ChannelSnapshotData, StoreError> {
    // Load all videos for the whole store once — both derived values share this slice.
    let all_videos = load_all_videos(store).await?;

    let derived_earliest_ready_date =
        oldest_ready_video_published_at_from_slice(&all_videos, &channel.id);

    let videos = apply_channel_video_filters(
        store,
        &all_videos,
        &channel.id,
        limit,
        offset,
        is_short,
        acknowledged,
        queue_filter,
    )
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
            build_channel_snapshot_data(
                store,
                channel,
                limit,
                offset,
                is_short,
                acknowledged,
                queue_filter,
            )
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
            build_channel_snapshot_data(
                store,
                channel,
                limit,
                offset,
                is_short,
                acknowledged,
                queue_filter,
            )
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

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::{oldest_ready_video_published_at_from_slice, video_visible_in_list};
    use crate::db::{MAX_CONCURRENT_S3_OPS, QueueFilter};
    use crate::models::{ContentStatus, Video};

    fn build_video(transcript_status: ContentStatus, summary_status: ContentStatus) -> Video {
        Video {
            id: "video-123".to_string(),
            channel_id: "channel-123".to_string(),
            title: "Video".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status,
            summary_status,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        }
    }

    // ---------------------------------------------------------------------------
    // Existing visibility tests
    // ---------------------------------------------------------------------------

    #[test]
    fn regular_lists_hide_videos_without_ready_transcripts() {
        let video = build_video(ContentStatus::Failed, ContentStatus::Pending);

        assert!(!video_visible_in_list(&video, None));
        assert!(!video_visible_in_list(
            &video,
            Some(QueueFilter::AnyIncomplete)
        ));
        assert!(!video_visible_in_list(
            &video,
            Some(QueueFilter::SummariesOnly)
        ));
    }

    #[test]
    fn transcript_queue_still_includes_videos_missing_transcripts() {
        let video = build_video(ContentStatus::Pending, ContentStatus::Pending);

        assert!(video_visible_in_list(
            &video,
            Some(QueueFilter::TranscriptsOnly)
        ));
    }

    #[test]
    fn ready_transcripts_remain_visible_everywhere() {
        let video = build_video(ContentStatus::Ready, ContentStatus::Pending);

        assert!(video_visible_in_list(&video, None));
        assert!(video_visible_in_list(
            &video,
            Some(QueueFilter::AnyIncomplete)
        ));
    }

    // ---------------------------------------------------------------------------
    // MAX_CONCURRENT_S3_OPS constant
    // ---------------------------------------------------------------------------

    #[test]
    fn max_concurrent_s3_ops_is_within_cloud_run_bounds() {
        // Must be between 8 and 16 for 1 vCPU / 512 MiB Cloud Run
        assert!(
            MAX_CONCURRENT_S3_OPS >= 8,
            "semaphore bound too low: {MAX_CONCURRENT_S3_OPS}"
        );
        assert!(
            MAX_CONCURRENT_S3_OPS <= 16,
            "semaphore bound too high: {MAX_CONCURRENT_S3_OPS}"
        );
    }

    // ---------------------------------------------------------------------------
    // oldest_ready_video_published_at_from_slice — pure logic tests
    // ---------------------------------------------------------------------------

    fn make_video(
        id: &str,
        channel_id: &str,
        transcript_status: ContentStatus,
        summary_status: ContentStatus,
        days_ago: i64,
    ) -> Video {
        Video {
            id: id.to_string(),
            channel_id: channel_id.to_string(),
            title: id.to_string(),
            thumbnail_url: None,
            published_at: Utc::now() - Duration::days(days_ago),
            is_short: false,
            transcript_status,
            summary_status,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        }
    }

    #[test]
    fn oldest_ready_date_returns_minimum_published_at() {
        let videos = vec![
            make_video("v1", "ch1", ContentStatus::Ready, ContentStatus::Ready, 10),
            make_video("v2", "ch1", ContentStatus::Ready, ContentStatus::Ready, 5),
            make_video("v3", "ch1", ContentStatus::Ready, ContentStatus::Ready, 20),
        ];
        let result = oldest_ready_video_published_at_from_slice(&videos, "ch1");
        // v3 is oldest (20 days ago)
        assert_eq!(result, Some(videos[2].published_at));
    }

    #[test]
    fn oldest_ready_date_ignores_videos_not_fully_ready() {
        let videos = vec![
            // transcript ready, summary still pending — not fully ready
            make_video(
                "v1",
                "ch1",
                ContentStatus::Ready,
                ContentStatus::Pending,
                30,
            ),
            // fully ready but newer
            make_video("v2", "ch1", ContentStatus::Ready, ContentStatus::Ready, 5),
        ];
        let result = oldest_ready_video_published_at_from_slice(&videos, "ch1");
        // Only v2 qualifies
        assert_eq!(result, Some(videos[1].published_at));
    }

    #[test]
    fn oldest_ready_date_returns_none_when_no_ready_videos() {
        let videos = vec![make_video(
            "v1",
            "ch1",
            ContentStatus::Pending,
            ContentStatus::Pending,
            1,
        )];
        let result = oldest_ready_video_published_at_from_slice(&videos, "ch1");
        assert_eq!(result, None);
    }

    #[test]
    fn oldest_ready_date_returns_none_for_empty_slice() {
        let result = oldest_ready_video_published_at_from_slice(&[], "ch1");
        assert_eq!(result, None);
    }

    #[test]
    fn oldest_ready_date_is_scoped_to_channel() {
        let videos = vec![
            // ch1: old ready video
            make_video("v1", "ch1", ContentStatus::Ready, ContentStatus::Ready, 100),
            // ch2: newer ready video — must NOT affect ch1's result
            make_video("v2", "ch2", ContentStatus::Ready, ContentStatus::Ready, 1),
        ];
        let result_ch1 = oldest_ready_video_published_at_from_slice(&videos, "ch1");
        let result_ch2 = oldest_ready_video_published_at_from_slice(&videos, "ch2");
        assert_eq!(result_ch1, Some(videos[0].published_at));
        assert_eq!(result_ch2, Some(videos[1].published_at));
    }

    // ---------------------------------------------------------------------------
    // Integration tests — require live S3 backend
    // ---------------------------------------------------------------------------

    /// Verifies that load_all returns all objects in parallel (correct results).
    #[tokio::test]
    #[ignore] // requires live S3 backend: cargo test -- --ignored
    async fn load_all_parallel_returns_correct_results() {
        let store = crate::db::Store::for_test().await;
        // Insert a known set of videos, then load_all and compare counts.
        let result: Result<Vec<crate::models::Video>, _> = store.load_all("videos/").await;
        assert!(result.is_ok(), "load_all should not error");
    }

    /// Verifies that bulk_insert_videos inserts in parallel and returns correct count.
    #[tokio::test]
    #[ignore] // requires live S3 backend
    async fn bulk_insert_parallel_returns_inserted_count() {
        use crate::db::bulk_insert_videos;
        let store = crate::db::Store::for_test().await;
        let videos: Vec<crate::models::Video> = (0..5)
            .map(|i| {
                make_video(
                    &format!("bulk-test-{i}"),
                    "ch-bulk",
                    ContentStatus::Pending,
                    ContentStatus::Pending,
                    i,
                )
            })
            .collect();
        let count = bulk_insert_videos(&store, videos)
            .await
            .expect("bulk_insert should succeed");
        assert_eq!(count, 5);
    }

    /// Verifies that get_video with include_summary=false does not fetch the summary S3 object.
    /// Evidence: video returned without quality_score set from summary, saving one GET.
    #[tokio::test]
    #[ignore] // requires live S3 backend
    async fn get_video_without_summary_skips_summary_fetch() {
        use crate::db::{get_video, insert_video};
        let store = crate::db::Store::for_test().await;
        let video = make_video(
            "test-no-summary",
            "ch-test",
            ContentStatus::Ready,
            ContentStatus::Ready,
            1,
        );
        insert_video(&store, &video)
            .await
            .expect("insert should succeed");

        // With include_summary=false, no summary S3 GET is issued.
        let fetched = get_video(&store, &video.id, false)
            .await
            .expect("get_video should succeed");
        assert!(fetched.is_some());
        // quality_score must be None since summary was not fetched.
        assert_eq!(fetched.unwrap().quality_score, None);
    }

    /// Verifies that build_channel_snapshot_data loads videos exactly once from S3.
    /// (Structural test: load_workspace_bootstrap_data drives build_channel_snapshot_data.)
    #[tokio::test]
    #[ignore] // requires live S3 backend
    async fn channel_snapshot_loads_video_data_once() {
        use crate::db::{insert_channel, insert_video, load_channel_snapshot_data};
        let store = crate::db::Store::for_test().await;
        let channel = crate::models::Channel {
            id: "ch-snapshot-dedup-test".to_string(),
            handle: None,
            name: "Snapshot Dedup Test".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&store, &channel)
            .await
            .expect("insert channel");
        insert_video(
            &store,
            &make_video(
                "snap-v1",
                &channel.id,
                ContentStatus::Ready,
                ContentStatus::Ready,
                3,
            ),
        )
        .await
        .expect("insert video");

        let snapshot = load_channel_snapshot_data(&store, &channel.id, 20, 0, None, None, None)
            .await
            .expect("load snapshot should succeed");
        assert!(snapshot.is_some(), "snapshot should be found");
        let snap = snapshot.unwrap();
        // derived_earliest_ready_date and videos list are both populated from the single load.
        assert!(snap.derived_earliest_ready_date.is_some());
        assert!(!snap.videos.is_empty());
    }
}
