import { tick } from "svelte";

import { authState } from "$lib/auth-state.svelte";
import {
  backfillChannelVideos,
  deleteChannel,
  getChannelSnapshot,
  getChannelSyncDepth,
  getVideo,
  getWorkspaceBootstrapWhenAvailable,
  listVideos,
  refreshChannel,
  RateLimitedError,
  type BackfillChannelVideosResponse,
} from "$lib/api";
import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
import { closeSummarySession } from "$lib/analytics/read-time";
import { track } from "$lib/analytics/tracker";
import { resolveInitialChannelSelection } from "$lib/channel-workspace";
import {
  buildChannelViewCacheKey,
  cloneSyncDepthState,
  cloneVideos,
  createChannelViewCache,
  type ChannelSyncDepthState,
} from "$lib/channel-view-cache";
import { buildHomeWorkspaceChannelViewCacheKey } from "$lib/workspace/home-workspace-cache-key";
import {
  putCachedBootstrapMeta,
  putCachedChannels,
  putCachedViewSnapshot,
  removeCachedChannel,
} from "$lib/workspace-cache";
import {
  loadChannelSnapshotWithRefresh,
  resolveNextChannelSelection,
} from "$lib/workspace/route-helpers";
import type {
  AcknowledgedFilter,
  WorkspaceContentMode,
} from "$lib/workspace/types";
import {
  isWorkspaceContentMode,
  resolveAcknowledgedParam,
} from "$lib/workspace/types";
import type {
  ChannelSnapshot,
  Highlight,
  SearchResult,
  Video,
  VideoTypeFilter,
} from "$lib/types";

import { createContentState } from "$lib/workspace/content-state.svelte";
import { createSidebarState } from "$lib/workspace/sidebar-state.svelte";

export type CachedChannelVideoState = {
  videos: Video[];
  offset: number;
  hasMore: boolean;
  historyExhausted: boolean;
  backfillingHistory: boolean;
  allowLoadedVideoSyncDepthOverride: boolean;
  syncDepth: ChannelSyncDepthState | null;
};

const CHANNEL_REFRESH_TTL_MS = 5 * 60 * 1000;
const MIN_BACKFILL_INTERVAL_MS = 2100;
const SELECTED_VIDEO_SCAN_PAGE_LIMIT = 8;

