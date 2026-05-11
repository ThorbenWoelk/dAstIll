import { tick } from "svelte";
import { SvelteMap } from "svelte/reactivity";
import {
  getWorkspaceBootstrapWhenAvailable,
  getChannelSnapshot,
  listVideos,
  refreshChannel,
  backfillChannelVideos,
  RateLimitedError,
  type BackfillChannelVideosResponse,
} from "$lib/api";
import {
  applySavedChannelOrder,
  resolveInitialChannelSelection,
} from "$lib/workspace/channel-workspace";
import {
  putCachedChannels,
  putCachedBootstrapMeta,
  putCachedViewSnapshot,
} from "$lib/workspace/workspace-cache";
import { resolveAcknowledgedParam } from "$lib/workspace/types";
import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth/required-notice";
import { track } from "$lib/analytics/tracker";
import type { SidebarStateResult } from "$lib/workspace/sidebar-state.svelte";
import type {
  AiStatus,
  SearchStatus,
  ChannelSnapshot,
  VideoTypeFilter,
} from "$lib/types";

export function createWorkspaceState(options: {
  sidebarState: SidebarStateResult;
  getMobileViewportMq: () => boolean;
  getMobileBrowseOpen: () => boolean;
  setMobileBrowseOpen: (v: boolean) => void;
  getAiAvailable: () => boolean | null;
  setAiAvailable: (v: boolean | null) => void;
  getAiStatus: () => AiStatus | null;
  setAiStatus: (v: AiStatus | null) => void;
  getSearchStatus: () => SearchStatus | null;
  setSearchStatus: (v: SearchStatus | null) => void;
  getErrorMessage: () => string | null;
  setErrorMessage: (v: string | null) => void;
  clearSelectedVideoState: () => void;
  loadContent: () => Promise<void>;
  hydrateSelectedVideo: (
    videoId: string | null,
    acknowledged: boolean | undefined,
  ) => Promise<void>;
  loadSyncDepth: () => Promise<void>;
  buildWorkspaceSnapshotCacheKey: (
    channelId: string,
    type: VideoTypeFilter,
    acknowledged: boolean | undefined,
  ) => string;
}) {
  const { sidebarState } = options;
  const MIN_BACKFILL_INTERVAL_MS = 2100;
  let lastBackfillRequestAtMs = 0;

  async function applyChannelSnapshot(
    channelId: string,
    snapshot: ChannelSnapshot,
    preferredVideoId: string | null,
    silent = false,
  ) {
    if (!silent) {
      sidebarState.setVideoLoadingState(true);
      sidebarState.clearSelectedVideoSelection();
      options.setErrorMessage(null);
    }
    try {
      if (sidebarState.selectedChannelId !== channelId) {
        return;
      }

      const isAck = resolveAcknowledgedParam(sidebarState.acknowledgedFilter);
      sidebarState.applyChannelSnapshotState({
        videos: snapshot.videos,
        has_more: snapshot.has_more,
        next_offset: snapshot.next_offset,
        sync_depth: snapshot.sync_depth,
      });
      track({
        event: "channel_snapshot_loaded",
        channel_id: channelId,
        video_count: snapshot.channel_video_count ?? snapshot.videos.length,
      });
      void putCachedViewSnapshot(
        options.buildWorkspaceSnapshotCacheKey(
          channelId,
          sidebarState.videoTypeFilter,
          isAck,
        ),
        snapshot,
      );
      await options.hydrateSelectedVideo(preferredVideoId, isAck);
    } catch (error) {
      if (presentAuthRequiredNoticeIfNeeded(error)) {
        // Modal only
      } else if (!silent || !options.getErrorMessage()) {
        options.setErrorMessage((error as Error).message);
      }
    } finally {
      if (!silent) {
        sidebarState.setVideoLoadingState(false);
      }
    }
  }

  async function refreshAndLoadVideos(
    channelId: string,
    bypassTtl = false,
    preferredVideoId: string | null = sidebarState.selectedVideoId,
    silentInitialSnapshot = false,
  ) {
    // This could be simplified by using sidebarState.refreshAndLoadVideos
    // but we need the applyChannelSnapshot override.
    const isAck = resolveAcknowledgedParam(sidebarState.acknowledgedFilter);
    const { loadChannelSnapshotWithRefresh } =
      await import("$lib/workspace/route-helpers");

    void loadChannelSnapshotWithRefresh({
      channelId,
      refreshedAtByChannel: new SvelteMap(),
      ttlMs: 5 * 60 * 1000,
      bypassTtl,
      initialSilent: silentInitialSnapshot,
      getMutationEpoch: () => sidebarState.getVideoListMutationEpoch(),
      loadSnapshot: () =>
        getChannelSnapshot(channelId, {
          limit: sidebarState.limit,
          offset: sidebarState.offset,
          videoType: sidebarState.videoTypeFilter,
          acknowledged: isAck,
        }),
      applySnapshot: (snapshot, silent = false) =>
        applyChannelSnapshot(channelId, snapshot, preferredVideoId, silent),
      refreshChannel: () => refreshChannel(channelId),
      shouldReloadAfterRefresh: () =>
        sidebarState.selectedChannelId === channelId,
      onRefreshingChange: (refreshing: boolean) => {
        sidebarState.setRefreshingChannelState(refreshing);
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

    if (!silent) sidebarState.setVideoLoadingState(true);
    if (!silent) options.setErrorMessage(null);

    try {
      const isAck = resolveAcknowledgedParam(sidebarState.acknowledgedFilter);
      const list = await listVideos(
        sidebarState.selectedChannelId,
        sidebarState.limit,
        reset ? 0 : sidebarState.offset,
        sidebarState.videoTypeFilter,
        isAck,
      );

      if (
        !sidebarState.isCurrentSelection(
          sidebarState.selectedChannelId,
          sidebarState.selectedVideoId,
        )
      )
        return;

      sidebarState.applyVideoPageState(list, { reset });

      if (reset) {
        await options.hydrateSelectedVideo(sidebarState.selectedVideoId, isAck);
      }
    } catch (error) {
      if (presentAuthRequiredNoticeIfNeeded(error)) {
        // Modal only
      } else if (!silent || !options.getErrorMessage()) {
        options.setErrorMessage((error as Error).message);
      }
    } finally {
      if (!silent) {
        sidebarState.setVideoLoadingState(false);
      }
    }
  }

  async function loadMoreVideos() {
    if (
      !sidebarState.selectedChannelId ||
      sidebarState.loadingVideos ||
      sidebarState.backfillingHistory
    )
      return;

    if (sidebarState.hasMore) {
      await loadVideos(false);
      return;
    }

    sidebarState.setBackfillState({ backfillingHistory: true });
    options.setErrorMessage(null);

    try {
      const channelId = sidebarState.selectedChannelId;
      if (!channelId) return;

      const throttleWait = Math.max(
        0,
        MIN_BACKFILL_INTERVAL_MS - (Date.now() - lastBackfillRequestAtMs),
      );
      if (throttleWait > 0) {
        await new Promise((r) => setTimeout(r, throttleWait));
      }

      let result: BackfillChannelVideosResponse | undefined;
      const maxAttempts = 12;
      for (let attempt = 1; attempt <= maxAttempts; attempt++) {
        lastBackfillRequestAtMs = Date.now();
        try {
          result = await backfillChannelVideos(channelId, 50);
          break;
        } catch (e) {
          if (e instanceof RateLimitedError && attempt < maxAttempts) {
            await new Promise((r) => setTimeout(r, e.retryAfterMs));
            continue;
          }
          throw e;
        }
      }
      if (!result) return;

      if (result.exhausted) {
        sidebarState.setBackfillState({ historyExhausted: true });
      }

      await loadVideos(false);
      await options.loadSyncDepth();
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        options.setErrorMessage((error as Error).message);
      }
    } finally {
      sidebarState.setBackfillState({ backfillingHistory: false });
    }
  }

  async function loadBootstrapRefresh(options_local?: { silent?: boolean }) {
    const silent = options_local?.silent ?? false;
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

      sidebarState.setChannels(
        applySavedChannelOrder(bootstrap.channels, sidebarState.channelOrder),
      );
      sidebarState.syncChannelOrderFromList();
      void putCachedChannels(sidebarState.channels);

      options.setAiAvailable(bootstrap.ai_available);
      options.setAiStatus(bootstrap.ai_status);
      options.setSearchStatus(bootstrap.search_status);
      void putCachedBootstrapMeta({
        ai_available: bootstrap.ai_available,
        ai_status: bootstrap.ai_status,
        search_status: bootstrap.search_status,
      });

      const selectionChannelId = sidebarState.selectedChannelId;
      const selectionVideoId = sidebarState.selectedVideoId;

      const initialChannelId = resolveInitialChannelSelection(
        bootstrap.channels,
        selectionChannelId ?? previousSelectedChannelId,
        null,
      );

      if (!initialChannelId) {
        sidebarState.clearChannelSelectionState();
        options.setMobileBrowseOpen(true);
        options.clearSelectedVideoState();
      } else {
        const preferredVideoId =
          initialChannelId === selectionChannelId ? selectionVideoId : null;
        const canReuseRenderedSnapshot =
          initialChannelId === selectionChannelId &&
          sidebarState.videos.length > 0;

        sidebarState.applySelectionState({
          selectedChannelId: initialChannelId,
        });
        options.clearSelectedVideoState(); // Note: in page.svelte it also cleared contentMode, but we want to be more specific

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
          options.clearSelectedVideoState();
          sidebarState.resetVideoListState({
            selectedVideoId: preferredVideoId,
          });
          if (!silent) {
            sidebarState.setVideoLoadingState(true);
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
        // Modal only
      } else if (!silent || !options.getErrorMessage()) {
        options.setErrorMessage((error as Error).message);
      }
    } finally {
      if (!silent) {
        sidebarState.setLoadingChannels(false);
        sidebarState.setLoadingVideos(false);
      }
    }
  }

  return {
    loadBootstrapRefresh,
    loadMoreVideos,
    loadVideos,
    refreshAndLoadVideos,
    applyChannelSnapshot,
  };
}
