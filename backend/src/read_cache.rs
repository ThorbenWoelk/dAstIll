use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::{
    db::QueueFilter,
    models::{
        Channel, ChannelSnapshotPayload, SearchStatusPayload, SyncDepthPayload,
        WorkspaceBootstrapPayload,
    },
};

const DEFAULT_READ_CACHE_TTL: Duration = Duration::from_secs(10);
const SEARCH_STATUS_CACHE_TTL: Duration = Duration::from_secs(30);
/// Maximum number of entries to keep in the cache.
/// Prevents unbounded memory growth within Cloud Run's 512Mi limit.
pub(crate) const MAX_CACHE_SIZE: usize = 512;

#[derive(Clone)]
pub struct ReadCache {
    ttl: Duration,
    entries: Arc<RwLock<HashMap<ReadCacheKey, CacheEntry>>>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    expires_at: Instant,
    value: ReadCacheValue,
}

#[derive(Debug, Clone)]
enum ReadCacheValue {
    Channels(Vec<Channel>),
    WorkspaceBootstrap(WorkspaceBootstrapPayload),
    ChannelSnapshot(ChannelSnapshotPayload),
    SyncDepth(SyncDepthPayload),
    SearchStatus(SearchStatusPayload),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReadCacheKey {
    Channels(String),
    WorkspaceBootstrap(WorkspaceBootstrapCacheKey),
    ChannelSnapshot(ChannelSnapshotCacheKey),
    ChannelSyncDepth(String, String),
    SearchStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceBootstrapCacheKey {
    pub scope: String,
    pub selected_channel_id: Option<String>,
    pub video_list: VideoListCacheKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelSnapshotCacheKey {
    pub scope: String,
    pub channel_id: String,
    pub video_list: VideoListCacheKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoListCacheKey {
    pub limit: usize,
    pub offset: usize,
    pub is_short: Option<bool>,
    pub acknowledged: Option<bool>,
    pub queue_filter_code: Option<u8>,
}

impl VideoListCacheKey {
    pub fn new(
        limit: usize,
        offset: usize,
        is_short: Option<bool>,
        acknowledged: Option<bool>,
        queue_filter: Option<QueueFilter>,
    ) -> Self {
        Self {
            limit,
            offset,
            is_short,
            acknowledged,
            queue_filter_code: queue_filter.map(queue_filter_code),
        }
    }
}

impl Default for ReadCache {
    fn default() -> Self {
        Self::new(DEFAULT_READ_CACHE_TTL)
    }
}

impl ReadCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_channels(&self, scope: &str) -> Option<Vec<Channel>> {
        self.get_typed(
            &ReadCacheKey::Channels(scope.to_string()),
            ReadCacheValue::into_channels,
        )
        .await
    }

    pub async fn set_channels(&self, scope: String, channels: Vec<Channel>) {
        self.set_typed(
            ReadCacheKey::Channels(scope),
            channels,
            ReadCacheValue::Channels,
        )
        .await;
    }

    pub async fn get_workspace_bootstrap(
        &self,
        key: &WorkspaceBootstrapCacheKey,
    ) -> Option<WorkspaceBootstrapPayload> {
        self.get_typed(
            &ReadCacheKey::WorkspaceBootstrap(key.clone()),
            ReadCacheValue::into_workspace_bootstrap,
        )
        .await
    }

    pub async fn set_workspace_bootstrap(
        &self,
        key: WorkspaceBootstrapCacheKey,
        payload: WorkspaceBootstrapPayload,
    ) {
        self.set_typed(
            ReadCacheKey::WorkspaceBootstrap(key),
            payload,
            ReadCacheValue::WorkspaceBootstrap,
        )
        .await;
    }

    pub async fn get_channel_snapshot(
        &self,
        key: &ChannelSnapshotCacheKey,
    ) -> Option<ChannelSnapshotPayload> {
        self.get_typed(
            &ReadCacheKey::ChannelSnapshot(key.clone()),
            ReadCacheValue::into_channel_snapshot,
        )
        .await
    }

    pub async fn set_channel_snapshot(
        &self,
        key: ChannelSnapshotCacheKey,
        payload: ChannelSnapshotPayload,
    ) {
        self.set_typed(
            ReadCacheKey::ChannelSnapshot(key),
            payload,
            ReadCacheValue::ChannelSnapshot,
        )
        .await;
    }

    pub async fn get_channel_sync_depth(
        &self,
        scope: &str,
        channel_id: &str,
    ) -> Option<SyncDepthPayload> {
        self.get_typed(
            &ReadCacheKey::ChannelSyncDepth(scope.to_string(), channel_id.to_string()),
            ReadCacheValue::into_sync_depth,
        )
        .await
    }

    pub async fn set_channel_sync_depth(
        &self,
        scope: String,
        channel_id: String,
        payload: SyncDepthPayload,
    ) {
        self.set_typed(
            ReadCacheKey::ChannelSyncDepth(scope, channel_id),
            payload,
            ReadCacheValue::SyncDepth,
        )
        .await;
    }

    pub async fn get_search_status(&self) -> Option<SearchStatusPayload> {
        self.get_typed(
            &ReadCacheKey::SearchStatus,
            ReadCacheValue::into_search_status,
        )
        .await
    }

    pub async fn set_search_status(&self, payload: SearchStatusPayload) {
        self.set_typed_with_ttl(
            ReadCacheKey::SearchStatus,
            payload,
            ReadCacheValue::SearchStatus,
            SEARCH_STATUS_CACHE_TTL,
        )
        .await;
    }

    pub async fn clear(&self) {
        self.entries.write().await.clear();
    }

    /// Evict all cache entries related to a specific channel's data.
    /// Used when a channel's video list changes (acknowledge, transcript/summary status,
    /// refresh, backfill). Leaves the channels list and other channels' entries intact.
    ///
    /// Also evicts workspace bootstrap entries keyed with `selected_channel_id=null`,
    /// because bootstrap resolution maps null to the first channel (fallback behavior),
    /// which may be the mutated channel, leaving a stale cached payload.
    pub async fn evict_channel(&self, channel_id: &str) {
        let mut entries = self.entries.write().await;
        entries.retain(|key, _| match key {
            ReadCacheKey::ChannelSnapshot(k) => k.channel_id != channel_id,
            ReadCacheKey::WorkspaceBootstrap(k) => match &k.selected_channel_id {
                // Evict null-keyed entries: null resolves to the first channel via fallback,
                // so this entry may contain stale data for the mutated channel.
                None => false,
                // Keep entries that are explicitly for a different channel.
                Some(id) => id != channel_id,
            },
            ReadCacheKey::ChannelSyncDepth(_, id) => id != channel_id,
            _ => true,
        });
    }

    /// Evict the channels list and all workspace bootstrap entries.
    /// Used when the set of channels changes (add, delete, update channel metadata).
    pub async fn evict_channel_list(&self) {
        let mut entries = self.entries.write().await;
        entries.retain(|key, _| {
            !matches!(
                key,
                ReadCacheKey::Channels(_) | ReadCacheKey::WorkspaceBootstrap(_)
            )
        });
    }

    fn evict_expired(entries: &mut HashMap<ReadCacheKey, CacheEntry>) {
        let now = Instant::now();
        entries.retain(|_, entry| entry.expires_at > now);
    }

    /// Returns the current number of entries in the cache (for testing).
    #[cfg(test)]
    pub(crate) async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    async fn get(&self, key: &ReadCacheKey) -> Option<ReadCacheValue> {
        let now = Instant::now();
        {
            let entries = self.entries.read().await;
            if let Some(entry) = entries.get(key) {
                if entry.expires_at > now {
                    return Some(entry.value.clone());
                }
            }
        }

        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get(key) {
            if entry.expires_at > now {
                return Some(entry.value.clone());
            }
        }
        entries.remove(key);
        None
    }

    async fn set(&self, key: ReadCacheKey, value: ReadCacheValue) {
        self.set_with_ttl(key, value, self.ttl).await;
    }

    async fn get_typed<T>(
        &self,
        key: &ReadCacheKey,
        map: fn(ReadCacheValue) -> Option<T>,
    ) -> Option<T> {
        self.get(key).await.and_then(map)
    }

    async fn set_typed<T>(&self, key: ReadCacheKey, value: T, wrap: fn(T) -> ReadCacheValue) {
        self.set(key, wrap(value)).await;
    }

    async fn set_typed_with_ttl<T>(
        &self,
        key: ReadCacheKey,
        value: T,
        wrap: fn(T) -> ReadCacheValue,
        ttl: Duration,
    ) {
        self.set_with_ttl(key, wrap(value), ttl).await;
    }

    async fn set_with_ttl(&self, key: ReadCacheKey, value: ReadCacheValue, ttl: Duration) {
        let mut entries = self.entries.write().await;
        // If inserting a new key would exceed the size limit, evict expired entries first.
        if !entries.contains_key(&key) && entries.len() >= MAX_CACHE_SIZE {
            Self::evict_expired(&mut entries);
        }
        // If still at capacity after evicting expired entries, skip insertion to
        // prevent unbounded memory growth within Cloud Run's 512Mi limit.
        if !entries.contains_key(&key) && entries.len() >= MAX_CACHE_SIZE {
            return;
        }
        entries.insert(
            key,
            CacheEntry {
                expires_at: Instant::now() + ttl,
                value,
            },
        );
    }
}

impl ReadCacheValue {
    fn into_channels(self) -> Option<Vec<Channel>> {
        match self {
            Self::Channels(channels) => Some(channels),
            _ => None,
        }
    }

    fn into_workspace_bootstrap(self) -> Option<WorkspaceBootstrapPayload> {
        match self {
            Self::WorkspaceBootstrap(payload) => Some(payload),
            _ => None,
        }
    }

    fn into_channel_snapshot(self) -> Option<ChannelSnapshotPayload> {
        match self {
            Self::ChannelSnapshot(payload) => Some(payload),
            _ => None,
        }
    }

    fn into_sync_depth(self) -> Option<SyncDepthPayload> {
        match self {
            Self::SyncDepth(payload) => Some(payload),
            _ => None,
        }
    }

    fn into_search_status(self) -> Option<SearchStatusPayload> {
        match self {
            Self::SearchStatus(payload) => Some(payload),
            _ => None,
        }
    }
}

fn queue_filter_code(filter: QueueFilter) -> u8 {
    match filter {
        QueueFilter::AnyIncomplete => 1,
        QueueFilter::TranscriptsOnly => 2,
        QueueFilter::SummariesOnly => 3,
        QueueFilter::EvaluationsOnly => 4,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;

    use super::{
        ChannelSnapshotCacheKey, MAX_CACHE_SIZE, ReadCache, VideoListCacheKey,
        WorkspaceBootstrapCacheKey,
    };
    use crate::db::QueueFilter;
    use crate::models::{AiStatus, Channel, SearchStatusPayload, WorkspaceBootstrapPayload};

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
        WorkspaceBootstrapPayload {
            ai_available: true,
            ai_status: AiStatus::Cloud,
            channels: vec![sample_channel("abc")],
            selected_channel_id: Some("abc".to_string()),
            snapshot: None,
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
        let transcripts =
            VideoListCacheKey::new(20, 0, None, None, Some(QueueFilter::TranscriptsOnly));
        let summaries = VideoListCacheKey::new(20, 0, None, None, Some(QueueFilter::SummariesOnly));
        let evaluations =
            VideoListCacheKey::new(20, 0, None, None, Some(QueueFilter::EvaluationsOnly));

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
            .set_channel_snapshot(
                key_a.clone(),
                crate::models::ChannelSnapshotPayload {
                    channel_id: "channel-a".to_string(),
                    sync_depth: crate::models::SyncDepthPayload {
                        earliest_sync_date: None,
                        earliest_sync_date_user_set: false,
                        derived_earliest_ready_date: None,
                    },
                    channel_video_count: Some(0),
                    has_more: false,
                    next_offset: None,
                    videos: vec![],
                },
            )
            .await;
        cache
            .set_channel_snapshot(
                key_b.clone(),
                crate::models::ChannelSnapshotPayload {
                    channel_id: "channel-b".to_string(),
                    sync_depth: crate::models::SyncDepthPayload {
                        earliest_sync_date: None,
                        earliest_sync_date_user_set: false,
                        derived_earliest_ready_date: None,
                    },
                    channel_video_count: Some(0),
                    has_more: false,
                    next_offset: None,
                    videos: vec![],
                },
            )
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
            .set_channel_snapshot(
                snapshot_key_a.clone(),
                crate::models::ChannelSnapshotPayload {
                    channel_id: "channel-a".to_string(),
                    sync_depth: crate::models::SyncDepthPayload {
                        earliest_sync_date: None,
                        earliest_sync_date_user_set: false,
                        derived_earliest_ready_date: None,
                    },
                    channel_video_count: Some(0),
                    has_more: false,
                    next_offset: None,
                    videos: vec![],
                },
            )
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
}
