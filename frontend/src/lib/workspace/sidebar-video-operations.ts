import type { SvelteMap } from "svelte/reactivity";
import {
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
  cloneSyncDepthState,
  cloneVideos,
  type ChannelSyncDepthState,
} from "$lib/channel-view-cache";
import {
  applyAcknowledgedFilterChange,
  applyVideoTypeFilterChange,
  clearSidebarVideoFilters,
  dedupeVideosById,
  loadChannelSnapshotWithRefresh,
  resolveSnapshotPageState,
} from "$lib/workspace/route-helpers";
import { putCachedChannels } from "$lib/workspace-cache";
import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
import { resolveAcknowledgedParam, type AcknowledgedFilter } from "./types";
import type {
  CachedVideoState,
  SidebarStateOptions,
} from "./sidebar-state.svelte";
import type {
  Channel,
  ChannelSnapshot,
  Video,
  VideoTypeFilter,
} from "$lib/types";
import { OTHERS_CHANNEL_ID } from "$lib/types";

const CHANNEL_REFRESH_TTL_MS = 5 * 60 * 1000;

type SidebarVideoStateCache = {
  get(key: string): CachedVideoState | null;
  delete(key: string): void;
};

type SidebarVideoOperationsContext = {
  options: SidebarStateOptions;
  limit: number;
  channelLastRefreshedAt: SvelteMap<string, number>;
  videoStateCache: SidebarVideoStateCache;
  getVideoStateKey: (channelId: string) => string;
  getChannelOrder: () => string[];
  getSelectedChannelId: () => string | null;
  getSelectedVideoId: () => string | null;
  getVideos: () => Video[];
  getOffset: () => number;
  getVideoTypeFilter: () => VideoTypeFilter;
  getAcknowledgedFilter: () => AcknowledgedFilter;
  getLoadingVideos: () => boolean;
  getVideoListMutationEpoch: () => number;
  applyLoadedChannelsState: (
    channels: Channel[],
    channelOrder?: string[],
  ) => void;
  applySelectionState: (options: {
    selectedChannelId?: string | null;
    selectedVideoId?: string | null;
  }) => void;
  clearChannelSelectionState: () => void;
  resetVideoListState: (options?: {
    videos?: Video[];
    offset?: number;
    hasMore?: boolean;
    historyExhausted?: boolean;
    backfillingHistory?: boolean;
    syncDepth?: ChannelSyncDepthState | null;
    selectedVideoId?: string | null;
  }) => void;
  applyChannelSnapshotState: (snapshot: {
    videos: Video[];
    has_more: boolean;
    next_offset: number | null;
    sync_depth: ChannelSyncDepthState | null;
  }) => void;
  applyVideoPageState: (
    page: {
      videos: Video[];
      has_more: boolean;
      next_offset: number | null;
    },
    options?: { reset?: boolean },
  ) => void;
  setChannelLoadingState: (loading: boolean) => void;
  setVideoLoadingState: (loading: boolean) => void;
  setRefreshingChannelState: (refreshing: boolean) => void;
  setSyncDepthState: (depth: ChannelSyncDepthState | null) => void;
  setVideos: (videos: Video[]) => void;
  setVideoTypeFilter: (filter: VideoTypeFilter) => void;
  setAcknowledgedFilter: (filter: AcknowledgedFilter) => void;
};

function videosBelongToChannel(channelId: string, videos: Video[]) {
  if (channelId === OTHERS_CHANNEL_ID) {
    return true;
  }
  return videos.every((video) => video.channel_id === channelId);
}

function cacheChannels(options: SidebarStateOptions, channels: Channel[]) {
  const writeChannels =
    options.cacheChannels ??
    ((next: Channel[]) => void putCachedChannels(next));
  writeChannels(channels);
}

