use std::time::Duration;

use chrono::Utc;

use super::{
    ChannelSnapshotCacheKey, MAX_CACHE_SIZE, ReadCache, VideoListCacheKey,
    WorkspaceBootstrapCacheKey,
};
use crate::db::QueueFilter;
use crate::models::{
    AiStatus, Channel, LibraryBootstrapPayload, SearchStatusPayload, WorkspaceBootstrapPayload,
};

fn sample_channel(id: &str) -> Channel {
    Channel {
        id: id.to_string(),
        handle: Some(format!("@{id}")),
        name: format!("Channel {id}"),
        thumbnail_url: None,
        added_at: Utc::now(),
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    }
}

fn sample_bootstrap() -> WorkspaceBootstrapPayload {
    let channel = sample_channel("abc");
    WorkspaceBootstrapPayload {
        ai_available: true,
        ai_status: AiStatus::Cloud,
        analytics_enabled: true,
        containers: vec![crate::models::youtube_series_container(&channel)],
        sources: vec![crate::models::youtube_content_source(&channel)],
        channels: vec![channel],
        selected_source_id: Some("abc".to_string()),
        selected_channel_id: Some("abc".to_string()),
        selected_item_id: None,
        snapshot: None,
        library: LibraryBootstrapPayload {
            sections: Vec::new(),
            sources: Vec::new(),
            selected_source_id: None,
            selected_source: None,
            selected_items: Vec::new(),
            website_folders: Vec::new(),
        },
        search_status: SearchStatusPayload {
            available: true,
            model: "embeddinggemma".to_string(),
            dimensions: 768,
            pending: 0,
            indexing: 0,
            ready: 1,
            failed: 0,
            total_sources: 1,
            total_chunk_count: 3,
            embedded_chunk_count: 3,
            vector_index_ready: true,
            retrieval_mode: "hybrid_ann".to_string(),
        },
    }
}

fn sample_channel_snapshot(channel_id: &str) -> crate::models::ChannelSnapshotPayload {
    let channel = sample_channel(channel_id);
    crate::models::ChannelSnapshotPayload {
        channel_id: channel_id.to_string(),
        source_id: channel_id.to_string(),
        container: crate::models::youtube_series_container(&channel),
        source: crate::models::youtube_content_source(&channel),
        sync_depth: crate::models::SyncDepthPayload {
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
            derived_earliest_ready_date: None,
        },
        channel_video_count: Some(0),
        has_more: false,
        next_offset: None,
        videos: vec![],
        items: vec![],
        parts: vec![],
    }
}

fn sample_search_status() -> SearchStatusPayload {
    SearchStatusPayload {
        available: true,
        model: "embeddinggemma".to_string(),
        dimensions: 768,
        pending: 0,
        indexing: 0,
        ready: 1,
        failed: 0,
        total_sources: 1,
        total_chunk_count: 3,
        embedded_chunk_count: 3,
        vector_index_ready: true,
        retrieval_mode: "hybrid_ann".to_string(),
    }
}

#[tokio::test]
async fn returns_cached_channels_before_ttl_expiry() {
    let cache = ReadCache::new(Duration::from_secs(60));
    let channels = vec![sample_channel("abc")];
    let scope = "user:test";

    cache
        .set_channels(scope.to_string(), channels.clone())
        .await;

    let cached = cache
        .get_channels(scope)
        .await
        .expect("channels should be cached");
    assert_eq!(cached.len(), channels.len());
    assert_eq!(cached[0].id, channels[0].id);
}

#[tokio::test]
async fn expires_entries_after_ttl() {
    let cache = ReadCache::new(Duration::from_millis(1));
    let key = WorkspaceBootstrapCacheKey {
        scope: "anonymous".to_string(),
        selected_channel_id: Some("abc".to_string()),
        video_list: VideoListCacheKey::new(20, 0, None, None, None),
    };

    cache
        .set_workspace_bootstrap(key.clone(), sample_bootstrap())
        .await;
    tokio::time::sleep(Duration::from_millis(5)).await;

    assert!(cache.get_workspace_bootstrap(&key).await.is_none());
}

