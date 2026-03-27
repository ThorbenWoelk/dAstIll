import { browser } from "$app/environment";
import {
  getChannelSnapshot,
  listVideos,
  refreshChannel,
  updateChannel,
} from "$lib/api";
import { resolveDisplayedSyncDepthIso } from "$lib/sync-depth";
import type {
  Channel,
  ChannelSnapshot,
  QueueTab,
  SyncDepth,
  Video,
  VideoTypeFilter,
} from "$lib/types";
import { OTHERS_CHANNEL_ID } from "$lib/types";
import {
  dedupeVideosById,
  filterVideosByAcknowledged,
  filterVideosByType,
  resolveInitialPreviewExpandedChannelId,
  resolveVirtualWindow,
  shouldForceReloadMissingSelectedVideo,
  shouldLoadAllChannelVideosForSelection,
} from "$lib/workspace/route-helpers";
import {
  getSidebarPreviewSession,
  pruneSidebarPreviewCollections,
  resolvePreferredExpandedSidebarPreviewCollectionId,
  setSidebarPreviewSession,
  setSingleExpandedSidebarPreviewCollection,
  type SidebarPreviewCollectionSnapshot,
} from "$lib/workspace/sidebar-preview-session";
import {
  resolveAcknowledgedParam,
  type AcknowledgedFilter,
} from "$lib/workspace/types";
import {
  resolveSyncDateInputValue,
  toIsoDateStart,
} from "$lib/workspace/sidebar-sync-date";

const PREVIEW_VISIBLE_VIDEO_COUNT = 5;
const PREVIEW_FETCH_LIMIT = PREVIEW_VISIBLE_VIDEO_COUNT + 1;
const EXPANDED_PAGE_SIZE = 30;
const VIRTUALIZATION_THRESHOLD = 24;
const VIRTUALIZED_ROW_HEIGHT = 56;
const VIRTUALIZED_OVERSCAN = 8;
const VIRTUALIZED_VIEWPORT_HEIGHT = 336;

type ChannelVideoCollectionLoadMode = "preview" | "paged";

export type ChannelVideoCollectionState = {
  videos: Video[];
  expanded: boolean;
  loadingInitial: boolean;
  loadingMore: boolean;
  loadedMode: ChannelVideoCollectionLoadMode | null;
  hasMore: boolean;
  nextOffset: number;
  channelVideoCount: number | null;
  filterKey: string | null;
  requestKey: string | null;
  syncDepth: SyncDepth | null;
  earliestSyncDateInput: string;
  savingSyncDate: boolean;
  selectedVideoReloadProbeKey: string | null;
  scrollTop: number;
};

export type RenderedCollectionVideos = {
  videos: Video[];
  topSpacer: number;
  bottomSpacer: number;
  virtualized: boolean;
};

type VideoAcknowledgeSync = {
  seq: number;
  video: Video;
  confirmed: boolean;
};

type SidebarPreviewControllerOptions = {
  getEnabled: () => boolean;
  getChannels: () => Channel[];
  getFilteredChannels: () => Channel[];
  getSelectedChannelId: () => string | null;
  getSelectedChannel: () => Channel | null;
  getSelectedVideoId: () => string | null;
  getVideoTypeFilter: () => VideoTypeFilter;
  getAcknowledgedFilter: () => AcknowledgedFilter;
  getHasActiveVideoFilters: () => boolean;
  getReadOnly: () => boolean;
  getInitialChannelPreviews: () => Record<string, ChannelSnapshot>;
  getInitialChannelPreviewsFilterKey: () => string | undefined;
  getChannelSnapshotQueueTab: () => QueueTab | undefined;
  getChannelQueueSnapshotUnified: () => boolean;
  getQueueVideoRefreshTick: () => number;
  getVideoAcknowledgeSync: () => VideoAcknowledgeSync | null;
  getPreviewSessionKey: () => string | undefined;
  onChannelUpdated?: (channel: Channel) => void | Promise<void>;
  onChannelSyncDateSaved?: (channelId: string) => void | Promise<void>;
};

