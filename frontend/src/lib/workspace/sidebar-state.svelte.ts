/**
 * Shared channel + video list state for WorkspaceSidebar usage across routes.
 *
 * Encapsulates all reactive state and operations that are common to the main
 * workspace route and the download-queue route: channel listing, selection,
 * add/delete/reorder, video listing, paginating, and filtering. Each route
 * supplies route-specific hooks for video selection and channel lifecycle events.
 *
 * Returns fully-typed `channelState`, `channelActions`, `videoState`, and
 * `videoActions` objects that can be passed directly to `WorkspaceSidebar`.
 */

import {
  addChannel,
  deleteChannel,
  getChannelSnapshot,
  listChannelsWhenAvailable,
  listVideos,
  refreshChannel,
} from "$lib/api";
import {
  applySavedChannelOrder,
  resolveInitialChannelSelection,
} from "$lib/channel-workspace";
import {
  buildChannelViewCacheKey,
  cloneDate,
  cloneSyncDepthState,
  cloneVideos,
  createChannelViewCache,
  type ChannelSyncDepthState,
} from "$lib/channel-view-cache";
import type {
  WorkspaceSidebarVideoActions,
  WorkspaceSidebarVideoState,
  WorkspaceSidebarChannelActions,
  WorkspaceSidebarChannelState,
  WorkspaceVideoSelectContext,
} from "$lib/workspace/component-props";
import {
  buildOptimisticChannel,
  removeChannelFromCollection,
  removeChannelId,
  replaceOptimisticChannel,
  replaceOptimisticChannelId,
} from "$lib/workspace/channel-actions";
import { channelOrderFromList } from "$lib/workspace/channels";
import {
  filterVideosByAcknowledged,
  filterVideosByType,
  loadChannelSnapshotWithRefresh,
  resolveNextChannelSelection,
} from "$lib/workspace/route-helpers";
import type { AcknowledgedFilter, ChannelSortMode } from "$lib/workspace/types";
import { resolveAcknowledgedParam } from "$lib/workspace/types";
import type {
  Channel,
  ChannelSnapshot,
  Video,
  VideoTypeFilter,
} from "$lib/types";
import { putCachedChannels } from "$lib/workspace-cache";

const CHANNEL_REFRESH_TTL_MS = 5 * 60 * 1000;

export type SidebarStateOptions = {
  /**
   * Max video count per page for channel snapshots. Defaults to 20.
   */
  limit?: number;

  /**
   * Called when a video is clicked in the sidebar. Route provides the handler
   * (workspace: select in-place; queue: navigate to workspace).
   */
  onSelectVideo: (
    videoId: string,
    context?: WorkspaceVideoSelectContext,
  ) => Promise<void> | void;

  initialChannelId?: string | null;
  initialVideoId?: string | null;
  initialVideoTypeFilter?: VideoTypeFilter;
  initialAcknowledgedFilter?: AcknowledgedFilter;

  /**
   * Optional: called when a channel is selected. Route can use this to reset
   * content state, clear pending video selection, etc.
   */
  onChannelSelected?: (channelId: string) => void;

  /**
   * Optional: called when all channels are deselected (no selection).
   */
  onChannelDeselected?: () => void;

  /**
   * Optional: called before video list is reset so route can reset
   * workspace-specific video state (historyExhausted, backfillingHistory, etc.).
   */
  onVideoListReset?: () => void;

  /**
   * Optional: called after videos are loaded/reset so route can perform
   * post-load actions (e.g. hydrate selected video in workspace).
   */
  onVideosLoaded?: (options: {
    reset: boolean;
    videos: Video[];
  }) => Promise<void> | void;

  /**
   * Optional: hook to report errors to the route's error message state.
   */
  onError?: (message: string) => void;

  /**
   * Optional: called after a channel is added successfully.
   * Route can redirect or select the new channel without this composable
   * needing to know about routing.
   */
  onChannelAdded?: (channel: Channel) => Promise<void> | void;

  /**
   * Optional: called after a channel is successfully deleted (the composable
   * handles the state removal and next-channel selection internally).
   */
  onChannelDeleted?: (channelId: string) => void;

  /**
   * Optional: hook to allow the route to override `onAddChannel` result cache logic.
   * Defaults to `putCachedChannels`.
   */
  cacheChannels?: (channels: Channel[]) => void;

  onPersistWorkspaceState?: (state: {
    selectedChannelId: string | null;
    channelOrder: string[];
    channelSortMode: ChannelSortMode;
  }) => void;
  onPersistViewUrl?: (state: { selectedChannelId: string | null }) => void;
  onLoadInitial?: (options?: { silent?: boolean }) => Promise<void>;
  onLoadChannelSnapshot?: (
    channelId: string,
    snapshotOptions: any,
    silent: boolean,
  ) => Promise<ChannelSnapshot>;
  onRefreshChannel?: (channelId: string) => Promise<any>;
  onListVideos?: (
    channelId: string,
    limit: number,
    offset: number,
    videoTypeFilter: VideoTypeFilter,
    acknowledgedFilter: any,
    includeOptimistic: boolean,
  ) => Promise<Video[]>;
  onVideoTypeFilterChange?: (filter: VideoTypeFilter) => void;
  onAcknowledgedFilterChange?: (ack: boolean | undefined) => void;
};