#[tokio::test]
async fn search_status_uses_longer_ttl_than_default_entries() {
    let cache = ReadCache::new(Duration::from_millis(1));

    cache.set_search_status(sample_search_status()).await;
    tokio::time::sleep(Duration::from_millis(5)).await;

    assert!(cache.get_search_status().await.is_some());
}

#[tokio::test]
async fn clear_invalidates_cached_values() {
    let cache = ReadCache::new(Duration::from_secs(60));
    let scope = "user:test";
    let key = WorkspaceBootstrapCacheKey {
        scope: scope.to_string(),
        selected_channel_id: Some("abc".to_string()),
        video_list: VideoListCacheKey::new(20, 0, None, None, None),
    };

    cache
        .set_channels(scope.to_string(), vec![sample_channel("abc")])
        .await;
    cache
        .set_workspace_bootstrap(key.clone(), sample_bootstrap())
        .await;
    cache.set_search_status(sample_search_status()).await;

    cache.clear().await;

    assert!(cache.get_channels(scope).await.is_none());
    assert!(cache.get_workspace_bootstrap(&key).await.is_none());
    assert!(cache.get_search_status().await.is_none());
}

#[test]
fn video_list_cache_key_distinguishes_queue_filters() {
    let any_incomplete =
        VideoListCacheKey::new(20, 0, None, None, Some(QueueFilter::AnyIncomplete));
    let transcripts = VideoListCacheKey::new(20, 0, None, None, Some(QueueFilter::TranscriptsOnly));
    let summaries = VideoListCacheKey::new(20, 0, None, None, Some(QueueFilter::SummariesOnly));
    let evaluations = VideoListCacheKey::new(20, 0, None, None, Some(QueueFilter::EvaluationsOnly));

    assert_ne!(any_incomplete, transcripts);
    assert_ne!(transcripts, summaries);
    assert_ne!(summaries, evaluations);
}

#[tokio::test]
async fn evict_channel_removes_only_matching_channel_snapshot_entries() {
    let cache = ReadCache::new(Duration::from_secs(60));
    let scope_a = "user:a";
    let scope_b = "user:b";
    let key_a = ChannelSnapshotCacheKey {
        scope: scope_a.to_string(),
        channel_id: "channel-a".to_string(),
        video_list: VideoListCacheKey::new(20, 0, None, None, None),
    };
    let key_b = ChannelSnapshotCacheKey {
        scope: scope_b.to_string(),
        channel_id: "channel-b".to_string(),
        video_list: VideoListCacheKey::new(20, 0, None, None, None),
    };
    let bootstrap_key_a = WorkspaceBootstrapCacheKey {
        scope: scope_a.to_string(),
        selected_channel_id: Some("channel-a".to_string()),
        video_list: VideoListCacheKey::new(20, 0, None, None, None),
    };
    let bootstrap_key_b = WorkspaceBootstrapCacheKey {
        scope: scope_b.to_string(),
        selected_channel_id: Some("channel-b".to_string()),
        video_list: VideoListCacheKey::new(20, 0, None, None, None),
    };

    // Populate cache for both channels
    cache
        .set_channels(
            scope_a.to_string(),
            vec![sample_channel("channel-a"), sample_channel("channel-b")],
        )
        .await;
    cache
        .set_channels(scope_b.to_string(), vec![sample_channel("channel-b")])
        .await;
    cache
        .set_channel_snapshot(key_a.clone(), sample_channel_snapshot("channel-a"))
        .await;
    cache
        .set_channel_snapshot(key_b.clone(), sample_channel_snapshot("channel-b"))
        .await;
    cache
        .set_workspace_bootstrap(bootstrap_key_a.clone(), sample_bootstrap())
        .await;
    cache
        .set_workspace_bootstrap(bootstrap_key_b.clone(), sample_bootstrap())
        .await;
    cache
        .set_channel_sync_depth(
            scope_a.to_string(),
            "channel-a".to_string(),
            crate::models::SyncDepthPayload {
                earliest_sync_date: None,
                earliest_sync_date_user_set: false,
                derived_earliest_ready_date: None,
            },
        )
        .await;

    // Evict only channel-a
    cache.evict_channel("channel-a").await;

    // channel-a entries are evicted
    assert!(
        cache.get_channel_snapshot(&key_a).await.is_none(),
        "channel-a snapshot should be evicted"
    );
    assert!(
        cache
            .get_workspace_bootstrap(&bootstrap_key_a)
            .await
            .is_none(),
        "channel-a workspace bootstrap should be evicted"
    );
    assert!(
        cache
            .get_channel_sync_depth(scope_a, "channel-a")
            .await
            .is_none(),
        "channel-a sync depth should be evicted"
    );

    // channel-b entries are still present
    assert!(
        cache.get_channel_snapshot(&key_b).await.is_some(),
        "channel-b snapshot should remain"
    );
    assert!(
        cache
            .get_workspace_bootstrap(&bootstrap_key_b)
            .await
            .is_some(),
        "channel-b workspace bootstrap should remain"
    );

    // channels list is untouched
    assert!(
        cache.get_channels(scope_a).await.is_some(),
        "scope-a channels list should remain after evict_channel"
    );
    assert!(
        cache.get_channels(scope_b).await.is_some(),
        "scope-b channels list should remain after evict_channel"
    );
}

