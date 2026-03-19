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
    Channels,
    WorkspaceBootstrap(WorkspaceBootstrapCacheKey),
    ChannelSnapshot(ChannelSnapshotCacheKey),
    ChannelSyncDepth(String),
    SearchStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceBootstrapCacheKey {
    pub selected_channel_id: Option<String>,
    pub video_list: VideoListCacheKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelSnapshotCacheKey {
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

    pub async fn get_channels(&self) -> Option<Vec<Channel>> {
        self.get_typed(&ReadCacheKey::Channels, ReadCacheValue::into_channels)
            .await
    }

    pub async fn set_channels(&self, channels: Vec<Channel>) {
        self.set_typed(ReadCacheKey::Channels, channels, ReadCacheValue::Channels)
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

    pub async fn get_channel_sync_depth(&self, channel_id: &str) -> Option<SyncDepthPayload> {
        self.get_typed(
            &ReadCacheKey::ChannelSyncDepth(channel_id.to_string()),
            ReadCacheValue::into_sync_depth,
        )
        .await
    }

    pub async fn set_channel_sync_depth(&self, channel_id: String, payload: SyncDepthPayload) {
        self.set_typed(
            ReadCacheKey::ChannelSyncDepth(channel_id),
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
        self.entries.write().await.insert(
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

    use super::{ReadCache, VideoListCacheKey, WorkspaceBootstrapCacheKey};
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

        cache.set_channels(channels.clone()).await;

        let cached = cache
            .get_channels()
            .await
            .expect("channels should be cached");
        assert_eq!(cached.len(), channels.len());
        assert_eq!(cached[0].id, channels[0].id);
    }

    #[tokio::test]
    async fn expires_entries_after_ttl() {
        let cache = ReadCache::new(Duration::from_millis(1));
        let key = WorkspaceBootstrapCacheKey {
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
        let key = WorkspaceBootstrapCacheKey {
            selected_channel_id: Some("abc".to_string()),
            video_list: VideoListCacheKey::new(20, 0, None, None, None),
        };

        cache.set_channels(vec![sample_channel("abc")]).await;
        cache
            .set_workspace_bootstrap(key.clone(), sample_bootstrap())
            .await;
        cache.set_search_status(sample_search_status()).await;

        cache.clear().await;

        assert!(cache.get_channels().await.is_none());
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
    async fn workspace_bootstrap_cache_keeps_entries_separate_by_video_filter() {
        let cache = ReadCache::new(Duration::from_secs(60));
        let long_videos_key = WorkspaceBootstrapCacheKey {
            selected_channel_id: Some("abc".to_string()),
            video_list: VideoListCacheKey::new(20, 0, Some(false), None, None),
        };
        let queued_videos_key = WorkspaceBootstrapCacheKey {
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
