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
  loadChannelSnapshotWithRefresh,
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
        await refreshAndLoadVideos(initialChannelId, silent);
      }
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        context.options.onError?.((error as Error).message);
      }
    } finally {
      if (!silent) {
        context.setChannelLoadingState(false);
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
      context.applyChannelSnapshotState({
        videos: snapshot.videos,
        has_more: snapshot.videos.length === context.limit,
        next_offset: snapshot.videos.length,
        sync_depth: snapshot.sync_depth,
      });

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
      context.resetVideoListState({
        videos: cloneVideos(cached.videos),
        offset: cached.offset,
        hasMore: cached.hasMore,
        syncDepth: cloneSyncDepthState(cached.syncDepth),
      });
      context.setVideoLoadingState(false);
      void refreshAndLoadVideos(channelId, true);
      return;
    }

    context.resetVideoListState({
      videos: selectedVideoHint ? [selectedVideoHint] : [],
    });
    context.options.onVideoListReset?.();
    await refreshAndLoadVideos(channelId, !fromUserInteraction);
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
    reloadSelectedChannelVideos,
    setVideoTypeFilterAndReload,
    setAcknowledgedFilterAndReload,
    clearAllFiltersAndReload,
  };
}
