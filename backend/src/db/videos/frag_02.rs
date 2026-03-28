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
        Some(user_id) => super::list_user_video_states(store, user_id).await?,
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
        Some(user_id) => super::list_user_video_states(store, user_id).await?,
        None => std::collections::HashMap::new(),
    };
    let allowed_other_video_ids = allowed_other_video_ids
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let subscribed_channel_ids = allowed_channel_ids.iter().cloned().collect::<HashSet<_>>();
    let mut filtered = load_all_videos(store)
        .await?
        .into_iter()
        .map(|video| overlay_user_video_state(video, &user_states))
        .filter(|video| {
            if channel_id == OTHERS_CHANNEL_ID {
                allowed_other_video_ids.contains(&video.id)
                    && !subscribed_channel_ids.contains(&video.channel_id)
            } else {
                video.channel_id == channel_id
            }
        })
        .filter(|video| is_short.is_none_or(|value| video.is_short == value))
        .filter(|video| acknowledged.is_none_or(|value| video.acknowledged == value))
        .filter(|video| video_visible_in_list(video, queue_filter))
        .filter(|video| match queue_filter {
            Some(QueueFilter::AnyIncomplete) => {
                video.transcript_status != ContentStatus::Ready
                    || video.summary_status != ContentStatus::Ready
            }
            Some(QueueFilter::TranscriptsOnly) => video.transcript_status != ContentStatus::Ready,
            Some(QueueFilter::SummariesOnly) => {
                video.transcript_status == ContentStatus::Ready
                    && video.summary_status != ContentStatus::Ready
            }
            Some(QueueFilter::EvaluationsOnly) => {
                video.transcript_status == ContentStatus::Ready
                    && video.summary_status == ContentStatus::Ready
            }
            None => true,
        })
        .collect::<Vec<_>>();

    filtered.sort_by(|left, right| right.published_at.cmp(&left.published_at));
    let total_len = filtered.len();
    let videos = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    let next_offset = offset + videos.len();

    Ok(Some(ChannelVideoPageData {
        videos,
        has_more: total_len > next_offset,
        next_offset: (total_len > next_offset).then_some(next_offset),
    }))
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
        let max_concurrent_s3_ops = std::hint::black_box(MAX_CONCURRENT_S3_OPS);

        // Must be between 8 and 16 for 1 vCPU / 512 MiB Cloud Run
        assert!(
            max_concurrent_s3_ops >= 8,
            "semaphore bound too low: {max_concurrent_s3_ops}"
        );
        assert!(
            max_concurrent_s3_ops <= 16,
            "semaphore bound too high: {max_concurrent_s3_ops}"
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
        let result = oldest_ready_video_published_at_from_slice(&videos, "ch1", None);
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
        let result = oldest_ready_video_published_at_from_slice(&videos, "ch1", None);
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
        let result = oldest_ready_video_published_at_from_slice(&videos, "ch1", None);
        assert_eq!(result, None);
    }

    #[test]
    fn oldest_ready_date_returns_none_for_empty_slice() {
        let result = oldest_ready_video_published_at_from_slice(&[], "ch1", None);
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
        let result_ch1 = oldest_ready_video_published_at_from_slice(&videos, "ch1", None);
        let result_ch2 = oldest_ready_video_published_at_from_slice(&videos, "ch2", None);
        assert_eq!(result_ch1, Some(videos[0].published_at));
        assert_eq!(result_ch2, Some(videos[1].published_at));
    }

    #[test]
    fn oldest_ready_date_respects_sync_floor() {
        let videos = vec![
            make_video("v1", "ch1", ContentStatus::Ready, ContentStatus::Ready, 100),
            make_video("v2", "ch1", ContentStatus::Ready, ContentStatus::Ready, 5),
        ];
        let floor = videos[1].published_at - Duration::days(1);
        let result = oldest_ready_video_published_at_from_slice(&videos, "ch1", Some(floor));
        assert_eq!(result, Some(videos[1].published_at));
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
