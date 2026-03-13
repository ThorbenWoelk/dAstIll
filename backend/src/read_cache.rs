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
        match self.get(&ReadCacheKey::Channels).await {
            Some(ReadCacheValue::Channels(channels)) => Some(channels),
            _ => None,
        }
    }

    pub async fn set_channels(&self, channels: Vec<Channel>) {
        self.set(ReadCacheKey::Channels, ReadCacheValue::Channels(channels))
            .await;
    }

    pub async fn get_workspace_bootstrap(
        &self,
        key: &WorkspaceBootstrapCacheKey,
    ) -> Option<WorkspaceBootstrapPayload> {
        match self
            .get(&ReadCacheKey::WorkspaceBootstrap(key.clone()))
            .await
        {
            Some(ReadCacheValue::WorkspaceBootstrap(payload)) => Some(payload),
            _ => None,
        }
    }

    pub async fn set_workspace_bootstrap(
        &self,
        key: WorkspaceBootstrapCacheKey,
        payload: WorkspaceBootstrapPayload,
    ) {
        self.set(
            ReadCacheKey::WorkspaceBootstrap(key),
            ReadCacheValue::WorkspaceBootstrap(payload),
        )
        .await;
    }

    pub async fn get_channel_snapshot(
        &self,
        key: &ChannelSnapshotCacheKey,
    ) -> Option<ChannelSnapshotPayload> {
        match self.get(&ReadCacheKey::ChannelSnapshot(key.clone())).await {
            Some(ReadCacheValue::ChannelSnapshot(payload)) => Some(payload),
            _ => None,
        }
    }

    pub async fn set_channel_snapshot(
        &self,
        key: ChannelSnapshotCacheKey,
        payload: ChannelSnapshotPayload,
    ) {
        self.set(
            ReadCacheKey::ChannelSnapshot(key),
            ReadCacheValue::ChannelSnapshot(payload),
        )
        .await;
    }

    pub async fn get_channel_sync_depth(&self, channel_id: &str) -> Option<SyncDepthPayload> {
        match self
            .get(&ReadCacheKey::ChannelSyncDepth(channel_id.to_string()))
            .await
        {
            Some(ReadCacheValue::SyncDepth(payload)) => Some(payload),
            _ => None,
        }
    }

    pub async fn set_channel_sync_depth(&self, channel_id: String, payload: SyncDepthPayload) {
        self.set(
            ReadCacheKey::ChannelSyncDepth(channel_id),
            ReadCacheValue::SyncDepth(payload),
        )
        .await;
    }

    pub async fn get_search_status(&self) -> Option<SearchStatusPayload> {
        match self.get(&ReadCacheKey::SearchStatus).await {
            Some(ReadCacheValue::SearchStatus(payload)) => Some(payload),
            _ => None,
        }
    }

    pub async fn set_search_status(&self, payload: SearchStatusPayload) {
        self.set_with_ttl(
            ReadCacheKey::SearchStatus,
            ReadCacheValue::SearchStatus(payload),
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
                vector_index_ready: true,
                retrieval_mode: "fts".to_string(),
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
            vector_index_ready: true,
            retrieval_mode: "fts".to_string(),
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
}