#[tokio::test]
async fn evict_channel_does_not_affect_workspace_bootstrap_for_other_channels() {
    let cache = ReadCache::new(Duration::from_secs(60));
    let scope = "user:test";
    // A workspace bootstrap with NO selected channel (e.g., first load with no channel selected).
    // Null resolves to the first channel via fallback, so evict_channel must invalidate it
    // to prevent serving a stale payload after a mutation to that first channel.
    let bootstrap_key_none = WorkspaceBootstrapCacheKey {
        scope: scope.to_string(),
        selected_channel_id: None,
        video_list: VideoListCacheKey::new(20, 0, None, None, None),
    };
    // A workspace bootstrap explicitly for channel-b (a different channel).
    let bootstrap_key_b = WorkspaceBootstrapCacheKey {
        scope: scope.to_string(),
        selected_channel_id: Some("channel-b".to_string()),
        video_list: VideoListCacheKey::new(20, 0, None, None, None),
    };

    cache
        .set_workspace_bootstrap(bootstrap_key_none.clone(), sample_bootstrap())
        .await;
    cache
        .set_workspace_bootstrap(bootstrap_key_b.clone(), sample_bootstrap())
        .await;

    cache.evict_channel("channel-a").await;

    // The null-keyed entry is evicted because it may resolve to channel-a via fallback.
    assert!(
        cache
            .get_workspace_bootstrap(&bootstrap_key_none)
            .await
            .is_none(),
        "null-keyed bootstrap should be evicted (null resolves to first channel via fallback)"
    );
    // The channel-b entry is explicitly for a different channel and must remain intact.
    assert!(
        cache
            .get_workspace_bootstrap(&bootstrap_key_b)
            .await
            .is_some(),
        "channel-b workspace bootstrap should not be evicted when evicting channel-a"
    );
}

