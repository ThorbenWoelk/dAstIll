import { untrack } from "svelte";
import { SvelteSet } from "svelte/reactivity";
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
  shouldForceReloadMissingSelectedVideo,
  shouldLoadAllChannelVideosForSelection,
} from "$lib/workspace/route-helpers";
import {
  demoteOtherPagedSidebarPreviewCollections,
  getSidebarPreviewSession,
  pruneSidebarPreviewCollections,
  resolvePreferredExpandedSidebarPreviewCollectionId,
  setSidebarPreviewCollectionExpanded,
  setSidebarPreviewSession,
  type SidebarPreviewCollectionSnapshot,
} from "$lib/workspace/sidebar-preview-session";
import {
  resolveAcknowledgedParam,
  type AcknowledgedFilter,
} from "$lib/workspace/types";
import type {
  WorkspaceSidebarPreviewProps,
  WorkspaceSidebarPreviewScope,
} from "$lib/workspace/component-props";
import {
  resolveSyncDateInputValue,
  toIsoDateStart,
} from "$lib/workspace/sidebar-sync-date";

const PREVIEW_VISIBLE_VIDEO_COUNT = 5;
const PREVIEW_FETCH_LIMIT = PREVIEW_VISIBLE_VIDEO_COUNT + 1;
const EXPANDED_PAGE_SIZE = 30;
const PREVIEW_WARMUP_CONCURRENCY = 2;

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
  getPreviewScope: () => WorkspaceSidebarPreviewScope;
  getQueueVideoRefreshTick: () => number;
  getVideoAcknowledgeSync: () => VideoAcknowledgeSync | null;
  getPreviewSessionKey: () => string | undefined;
  onChannelPreviewSnapshotLoaded?: WorkspaceSidebarPreviewProps["onChannelPreviewSnapshotLoaded"];
  onChannelUpdated?: (channel: Channel) => void | Promise<void>;
  onChannelSyncDateSaved?: (channelId: string) => void | Promise<void>;
};