export function createHomeWorkspaceDataController(options: {
  sidebarState: ReturnType<typeof createSidebarState>;
  content: ReturnType<typeof createContentState>;
  channelLastRefreshedAt: Map<string, number>;
  channelVideoStateCache: ReturnType<
    typeof createChannelViewCache<CachedChannelVideoState>
  >;
  getAllowLoadedVideoSyncDepthOverride: () => boolean;
  setAllowLoadedVideoSyncDepthOverride: (value: boolean) => void;
  getPendingSelectedVideo: () => Video | null;
  setPendingSelectedVideo: (value: Video | null) => void;
  getErrorMessage: () => string | null;
  setErrorMessage: (value: string | null) => void;
  getMobileBrowseOpen: () => boolean;
  setMobileBrowseOpen: (value: boolean) => void;
  getMobileViewportMq: () => boolean;
  getWorkspaceCacheScopeKey: () => string;
  getVideoHighlightsByVideoId: () => Record<string, Highlight[]>;
  hydrateVideoHighlights: (
    videoId: string,
    options?: { showError?: boolean },
  ) => Promise<Highlight[] | null>;
}) {
  const { sidebarState, content } = options;
  let lastBackfillRequestAtMs = 0;

  function getChannelViewKey(channelId: string) {
    return buildHomeWorkspaceChannelViewCacheKey({
      channelId,
      workspaceCacheScopeKey: options.getWorkspaceCacheScopeKey(),
      videoTypeFilter: sidebarState.videoState.videoTypeFilter,
      acknowledgedFilter: sidebarState.videoState.acknowledgedFilter,
    });
  }

  function restoreCachedChannelVideoState(state: CachedChannelVideoState) {
    sidebarState.resetVideoListState({
      videos: cloneVideos(state.videos),
      offset: state.offset,
      hasMore: state.hasMore,
      historyExhausted: state.historyExhausted,
      backfillingHistory: state.backfillingHistory,
      syncDepth: state.syncDepth,
    });
    options.setAllowLoadedVideoSyncDepthOverride(
      state.allowLoadedVideoSyncDepthOverride,
    );
  }

  $effect(() => {
    const selectedChannelId = sidebarState.selectedChannelId;
    if (!selectedChannelId) return;

    options.channelVideoStateCache.set(getChannelViewKey(selectedChannelId), {
      videos: cloneVideos(sidebarState.videos),
      offset: sidebarState.offset,
      hasMore: sidebarState.hasMore,
      historyExhausted: sidebarState.historyExhausted,
      backfillingHistory: sidebarState.backfillingHistory,
      allowLoadedVideoSyncDepthOverride:
        options.getAllowLoadedVideoSyncDepthOverride(),
      syncDepth: cloneSyncDepthState(sidebarState.syncDepth),
    });
  });

  async function loadSyncDepth() {
    if (!sidebarState.selectedChannelId) {
      sidebarState.setSyncDepth(null);
      return;
    }
    try {
      const depth = await getChannelSyncDepth(sidebarState.selectedChannelId);
      sidebarState.setSyncDepth(depth as ChannelSyncDepthState);
    } catch {
      sidebarState.setSyncDepth(null);
    }
  }

  async function handleChannelSyncDateSaved(channelId: string) {
    options.channelVideoStateCache.deleteByPrefix(
      buildChannelViewCacheKey(channelId, options.getWorkspaceCacheScopeKey()),
    );
    if (sidebarState.selectedChannelId === channelId) {
      await loadSyncDepth();
    }
  }

  function clearSelectedVideoState() {
    sidebarState.clearSelectedVideoSelection();
    options.setPendingSelectedVideo(null);
    content.clear();
  }

  async function resolvePendingSelectedVideo(
    videoId: string,
    channelId: string,
  ) {
    try {
      const video = await getVideo(videoId);
      if (
        sidebarState.selectedChannelId !== channelId ||
        sidebarState.selectedVideoId !== videoId
      ) {
        return null;
      }
      options.setPendingSelectedVideo(video);
      return video;
    } catch {
      return null;
    }
  }

  async function hydrateSelectedVideo(
    preferredVideoId: string | null,
    acknowledged: boolean | undefined,
  ) {
    if (sidebarState.videos.length === 0) {
      clearSelectedVideoState();
      return;
    }

    if (!preferredVideoId) {
      options.setPendingSelectedVideo(null);
      void selectVideo(sidebarState.videos[0].id);
      return;
    }

    sidebarState.applySelectionState({ selectedVideoId: preferredVideoId });
    const cachedHighlights =
      options.getVideoHighlightsByVideoId()[preferredVideoId];
    if (!cachedHighlights) {
      void options.hydrateVideoHighlights(preferredVideoId);
    }
    let hasSelectedVideo = sidebarState.videos.some(
      (video) => video.id === preferredVideoId,
    );
    let scannedPages = 0;
    const targetChannelId = sidebarState.selectedChannelId;
    const pendingSelectedVideoRequest =
      hasSelectedVideo || !targetChannelId
        ? Promise.resolve(null)
        : resolvePendingSelectedVideo(preferredVideoId, targetChannelId);

    void content.loadContent();

    while (
      !hasSelectedVideo &&
      sidebarState.hasMore &&
      scannedPages < SELECTED_VIDEO_SCAN_PAGE_LIMIT &&
      targetChannelId &&
      sidebarState.selectedChannelId === targetChannelId &&
      sidebarState.selectedVideoId === preferredVideoId
    ) {
      const next = await listVideos(
        targetChannelId,
        sidebarState.limit,
        sidebarState.offset,
        sidebarState.videoTypeFilter,
        acknowledged,
      );
      scannedPages += 1;
      if (next.videos.length === 0) {
        sidebarState.applyVideoPageState(next);
        break;
      }

      sidebarState.applyVideoPageState(next);
      hasSelectedVideo = sidebarState.videos.some(
        (video) => video.id === preferredVideoId,
      );
    }

    if (!hasSelectedVideo) {
      const restoredVideo = await pendingSelectedVideoRequest;
      if (
        restoredVideo &&
        sidebarState.selectedChannelId === targetChannelId &&
        sidebarState.selectedVideoId === preferredVideoId
      ) {
        return;
      }

      void selectVideo(sidebarState.videos[0].id);
      return;
    }

    options.setPendingSelectedVideo(null);
  }

  function buildWorkspaceSnapshotCacheKey(
    channelId: string,
    type: VideoTypeFilter,
    acknowledged: boolean | undefined,
  ) {
    const acknowledgedKey =
      acknowledged === undefined ? "all" : acknowledged ? "ack" : "unack";
    return `workspace:${channelId}:type=${type}:ack=${acknowledgedKey}:limit=${sidebarState.limit}`;
  }

  async function applyChannelSnapshot(
    channelId: string,
    snapshot: ChannelSnapshot,
    preferredVideoId: string | null,
    silent = false,
  ) {
    if (!silent) {
      sidebarState.setLoadingVideos(true);
      sidebarState.clearSelectedVideoSelection();
      options.setErrorMessage(null);
    }
    try {
      if (sidebarState.selectedChannelId !== channelId) {
        return;
      }

      const acknowledged = resolveAcknowledgedParam(
        sidebarState.acknowledgedFilter,
      );
      options.setAllowLoadedVideoSyncDepthOverride(false);
      sidebarState.applyChannelSnapshotState({
        videos: snapshot.videos,
        has_more: snapshot.videos.length === sidebarState.limit,
        next_offset: snapshot.videos.length,
        sync_depth: snapshot.sync_depth,
      });
      track({
        event: "channel_snapshot_loaded",
        channel_id: channelId,
        video_count: snapshot.channel_video_count ?? snapshot.videos.length,
      });
      void putCachedViewSnapshot(
        buildWorkspaceSnapshotCacheKey(
          channelId,
          sidebarState.videoTypeFilter,
          acknowledged,
        ),
        snapshot,
        options.getWorkspaceCacheScopeKey(),
      );
      await hydrateSelectedVideo(preferredVideoId, acknowledged);
    } catch (error) {
      if (presentAuthRequiredNoticeIfNeeded(error)) {
        return;
      }
      if (!silent || !options.getErrorMessage()) {
        options.setErrorMessage((error as Error).message);
      }
    } finally {
      if (!silent) {
        sidebarState.setLoadingVideos(false);
      }
    }
  }

  async function handleSearchResultSelection(
    result: SearchResult,
    targetMode: "transcript" | "summary",
  ) {
    if (sidebarState.selectedChannelId !== result.channel_id) {
      await selectChannel(result.channel_id, result.video_id, true);
    } else {
      await selectVideo(result.video_id, true);
    }

    if (content.contentMode !== targetMode) {
      content.setMode(targetMode);
      await content.loadContent();
    }
  }

  async function loadBootstrapRefresh(opts: { silent?: boolean } = {}) {
    const silent = opts.silent ?? false;
    const previousSelectedChannelId = sidebarState.selectedChannelId;

    if (!silent) {
      sidebarState.setLoadingChannels(true);
      options.setErrorMessage(null);
    }

    try {
      const bootstrap = await getWorkspaceBootstrapWhenAvailable({
        selectedChannelId: previousSelectedChannelId,
        videoType: sidebarState.videoTypeFilter,
        acknowledged: resolveAcknowledgedParam(sidebarState.acknowledgedFilter),
        limit: sidebarState.limit,
        retryDelayMs: 500,
      });

      sidebarState.applyLoadedChannelsState(
        bootstrap.channels,
        sidebarState.channelOrder,
      );
      void putCachedChannels(
        sidebarState.channels,
        options.getWorkspaceCacheScopeKey(),
      );

      void putCachedBootstrapMeta(
        {
          ai_available: bootstrap.ai_available,
          ai_status: bootstrap.ai_status,
          search_status: bootstrap.search_status,
        },
        options.getWorkspaceCacheScopeKey(),
      );

      const selectionChannelId = sidebarState.selectedChannelId;
      const selectionVideoId = sidebarState.selectedVideoId;

      const initialChannelId = resolveInitialChannelSelection(
        bootstrap.channels,
        selectionChannelId ?? previousSelectedChannelId,
        selectionChannelId,
      );

      if (!initialChannelId) {
        sidebarState.clearChannelSelectionState();
        options.setMobileBrowseOpen(true);
        clearSelectedVideoState();
        options.setAllowLoadedVideoSyncDepthOverride(false);
      } else {
        const preferredVideoId =
          initialChannelId === selectionChannelId ? selectionVideoId : null;
        const canReuseRenderedSnapshot =
          initialChannelId === selectionChannelId &&
          sidebarState.videos.length > 0;

        sidebarState.applySelectionState({
          selectedChannelId: initialChannelId,
        });

        if (!silent || preferredVideoId !== selectionVideoId) {
          content.resetViewState();
        }

        if (
          bootstrap.snapshot &&
          bootstrap.selected_channel_id === initialChannelId
        ) {
          await applyChannelSnapshot(
            initialChannelId,
            bootstrap.snapshot,
            preferredVideoId,
            canReuseRenderedSnapshot,
          );
        } else if (!canReuseRenderedSnapshot) {
          clearSelectedVideoState();
          sidebarState.resetVideoListState({
            selectedVideoId: preferredVideoId,
          });
          options.setAllowLoadedVideoSyncDepthOverride(false);
          if (!silent) {
            sidebarState.setLoadingVideos(true);
          }
          await tick();
          await refreshAndLoadVideos(
            initialChannelId,
            false,
            preferredVideoId,
            canReuseRenderedSnapshot,
          );
        }
      }
    } catch (error) {
      if (presentAuthRequiredNoticeIfNeeded(error)) {
        return;
      }
      if (!silent || !options.getErrorMessage()) {
        options.setErrorMessage((error as Error).message);
      }
    } finally {
      if (!silent) {
        sidebarState.setLoadingChannels(false);
        sidebarState.setLoadingVideos(false);
      }
    }
  }

  async function handleDeleteChannel(channelId: string) {
    if (authState.current.authState !== "authenticated") {
      return "auth_required" as const;
    }

    sidebarState.setChannelIdToDelete(channelId);
    sidebarState.setShowDeleteConfirmation(true);
    return "queued" as const;
  }

  async function confirmDeleteChannel() {
    if (
      !sidebarState.channelIdToDelete ||
      authState.current.authState !== "authenticated"
    ) {
      return;
    }
    const channelId = sidebarState.channelIdToDelete;
    sidebarState.setShowDeleteConfirmation(false);
    sidebarState.setChannelIdToDelete(null);

    const previousChannels = [...sidebarState.channels];
    const previousSelectedChannelId = sidebarState.selectedChannelId;

    sidebarState.removeChannel(channelId);

    if (sidebarState.selectedChannelId === channelId) {
      const nextChannelId = resolveNextChannelSelection(
        previousChannels,
        channelId,
      );
      if (nextChannelId) {
        await selectChannel(nextChannelId);
      } else {
        sidebarState.clearChannelSelectionState();
        options.setMobileBrowseOpen(true);
        content.clearDisplayedContent();
      }
    }

    try {
      await deleteChannel(channelId);
      void removeCachedChannel(channelId, options.getWorkspaceCacheScopeKey());
      options.channelVideoStateCache.deleteByPrefix(
        buildChannelViewCacheKey(
          channelId,
          options.getWorkspaceCacheScopeKey(),
        ),
      );
    } catch (error) {
      sidebarState.setChannels(previousChannels);
      sidebarState.setSelectedChannelId(previousSelectedChannelId);
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        options.setErrorMessage((error as Error).message);
      }
    }
  }

  async function selectChannel(
    channelId: string | null,
    videoId: string | null = null,
    _scroll = true,
  ) {
    if (sidebarState.selectedChannelId === channelId && !videoId) return;

    sidebarState.applySelectionState({ selectedChannelId: channelId });
    if (!channelId) return;

    if (!videoId) {
      clearSelectedVideoState();
    }

    const channelViewKey = getChannelViewKey(channelId);
    const cachedChannelVideoState =
      options.channelVideoStateCache.get(channelViewKey);
    const hasCachedChannelVideoState =
      !!cachedChannelVideoState && cachedChannelVideoState.videos.length > 0;

    content.clearFormattingFeedback();
    if (hasCachedChannelVideoState && cachedChannelVideoState) {
      restoreCachedChannelVideoState(cachedChannelVideoState);
      sidebarState.setLoadingVideos(false);
      void refreshAndLoadVideos(channelId, false, videoId, true);
      return;
    }

    sidebarState.resetVideoListState();
    options.setAllowLoadedVideoSyncDepthOverride(false);
    await refreshAndLoadVideos(channelId, false, videoId);
  }

  async function refreshAndLoadVideos(
    channelId: string,
    bypassTtl = false,
    preferredVideoId: string | null = sidebarState.selectedVideoId,
    silentInitialSnapshot = false,
  ) {
    const acknowledged = resolveAcknowledgedParam(
      sidebarState.acknowledgedFilter,
    );
    await loadChannelSnapshotWithRefresh({
      channelId,
      refreshedAtByChannel: options.channelLastRefreshedAt,
      ttlMs: CHANNEL_REFRESH_TTL_MS,
      bypassTtl,
      initialSilent: silentInitialSnapshot,
      getMutationEpoch: () => sidebarState.getVideoListMutationEpoch(),
      loadSnapshot: () =>
        getChannelSnapshot(channelId, {
          limit: sidebarState.limit,
          offset: sidebarState.offset,
          videoType: sidebarState.videoTypeFilter,
          acknowledged,
        }),
      applySnapshot: (snapshot, silent = false) =>
        applyChannelSnapshot(channelId, snapshot, preferredVideoId, silent),
      refreshChannel: () => refreshChannel(channelId),
      shouldReloadAfterRefresh: () =>
        sidebarState.selectedChannelId === channelId,
      onRefreshingChange: (refreshing: boolean) => {
        sidebarState.setRefreshingChannel(refreshing);
      },
      onError: (message) => {
        if (!options.getErrorMessage()) {
          options.setErrorMessage(message);
        }
      },
    });
  }

  async function loadVideos(reset = false, silent = false) {
    if (!sidebarState.selectedChannelId) return;
    if (sidebarState.loadingVideos && !silent) return;

    if (!silent) sidebarState.setLoadingVideos(true);
    if (!silent) options.setErrorMessage(null);

    try {
      const acknowledged = resolveAcknowledgedParam(
        sidebarState.acknowledgedFilter,
      );
      const list = await listVideos(
        sidebarState.selectedChannelId,
        sidebarState.limit,
        reset ? 0 : sidebarState.offset,
        sidebarState.videoTypeFilter,
        acknowledged,
      );

      if (
        !sidebarState.isCurrentSelection(
          sidebarState.selectedChannelId,
          sidebarState.selectedVideoId,
        )
      ) {
        return;
      }

      sidebarState.applyVideoPageState(list, { reset });
      if (reset) {
        options.setAllowLoadedVideoSyncDepthOverride(false);
        await hydrateSelectedVideo(sidebarState.selectedVideoId, acknowledged);
      }
    } catch (error) {
      if (presentAuthRequiredNoticeIfNeeded(error)) {
        return;
      }
      if (!silent || !options.getErrorMessage()) {
        options.setErrorMessage((error as Error).message);
      }
    } finally {
      if (!silent) {
        sidebarState.setLoadingVideos(false);
      }
    }
  }

  async function loadMoreVideos() {
    if (
      !sidebarState.selectedChannelId ||
      sidebarState.loadingVideos ||
      sidebarState.backfillingHistory
    ) {
      return;
    }

    if (sidebarState.hasMore) {
      await loadVideos(false);
      options.setAllowLoadedVideoSyncDepthOverride(true);
      return;
    }

    sidebarState.setBackfillingHistory(true);
    options.setErrorMessage(null);

    try {
      const channelId = sidebarState.selectedChannelId;
      if (!channelId) return;

      const throttleWait = Math.max(
        0,
        MIN_BACKFILL_INTERVAL_MS - (Date.now() - lastBackfillRequestAtMs),
      );
      if (throttleWait > 0) {
        await new Promise((resolve) => setTimeout(resolve, throttleWait));
      }

      let result: BackfillChannelVideosResponse | undefined;
      const maxAttempts = 12;
      for (let attempt = 1; attempt <= maxAttempts; attempt++) {
        lastBackfillRequestAtMs = Date.now();
        try {
          result = await backfillChannelVideos(channelId, 50);
          break;
        } catch (error) {
          if (error instanceof RateLimitedError && attempt < maxAttempts) {
            await new Promise((resolve) =>
              setTimeout(resolve, error.retryAfterMs),
            );
            continue;
          }
          throw error;
        }
      }
      if (!result) return;

      if (result.exhausted) {
        sidebarState.setHistoryExhausted(true);
      }

      await loadVideos(false);
      await loadSyncDepth();
      options.setAllowLoadedVideoSyncDepthOverride(true);
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        options.setErrorMessage((error as Error).message);
      }
    } finally {
      sidebarState.setBackfillingHistory(false);
    }
  }

  async function loadAllVideosForMobileBrowse(isAborted: () => boolean) {
    const channelId = sidebarState.selectedChannelId;
    if (
      !channelId ||
      !options.getMobileBrowseOpen() ||
      !options.getMobileViewportMq()
    ) {
      return;
    }

    let bootWait = 0;
    while (bootWait++ < 100 && !isAborted()) {
      if (
        !options.getMobileBrowseOpen() ||
        sidebarState.selectedChannelId !== channelId ||
        !options.getMobileViewportMq()
      ) {
        return;
      }
      const hasList = sidebarState.videos.length > 0 || !sidebarState.hasMore;
      if (hasList) break;
      await tick();
      await new Promise((resolve) => setTimeout(resolve, 40));
    }

    let safety = 0;
    while (safety++ < 2000 && !isAborted()) {
      if (
        !options.getMobileBrowseOpen() ||
        sidebarState.selectedChannelId !== channelId ||
        !options.getMobileViewportMq()
      ) {
        return;
      }

      while (
        (sidebarState.loadingVideos ||
          sidebarState.backfillingHistory ||
          sidebarState.refreshingChannel) &&
        !isAborted()
      ) {
        await tick();
        await new Promise((resolve) => setTimeout(resolve, 30));
      }
      if (isAborted()) return;
      if (
        !options.getMobileBrowseOpen() ||
        sidebarState.selectedChannelId !== channelId
      ) {
        return;
      }

      if (sidebarState.hasMore) {
        await loadVideos(false);
        continue;
      }
      if (sidebarState.historyExhausted) {
        break;
      }
      await loadMoreVideos();
    }
  }

  async function loadDbPagesOnlyForMobileBrowse() {
    const channelId = sidebarState.selectedChannelId;
    if (
      !channelId ||
      !options.getMobileBrowseOpen() ||
      !options.getMobileViewportMq()
    ) {
      return;
    }

    let safety = 0;
    while (sidebarState.hasMore && safety++ < 500) {
      if (
        sidebarState.selectedChannelId !== channelId ||
        !options.getMobileBrowseOpen() ||
        !options.getMobileViewportMq()
      ) {
        return;
      }
      while (sidebarState.loadingVideos || sidebarState.backfillingHistory) {
        await tick();
        await new Promise((resolve) => setTimeout(resolve, 30));
      }
      if (
        sidebarState.selectedChannelId !== channelId ||
        !options.getMobileBrowseOpen() ||
        !options.getMobileViewportMq()
      ) {
        return;
      }
      if (!sidebarState.hasMore) break;
      await loadVideos(false);
    }
  }

  async function onBrowseVideoTypeFilterChange(nextValue: VideoTypeFilter) {
    await sidebarState.videoActions.onVideoTypeFilterChange(nextValue);
    await loadDbPagesOnlyForMobileBrowse();
  }

  async function onBrowseAcknowledgedFilterChange(
    nextValue: AcknowledgedFilter,
  ) {
    await sidebarState.videoActions.onAcknowledgedFilterChange(nextValue);
    await loadDbPagesOnlyForMobileBrowse();
  }

  async function selectVideo(
    videoId: string | null,
    fromUserInteraction = false,
    _forceReload = false,
  ) {
    if (fromUserInteraction) {
      options.setMobileBrowseOpen(false);
    }
    if (content.contentMode === "summary" && sidebarState.selectedVideoId) {
      closeSummarySession();
    }
    sidebarState.applySelectionState({ selectedVideoId: videoId });
    if (videoId && sidebarState.selectedChannelId) {
      track({
        event: "video_opened",
        video_id: videoId,
        channel_id: sidebarState.selectedChannelId,
      });
    }
    content.clearDisplayedContent();
    const cachedHighlights = videoId
      ? options.getVideoHighlightsByVideoId()[videoId]
      : null;
    if (videoId && !cachedHighlights) {
      void options.hydrateVideoHighlights(videoId);
    }
    content.resetViewState();
    await content.loadContent();
  }

  async function setMode(mode: WorkspaceContentMode) {
    if (!isWorkspaceContentMode(mode) || content.contentMode === mode) return;
    const previousMode = content.contentMode;
    if (previousMode === "summary" && sidebarState.selectedVideoId) {
      closeSummarySession();
    }
    content.setMode(mode);
    if (sidebarState.selectedVideoId && sidebarState.selectedChannelId) {
      track({
        event: "content_mode_changed",
        video_id: sidebarState.selectedVideoId,
        channel_id: sidebarState.selectedChannelId,
        from_mode: previousMode,
        to_mode: mode,
      });
    }
    content.resetViewState();
    await content.loadContent();
  }

  async function clearBrowseVideoFilters() {
    const actions = sidebarState.videoActions;
    if (actions.onClearAllFilters) {
      await actions.onClearAllFilters();
    } else {
      await actions.onVideoTypeFilterChange("all");
      await actions.onAcknowledgedFilterChange("all");
    }
    if (options.getMobileBrowseOpen() && options.getMobileViewportMq()) {
      await loadDbPagesOnlyForMobileBrowse();
    }
  }

  return {
    buildWorkspaceSnapshotCacheKey,
    handleChannelSyncDateSaved,
    clearSelectedVideoState,
    handleSearchResultSelection,
    loadBootstrapRefresh,
    handleDeleteChannel,
    confirmDeleteChannel,
    selectChannel,
    loadMoreVideos,
    loadAllVideosForMobileBrowse,
    onBrowseVideoTypeFilterChange,
    onBrowseAcknowledgedFilterChange,
    selectVideo,
    setMode,
    clearBrowseVideoFilters,
    applyChannelSnapshot,
    syncChannelOrderFromList: sidebarState.syncChannelOrderFromList,
  };
}
