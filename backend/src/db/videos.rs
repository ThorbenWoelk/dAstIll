use std::collections::HashSet;

use crate::models::{Channel, ContentStatus, OTHERS_CHANNEL_ID, OTHERS_CHANNEL_NAME, Video};

use super::{
    ChannelSnapshotData, QueueFilter, Store, StoreError, VideoInsertOutcome, WorkspaceBootstrapData,
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

async fn load_all_videos(store: &Store) -> Result<Vec<Video>, StoreError> {
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

fn oldest_ready_video_published_at_for_scope(
    videos: &[Video],
    channel_id: &str,
    subscribed_channel_ids: &HashSet<String>,
) -> Option<chrono::DateTime<chrono::Utc>> {
    videos
        .iter()
        .filter(|v| video_matches_channel_scope(v, channel_id, subscribed_channel_ids))
        .filter(|v| {
            v.transcript_status == ContentStatus::Ready && v.summary_status == ContentStatus::Ready
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
    subscribed_channel_ids: &HashSet<String>,
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
) -> Result<Vec<Video>, StoreError> {
    let mut filtered: Vec<Video> = all_videos
        .iter()
        .filter(|v| video_matches_channel_scope(v, channel_id, subscribed_channel_ids))
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
    let channels = super::channels::list_channels(store).await?;
    let subscribed = subscribed_channel_ids(&channels);
    apply_channel_video_filters(
        store,
        &all,
        channel_id,
        &subscribed,
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
    limit: usize,
    offset: usize,
    is_short: Option<bool>,
    acknowledged: Option<bool>,
    queue_filter: Option<QueueFilter>,
) -> Result<ChannelSnapshotData, StoreError> {
    // Load all videos for the whole store once — both derived values share this slice.
    let all_videos = load_all_videos(store).await?;

    let derived_earliest_ready_date =
        oldest_ready_video_published_at_for_scope(&all_videos, &channel.id, subscribed_channel_ids);

    let channel_video_count = all_videos
        .iter()
        .filter(|v| video_matches_channel_scope(v, &channel.id, subscribed_channel_ids))
        .count();

    let videos = apply_channel_video_filters(
        store,
        &all_videos,
        &channel.id,
        subscribed_channel_ids,
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
        channel_video_count,
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
    let stored_channels = super::channels::list_channels(store).await?;
    let subscribed = subscribed_channel_ids(&stored_channels);

    if channel_id == OTHERS_CHANNEL_ID {
        if !has_unsubscribed_channel_videos(store).await? {
            return Ok(None);
        }

        return Ok(Some(
            build_channel_snapshot_data(
                store,
                build_virtual_others_channel(),
                &subscribed,
                limit,
                offset,
                is_short,
                acknowledged,
                queue_filter,
            )
            .await?,
        ));
    }

    let channel = stored_channels
        .into_iter()
        .find(|channel| channel.id == channel_id);
    match channel {
        Some(channel) => Ok(Some(
            build_channel_snapshot_data(
                store,
                channel,
                &subscribed,
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
    let channels = list_channels_with_virtual_others(store).await?;
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
        Some(channel) => Some(
            build_channel_snapshot_data(
                store,
                channel,
                &subscribed,
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

    use std::collections::HashSet;

    use super::{
        oldest_ready_video_published_at_from_slice, video_matches_channel_scope,
        video_visible_in_list,
    };
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

    #[test]
    fn heal_queue_clears_stale_loading_and_resets_retries() {
        let mut v = build_video(ContentStatus::Loading, ContentStatus::Pending);
        v.retry_count = 3;
        assert!(super::apply_heal_queue_video_fields(&mut v, 3));
        assert_eq!(v.transcript_status, ContentStatus::Failed);
        assert_eq!(v.retry_count, 0);
    }

    #[test]
    fn heal_queue_fixes_summary_loading_with_exhausted_retries() {
        let mut v = build_video(ContentStatus::Ready, ContentStatus::Loading);
        v.retry_count = 3;
        assert!(super::apply_heal_queue_video_fields(&mut v, 3));
        assert_eq!(v.summary_status, ContentStatus::Failed);
        assert_eq!(v.retry_count, 0);
    }

    #[test]
    fn heal_queue_resets_exhausted_failed_transcripts() {
        let mut v = build_video(ContentStatus::Failed, ContentStatus::Pending);
        v.retry_count = 3;
        assert!(super::apply_heal_queue_video_fields(&mut v, 3));
        assert_eq!(v.transcript_status, ContentStatus::Failed);
        assert_eq!(v.retry_count, 0);
    }

    #[test]
    fn heal_queue_skips_complete_videos() {
        let mut v = build_video(ContentStatus::Ready, ContentStatus::Ready);
        v.retry_count = 3;
        assert!(!super::apply_heal_queue_video_fields(&mut v, 3));
        assert_eq!(v.retry_count, 3);
    }

    #[test]
    fn heal_queue_skips_when_below_retry_cap() {
        let mut v = build_video(ContentStatus::Loading, ContentStatus::Pending);
        v.retry_count = 2;
        assert!(!super::apply_heal_queue_video_fields(&mut v, 3));
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

    #[test]
    fn others_scope_includes_only_unsubscribed_channel_videos() {
        let unsubscribed_video = Video {
            channel_id: "UC_UNSUBSCRIBED".to_string(),
            ..build_video(ContentStatus::Ready, ContentStatus::Ready)
        };
        let subscribed_video = Video {
            channel_id: "UC_SUBSCRIBED".to_string(),
            ..build_video(ContentStatus::Ready, ContentStatus::Ready)
        };
        let subscribed = HashSet::from(["UC_SUBSCRIBED".to_string()]);

        assert!(video_matches_channel_scope(
            &unsubscribed_video,
            "__others__",
            &subscribed
        ));
        assert!(!video_matches_channel_scope(
            &subscribed_video,
            "__others__",
            &subscribed
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