type CachedVideoState = {
  videos: Video[];
  offset: number;
  hasMore: boolean;
  lastSyncedAt: Date | null;
  syncDepth: ChannelSyncDepthState | null;
};

export type { WorkspaceSidebarVideoState } from "$lib/workspace/component-props";

export type SidebarStateResult = {
  // Reactive state properties (getters/setters for runes)
  channels: Channel[];
  channelOrder: string[];
  channelSortMode: ChannelSortMode;
  selectedChannelId: string | null;
  selectedVideoId: string | null;
  videos: Video[];
  loadingChannels: boolean;
  loadingVideos: boolean;
  refreshingChannel: boolean;
  addingChannel: boolean;
  hasMore: boolean;
  historyExhausted: boolean;
  backfillingHistory: boolean;
  syncDepth: ChannelSyncDepthState | null;
  offset: number;
  channelIdToDelete: string | null;
  showDeleteConfirmation: boolean;

  // Read-only logic-derived state
  readonly selectedChannel: Channel | null;
  readonly videoTypeFilter: VideoTypeFilter;
  readonly acknowledgedFilter: AcknowledgedFilter;
  readonly limit: number;
  readonly sidebarCollapsed: boolean;
  readonly sidebarWidth: number | undefined;

  // Explicit mutator methods
  setChannels: (v: Channel[]) => void;
  setChannelOrder: (v: string[]) => void;
  setSelectedChannelId: (v: string | null) => void;
  setSelectedVideoId: (v: string | null) => void;
  setVideos: (v: Video[]) => void;
  setSyncDepth: (v: ChannelSyncDepthState | null) => void;
  setHasMore: (v: boolean) => void;
  setOffset: (v: number) => void;
  setVideoTypeFilter: (v: VideoTypeFilter) => void;
  setAcknowledgedFilter: (v: AcknowledgedFilter) => void;
  setChannelSortMode: (v: ChannelSortMode) => void;
  setLoadingVideos: (v: boolean) => void;
  setLoadingChannels: (v: boolean) => void;
  setRefreshingChannel: (v: boolean) => void;
  setAddingChannel: (v: boolean) => void;
  setChannelIdToDelete: (v: string | null) => void;
  setShowDeleteConfirmation: (v: boolean) => void;

  // Operations
  syncChannelOrderFromList: () => void;
  loadInitial: (options?: { silent?: boolean }) => Promise<void>;
  selectChannel: (
    channelId: string,
    videoId?: string | null,
    fromUserInteraction?: boolean,
  ) => Promise<void>;
  refreshAndLoadVideos: (channelId: string, silent?: boolean) => Promise<void>;
  loadVideos: (reset?: boolean, silent?: boolean) => Promise<void>;
  handleAddChannel: (input: string) => Promise<boolean>;
  handleDeleteChannel: (
    channelId: string,
    isOperator: boolean,
    onAccessRequired: () => void,
  ) => Promise<void>;
  confirmDeleteChannel: (
    channelId: string,
    isOperator: boolean,
  ) => Promise<void>;
  reorderChannels: (nextOrder: string[]) => void;
  setVideoTypeFilterAndReload: (value: VideoTypeFilter) => Promise<void>;
  setAcknowledgedFilterAndReload: (value: AcknowledgedFilter) => Promise<void>;
  /** After local video list mutations (e.g. read toggle), bump so stale snapshots are not applied. */
  bumpVideoListMutationEpoch: () => void;
  getVideoListMutationEpoch: () => number;
  updateChannel: (updated: Channel) => void;
  toggleSidebar: () => void;
  addOptimisticChannel: (channel: Channel) => void;
  replaceOptimisticChannelId: (tempId: string, realId: string) => void;
  removeChannel: (channelId: string) => void;
  isCurrentSelection: (
    channelId: string | null,
    videoId: string | null,
  ) => boolean;

  // WorkspaceSidebar-ready prop objects
  readonly channelState: WorkspaceSidebarChannelState;
  readonly channelActions: WorkspaceSidebarChannelActions;
  readonly videoState: WorkspaceSidebarVideoState;
  readonly videoActions: WorkspaceSidebarVideoActions;
};