export function createEmptyChannelVideoCollection(): ChannelVideoCollectionState {
  return {
    videos: [],
    expanded: false,
    loadingInitial: false,
    loadingMore: false,
    loadedMode: null,
    hasMore: false,
    nextOffset: 0,
    channelVideoCount: null,
    filterKey: null,
    requestKey: null,
    syncDepth: null,
    earliestSyncDateInput: "",
    savingSyncDate: false,
    selectedVideoReloadProbeKey: null,
    scrollTop: 0,
  };
}

export function createSidebarPreviewController(
  options: SidebarPreviewControllerOptions,
) {
  const emptyChannelVideoCollection = createEmptyChannelVideoCollection();

  let channelVideoCollections = $state<
    Record<string, ChannelVideoCollectionState>
  >({});
  let hydratedPreviewSessionKey = $state<string | null>(null);
  let lastAppliedVideoAcknowledgeSeq = $state(0);
  let lastAutoExpandedChannelId = $state<string | null>(null);
  let syncDatePickerChannelId = $state<string | null>(null);

  function channelListEmptyCaption(channelVideoCount: number | null): string {
    if (channelVideoCount === null) {
      return "Nothing to show.";
    }
    if (channelVideoCount === 0) {
      return "No videos yet.";
    }
    if (options.getHasActiveVideoFilters()) {
      return "Nothing matches the current filters.";
    }
    return "Nothing to show.";
  }

  function getChannelVideoCollectionFilterKey() {
    const queueSegment = options.getChannelSnapshotQueueTab()
      ? options.getChannelSnapshotQueueTab()
      : options.getChannelQueueSnapshotUnified()
        ? "unified"
        : "default";
    return `${options.getVideoTypeFilter()}:${options.getAcknowledgedFilter()}:${queueSegment}`;
  }

  function supportsMode(
    state: ChannelVideoCollectionState,
    filterKey: string,
    mode: ChannelVideoCollectionLoadMode,
  ) {
    if (state.filterKey !== filterKey) {
      return false;
    }

    if (mode === "preview") {
      return state.loadedMode === "preview" || state.loadedMode === "paged";
    }

    return state.loadedMode === "paged";
  }

  function constrainVideosToChannel(channelId: string, videos: Video[]) {
    if (channelId === OTHERS_CHANNEL_ID) {
      return dedupeVideosById(videos);
    }

    return dedupeVideosById(
      videos.filter((video) => video.channel_id === channelId),
    );
  }

  function ensureChannelVideoCollection(channelId: string) {
    const existingCollection = channelVideoCollections[channelId];
    if (existingCollection) {
      return existingCollection;
    }

    const nextCollection = createEmptyChannelVideoCollection();
    channelVideoCollections[channelId] = nextCollection;
    return nextCollection;
  }

  function restoreChannelVideoCollections(
    collections: Record<string, SidebarPreviewCollectionSnapshot>,
  ): Record<string, ChannelVideoCollectionState> {
    const restored: Record<string, ChannelVideoCollectionState> = {};

    for (const [channelId, collection] of Object.entries(collections)) {
      restored[channelId] = {
        ...createEmptyChannelVideoCollection(),
        ...collection,
        videos: constrainVideosToChannel(channelId, collection.videos),
      };
    }

    return restored;
  }

  function setExpandedPreviewChannel(channelId: string | null) {
    setSingleExpandedSidebarPreviewCollection(
      channelVideoCollections,
      channelId,
    );
  }

  function resolveVisibleCollectionVideos(
    collection: ChannelVideoCollectionState,
  ): Video[] {
    if (collection.loadedMode === "preview") {
      return collection.videos.slice(0, PREVIEW_VISIBLE_VIDEO_COUNT);
    }

    return collection.videos;
  }

  function resolveRenderedCollectionVideos(
    collection: ChannelVideoCollectionState,
  ): RenderedCollectionVideos {
    const visibleVideos = resolveVisibleCollectionVideos(collection);
    if (
      collection.loadedMode !== "paged" ||
      visibleVideos.length <= VIRTUALIZATION_THRESHOLD
    ) {
      return {
        videos: visibleVideos,
        topSpacer: 0,
        bottomSpacer: 0,
        virtualized: false,
      };
    }

    const selectedVideoId = options.getSelectedVideoId();
    const selectedIndex = selectedVideoId
      ? visibleVideos.findIndex((video) => video.id === selectedVideoId)
      : -1;
    const window = resolveVirtualWindow({
      itemCount: visibleVideos.length,
      itemHeight: VIRTUALIZED_ROW_HEIGHT,
      viewportHeight: VIRTUALIZED_VIEWPORT_HEIGHT,
      scrollTop: collection.scrollTop,
      overscan: VIRTUALIZED_OVERSCAN,
    });
    let start = window.startIndex;
    let end = window.endIndex;

    if (selectedIndex >= 0 && (selectedIndex < start || selectedIndex >= end)) {
      const renderCount = end - start;
      start = Math.max(0, selectedIndex - VIRTUALIZED_OVERSCAN);
      end = Math.min(visibleVideos.length, start + renderCount);
    }

    return {
      videos: visibleVideos.slice(start, end),
      topSpacer: window.offsetTop,
      bottomSpacer: Math.max(
        0,
        window.totalHeight -
          window.offsetTop -
          (end - start) * VIRTUALIZED_ROW_HEIGHT,
      ),
      virtualized: true,
    };
  }

  async function loadChannelVideoCollection(
    channel: Channel,
    mode: ChannelVideoCollectionLoadMode,
    requestOptions?: { force?: boolean; append?: boolean },
  ) {
    const force = requestOptions?.force ?? false;
    const append = requestOptions?.append ?? false;
    const state = ensureChannelVideoCollection(channel.id);
    const filterKey = getChannelVideoCollectionFilterKey();

    if (
      append &&
      (!state.hasMore ||
        state.loadingMore ||
        state.filterKey !== filterKey ||
        state.requestKey !== null)
    ) {
      return;
    }

    if (
      !append &&
      (state.loadingInitial || state.loadingMore) &&
      state.filterKey === filterKey &&
      !force
    ) {
      return;
    }

    if (!append && !force && supportsMode(state, filterKey, mode)) {
      return;
    }

    const initialChannelPreviewsFilterKey =
      options.getInitialChannelPreviewsFilterKey();
    const initialChannelPreviews = options.getInitialChannelPreviews();
    if (
      !force &&
      !append &&
      mode === "preview" &&
      initialChannelPreviewsFilterKey &&
      filterKey === initialChannelPreviewsFilterKey &&
      channel.id in initialChannelPreviews
    ) {
      const preloaded = initialChannelPreviews[channel.id];
      state.videos = constrainVideosToChannel(channel.id, preloaded.videos);
      state.loadedMode = "preview";
      state.filterKey = filterKey;
      state.loadingInitial = false;
      state.loadingMore = false;
      state.hasMore = preloaded.has_more;
      state.nextOffset = preloaded.next_offset ?? preloaded.videos.length;
      state.channelVideoCount = preloaded.channel_video_count ?? null;
      state.syncDepth = preloaded.sync_depth;
      state.earliestSyncDateInput = resolveSyncDateInputValue(
        channel,
        preloaded.sync_depth,
      );
      return;
    }

    const requestOffset = append ? state.nextOffset : 0;
    const requestKey = `${channel.id}:${filterKey}:${mode}:${requestOffset}:${Date.now()}`;
    if (append) {
      state.loadingMore = true;
    } else {
      state.loadingInitial = true;
    }
    state.filterKey = filterKey;
    state.requestKey = requestKey;

    const acknowledged = resolveAcknowledgedParam(
      options.getAcknowledgedFilter(),
    );
    const pageLimit =
      mode === "paged" ? EXPANDED_PAGE_SIZE : PREVIEW_FETCH_LIMIT;
    const queueOnly =
      Boolean(options.getChannelSnapshotQueueTab()) ||
      options.getChannelQueueSnapshotUnified();

    try {
      const current = channelVideoCollections[channel.id];
      if (!current || current.requestKey !== requestKey) {
        return;
      }

      if (!append) {
        const snapshot = await getChannelSnapshot(channel.id, {
          limit: pageLimit,
          offset: 0,
          videoType: options.getVideoTypeFilter(),
          acknowledged,
          queueOnly: queueOnly ? true : undefined,
          queueTab: options.getChannelSnapshotQueueTab(),
          bypassCache: force,
        });

        if (current.requestKey !== requestKey) {
          return;
        }

        current.videos = constrainVideosToChannel(channel.id, snapshot.videos);
        current.loadedMode = mode;
        current.loadingInitial = false;
        current.loadingMore = false;
        current.filterKey = filterKey;
        current.requestKey = null;
        current.hasMore = snapshot.has_more;
        current.nextOffset = snapshot.next_offset ?? current.videos.length;
        current.channelVideoCount = snapshot.channel_video_count ?? null;
        current.syncDepth = snapshot.sync_depth;
        current.earliestSyncDateInput = resolveSyncDateInputValue(
          channel,
          snapshot.sync_depth,
        );
        if (mode !== "paged") {
          current.scrollTop = 0;
        }
        return;
      }

      const page = await listVideos(
        channel.id,
        pageLimit,
        requestOffset,
        options.getVideoTypeFilter(),
        acknowledged,
        queueOnly,
        options.getChannelSnapshotQueueTab(),
        force,
      );

      if (current.requestKey !== requestKey) {
        return;
      }

      current.videos = constrainVideosToChannel(channel.id, [
        ...current.videos,
        ...page.videos,
      ]);
      current.loadedMode = "paged";
      current.loadingInitial = false;
      current.loadingMore = false;
      current.filterKey = filterKey;
      current.requestKey = null;
      current.hasMore = page.has_more;
      current.nextOffset = page.next_offset ?? current.videos.length;
    } catch {
      const current = channelVideoCollections[channel.id];
      if (!current || current.requestKey !== requestKey) {
        return;
      }

      current.loadingInitial = false;
      current.loadingMore = false;
      current.requestKey = null;
    }
  }

  async function toggleChannelVideoCollection(channel: Channel) {
    const state = ensureChannelVideoCollection(channel.id);
    if (state.expanded) {
      setExpandedPreviewChannel(null);
      return;
    }

    setExpandedPreviewChannel(channel.id);
    const nextState = ensureChannelVideoCollection(channel.id);
    nextState.scrollTop = 0;

    const filterKey = getChannelVideoCollectionFilterKey();
    if (
      nextState.filterKey === filterKey &&
      nextState.loadedMode === "preview"
    ) {
      nextState.loadedMode = "paged";
      if (nextState.videos.length < EXPANDED_PAGE_SIZE && nextState.hasMore) {
        await loadChannelVideoCollection(channel, "paged", {
          append: true,
        });
      }
      return;
    }

    await loadChannelVideoCollection(channel, "paged");
  }

  async function loadNextChannelVideoPage(channel: Channel) {
    const state = ensureChannelVideoCollection(channel.id);
    if (!state.expanded || !state.hasMore) {
      return;
    }

    await loadChannelVideoCollection(channel, "paged", { append: true });
  }

  function handleChannelCollectionScroll(channel: Channel, event: Event) {
    const currentTarget = event.currentTarget;
    if (!(currentTarget instanceof HTMLDivElement)) {
      return;
    }
    const state = ensureChannelVideoCollection(channel.id);
    state.scrollTop = currentTarget.scrollTop;

    if (
      state.loadedMode !== "paged" ||
      !state.hasMore ||
      state.loadingMore ||
      state.loadingInitial
    ) {
      return;
    }

    const remaining =
      currentTarget.scrollHeight -
      (currentTarget.scrollTop + currentTarget.clientHeight);
    if (remaining <= VIRTUALIZED_ROW_HEIGHT * 2) {
      void loadNextChannelVideoPage(channel);
    }
  }

  function toggleSyncDatePicker(
    channel: Channel,
    depth: SyncDepth | null,
    collection: ChannelVideoCollectionState | undefined,
  ) {
    if (syncDatePickerChannelId === channel.id) {
      syncDatePickerChannelId = null;
      return;
    }
    syncDatePickerChannelId = channel.id;
    if (collection) {
      collection.earliestSyncDateInput = resolveSyncDateInputValue(
        channel,
        depth,
      );
    }
  }

  async function saveChannelSyncDate(channel: Channel) {
    const state = ensureChannelVideoCollection(channel.id);
    if (!state.earliestSyncDateInput || state.savingSyncDate) {
      return;
    }

    state.savingSyncDate = true;

    try {
      const updatedChannel = await updateChannel(channel.id, {
        earliest_sync_date: toIsoDateStart(state.earliestSyncDateInput),
        earliest_sync_date_user_set: true,
      });
      await options.onChannelUpdated?.(updatedChannel);
      await refreshChannel(channel.id);
      await loadChannelVideoCollection(
        updatedChannel,
        state.expanded ? "paged" : "preview",
      );
      syncDatePickerChannelId = null;
      await options.onChannelSyncDateSaved?.(channel.id);
    } finally {
      const current = ensureChannelVideoCollection(channel.id);
      current.savingSyncDate = false;
    }
  }

  $effect(() => {
    const previewSessionKey = options.getPreviewSessionKey();
    if (!options.getEnabled() || !previewSessionKey) {
      hydratedPreviewSessionKey = null;
      return;
    }

    if (hydratedPreviewSessionKey === previewSessionKey) {
      return;
    }

    channelVideoCollections = restoreChannelVideoCollections(
      getSidebarPreviewSession(previewSessionKey) ?? {},
    );
    setExpandedPreviewChannel(
      resolvePreferredExpandedSidebarPreviewCollectionId(
        channelVideoCollections,
        options.getSelectedChannelId(),
      ),
    );
    hydratedPreviewSessionKey = previewSessionKey;
  });

  $effect(() => {
    if (!browser || !options.getChannelQueueSnapshotUnified()) return;
    const refreshTick = options.getQueueVideoRefreshTick();
    if (refreshTick === 0) return;
    if (!options.getEnabled()) return;

    for (const channel of options.getChannels()) {
      const state = channelVideoCollections[channel.id];
      if (!state?.expanded) continue;
      const mode = state.loadedMode === "preview" ? "preview" : "paged";
      void loadChannelVideoCollection(channel, mode, { force: true });
    }
  });

  $effect(() => {
    if (!options.getEnabled()) {
      return;
    }
    const sync = options.getVideoAcknowledgeSync();
    if (!sync || sync.seq <= lastAppliedVideoAcknowledgeSeq) {
      return;
    }
    lastAppliedVideoAcknowledgeSeq = sync.seq;
    const { video, confirmed } = sync;
    const state = channelVideoCollections[video.channel_id];
    if (!state) {
      return;
    }
    const merged = state.videos.map((v) => (v.id === video.id ? video : v));
    const byType = filterVideosByType(merged, options.getVideoTypeFilter());
    const filtered = filterVideosByAcknowledged(
      byType,
      options.getAcknowledgedFilter(),
    );
    state.videos = constrainVideosToChannel(video.channel_id, filtered);

    if (!confirmed) {
      return;
    }

    const channel = options
      .getChannels()
      .find((candidate) => candidate.id === video.channel_id);
    if (!channel || !state.expanded) {
      return;
    }

    if (filtered.length === 0) {
      void loadChannelVideoCollection(channel, "paged", { force: true });
    }
  });

  $effect(() => {
    const previewSessionKey = options.getPreviewSessionKey();
    if (!options.getEnabled() || !previewSessionKey) {
      return;
    }

    setSidebarPreviewSession(
      previewSessionKey,
      pruneSidebarPreviewCollections(
        channelVideoCollections,
        options.getChannels().map((channel) => channel.id),
      ),
    );
  });

  $effect(() => {
    if (!options.getEnabled()) {
      return;
    }

    const filterKey = getChannelVideoCollectionFilterKey();
    const visibleChannelIds = options
      .getFilteredChannels()
      .map((channel) => channel.id);

    for (const channel of options.getFilteredChannels()) {
      const state = ensureChannelVideoCollection(channel.id);
      if (!state.expanded) continue;
      if (supportsMode(state, filterKey, "paged")) continue;
      void loadChannelVideoCollection(channel, "paged");
    }

    for (const channelId of Object.keys(channelVideoCollections)) {
      if (
        !visibleChannelIds.includes(channelId) &&
        !options.getChannels().some((channel) => channel.id === channelId)
      ) {
        delete channelVideoCollections[channelId];
      }
    }
  });

  $effect(() => {
    if (!options.getEnabled()) return;
    const targetChannelId = resolveInitialPreviewExpandedChannelId(
      options.getFilteredChannels(),
      options.getSelectedChannelId(),
      OTHERS_CHANNEL_ID,
    );
    if (!targetChannelId || targetChannelId === lastAutoExpandedChannelId) {
      return;
    }

    const targetChannel = options
      .getChannels()
      .find((channel) => channel.id === targetChannelId);
    if (!targetChannel || targetChannel.id === OTHERS_CHANNEL_ID) {
      return;
    }

    setExpandedPreviewChannel(targetChannel.id);
    const nextState = ensureChannelVideoCollection(targetChannel.id);
    lastAutoExpandedChannelId = targetChannel.id;

    const preferredMode =
      options.getSelectedVideoId() &&
      options.getSelectedChannelId() === targetChannel.id
        ? "paged"
        : "preview";
    if (
      !supportsMode(
        nextState,
        getChannelVideoCollectionFilterKey(),
        preferredMode,
      )
    ) {
      void loadChannelVideoCollection(targetChannel, preferredMode);
    }
  });

  $effect(() => {
    if (!options.getEnabled()) return;

    const selectedChannel = options.getSelectedChannel();
    const selectedVideoId = options.getSelectedVideoId();
    if (!selectedChannel || !selectedVideoId) return;
    if (selectedChannel.id === OTHERS_CHANNEL_ID) return;

    const state = ensureChannelVideoCollection(selectedChannel.id);
    if (!state.expanded) {
      setExpandedPreviewChannel(selectedChannel.id);
    }

    const nextState = ensureChannelVideoCollection(selectedChannel.id);
    if (nextState.loadingInitial || nextState.loadingMore) return;
    if (
      shouldLoadAllChannelVideosForSelection({
        selectedVideoId,
        videos: nextState.videos,
        loadedMode: nextState.loadedMode,
        hasMore: nextState.hasMore,
      })
    ) {
      if (nextState.loadedMode === "preview") {
        nextState.loadedMode = "paged";
      }
      void loadNextChannelVideoPage(selectedChannel);
      return;
    }

    const probeKey = `${selectedChannel.id}:${selectedVideoId}:${getChannelVideoCollectionFilterKey()}`;
    if (
      !shouldForceReloadMissingSelectedVideo({
        selectedVideoId,
        videos: nextState.videos,
        probeKey,
        lastProbeKey: nextState.selectedVideoReloadProbeKey,
      })
    ) {
      if (nextState.videos.some((video) => video.id === selectedVideoId)) {
        nextState.selectedVideoReloadProbeKey = null;
      }
      return;
    }

    nextState.selectedVideoReloadProbeKey = probeKey;
    void loadChannelVideoCollection(selectedChannel, "paged", { force: true });
  });

  return {
    emptyChannelVideoCollection,
    get channelVideoCollections() {
      return channelVideoCollections;
    },
    get syncDatePickerChannelId() {
      return syncDatePickerChannelId;
    },
    channelListEmptyCaption,
    ensureChannelVideoCollection,
    handleChannelCollectionScroll,
    loadNextChannelVideoPage,
    resolveDisplayedSyncDepthIso,
    resolveRenderedCollectionVideos,
    saveChannelSyncDate,
    toggleChannelVideoCollection,
    toggleSyncDatePicker,
  };
}