export function shouldSkipAutoExpandForCollapsedSelection(params: {
  targetChannelId: string;
  selectedChannelId: string | null;
  selectedVideoId: string | null;
  userCollapsedSelectionKey: string | null;
}) {
  if (
    !params.selectedVideoId ||
    params.selectedChannelId !== params.targetChannelId
  ) {
    return false;
  }

  return (
    params.userCollapsedSelectionKey ===
    `${params.targetChannelId}:${params.selectedVideoId}`
  );
}

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
  let userCollapsedSelectionKey = $state<string | null>(null);
  let syncDatePickerChannelId = $state<string | null>(null);
  let previewWarmupSeq = 0;
  let userChangedExpandedState = false;
  const manuallyCollapsedChannelIds = new SvelteSet<string>();

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
    return `${options.getVideoTypeFilter()}:${options.getAcknowledgedFilter()}:default`;
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

  function setPreviewChannelExpanded(channelId: string, expanded: boolean) {
    ensureChannelVideoCollection(channelId);
    setSidebarPreviewCollectionExpanded(
      channelVideoCollections,
      channelId,
      expanded,
    );
  }

  function demoteOtherPagedCollections(exceptChannelId: string) {
    demoteOtherPagedSidebarPreviewCollections(
      channelVideoCollections,
      exceptChannelId,
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
    return {
      videos: resolveVisibleCollectionVideos(collection),
      topSpacer: 0,
      bottomSpacer: 0,
      virtualized: false,
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

    const videoTypeFilter = options.getVideoTypeFilter();
    const acknowledgedFilter = options.getAcknowledgedFilter();
    const acknowledged = resolveAcknowledgedParam(acknowledgedFilter);
    const pageLimit =
      mode === "paged" ? EXPANDED_PAGE_SIZE : PREVIEW_FETCH_LIMIT;

    try {
      const current = channelVideoCollections[channel.id];
      if (!current || current.requestKey !== requestKey) {
        return;
      }

      if (!append) {
        const snapshot = await getChannelSnapshot(channel.id, {
          limit: pageLimit,
          offset: 0,
          videoType: videoTypeFilter,
          acknowledged,
          bypassCache: force,
        });

        if (current.requestKey !== requestKey) {
          return;
        }

        if (mode === "paged") {
          demoteOtherPagedCollections(channel.id);
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
        if (
          mode === "preview" &&
          current.videos.length > 0 &&
          !manuallyCollapsedChannelIds.has(channel.id)
        ) {
          current.expanded = true;
        }
        if (mode !== "paged") {
          current.scrollTop = 0;
        }
        void options.onChannelPreviewSnapshotLoaded?.(channel.id, snapshot, {
          videoTypeFilter,
          acknowledgedFilter,
        });
        return;
      }

      const page = await listVideos(
        channel.id,
        pageLimit,
        requestOffset,
        videoTypeFilter,
        acknowledged,
        false,
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
    userChangedExpandedState = true;

    if (state.expanded) {
      manuallyCollapsedChannelIds.add(channel.id);
      const selectedChannelId = options.getSelectedChannelId();
      const selectedVideoId = options.getSelectedVideoId();
      if (selectedChannelId === channel.id && selectedVideoId) {
        userCollapsedSelectionKey = `${channel.id}:${selectedVideoId}`;
      }
      if (state.loadedMode === "paged") {
        state.loadedMode = "preview";
      }
      state.scrollTop = 0;
      setPreviewChannelExpanded(channel.id, false);
      return;
    }

    userCollapsedSelectionKey = null;
    manuallyCollapsedChannelIds.delete(channel.id);
    setPreviewChannelExpanded(channel.id, true);
    const nextState = ensureChannelVideoCollection(channel.id);
    nextState.scrollTop = 0;

    const filterKey = getChannelVideoCollectionFilterKey();
    if (!supportsMode(nextState, filterKey, "preview")) {
      await loadChannelVideoCollection(channel, "preview");
    }
  }

  async function promoteChannelVideoCollectionToPaged(channel: Channel) {
    const state = ensureChannelVideoCollection(channel.id);
    userChangedExpandedState = true;
    userCollapsedSelectionKey = null;
    manuallyCollapsedChannelIds.delete(channel.id);
    demoteOtherPagedCollections(channel.id);
    setPreviewChannelExpanded(channel.id, true);
    state.scrollTop = 0;

    const filterKey = getChannelVideoCollectionFilterKey();
    if (state.filterKey === filterKey && state.loadedMode === "preview") {
      state.loadedMode = "paged";
      if (state.videos.length < EXPANDED_PAGE_SIZE && state.hasMore) {
        await loadChannelVideoCollection(channel, "paged", { append: true });
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

  async function warmChannelVideoPreviews(
    channels: Channel[],
    filterKey: string,
    seq: number,
  ) {
    const candidates = channels.filter((channel) => {
      if (channel.id === OTHERS_CHANNEL_ID) {
        return false;
      }
      const state = untrack(() => channelVideoCollections[channel.id]);
      return !state || !supportsMode(state, filterKey, "preview");
    });
    let nextIndex = 0;

    async function worker() {
      for (;;) {
        if (seq !== previewWarmupSeq) {
          return;
        }
        const channel = candidates[nextIndex++];
        if (!channel) {
          return;
        }
        await loadChannelVideoCollection(channel, "preview");
      }
    }

    const workers = Array.from({
      length: Math.min(PREVIEW_WARMUP_CONCURRENCY, candidates.length),
    }).map(() => worker());
    await Promise.all(workers);
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

    if (
      userChangedExpandedState &&
      Object.keys(channelVideoCollections).length > 0
    ) {
      hydratedPreviewSessionKey = previewSessionKey;
      return;
    }

    const expandedChannelIds = Object.entries(channelVideoCollections)
      .filter(([, collection]) => collection.expanded)
      .map(([channelId]) => channelId);
    channelVideoCollections = restoreChannelVideoCollections(
      getSidebarPreviewSession(previewSessionKey) ?? {},
    );
    for (const channelId of expandedChannelIds) {
      if (options.getChannels().some((channel) => channel.id === channelId)) {
        setPreviewChannelExpanded(channelId, true);
      }
    }
    const preferredExpandedChannelId =
      resolvePreferredExpandedSidebarPreviewCollectionId(
        channelVideoCollections,
        options.getSelectedChannelId(),
      );
    if (preferredExpandedChannelId) {
      setPreviewChannelExpanded(preferredExpandedChannelId, true);
    }
    hydratedPreviewSessionKey = previewSessionKey;
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

    const seq = ++previewWarmupSeq;
    const filterKey = getChannelVideoCollectionFilterKey();
    const channels = options.getFilteredChannels();
    void warmChannelVideoPreviews(channels, filterKey, seq);

    return () => {
      if (previewWarmupSeq === seq) {
        previewWarmupSeq += 1;
      }
    };
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
      if (state.loadedMode !== "paged") continue;
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

    const selectedVideoId = options.getSelectedVideoId();
    if (
      shouldSkipAutoExpandForCollapsedSelection({
        targetChannelId: targetChannel.id,
        selectedChannelId: options.getSelectedChannelId(),
        selectedVideoId,
        userCollapsedSelectionKey,
      })
    ) {
      return;
    }

    setPreviewChannelExpanded(targetChannel.id, true);
    const nextState = ensureChannelVideoCollection(targetChannel.id);
    lastAutoExpandedChannelId = targetChannel.id;

    const preferredMode =
      selectedVideoId && options.getSelectedChannelId() === targetChannel.id
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

    const selectionKey = `${selectedChannel.id}:${selectedVideoId}`;
    if (userCollapsedSelectionKey !== selectionKey) {
      userCollapsedSelectionKey = null;
    }

    const state = ensureChannelVideoCollection(selectedChannel.id);
    if (!state.expanded && userCollapsedSelectionKey !== selectionKey) {
      untrack(() => setPreviewChannelExpanded(selectedChannel.id, true));
    }

    if (userCollapsedSelectionKey === selectionKey) {
      return;
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
        demoteOtherPagedCollections(selectedChannel.id);
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
    loadNextChannelVideoPage,
    resolveDisplayedSyncDepthIso,
    resolveRenderedCollectionVideos,
    saveChannelSyncDate,
    promoteChannelVideoCollectionToPaged,
    toggleChannelVideoCollection,
    toggleSyncDatePicker,
  };
}