export function createSidebarVideoOperations(
  context: SidebarVideoOperationsContext,
) {
  async function loadInitial(options?: { silent?: boolean }) {
    if (context.options.onLoadInitial) {
      return context.options.onLoadInitial(options);
    }
    const silent = options?.silent ?? false;
    if (!silent) {
      context.setChannelLoadingState(true);
    }

    try {
      const channelList = await listChannelsWhenAvailable({
        retryDelayMs: 500,
      });
      const orderedChannels = applySavedChannelOrder(
        channelList,
        context.getChannelOrder(),
      );
      context.applyLoadedChannelsState(channelList, context.getChannelOrder());
      cacheChannels(context.options, orderedChannels);

      const initialChannelId = resolveInitialChannelSelection(
        orderedChannels,
        context.getSelectedChannelId(),
        context.getChannelOrder()[0],
      );

      if (!initialChannelId) {
        context.clearChannelSelectionState();
      } else {
        context.applySelectionState({ selectedChannelId: initialChannelId });
        // Set video loading eagerly so the skeleton shows during the network
        // fetch instead of flashing "No videos yet."
        if (!silent) {
          context.setVideoLoadingState(true);
        }
        await refreshAndLoadVideos(initialChannelId, silent);
      }
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        context.options.onError?.((error as Error).message);
      }
    } finally {
      if (!silent) {
        context.setChannelLoadingState(false);
        // applyChannelSnapshot resets this in the happy path; this covers
        // the case where loadSnapshot() throws before applySnapshot runs.
        context.setVideoLoadingState(false);
      }
    }
  }

  async function applyChannelSnapshot(
    channelId: string,
    snapshot: ChannelSnapshot,
    silent = false,
  ) {
    if (!silent) {
      context.setVideoLoadingState(true);
    }
    try {
      if (context.getSelectedChannelId() !== channelId) return;
      context.applyChannelSnapshotState(resolveSnapshotPageState(snapshot));

      if (context.options.onVideosLoaded) {
        await context.options.onVideosLoaded({
          reset: true,
          videos: snapshot.videos,
        });
      }
    } finally {
      if (!silent) {
        context.setVideoLoadingState(false);
      }
    }
  }

  async function refreshAndLoadVideos(channelId: string, silent = false) {
    const acknowledged = resolveAcknowledgedParam(
      context.getAcknowledgedFilter(),
    );
    const snapshotOptions = {
      limit: context.limit,
      offset: 0,
      videoType: context.getVideoTypeFilter(),
      acknowledged,
    };
    await loadChannelSnapshotWithRefresh({
      channelId,
      refreshedAtByChannel: context.channelLastRefreshedAt,
      ttlMs: CHANNEL_REFRESH_TTL_MS,
      enableRefresh: context.options.enableBackgroundRefresh ?? true,
      initialSilent: silent,
      getMutationEpoch: context.getVideoListMutationEpoch,
      loadSnapshot: () =>
        context.options.onLoadChannelSnapshot
          ? context.options.onLoadChannelSnapshot(
              channelId,
              snapshotOptions,
              silent,
            )
          : getChannelSnapshot(channelId, snapshotOptions),
      applySnapshot: (snapshot, snapshotSilent = false) =>
        applyChannelSnapshot(channelId, snapshot, snapshotSilent),
      refreshChannel: () =>
        context.options.onRefreshChannel
          ? context.options.onRefreshChannel(channelId)
          : refreshChannel(channelId),
      shouldReloadAfterRefresh: () =>
        context.getSelectedChannelId() === channelId,
      onRefreshingChange: context.setRefreshingChannelState,
      onError: (message) => {
        context.options.onError?.(message);
      },
    });
  }

  function mergeSelectedVideoHint(
    channelId: string,
    videoId: string,
    selectedVideoHint: Video | null,
  ) {
    if (!selectedVideoHint || selectedVideoHint.channel_id !== channelId) {
      return;
    }
    const currentVideos = context.getVideos();
    if (currentVideos.some((video) => video.id === videoId)) {
      return;
    }
    if (!videosBelongToChannel(channelId, currentVideos)) {
      return;
    }

    context.setVideos(dedupeVideosById([selectedVideoHint, ...currentVideos]));
  }

  function restoreCachedVideoState(
    cached: CachedVideoState,
    channelId: string,
    videoId: string | null = null,
    selectedVideoHint: Video | null = null,
  ) {
    const cachedVideos =
      videoId && selectedVideoHint && selectedVideoHint.channel_id === channelId
        ? dedupeVideosById([selectedVideoHint, ...cloneVideos(cached.videos)])
        : cloneVideos(cached.videos);

    context.resetVideoListState({
      videos: cachedVideos,
      offset: cached.offset,
      hasMore: cached.hasMore,
      syncDepth: cloneSyncDepthState(cached.syncDepth),
      ...(videoId ? { selectedVideoId: videoId } : {}),
    });
  }

  async function hydrateChannelSelectionInBackground(channelId: string) {
    try {
      await refreshAndLoadVideos(channelId, true);
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        context.options.onError?.((error as Error).message);
      }
    } finally {
      if (context.getSelectedChannelId() === channelId) {
        context.setVideoLoadingState(false);
      }
    }
  }

  function selectChannelVideoOptimistically(
    channelId: string,
    videoId: string,
    selectedVideoHint: Video | null = null,
  ) {
    const cacheKey = context.getVideoStateKey(channelId);
    const cached = context.videoStateCache.get(cacheKey);
    const hasCached =
      !!cached &&
      cached.videos.length > 0 &&
      videosBelongToChannel(channelId, cached.videos);

    if (cached && !hasCached) {
      context.videoStateCache.delete(cacheKey);
    }

    context.applySelectionState({
      selectedChannelId: channelId,
      selectedVideoId: videoId,
    });
    context.options.onChannelSelected?.(channelId);

    if (hasCached && cached) {
      restoreCachedVideoState(cached, channelId, videoId, selectedVideoHint);
      context.setVideoLoadingState(false);
      return hydrateChannelSelectionInBackground(channelId);
    }

    context.resetVideoListState({
      videos:
        selectedVideoHint && selectedVideoHint.channel_id === channelId
          ? [selectedVideoHint]
          : [],
      selectedVideoId: videoId,
    });
    context.options.onVideoListReset?.();
    context.setVideoLoadingState(true);
    return hydrateChannelSelectionInBackground(channelId);
  }

  async function loadVideos(reset = false, silent = false) {
    const selectedChannelId = context.getSelectedChannelId();
    if (!selectedChannelId) return;
    if (context.getLoadingVideos() && !silent) return;

    if (!silent) {
      context.setVideoLoadingState(true);
    }

    try {
      const acknowledged = resolveAcknowledgedParam(
        context.getAcknowledgedFilter(),
      );
      const currentOffset = context.getOffset();
      const list = context.options.onListVideos
        ? await context.options.onListVideos(
            selectedChannelId,
            context.limit,
            reset ? 0 : currentOffset,
            context.getVideoTypeFilter(),
            acknowledged,
            false,
          )
        : await listVideos(
            selectedChannelId,
            context.limit,
            reset ? 0 : currentOffset,
            context.getVideoTypeFilter(),
            acknowledged,
          );
      const page = Array.isArray(list)
        ? {
            videos: list,
            has_more: list.length === context.limit,
            next_offset: (reset ? 0 : currentOffset) + list.length,
          }
        : list;
      context.applyVideoPageState(page, { reset });

      if (context.options.onVideosLoaded) {
        await context.options.onVideosLoaded({
          reset,
          videos: reset ? page.videos : context.getVideos(),
        });
      }
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        context.options.onError?.((error as Error).message);
      }
    } finally {
      if (!silent) {
        context.setVideoLoadingState(false);
      }
    }
  }

  async function selectChannel(
    channelId: string,
    videoId: string | null = null,
    fromUserInteraction = false,
    selectedVideoHint: Video | null = null,
  ) {
    const cacheKey = context.getVideoStateKey(channelId);
    const cached = context.videoStateCache.get(cacheKey);
    const hasCached =
      !!cached &&
      cached.videos.length > 0 &&
      videosBelongToChannel(channelId, cached.videos);

    if (cached && !hasCached) {
      context.videoStateCache.delete(cacheKey);
    }

    context.applySelectionState({
      selectedChannelId: channelId,
      selectedVideoId: videoId ?? null,
    });
    context.options.onChannelSelected?.(channelId);

    if (hasCached && cached) {
      restoreCachedVideoState(cached, channelId);
      context.setVideoLoadingState(false);
      void refreshAndLoadVideos(channelId, true);
      return;
    }

    context.resetVideoListState({
      videos: selectedVideoHint ? [selectedVideoHint] : [],
    });
    context.options.onVideoListReset?.();
    // Set loading eagerly in both paths: user-initiated (non-silent) and
    // programmatic (silent snapshot but still show skeleton in the UI).
    context.setVideoLoadingState(true);
    try {
      await refreshAndLoadVideos(channelId, !fromUserInteraction);
    } finally {
      // applyChannelSnapshot resets this in the happy path via its own
      // finally block; this clears it if loadSnapshot() throws first.
      context.setVideoLoadingState(false);
    }
  }

  function selectVideo(videoId: string | null) {
    context.applySelectionState({ selectedVideoId: videoId });
  }

  async function reloadSelectedChannelVideos(
    options_local: {
      reset?: boolean;
      silent?: boolean;
      refresh?: boolean;
      clearMissingSelectedVideo?: boolean;
    } = {},
  ) {
    const selectedChannelId = context.getSelectedChannelId();
    if (!selectedChannelId) return;

    const selectedVideoId = context.getSelectedVideoId();
    const reset = options_local.reset ?? false;
    const silent = options_local.silent ?? false;

    if (reset && options_local.refresh) {
      context.resetVideoListState();
    }

    if (options_local.refresh) {
      await refreshAndLoadVideos(selectedChannelId, silent);
    } else {
      await loadVideos(reset, silent);
    }

    if (
      !reset ||
      !options_local.clearMissingSelectedVideo ||
      !selectedVideoId ||
      context.getSelectedChannelId() !== selectedChannelId ||
      context.getSelectedVideoId() !== selectedVideoId
    ) {
      return;
    }

    if (context.getVideos().some((video) => video.id === selectedVideoId)) {
      return;
    }

    selectVideo(null);
  }

  async function setVideoTypeFilterAndReload(nextValue: VideoTypeFilter) {
    await applyVideoTypeFilterChange({
      currentFilter: context.getVideoTypeFilter(),
      nextFilter: nextValue,
      videos: context.getVideos(),
      setFilter: context.setVideoTypeFilter,
      setVideos: context.setVideos,
      reload: () => loadVideos(true, true),
    });
  }

  async function setAcknowledgedFilterAndReload(nextValue: AcknowledgedFilter) {
    await applyAcknowledgedFilterChange({
      currentFilter: context.getAcknowledgedFilter(),
      nextFilter: nextValue,
      videos: context.getVideos(),
      setFilter: context.setAcknowledgedFilter,
      setVideos: context.setVideos,
      reload: () => loadVideos(true, true),
    });
  }

  async function clearAllFiltersAndReload() {
    await clearSidebarVideoFilters({
      videoTypeFilter: context.getVideoTypeFilter(),
      acknowledgedFilter: context.getAcknowledgedFilter(),
      setVideoTypeFilter: context.setVideoTypeFilter,
      setAcknowledgedFilter: context.setAcknowledgedFilter,
      reload: () => loadVideos(true, true),
    });
  }

  return {
    loadInitial,
    refreshAndLoadVideos,
    loadVideos,
    selectChannel,
    selectVideo,
    mergeSelectedVideoHint,
    selectChannelVideoOptimistically,
    reloadSelectedChannelVideos,
    setVideoTypeFilterAndReload,
    setAcknowledgedFilterAndReload,
    clearAllFiltersAndReload,
  };
}
