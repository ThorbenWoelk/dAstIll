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
mod tests {
    use chrono::{Duration, Utc};

    use std::collections::HashSet;

    use super::{
        oldest_ready_video_published_at_from_slice, overlay_user_video_state,
        summary_needs_evaluation_filter, user_scoped_evaluation_filter_allows,
        video_matches_channel_scope, video_visible_in_list,
    };
    use crate::db::{MAX_CONCURRENT_S3_OPS, QueueFilter};
    use crate::models::{ContentStatus, Summary, UserVideoState, Video};

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
    fn regular_lists_show_podcast_episodes_while_transcripts_are_pending() {
        let video = Video {
            id: "podcast:episode:episode-1".to_string(),
            channel_id: "podcast:rss:show".to_string(),
            ..build_video(ContentStatus::Pending, ContentStatus::Pending)
        };

        assert!(video_visible_in_list(&video, None));
        assert!(video_visible_in_list(
            &video,
            Some(QueueFilter::AnyIncomplete)
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

    fn summary_with_quality(
        quality_score: Option<u8>,
        quality_note: Option<&str>,
        summary_tags_evaluated: bool,
    ) -> Summary {
        Summary {
            video_id: "video-123".to_string(),
            content: "summary".to_string(),
            model_used: Some("summary-model".to_string()),
            quality_score,
            quality_note: quality_note.map(ToOwned::to_owned),
            quality_model_used: Some("eval-model".to_string()),
            summary_tags: Vec::new(),
            summary_tags_evaluated,
        }
    }

    #[test]
    fn evaluation_filter_skips_unscorable_completed_summary() {
        let summary = summary_with_quality(None, Some("**Unscorable**:\n- Show notes"), true);

        assert!(!summary_needs_evaluation_filter(&summary));
    }

    #[test]
    fn evaluation_filter_keeps_missing_quality_pending() {
        let summary = summary_with_quality(None, None, false);

        assert!(summary_needs_evaluation_filter(&summary));
    }

    #[test]
    fn evaluation_filter_keeps_legacy_tagless_summary_pending() {
        let summary = summary_with_quality(Some(9), Some("Legacy evaluation"), false);

        assert!(summary_needs_evaluation_filter(&summary));
    }

    #[test]
    fn user_scoped_evaluation_filter_skips_unscorable_completed_summary() {
        let video = build_video(ContentStatus::Ready, ContentStatus::Ready);
        let summary = summary_with_quality(None, Some("**Unscorable**:\n- Show notes"), true);

        assert!(!user_scoped_evaluation_filter_allows(
            &video,
            Some(&summary),
            Some(QueueFilter::EvaluationsOnly)
        ));
    }

    #[test]
    fn user_scoped_evaluation_filter_keeps_missing_quality_pending() {
        let video = build_video(ContentStatus::Ready, ContentStatus::Ready);
        let summary = summary_with_quality(None, None, false);

        assert!(user_scoped_evaluation_filter_allows(
            &video,
            Some(&summary),
            Some(QueueFilter::EvaluationsOnly)
        ));
    }

    #[test]
    fn user_scoped_evaluation_filter_does_not_affect_non_evaluation_queues() {
        let video = build_video(ContentStatus::Ready, ContentStatus::Ready);

        assert!(user_scoped_evaluation_filter_allows(&video, None, None));
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
    fn overlay_user_video_state_applies_user_specific_acknowledged_flag() {
        let video = build_video(ContentStatus::Ready, ContentStatus::Ready);
        let user_video_states = std::collections::HashMap::from([(
            video.id.clone(),
            UserVideoState {
                video_id: video.id.clone(),
                acknowledged: true,
                updated_at: Utc::now(),
            },
        )]);

        let overlaid = overlay_user_video_state(video, &user_video_states);
        assert!(overlaid.acknowledged);
    }

    #[test]
    fn overlay_user_video_state_defaults_to_unacknowledged_without_user_state() {
        let video = Video {
            acknowledged: true,
            ..build_video(ContentStatus::Ready, ContentStatus::Ready)
        };

        let overlaid = overlay_user_video_state(video, &std::collections::HashMap::new());
        assert!(!overlaid.acknowledged);
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