/**
 * Creates shared reactive sidebar channel+video state using Svelte 5 runes.
 *
 * Must be called synchronously during component initialization (i.e. at the
 * top level of a `<script>` block or in a function called from there), not
 * inside an async callback or `onMount`.
 */
export function createSidebarState(
  options_root: SidebarStateOptions,
): SidebarStateResult {
  const limit = options_root.limit ?? 20;
  const channelLastRefreshedAt = new Map<string, number>();

  const videoStateCache = createChannelViewCache<CachedVideoState>((state) => ({
    ...state,
    videos: cloneVideos(state.videos),
    lastSyncedAt: cloneDate(state.lastSyncedAt),
    syncDepth: cloneSyncDepthState(state.syncDepth),
  }));

  async function loadInitial(options?: { silent?: boolean }) {
    if (options_root.onLoadInitial) {
      return options_root.onLoadInitial(options);
    }
    const silent = options?.silent ?? false;
    if (!silent) {
      loadingChannels = true;
    }

    try {
      const channelList = await listChannelsWhenAvailable({
        retryDelayMs: 500,
      });
      channels = applySavedChannelOrder(channelList, channelOrder);
      syncChannelOrderFromList();
      void putCachedChannels(channels);

      const initialChannelId = resolveInitialChannelSelection(
        channels,
        selectedChannelId,
        channelOrder[0], // Pass a single string (the first channel ID) as the preference
      );

      if (!initialChannelId) {
        selectedChannelId = null;
        videos = [];
        syncDepth = null;
      } else {
        selectedChannelId = initialChannelId;
        await refreshAndLoadVideos(initialChannelId, silent);
      }
    } catch (error) {
      options_root.onError?.((error as Error).message);
    } finally {
      if (!silent) {
        loadingChannels = false;
      }
    }
  }

  // --- Core reactive state ---

  let channels = $state<Channel[]>([]);
  let channelOrder = $state<string[]>([]);
  let channelSortMode = $state<ChannelSortMode>("custom");
  let selectedChannelId = $state<string | null>(
    options_root.initialChannelId ?? null,
  );
  let selectedVideoId = $state<string | null>(
    options_root.initialVideoId ?? null,
  );
  let videos = $state<Video[]>([]);
  let offset = $state(0);
  let hasMore = $state(true);
  let historyExhausted = $state(false);
  let backfillingHistory = $state(false);
  let syncDepth = $state<ChannelSyncDepthState | null>(null);
  let loadingChannels = $state(false);
  let loadingVideos = $state(false);
  let refreshingChannel = $state(false);
  let addingChannel = $state(false);
  let videoTypeFilter = $state<VideoTypeFilter>(
    options_root.initialVideoTypeFilter ?? "all",
  );
  let acknowledgedFilter = $state<AcknowledgedFilter>(
    options_root.initialAcknowledgedFilter ?? "all",
  );
  let channelIdToDelete = $state<string | null>(null);
  let showDeleteConfirmation = $state(false);

  let sidebarCollapsed = $state(false);
  let sidebarWidth = $state<number | undefined>(undefined);

  /** Incremented when the client mutates the video list so in-flight snapshots are ignored. */
  let videoListMutationEpoch = 0;

  function bumpVideoListMutationEpoch() {
    videoListMutationEpoch += 1;
  }

  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
  }

  const selectedChannel = $derived(
    channels.find((ch) => ch.id === selectedChannelId) ?? null,
  );

  // --- Cache key helpers ---

  function getVideoStateKey(channelId: string) {
    return buildChannelViewCacheKey(
      channelId,
      videoTypeFilter,
      acknowledgedFilter,
    );
  }

  // --- Persist current state into the per-channel cache whenever it changes ---

  $effect(() => {
    if (!selectedChannelId) return;
    videoStateCache.set(getVideoStateKey(selectedChannelId), {
      videos: cloneVideos(videos),
      offset,
      hasMore,
      lastSyncedAt: null,
      syncDepth: cloneSyncDepthState(syncDepth),
    });
  });

  // --- Mutators ---

  function syncChannelOrderFromList() {
    channelOrder = channelOrderFromList(channels);
  }

  function setChannels(next: Channel[]) {
    channels = next;
  }
  function setChannelOrder(order: string[]) {
    channelOrder = order;
  }
  function setSelectedChannelId(id: string | null) {
    selectedChannelId = id;
  }
  function setSelectedVideoId(id: string | null) {
    selectedVideoId = id;
  }
  function setVideos(next: Video[]) {
    videos = next;
  }
  function setSyncDepth(depth: ChannelSyncDepthState | null) {
    syncDepth = depth;
  }
  function setHasMore(value: boolean) {
    hasMore = value;
  }
  function setOffset(value: number) {
    offset = value;
  }
  function setVideoTypeFilter(v: VideoTypeFilter) {
    videoTypeFilter = v;
    options_root.onVideoTypeFilterChange?.(v);
  }
  function setAcknowledgedFilter(v: AcknowledgedFilter) {
    acknowledgedFilter = v;
    options_root.onAcknowledgedFilterChange?.(
      v === "ack" ? true : v === "unack" ? false : undefined,
    );
  }
  function setChannelSortMode(mode: ChannelSortMode) {
    channelSortMode = mode;
  }
  function setLoadingVideos(v: boolean) {
    loadingVideos = v;
  }
  function setLoadingChannels(v: boolean) {
    loadingChannels = v;
  }
  function setRefreshingChannel(v: boolean) {
    refreshingChannel = v;
  }
  function setAddingChannel(v: boolean) {
    addingChannel = v;
  }
  function setChannelIdToDelete(v: string | null) {
    channelIdToDelete = v;
  }
  function setShowDeleteConfirmation(v: boolean) {
    showDeleteConfirmation = v;
  }

  function updateChannel(updated: Channel) {
    channels = channels.map((ch) => (ch.id === updated.id ? updated : ch));
  }

  function reorderChannels(nextOrder: string[]) {
    channels = applySavedChannelOrder(channels, nextOrder);
    channelOrder = nextOrder;
  }

  function addOptimisticChannel(channel: Channel) {
    channels = [channel, ...channels];
  }

  function replaceOptimisticChannelId(tempId: string, realId: string) {
    channels = channels.map((c) =>
      c.id === tempId ? { ...c, id: realId } : c,
    );
    if (selectedChannelId === tempId) {
      selectedChannelId = realId;
    }
  }

  function removeChannel(id: string) {
    channels = channels.filter((c) => c.id !== id);
  }

  function isCurrentSelection(channelId: string | null, vidId: string | null) {
    return selectedChannelId === channelId && selectedVideoId === vidId;
  }

  // --- Snapshot / video loading ---

  async function applyChannelSnapshot(
    channelId: string,
    snapshot: ChannelSnapshot,
    silent = false,
  ) {
    if (!silent) {
      loadingVideos = true;
    }
    try {
      if (selectedChannelId !== channelId) return;
      syncDepth = snapshot.sync_depth;
      videos = snapshot.videos;
      offset = snapshot.videos.length;
      hasMore = snapshot.videos.length === limit;

      if (options_root.onVideosLoaded) {
        await options_root.onVideosLoaded({
          reset: true,
          videos: snapshot.videos,
        });
      }
    } finally {
      if (!silent) {
        loadingVideos = false;
      }
    }
  }

  async function refreshAndLoadVideos(channelId: string, silent = false) {
    const isAck = resolveAcknowledgedParam(acknowledgedFilter);
    await loadChannelSnapshotWithRefresh({
      channelId,
      refreshedAtByChannel: channelLastRefreshedAt,
      ttlMs: CHANNEL_REFRESH_TTL_MS,
      initialSilent: silent,
      getMutationEpoch: () => videoListMutationEpoch,
      loadSnapshot: () =>
        getChannelSnapshot(channelId, {
          limit,
          offset: 0,
          videoType: videoTypeFilter,
          acknowledged: isAck,
        }),
      applySnapshot: (snapshot, snapshotSilent = false) =>
        applyChannelSnapshot(channelId, snapshot, snapshotSilent),
      refreshChannel: () => refreshChannel(channelId),
      shouldReloadAfterRefresh: () => selectedChannelId === channelId,
      onRefreshingChange: (r) => {
        refreshingChannel = r;
      },
      onError: (message) => {
        options_root.onError?.(message);
      },
    });
  }

  async function loadVideos(reset = false, silent = false) {
    if (!selectedChannelId) return;
    if (loadingVideos && !silent) return;

    if (!silent) {
      loadingVideos = true;
    }

    try {
      const isAck = resolveAcknowledgedParam(acknowledgedFilter);
      const list = await listVideos(
        selectedChannelId,
        limit,
        reset ? 0 : offset,
        videoTypeFilter,
        isAck,
      );
      videos = reset ? list : [...videos, ...list];
      offset = (reset ? 0 : offset) + list.length;
      hasMore = list.length === limit;

      if (options_root.onVideosLoaded) {
        await options_root.onVideosLoaded({ reset, videos });
      }
    } catch (error) {
      options_root.onError?.((error as Error).message);
    } finally {
      if (!silent) {
        loadingVideos = false;
      }
    }
  }

  // --- Channel selection ---

  async function selectChannel(
    channelId: string,
    videoId: string | null = null,
    fromUserInteraction = false,
  ) {
    const cacheKey = getVideoStateKey(channelId);
    const cached = videoStateCache.get(cacheKey);
    const hasCached = !!cached && cached.videos.length > 0;

    selectedChannelId = channelId;
    if (videoId) {
      selectedVideoId = videoId;
    }
    options_root.onChannelSelected?.(channelId);

    if (hasCached && cached) {
      videos = cloneVideos(cached.videos);
      offset = cached.offset;
      hasMore = cached.hasMore;
      syncDepth = cloneSyncDepthState(cached.syncDepth);
      loadingVideos = false;
      void refreshAndLoadVideos(channelId, true);
      return;
    }

    videos = [];
    offset = 0;
    hasMore = true;
    syncDepth = null;
    options_root.onVideoListReset?.();
    await refreshAndLoadVideos(channelId, !fromUserInteraction);
  }

  // --- Filter operations ---

  async function setVideoTypeFilterAndReload(nextValue: VideoTypeFilter) {
    if (videoTypeFilter === nextValue) return;
    videoTypeFilter = nextValue;
    videos = filterVideosByType(videos, nextValue);
    await loadVideos(true, true);
  }

  async function setAcknowledgedFilterAndReload(nextValue: AcknowledgedFilter) {
    if (acknowledgedFilter === nextValue) return;
    acknowledgedFilter = nextValue;
    videos = filterVideosByAcknowledged(videos, nextValue);
    await loadVideos(true, true);
  }

  // --- Channel CRUD ---

  async function handleAddChannel(input: string): Promise<boolean> {
    if (!input.trim()) return false;

    addingChannel = true;
    options_root.onError?.("");

    const previousChannels = [...channels];
    const previousSelectedId = selectedChannelId;

    const { optimisticChannel, tempId, trimmedInput } =
      buildOptimisticChannel(input);
    channels = [optimisticChannel, ...channels];
    channelOrder = [tempId, ...channelOrder];

    try {
      const channel = await addChannel(trimmedInput);
      channels = replaceOptimisticChannel(channels, tempId, channel);
      replaceOptimisticChannelId(tempId, channel.id);

      const cacheChannels =
        options_root.cacheChannels ??
        ((chs: Channel[]) => void putCachedChannels(chs));
      cacheChannels(channels);

      selectedChannelId = channel.id;
      if (options_root.onChannelAdded) {
        await options_root.onChannelAdded(channel);
      }
      return true;
    } catch (error) {
      channels = previousChannels;
      selectedChannelId = previousSelectedId;
      syncChannelOrderFromList();
      options_root.onError?.((error as Error).message);
      return false;
    } finally {
      addingChannel = false;
    }
  }

  async function handleDeleteChannel(
    channelId: string,
    isOperator: boolean,
    onAccessRequired: () => void,
  ) {
    if (!isOperator) {
      onAccessRequired();
      return;
    }
    channelIdToDelete = channelId;
    showDeleteConfirmation = true;
  }

  async function confirmDeleteChannel(channelId: string, isOperator: boolean) {
    if (!isOperator) return;

    const previousChannels = [...channels];
    channels = removeChannelFromCollection(channels, channelId);
    channelOrder = removeChannelId(channelOrder, channelId);

    if (selectedChannelId === channelId) {
      const nextChannelId = resolveNextChannelSelection(channels, channelId);
      if (nextChannelId) {
        await selectChannel(nextChannelId);
      } else {
        selectedChannelId = null;
        videos = [];
        syncDepth = null;
        options_root.onChannelDeselected?.();
      }
    }

    try {
      await deleteChannel(channelId);
      options_root.onChannelDeleted?.(channelId);
    } catch (error) {
      channels = previousChannels;
      syncChannelOrderFromList();
      options_root.onError?.((error as Error).message);
    } finally {
      channelIdToDelete = null;
      showDeleteConfirmation = false;
    }
  }

  // --- WorkspaceSidebar prop objects ---

  const channelState = $derived<WorkspaceSidebarChannelState>({
    channels,
    selectedChannelId,
    loadingChannels,
    addingChannel,
    channelSortMode,
    canDeleteChannels: true, // Will be overridden if needed
  });

  const channelActions: WorkspaceSidebarChannelActions = {
    onChannelSortModeChange: (next) => {
      channelSortMode = next;
    },
    onAddChannel: handleAddChannel,
    onSelectChannel: (channelId) => {
      if (channelId === selectedChannelId) {
        selectedChannelId = null;
        syncDepth = null;
        options_root.onChannelDeselected?.();
        return;
      }
      void selectChannel(channelId, null, true);
    },
    onDeleteChannel: (id: string) => {
      channelIdToDelete = id;
      showDeleteConfirmation = true;
    },
    onReorderChannels: reorderChannels,
    onChannelUpdated: updateChannel,
    onOpenChannelOverview: (id: string) => {}, // To be provided by route if needed
    onDeleteAccessRequired: () => {}, // To be provided by route if needed
  };

  const videoState = $derived<WorkspaceSidebarVideoState>({
    videos,
    selectedVideoId,
    selectedChannel,
    loadingVideos,
    refreshingChannel,
    hasMore,
    historyExhausted: false,
    backfillingHistory: false,
    videoTypeFilter,
    acknowledgedFilter,
    syncDepth,
    offset,
    allowLoadedVideoSyncDepthOverride: false,
  });

  const videoActions: WorkspaceSidebarVideoActions = {
    onSelectVideo: (videoId: string, context?: WorkspaceVideoSelectContext) => {
      void options_root.onSelectVideo(videoId, context);
    },
    onLoadMoreVideos: () => loadVideos(false),
    onVideoTypeFilterChange: setVideoTypeFilterAndReload,
    onAcknowledgedFilterChange: setAcknowledgedFilterAndReload,
    onSelectChannelVideo: (channelId: string, videoId: string) => {
      void (async () => {
        await selectChannel(channelId, videoId, true);
        void options_root.onSelectVideo(videoId, { forceReload: true });
      })();
    },
  };

  return {
    // Read-only logic-derived state
    get limit() {
      return limit;
    },
    get videoTypeFilter() {
      return videoTypeFilter;
    },
    get acknowledgedFilter() {
      return acknowledgedFilter;
    },
    get sidebarCollapsed() {
      return sidebarCollapsed;
    },
    get sidebarWidth() {
      return sidebarWidth;
    },

    // Reactive state properties (getters/setters for runes)
    get channels() {
      return channels;
    },
    set channels(v) {
      channels = v;
    },
    get channelOrder() {
      return channelOrder;
    },
    set channelOrder(v) {
      channelOrder = v;
    },
    get channelSortMode() {
      return channelSortMode;
    },
    set channelSortMode(v) {
      channelSortMode = v;
    },
    get selectedChannelId() {
      return selectedChannelId;
    },
    set selectedChannelId(v) {
      selectedChannelId = v;
    },
    get selectedVideoId() {
      return selectedVideoId;
    },
    set selectedVideoId(v) {
      selectedVideoId = v;
    },
    get selectedChannel() {
      return selectedChannel;
    },
    get videos() {
      return videos;
    },
    set videos(v) {
      videos = v;
    },
    get loadingChannels() {
      return loadingChannels;
    },
    set loadingChannels(v) {
      loadingChannels = v;
    },
    get loadingVideos() {
      return loadingVideos;
    },
    set loadingVideos(v) {
      loadingVideos = v;
    },
    get refreshingChannel() {
      return refreshingChannel;
    },
    set refreshingChannel(v) {
      refreshingChannel = v;
    },
    get addingChannel() {
      return addingChannel;
    },
    set addingChannel(v) {
      addingChannel = v;
    },
    get hasMore() {
      return hasMore;
    },
    set hasMore(v) {
      hasMore = v;
    },
    get historyExhausted() {
      return historyExhausted;
    },
    set historyExhausted(v) {
      historyExhausted = v;
    },
    get backfillingHistory() {
      return backfillingHistory;
    },
    set backfillingHistory(v) {
      backfillingHistory = v;
    },
    get syncDepth() {
      return syncDepth;
    },
    set syncDepth(v) {
      syncDepth = v;
    },
    get offset() {
      return offset;
    },
    set offset(v) {
      offset = v;
    },
    get channelIdToDelete() {
      return channelIdToDelete;
    },
    get showDeleteConfirmation() {
      return showDeleteConfirmation;
    },

    // Explicit mutator methods
    setChannels,
    setChannelOrder,
    setSelectedChannelId,
    setSelectedVideoId,
    setVideos,
    setSyncDepth,
    setHasMore,
    setOffset,
    setVideoTypeFilter,
    setAcknowledgedFilter,
    setChannelSortMode,
    setLoadingVideos,
    setLoadingChannels,
    setRefreshingChannel,
    setAddingChannel,
    setChannelIdToDelete,
    setShowDeleteConfirmation,

    // Operations
    loadInitial,
    selectChannel,
    refreshAndLoadVideos,
    loadVideos,
    handleAddChannel,
    handleDeleteChannel,
    confirmDeleteChannel,
    syncChannelOrderFromList,
    reorderChannels,
    setVideoTypeFilterAndReload,
    setAcknowledgedFilterAndReload,
    bumpVideoListMutationEpoch,
    getVideoListMutationEpoch: () => videoListMutationEpoch,
    updateChannel,
    toggleSidebar,
    addOptimisticChannel,
    replaceOptimisticChannelId,
    removeChannel,
    isCurrentSelection,

    // WorkspaceSidebar-ready prop objects
    get channelState() {
      return channelState;
    },
    get channelActions() {
      return channelActions;
    },
    get videoState() {
      return videoState;
    },
    get videoActions() {
      return videoActions;
    },
  };
}