#[tokio::test]
async fn evict_channel_list_removes_channels_and_all_workspace_bootstraps() {
    let cache = ReadCache::new(Duration::from_secs(60));
    let scope_a = "user:a";
    let scope_b = "user:b";
    let key_a = WorkspaceBootstrapCacheKey {
        scope: scope_a.to_string(),
        selected_channel_id: Some("channel-a".to_string()),
        video_list: VideoListCacheKey::new(20, 0, None, None, None),
    };
    let key_b = WorkspaceBootstrapCacheKey {
        scope: scope_b.to_string(),
        selected_channel_id: Some("channel-b".to_string()),
        video_list: VideoListCacheKey::new(20, 0, None, None, None),
    };
    let snapshot_key_a = ChannelSnapshotCacheKey {
        scope: scope_a.to_string(),
        channel_id: "channel-a".to_string(),
        video_list: VideoListCacheKey::new(20, 0, None, None, None),
    };

    cache
        .set_channels(scope_a.to_string(), vec![sample_channel("channel-a")])
        .await;
    cache
        .set_channels(scope_b.to_string(), vec![sample_channel("channel-b")])
        .await;
    cache
        .set_workspace_bootstrap(key_a.clone(), sample_bootstrap())
        .await;
    cache
        .set_workspace_bootstrap(key_b.clone(), sample_bootstrap())
        .await;
    cache
        .set_channel_snapshot(snapshot_key_a.clone(), sample_channel_snapshot("channel-a"))
        .await;
    cache.set_search_status(sample_search_status()).await;

    cache.evict_channel_list().await;

    // Channels list and ALL workspace bootstraps are evicted
    assert!(
        cache.get_channels(scope_a).await.is_none(),
        "scope-a channels list should be evicted"
    );
    assert!(
        cache.get_channels(scope_b).await.is_none(),
        "scope-b channels list should be evicted"
    );
    assert!(
        cache.get_workspace_bootstrap(&key_a).await.is_none(),
        "workspace bootstrap for channel-a should be evicted"
    );
    assert!(
        cache.get_workspace_bootstrap(&key_b).await.is_none(),
        "workspace bootstrap for channel-b should be evicted"
    );

    // Channel snapshot and search status remain (they don't depend on the channels list)
    assert!(
        cache.get_channel_snapshot(&snapshot_key_a).await.is_some(),
        "channel snapshot should remain"
    );
    assert!(
        cache.get_search_status().await.is_some(),
        "search status should remain"
    );
}

#[tokio::test]
async fn bounded_cache_does_not_exceed_max_size_when_full() {
    // Use a tiny cache size for testing
    use std::time::Duration;
    let cache = ReadCache::new(Duration::from_secs(60));
    // Override max size is not possible via the public API,
    // so we fill to MAX_CACHE_SIZE using ChannelSyncDepth keys and
    // verify we can still update an existing key (not a new insertion)
    // This test verifies the bounded invariant via the public constants.
    // Fill cache with unique ChannelSyncDepth entries
    let fill_count = crate::read_cache::MAX_CACHE_SIZE + 10;
    for i in 0..fill_count {
        let channel_id = format!("channel-{i}");
        cache
            .set_channel_sync_depth(
                "anonymous".to_string(),
                channel_id,
                crate::models::SyncDepthPayload {
                    earliest_sync_date: None,
                    earliest_sync_date_user_set: false,
                    derived_earliest_ready_date: None,
                },
            )
            .await;
    }
    // The cache should have at most MAX_CACHE_SIZE entries
    let entry_count = cache.len().await;
    assert!(
        entry_count <= MAX_CACHE_SIZE,
        "cache size {entry_count} exceeds MAX_CACHE_SIZE {MAX_CACHE_SIZE}"
    );
}

#[tokio::test]
async fn workspace_bootstrap_cache_keeps_entries_separate_by_video_filter() {
    let cache = ReadCache::new(Duration::from_secs(60));
    let long_videos_key = WorkspaceBootstrapCacheKey {
        scope: "anonymous".to_string(),
        selected_channel_id: Some("abc".to_string()),
        video_list: VideoListCacheKey::new(20, 0, Some(false), None, None),
    };
    let queued_videos_key = WorkspaceBootstrapCacheKey {
        scope: "anonymous".to_string(),
        selected_channel_id: Some("abc".to_string()),
        video_list: VideoListCacheKey::new(
            20,
            0,
            Some(false),
            None,
            Some(QueueFilter::SummariesOnly),
        ),
    };

    let mut long_videos = sample_bootstrap();
    long_videos.selected_channel_id = Some("long-only".to_string());
    let mut queued_videos = sample_bootstrap();
    queued_videos.selected_channel_id = Some("queued-only".to_string());

    cache
        .set_workspace_bootstrap(long_videos_key.clone(), long_videos)
        .await;
    cache
        .set_workspace_bootstrap(queued_videos_key.clone(), queued_videos)
        .await;

    assert_eq!(
        cache
            .get_workspace_bootstrap(&long_videos_key)
            .await
            .and_then(|payload| payload.selected_channel_id),
        Some("long-only".to_string())
    );
    assert_eq!(
        cache
            .get_workspace_bootstrap(&queued_videos_key)
            .await
            .and_then(|payload| payload.selected_channel_id),
        Some("queued-only".to_string())
    );
}
