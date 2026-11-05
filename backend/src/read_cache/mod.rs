use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use tokio::sync::{Mutex, RwLock};

use crate::{
    db::QueueFilter,
    models::{
        Channel, ChannelSnapshotPayload, SearchStatusPayload, SyncDepthPayload, UserVideoState,
        Video, WorkspaceBootstrapPayload,
    },
};

const DEFAULT_READ_CACHE_TTL: Duration = Duration::from_secs(10);
const SEARCH_STATUS_CACHE_TTL: Duration = Duration::from_secs(30);
const VIDEOS_CACHE_TTL: Duration = Duration::from_secs(600);
const VIDEO_SUGGESTION_CACHE_TTL: Duration = Duration::from_secs(600);
/// Read states change only through `put_user_video_state`. A fill merges any
/// write that arrived while the map was loading, so that write stays visible
/// for the rest of this TTL.
const USER_VIDEO_STATES_CACHE_TTL: Duration = Duration::from_secs(30 * 60);
/// Maximum number of entries to keep in the cache.
/// Prevents unbounded memory growth within Cloud Run's 512Mi limit.
pub(crate) const MAX_CACHE_SIZE: usize = 512;

#[derive(Debug, Clone)]
pub struct SuggestedVideo {
    pub id: String,
    pub channel_id: String,
    pub title: String,
    pub published_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ReadCache {
    ttl: Duration,
    entries: Arc<RwLock<HashMap<ReadCacheKey, CacheEntry>>>,
    /// Mark-read writes that arrived before that reader's state map was cached.
    /// Guarded together with `entries` by locking this mutex first.
    pending_user_video_states: Arc<Mutex<HashMap<String, HashMap<String, UserVideoState>>>>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    expires_at: Instant,
    value: ReadCacheValue,
}

#[derive(Debug, Clone)]
enum ReadCacheValue {
    Channels(Vec<Channel>),
    Videos(Vec<Video>),
    ScopedVideoSuggestions(Vec<SuggestedVideo>),
    WorkspaceBootstrap(WorkspaceBootstrapPayload),
    ChannelSnapshot(ChannelSnapshotPayload),
    SyncDepth(SyncDepthPayload),
    SearchStatus(SearchStatusPayload),
    UserVideoStates(HashMap<String, UserVideoState>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReadCacheKey {
    Channels(String),
    Videos,
    ScopedVideoSuggestions(String),
    WorkspaceBootstrap(WorkspaceBootstrapCacheKey),
    ChannelSnapshot(ChannelSnapshotCacheKey),
    ChannelSyncDepth(String, String),
    SearchStatus,
    UserVideoStates(String),
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
            pending_user_video_states: Arc::new(Mutex::new(HashMap::new())),
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

    pub async fn get_videos(&self) -> Option<Vec<Video>> {
        self.get_typed(&ReadCacheKey::Videos, ReadCacheValue::into_videos)
            .await
    }

    pub async fn evict_videos(&self) {
        self.entries.write().await.remove(&ReadCacheKey::Videos);
    }

    pub async fn set_videos(&self, videos: Vec<Video>) {
        self.set_typed_with_ttl(
            ReadCacheKey::Videos,
            videos,
            ReadCacheValue::Videos,
            VIDEOS_CACHE_TTL,
        )
        .await;
    }

    pub async fn get_scoped_video_suggestions(&self, scope: &str) -> Option<Vec<SuggestedVideo>> {
        self.get_typed(
            &ReadCacheKey::ScopedVideoSuggestions(scope.to_string()),
            ReadCacheValue::into_scoped_video_suggestions,
        )
        .await
    }

    pub async fn set_scoped_video_suggestions(&self, scope: String, videos: Vec<SuggestedVideo>) {
        self.set_typed_with_ttl(
            ReadCacheKey::ScopedVideoSuggestions(scope),
            videos,
            ReadCacheValue::ScopedVideoSuggestions,
            VIDEO_SUGGESTION_CACHE_TTL,
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

    pub async fn get_user_video_states(
        &self,
        user_id: &str,
    ) -> Option<HashMap<String, UserVideoState>> {
        self.get_typed(
            &ReadCacheKey::UserVideoStates(user_id.to_string()),
            ReadCacheValue::into_user_video_states,
        )
        .await
    }

    pub async fn set_user_video_states(
        &self,
        user_id: String,
        states: HashMap<String, UserVideoState>,
    ) {
        self.publish_user_video_states(user_id, states).await;
    }

    /// Installs a freshly loaded state map without hiding a mark-read that
    /// landed while the load was in flight. A second fill keeps the map
    /// already published, including writes applied after that first fill.
    pub async fn publish_user_video_states(
        &self,
        user_id: String,
        mut loaded: HashMap<String, UserVideoState>,
    ) -> HashMap<String, UserVideoState> {
        let mut pending = self.pending_user_video_states.lock().await;
        let mut entries = self.entries.write().await;
        let key = ReadCacheKey::UserVideoStates(user_id.clone());
        if let Some(states) = Self::fresh_user_video_states(entries.get(&key)) {
            let mut states = states.clone();
            if let Some(writes) = pending.remove(&user_id) {
                for state in writes.into_values() {
                    Self::merge_user_video_state(&mut states, state);
                }
                if let Some(CacheEntry {
                    value: ReadCacheValue::UserVideoStates(cached),
                    ..
                }) = entries.get_mut(&key)
                {
                    *cached = states.clone();
                }
            }
            return states;
        }

        if let Some(writes) = pending.remove(&user_id) {
            for state in writes.into_values() {
                Self::merge_user_video_state(&mut loaded, state);
            }
        }
        if !Self::remember_user_video_states(&mut entries, &key, loaded.clone()) {
            pending.insert(user_id, loaded.clone());
        }
        loaded
    }

    /// Keeps a cached state map in step with a write. When the map is not
    /// cached yet, remembers the write so the in-flight fill cannot drop it.
    pub async fn record_user_video_state(&self, user_id: &str, state: &UserVideoState) {
        let mut pending = self.pending_user_video_states.lock().await;
        let mut entries = self.entries.write().await;
        let key = ReadCacheKey::UserVideoStates(user_id.to_string());
        if let Some(CacheEntry {
            value: ReadCacheValue::UserVideoStates(states),
            expires_at,
        }) = entries.get_mut(&key)
            && *expires_at > Instant::now()
        {
            Self::merge_user_video_state(states, state.clone());
            return;
        }
        Self::merge_user_video_state(
            pending.entry(user_id.to_string()).or_default(),
            state.clone(),
        );
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
            ReadCacheKey::Videos => false, // Evict all videos cache if any channel changes
            ReadCacheKey::ScopedVideoSuggestions(_) => false,
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
                ReadCacheKey::Channels(_)
                    | ReadCacheKey::ScopedVideoSuggestions(_)
                    | ReadCacheKey::WorkspaceBootstrap(_)
                    | ReadCacheKey::Videos
            )
        });
    }

    fn fresh_user_video_states(
        entry: Option<&CacheEntry>,
    ) -> Option<&HashMap<String, UserVideoState>> {
        let entry = entry?;
        if entry.expires_at <= Instant::now() {
            return None;
        }
        match &entry.value {
            ReadCacheValue::UserVideoStates(states) => Some(states),
            _ => None,
        }
    }

    /// Keeps the newer write. Equal timestamps keep `state`, the write that just arrived.
    fn merge_user_video_state(states: &mut HashMap<String, UserVideoState>, state: UserVideoState) {
        if states
            .get(&state.video_id)
            .is_some_and(|existing| existing.updated_at > state.updated_at)
        {
            return;
        }
        states.insert(state.video_id.clone(), state);
    }

    fn remember_user_video_states(
        entries: &mut HashMap<ReadCacheKey, CacheEntry>,
        key: &ReadCacheKey,
        states: HashMap<String, UserVideoState>,
    ) -> bool {
        if !entries.contains_key(key) && entries.len() >= MAX_CACHE_SIZE {
            ReadCache::evict_expired(entries);
        }
        if !entries.contains_key(key) && entries.len() >= MAX_CACHE_SIZE {
            return false;
        }
        entries.insert(
            key.clone(),
            CacheEntry {
                expires_at: Instant::now() + USER_VIDEO_STATES_CACHE_TTL,
                value: ReadCacheValue::UserVideoStates(states),
            },
        );
        true
    }

    fn evict_expired(entries: &mut HashMap<ReadCacheKey, CacheEntry>) {
        let now = Instant::now();
        entries.retain(|_, entry| entry.expires_at > now);
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

    fn into_user_video_states(self) -> Option<HashMap<String, UserVideoState>> {
        match self {
            Self::UserVideoStates(states) => Some(states),
            _ => None,
        }
    }

    fn into_videos(self) -> Option<Vec<Video>> {
        match self {
            Self::Videos(videos) => Some(videos),
            _ => None,
        }
    }

    fn into_scoped_video_suggestions(self) -> Option<Vec<SuggestedVideo>> {
        match self {
            Self::ScopedVideoSuggestions(videos) => Some(videos),
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
#[path = "tests.rs"]
mod tests;
