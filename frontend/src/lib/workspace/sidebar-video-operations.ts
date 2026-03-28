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
  syncChannelOrderFromList: () => void;
  getVideoStateKey: (channelId: string) => string;
  getChannelOrder: () => string[];
  getSelectedChannelId: () => string | null;
  getVideos: () => Video[];
  getOffset: () => number;
  getVideoTypeFilter: () => VideoTypeFilter;
  getAcknowledgedFilter: () => AcknowledgedFilter;
  getLoadingVideos: () => boolean;
  getVideoListMutationEpoch: () => number;
  setChannels: (channels: Channel[]) => void;
  setSelectedChannelId: (channelId: string | null) => void;
  setSelectedVideoId: (videoId: string | null) => void;
  setVideos: (videos: Video[]) => void;
  setOffset: (offset: number) => void;
  setHasMore: (hasMore: boolean) => void;
  setSyncDepth: (syncDepth: ChannelSyncDepthState | null) => void;
  setLoadingChannels: (loading: boolean) => void;
  setLoadingVideos: (loading: boolean) => void;
  setRefreshingChannel: (refreshing: boolean) => void;
  setHistoryExhausted: (value: boolean) => void;
  setBackfillingHistory: (value: boolean) => void;
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
      context.setLoadingChannels(true);
    }

    try {
      const channelList = await listChannelsWhenAvailable({
        retryDelayMs: 500,
      });
      const orderedChannels = applySavedChannelOrder(
        channelList,
        context.getChannelOrder(),
      );
      context.setChannels(orderedChannels);
      context.syncChannelOrderFromList();
      cacheChannels(context.options, orderedChannels);

      const initialChannelId = resolveInitialChannelSelection(
        orderedChannels,
        context.getSelectedChannelId(),
        context.getChannelOrder()[0],
      );

      if (!initialChannelId) {
        context.setSelectedChannelId(null);
        context.setVideos([]);
        context.setSyncDepth(null);
      } else {
        context.setSelectedChannelId(initialChannelId);
        await refreshAndLoadVideos(initialChannelId, silent);
      }
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        context.options.onError?.((error as Error).message);
      }
    } finally {
      if (!silent) {
        context.setLoadingChannels(false);
      }
    }
  }

  async function applyChannelSnapshot(
    channelId: string,
    snapshot: ChannelSnapshot,
    silent = false,
  ) {
    if (!silent) {
      context.setLoadingVideos(true);
    }
    try {
      if (context.getSelectedChannelId() !== channelId) return;
      const deduped = snapshot.videos;
      context.setSyncDepth(snapshot.sync_depth);
      context.setVideos(deduped);
      context.setOffset(deduped.length);
      context.setHasMore(deduped.length === context.limit);

      if (context.options.onVideosLoaded) {
        await context.options.onVideosLoaded({
          reset: true,
          videos: deduped,
        });
      }
    } finally {
      if (!silent) {
        context.setLoadingVideos(false);
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
      onRefreshingChange: context.setRefreshingChannel,
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
      context.setLoadingVideos(true);
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
      const nextVideos = reset
        ? page.videos
        : [...context.getVideos(), ...page.videos];
      context.setVideos(nextVideos);
      context.setOffset(
        page.next_offset ?? (reset ? 0 : currentOffset) + page.videos.length,
      );
      context.setHasMore(page.has_more);

      if (context.options.onVideosLoaded) {
        await context.options.onVideosLoaded({
          reset,
          videos: context.getVideos(),
        });
      }
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        context.options.onError?.((error as Error).message);
      }
    } finally {
      if (!silent) {
        context.setLoadingVideos(false);
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

    context.setSelectedChannelId(channelId);
    context.setSelectedVideoId(videoId ?? null);
    context.options.onChannelSelected?.(channelId);

    if (hasCached && cached) {
      context.setVideos(cloneVideos(cached.videos));
      context.setOffset(cached.offset);
      context.setHasMore(cached.hasMore);
      context.setSyncDepth(cloneSyncDepthState(cached.syncDepth));
      context.setLoadingVideos(false);
      void refreshAndLoadVideos(channelId, true);
      return;
    }

    context.setVideos(selectedVideoHint ? [selectedVideoHint] : []);
    context.setOffset(0);
    context.setHasMore(true);
    context.setHistoryExhausted(false);
    context.setBackfillingHistory(false);
    context.setSyncDepth(null);
    context.options.onVideoListReset?.();
    await refreshAndLoadVideos(channelId, !fromUserInteraction);
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
    setVideoTypeFilterAndReload,
    setAcknowledgedFilterAndReload,
    clearAllFiltersAndReload,
  };
}
