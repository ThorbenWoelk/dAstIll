import { goto, replaceState as replacePageState } from "$app/navigation";
import { page } from "$app/state";
import { onMount, tick } from "svelte";
import type { Component } from "svelte";
import { authState } from "$lib/auth-state.svelte";
import { getAuthStorageScopeKey, getScopedStorageKey } from "$lib/auth-storage";
import {
  backfillChannelVideos,
  type BackfillChannelVideosResponse,
  createHighlight,
  deleteChannel,
  getChannelSnapshot,
  getChannelSyncDepth,
  getPreferences,
  getVideo,
  getVideoHighlights,
  getWorkspaceBootstrapWhenAvailable,
  listVideos,
  refreshChannel,
  savePreferences,
  updateAcknowledged,
  RateLimitedError,
} from "$lib/api";
import { resolveAiIndicatorPresentation } from "$lib/ai-status";
import { DOCS_URL } from "$lib/app-config";
import type {
  AiStatus,
  AddVideoResult,
  Channel,
  ChannelSnapshot,
  CreateHighlightRequest,
  Highlight,
  HighlightSource,
  SearchResult,
  SearchStatus,
  Video,
  VideoTypeFilter,
  VocabularyReplacement,
} from "$lib/types";
import {
  applySavedChannelOrder,
  loadWorkspaceState,
  restoreWorkspaceSnapshot,
  resolveInitialChannelSelection,
  saveWorkspaceState,
  type WorkspaceStateSnapshot,
} from "$lib/channel-workspace";
import { renderMarkdown } from "$lib/utils/markdown";
import {
  buildChannelAddFeedback,
  buildVideoAddFeedback,
  type AddSourceFeedback,
  resolveAddedChannelStatus,
  resolveAddedVideoStatus,
} from "$lib/workspace/add-source-feedback";
import {
  buildOptimisticHighlight,
  reconcileOptimisticHighlight,
} from "$lib/utils/highlights";
import {
  putCachedBootstrapMeta,
  putCachedChannels,
  putCachedViewSnapshot,
  removeCachedChannel,
} from "$lib/workspace-cache";
import { resolveBootstrapOnMount } from "$lib/ssr-bootstrap";
import { createAiStatusPoller } from "$lib/utils/ai-poller";
import { upsertVocabularyReplacement } from "$lib/vocabulary";
import { mobileWorkspaceBrowseIntent } from "$lib/mobile-navigation/mobileWorkspaceBrowseIntent";
import {
  buildWorkspaceViewHref,
  mergeWorkspaceViewState,
  type WorkspaceViewState,
} from "$lib/view-url";
import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
import {
  loadSessionHighlightsMap,
  resolveHighlightsScopeKey,
  saveSessionHighlightsMap,
  shouldUseSessionHighlights,
} from "$lib/workspace/session-highlights";
import { track } from "$lib/analytics/tracker";
import { closeSummarySession } from "$lib/analytics/read-time";
import {
  buildChannelViewCacheKey,
  cloneSyncDepthState,
  cloneVideos,
  createChannelViewCache,
  type ChannelSyncDepthState,
} from "$lib/channel-view-cache";
import { channelOrderFromList } from "$lib/workspace/channels";
import {
  buildOptimisticAcknowledgeSidebarList,
  isStillSelectedAfterAcknowledgeSuccess,
  matchesAcknowledgedFilterVideo,
  resolveRevertedVideoForAcknowledge,
  resolveVideoForAcknowledgeToggle,
  selectionDroppedAfterAcknowledgeOptimistic,
} from "$lib/workspace/acknowledge-toggle";
import {
  loadChannelSnapshotWithRefresh,
  resolveNextChannelSelection,
} from "$lib/workspace/route-helpers";
import { shouldRetryReadySummaryLoad } from "$lib/workspace/content";
import {
  mergeVideoHighlights,
  removeVideoHighlightFromState,
} from "$lib/workspace/highlight-actions";
import { createSidebarState } from "$lib/workspace/sidebar-state.svelte";
import { mobileBottomBar } from "$lib/mobile-navigation/mobileBottomBar";
import {
  type AcknowledgedFilter,
  type ChannelSortMode,
  type WorkspaceContentMode,
  isWorkspaceContentMode,
  isWorkspaceVideoTypeFilter,
  resolveAcknowledgedParam,
} from "$lib/workspace/types";
import { createGuideState } from "$lib/workspace/guide-state.svelte";
import { createHomeTourSteps } from "$lib/workspace/home-tour";
import { createContentState } from "$lib/workspace/content-state.svelte";
import { DASTILL_SET_WORKSPACE_CONTENT_MODE_EVENT } from "$lib/utils/keyboard-shortcuts";

export function createHomeWorkspacePage() {
  // Lazy-loaded dynamic components (Vite code-split boundaries)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let WorkspaceSearchBarComponent = $state<Component<any> | null>(null);

  const CHANNEL_REFRESH_TTL_MS = 5 * 60 * 1000;
  const SELECTED_VIDEO_SCAN_PAGE_LIMIT = 8;
  const channelLastRefreshedAt = new Map<string, number>();
  let preferencesSaveTimer: ReturnType<typeof setTimeout> | null = null;
  /** When false, skip PUT /api/preferences so a fast debounce cannot overwrite the server before GET preferences returns. */
  let preferencesHydrated = $state(false);

  type CachedChannelVideoState = {
    videos: Video[];
    offset: number;
    hasMore: boolean;
    historyExhausted: boolean;
    backfillingHistory: boolean;
    allowLoadedVideoSyncDepthOverride: boolean;
    syncDepth: ChannelSyncDepthState | null;
  };

  const channelVideoStateCache =
    createChannelViewCache<CachedChannelVideoState>((state) => ({
      ...state,
      videos: cloneVideos(state.videos),
      syncDepth: cloneSyncDepthState(state.syncDepth),
    }));

  let aiAvailable = $state<boolean | null>(null);
  let aiStatus = $state<AiStatus | null>(null);
  let searchStatus = $state<SearchStatus | null>(null);
  let vocabularyReplacements = $state<VocabularyReplacement[]>([]);

  let errorMessage = $state<string | null>(null);
  let showDeleteAccessPrompt = $state(false);
  let addSourceFeedback = $state<AddSourceFeedback | null>(null);
  let addSourceFeedbackDismissed = $state(false);
  let showResetVideoConfirmation = $state(false);
  let addSourceFeedbackPollSequence = 0;

  let allowLoadedVideoSyncDepthOverride = $state(false);
  /**
   * Default backend expensive limit is 120/min per client (`EXPENSIVE_RATE_LIMIT_PER_MINUTE`).
   * Space POST /backfill calls so mobile auto-load does not burst 429s.
   */
  const MIN_BACKFILL_INTERVAL_MS = 2100;
  let lastBackfillRequestAtMs = 0;

  /** Matches Tailwind `lg` (mobile-only UI). Used to avoid racing desktop channel snapshot loads. */
  let mobileViewportMq = $state(false);
  let mobileBrowseOpen = $state(true);

  let videoHighlightsByVideoId = $state<Record<string, Highlight[]>>({});
  let nextOptimisticHighlightId = -1;
  let creatingHighlight = $state(false);
  let creatingHighlightVideoId = $state<string | null>(null);
  let creatingVocabularyReplacement = $state(false);
  let vocabularyModalSource = $state<string | null>(null);
  let vocabularyModalValue = $state("");
  let deletingHighlightId = $state<number | null>(null);

  let workspaceStateHydrated = $state(false);
  /** SvelteKit's replaceState throws until the client router has started; sidebar restore runs in onMount before that. */
  let shallowUrlSyncReady = $state(false);
  let viewUrlHydrated = $state(false);
  let pendingSelectedVideo = $state<Video | null>(null);
  const workspaceStorageKey = $derived(
    getScopedStorageKey(
      "dastill.workspace.state.v1",
      getAuthStorageScopeKey(authState.current),
    ),
  );
  const workspaceCacheScopeKey = $derived(
    getAuthStorageScopeKey(authState.current),
  );

  // Sidebar State (using unified composable)
  const sidebarState = createSidebarState({
    initialChannelId: page.data.selectedChannelId,
    initialVideoId: page.data.selectedVideoId,
    initialVideoTypeFilter: page.data.videoTypeFilter ?? "all",
    initialAcknowledgedFilter: page.data.acknowledgedFilter ?? "all",
    onSelectVideo: (videoId: string, context?: { forceReload?: boolean }) => {
      return selectVideo(videoId, true, context?.forceReload ?? false);
    },
    onChannelSelected: (channelId: string) => {
      if (!sidebarState.selectedVideoId) {
        content.resetSummaryQuality();
        content.videoInfo = null;
      }
      const href = buildWorkspaceViewHref({
        selectedChannelId: channelId,
        selectedVideoId: sidebarState.selectedVideoId,
        contentMode,
        videoTypeFilter: sidebarState.videoTypeFilter,
        acknowledgedFilter: sidebarState.acknowledgedFilter,
      });
      replaceWorkspaceUrl(href);
    },
    onChannelDeleted: (channelId: string) => {
      if (sidebarState.selectedChannelId === channelId) {
        const nextChannelId = resolveNextChannelSelection(
          sidebarState.channels,
          channelId,
        );
        if (nextChannelId) {
          void sidebarState.selectChannel(nextChannelId);
        } else {
          sidebarState.setSelectedChannelId(null);
          sidebarState.setSelectedVideoId(null);
          sidebarState.setVideos([]);
          sidebarState.setSyncDepth(null);
          clearSelectedVideoState();
        }
      }
    },
    onVideoTypeFilterChange: (value: VideoTypeFilter) => {
      const href = buildWorkspaceViewHref({
        selectedChannelId: sidebarState.selectedChannelId,
        selectedVideoId,
        contentMode,
        videoTypeFilter: value,
        acknowledgedFilter: sidebarState.acknowledgedFilter,
      });
      replaceWorkspaceUrl(href);
    },
    onAcknowledgedFilterChange: (value: AcknowledgedFilter) => {
      const href = buildWorkspaceViewHref({
        selectedChannelId: sidebarState.selectedChannelId,
        selectedVideoId,
        contentMode,
        videoTypeFilter: sidebarState.videoTypeFilter,
        acknowledgedFilter: value,
      });
      replaceWorkspaceUrl(href);
    },
    onOpenChannelOverview: async (channelId: string) => {
      await goto(`/channels/${encodeURIComponent(channelId)}`);
    },
    onChannelAdded: (channel: Channel) => {
      void trackAddedChannel(channel);
    },
    onVideoAdded: (result: AddVideoResult) => {
      void trackAddedVideo(result);
    },
    onVideoListReset: () => {
      // Handled by sidebarState
    },
  });

  // Content State
  const content = createContentState({
    getSelectedVideoId: () => sidebarState.selectedVideoId,
    getSelectedChannelId: () => sidebarState.selectedChannelId,
    setVideoStatus: (videoId, transcriptStatus, summaryStatus) => {
      sidebarState.setVideoStatus(videoId, transcriptStatus, summaryStatus);
    },
    initialContentMode: page.data.contentMode ?? undefined,
  });

  // Derived state exported to UI
  const contentMode = $derived(content.contentMode);
  const loadingContent = $derived(content.loadingContent);
  const editing = $derived(content.editing);
  const contentText = $derived(content.contentText);
  const transcriptRenderMode = $derived(content.transcriptRenderMode);
  const videoInfo = $derived(content.videoInfo);
  const formattingContent = $derived(content.formattingContent);
  const formattingVideoId = $derived(content.formattingVideoId);
  const regeneratingSummaryVideoIds = $derived(
    content.regeneratingSummaryVideoIds,
  );
  const revertingContent = $derived(content.revertingContent);
  const revertingVideoId = $derived(content.revertingVideoId);
  const resettingVideo = $derived(content.resettingVideo);
  const resettingVideoId = $derived(content.resettingVideoId);
  const draft = $derived(content.draft);

  let videoAcknowledgeSeq = 0;
  let videoAcknowledgeSync = $state<{
    seq: number;
    video: Video;
    /** When false, only merge into per-channel lists; do not refetch (refetch would hit stale GET cache before PUT invalidates). */
    confirmed: boolean;
  } | null>(null);

  // Backward compatibility/UI specific derived states
  const selectedChannelId = $derived(sidebarState.selectedChannelId);
  const selectedChannel = $derived(sidebarState.selectedChannel);
  const videos = $derived(sidebarState.videos);
  const selectedVideoId = $derived(sidebarState.selectedVideoId);
  const selectedVideo = $derived(
    videos.find((video) => video.id === selectedVideoId) ??
      (pendingSelectedVideo?.id === selectedVideoId
        ? pendingSelectedVideo
        : null),
  );
  const selectedVideoHighlights = $derived(
    selectedVideoId ? (videoHighlightsByVideoId[selectedVideoId] ?? []) : [],
  );
  const contentHighlights = $derived(
    contentMode === "transcript" || contentMode === "summary"
      ? selectedVideoHighlights.filter(
          (highlight) => highlight.source === (contentMode as HighlightSource),
        )
      : [],
  );
  const videoTypeFilter = $derived(sidebarState.videoTypeFilter);
  const acknowledgedFilter = $derived(sidebarState.acknowledgedFilter);
  const syncDepth = $derived(sidebarState.syncDepth);
  const limit = $derived(sidebarState.limit);
  const offset = $derived(sidebarState.offset);
  const hasMore = $derived(sidebarState.hasMore);

  function getChannelViewKey(channelId: string) {
    const d = sidebarState.videoState.syncDepth;
    const syncKey = d
      ? `${d.earliest_sync_date ?? ""}:${d.earliest_sync_date_user_set}:${d.derived_earliest_ready_date ?? ""}`
      : "";
    return buildChannelViewCacheKey(
      channelId,
      sidebarState.videoState.backfillingHistory,
      sidebarState.videoState.videoTypeFilter,
      sidebarState.videoState.acknowledgedFilter,
      sidebarState.videoState.offset,
      syncKey,
    );
  }

  function restoreCachedChannelVideoState(state: CachedChannelVideoState) {
    sidebarState.setVideos(cloneVideos(state.videos));
    sidebarState.setOffset(state.offset);
    sidebarState.setHasMore(state.hasMore);
    sidebarState.setHistoryExhausted(state.historyExhausted);
    sidebarState.setBackfillingHistory(state.backfillingHistory);
    allowLoadedVideoSyncDepthOverride = state.allowLoadedVideoSyncDepthOverride;
    sidebarState.setSyncDepth(cloneSyncDepthState(state.syncDepth));
  }

  $effect(() => {
    if (!selectedChannelId) return;

    channelVideoStateCache.set(getChannelViewKey(selectedChannelId), {
      videos: cloneVideos(videos),
      offset,
      hasMore,
      historyExhausted: sidebarState.historyExhausted,
      backfillingHistory: sidebarState.backfillingHistory,
      allowLoadedVideoSyncDepthOverride,
      syncDepth: cloneSyncDepthState(syncDepth),
    });
  });

  async function loadSyncDepth() {
    if (!selectedChannelId) {
      sidebarState.setSyncDepth(null);
      return;
    }
    try {
      const depth = await getChannelSyncDepth(selectedChannelId);
      sidebarState.setSyncDepth(depth as ChannelSyncDepthState);
    } catch {
      sidebarState.setSyncDepth(null);
    }
  }

  async function handleChannelSyncDateSaved(channelId: string) {
    if (sidebarState.selectedChannelId === channelId) {
      await loadSyncDepth();
    }
  }

  function openGuide() {
    tour.open();
  }

  function closeGuide() {
    tour.close();
  }

  function setGuideStep(step: number) {
    tour.setStep(step);
  }

  function clearSelectedVideoState() {
    sidebarState.setSelectedVideoId(null);
    pendingSelectedVideo = null;
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
      pendingSelectedVideo = video;
      return video;
    } catch {
      return null;
    }
  }

  async function hydrateSelectedVideo(
    preferredVideoId: string | null,
    acknowledged: boolean | undefined,
  ) {
    if (videos.length === 0) {
      clearSelectedVideoState();
      return;
    }

    if (!preferredVideoId) {
      pendingSelectedVideo = null;
      void selectVideo(videos[0].id);
      return;
    }

    sidebarState.setSelectedVideoId(preferredVideoId);
    const cachedHighlights = videoHighlightsByVideoId[preferredVideoId];
    if (!cachedHighlights) {
      void hydrateVideoHighlights(preferredVideoId);
    }
    let hasSelectedVideo = videos.some(
      (video) => video.id === preferredVideoId,
    );
    let scannedPages = 0;
    const targetChannelId = selectedChannelId;
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
        sidebarState.setHasMore(next.has_more);
        break;
      }

      sidebarState.setVideos([...videos, ...next.videos]);
      sidebarState.setOffset(next.next_offset ?? offset + next.videos.length);
      sidebarState.setHasMore(next.has_more);
      hasSelectedVideo = videos.some((video) => video.id === preferredVideoId);
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

      void selectVideo(videos[0].id);
      return;
    }

    pendingSelectedVideo = null;
  }

  async function applyChannelSnapshot(
    channelId: string,
    snapshot: ChannelSnapshot,
    preferredVideoId: string | null,
    silent = false,
  ) {
    if (!silent) {
      sidebarState.setLoadingVideos(true);
      sidebarState.setSelectedVideoId(null);
      errorMessage = null;
    }
    try {
      if (selectedChannelId !== channelId) {
        return;
      }

      const isAck = resolveAcknowledgedParam(sidebarState.acknowledgedFilter);
      sidebarState.setSyncDepth(snapshot.sync_depth);
      allowLoadedVideoSyncDepthOverride = false;
      sidebarState.setVideos(snapshot.videos);
      sidebarState.setOffset(snapshot.videos.length);
      sidebarState.setHasMore(snapshot.videos.length === limit);
      track({
        event: "channel_snapshot_loaded",
        channel_id: channelId,
        video_count: snapshot.channel_video_count ?? snapshot.videos.length,
      });
      void putCachedViewSnapshot(
        buildWorkspaceSnapshotCacheKey(channelId, videoTypeFilter, isAck),
        snapshot,
        workspaceCacheScopeKey,
      );
      await hydrateSelectedVideo(preferredVideoId, isAck);
    } catch (error) {
      if (presentAuthRequiredNoticeIfNeeded(error)) {
        // Modal only; avoid duplicating sign-in copy in the workspace error strip.
      } else if (!silent || !errorMessage) {
        errorMessage = (error as Error).message;
      }
    } finally {
      if (!silent) {
        sidebarState.setLoadingVideos(false);
      }
    }
  }

  const selectedVideoYoutubeUrl = $derived(
    selectedVideoId
      ? `https://www.youtube.com/watch?v=${selectedVideoId}`
      : null,
  );
  const selectedOriginalTranscript = $derived(
    selectedVideoId
      ? (content.originalTranscriptByVideoId[selectedVideoId] ?? null)
      : null,
  );
  const hasUpdatedTranscript = $derived(
    contentMode === "transcript" &&
      selectedOriginalTranscript !== null &&
      content.contentText !== selectedOriginalTranscript,
  );
  const canRevertTranscript = $derived(
    contentMode === "transcript" &&
      selectedOriginalTranscript !== null &&
      (editing
        ? content.draft !== selectedOriginalTranscript
        : content.contentText !== selectedOriginalTranscript),
  );

  function storeVideoHighlights(videoId: string, highlights: Highlight[]) {
    videoHighlightsByVideoId = {
      ...videoHighlightsByVideoId,
      [videoId]: highlights,
    };
  }

  function mergeVideoHighlight(videoId: string, highlight: Highlight) {
    videoHighlightsByVideoId = mergeVideoHighlights(
      videoHighlightsByVideoId,
      videoId,
      highlight,
    );
  }

  function removeVideoHighlight(videoId: string, highlightId: number) {
    videoHighlightsByVideoId = removeVideoHighlightFromState(
      videoHighlightsByVideoId,
      videoId,
      highlightId,
    );
  }

  function persistSessionHighlightsIfNeeded() {
    if (!shouldUseSessionHighlights(authState.current)) return;
    const scope = resolveHighlightsScopeKey(authState.current);
    const existing = loadSessionHighlightsMap(scope);
    const next: Record<string, Highlight[]> = { ...existing };
    for (const [videoId, list] of Object.entries(videoHighlightsByVideoId)) {
      next[videoId] = list;
    }
    saveSessionHighlightsMap(scope, next);
  }

  async function hydrateVideoHighlights(
    videoId: string,
    options: { showError?: boolean } = {},
  ) {
    if (shouldUseSessionHighlights(authState.current)) {
      const scope = resolveHighlightsScopeKey(authState.current);
      const map = loadSessionHighlightsMap(scope);
      const highlights = map[videoId] ?? [];
      storeVideoHighlights(videoId, highlights);
      return highlights;
    }
    try {
      const highlights = await getVideoHighlights(videoId);
      storeVideoHighlights(videoId, highlights);
      return highlights;
    } catch (error) {
      if (options.showError) {
        if (!presentAuthRequiredNoticeIfNeeded(error)) {
          errorMessage = (error as Error).message;
        }
      }
      return null;
    }
  }

  async function saveSelectionHighlight(payload: CreateHighlightRequest) {
    if (
      !selectedVideoId ||
      (contentMode !== "transcript" && contentMode !== "summary")
    ) {
      return;
    }

    const targetVideoId = selectedVideoId;
    const optimisticHighlight = buildOptimisticHighlight(
      targetVideoId,
      payload,
      nextOptimisticHighlightId,
    );
    nextOptimisticHighlightId -= 1;

    mergeVideoHighlight(targetVideoId, optimisticHighlight);
    creatingHighlight = true;
    creatingHighlightVideoId = targetVideoId;
    errorMessage = null;

    if (shouldUseSessionHighlights(authState.current)) {
      try {
        persistSessionHighlightsIfNeeded();
        if (selectedChannelId) {
          track({
            event: "highlight_created",
            video_id: targetVideoId,
            channel_id: selectedChannelId,
            source: payload.source,
          });
        }
      } finally {
        creatingHighlight = false;
        creatingHighlightVideoId = null;
      }
      return;
    }

    if (!canManageLibrary) {
      removeVideoHighlight(targetVideoId, optimisticHighlight.id);
      creatingHighlight = false;
      creatingHighlightVideoId = null;
      return;
    }

    try {
      const highlight = await createHighlight(targetVideoId, payload);
      storeVideoHighlights(
        targetVideoId,
        reconcileOptimisticHighlight(
          videoHighlightsByVideoId[targetVideoId] ?? [],
          optimisticHighlight.id,
          highlight,
        ),
      );
      if (selectedChannelId) {
        track({
          event: "highlight_created",
          video_id: targetVideoId,
          channel_id: selectedChannelId,
          source: payload.source,
        });
      }
    } catch (error) {
      removeVideoHighlight(targetVideoId, optimisticHighlight.id);
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    } finally {
      creatingHighlight = false;
      creatingHighlightVideoId = null;
    }
  }

  async function saveVocabularyReplacement(selectedText: string) {
    const source = selectedText.trim();
    if (!source) {
      return;
    }
    vocabularyModalSource = source;
    vocabularyModalValue = source;
  }

  function closeVocabularyModal() {
    if (creatingVocabularyReplacement) {
      return;
    }
    vocabularyModalSource = null;
    vocabularyModalValue = "";
  }

  async function confirmVocabularyReplacement() {
    const source = vocabularyModalSource?.trim();
    const replacement = vocabularyModalValue.trim();
    if (!source || !replacement) {
      return;
    }

    const nextReplacements = upsertVocabularyReplacement(
      vocabularyReplacements,
      {
        from: source,
        to: replacement,
        added_at: new Date().toISOString(),
      },
    );
    if (nextReplacements === vocabularyReplacements) {
      return;
    }

    creatingVocabularyReplacement = true;
    errorMessage = null;

    try {
      vocabularyReplacements = nextReplacements;
      await savePreferences({
        channel_order: sidebarState.channelOrder,
        channel_sort_mode: sidebarState.channelSortMode,
        vocabulary_replacements: nextReplacements,
      });
      vocabularyModalSource = null;
      vocabularyModalValue = "";
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    } finally {
      creatingVocabularyReplacement = false;
      if (preferencesSaveTimer) {
        clearTimeout(preferencesSaveTimer);
        preferencesSaveTimer = null;
      }
    }
  }

  async function deleteExistingHighlight(highlightId: number) {
    const targetVideoId =
      selectedVideoId ??
      Object.keys(videoHighlightsByVideoId).find((videoId) =>
        (videoHighlightsByVideoId[videoId] ?? []).some(
          (h) => Number(h.id) === highlightId,
        ),
      );
    if (!targetVideoId) {
      return;
    }
    deletingHighlightId = highlightId;
    errorMessage = null;

    if (shouldUseSessionHighlights(authState.current)) {
      try {
        removeVideoHighlight(targetVideoId, highlightId);
        persistSessionHighlightsIfNeeded();
      } finally {
        deletingHighlightId = null;
      }
      return;
    }

    if (!canManageLibrary) {
      deletingHighlightId = null;
      return;
    }

    try {
      const { deleteHighlight } = await import("$lib/api");
      await deleteHighlight(highlightId);
      removeVideoHighlight(targetVideoId, highlightId);
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    } finally {
      deletingHighlightId = null;
    }
  }

  function syncChannelOrderFromList() {
    sidebarState.setChannelOrder(channelOrderFromList(sidebarState.channels));
  }

  function restoreWorkspaceState() {
    const urlState: Partial<WorkspaceViewState> = {};
    if (page.data.selectedChannelId)
      urlState.selectedChannelId = page.data.selectedChannelId;
    if (page.data.selectedVideoId)
      urlState.selectedVideoId = page.data.selectedVideoId;
    if (page.data.contentMode) urlState.contentMode = page.data.contentMode;
    if (page.data.videoTypeFilter)
      urlState.videoTypeFilter = page.data.videoTypeFilter;
    if (page.data.acknowledgedFilter)
      urlState.acknowledgedFilter = page.data.acknowledgedFilter;

    const restored = mergeWorkspaceViewState(
      restoreWorkspaceSnapshot(
        typeof localStorage === "undefined"
          ? null
          : loadWorkspaceState(localStorage, workspaceStorageKey),
        {
          includeSelectedVideoId: true,
          includeContentMode: true,
          includeVideoTypeFilter: true,
          includeAcknowledgedFilter: true,
          includeChannelSortMode: true,
        },
      ),
      urlState,
    );

    if ("selectedChannelId" in restored) {
      sidebarState.setSelectedChannelId(restored.selectedChannelId ?? null);
    }
    if ("selectedVideoId" in restored) {
      sidebarState.setSelectedVideoId(restored.selectedVideoId ?? null);
    }
    if (restored.contentMode && isWorkspaceContentMode(restored.contentMode)) {
      content.contentMode = restored.contentMode;
    }
    if (
      restored.videoTypeFilter &&
      isWorkspaceVideoTypeFilter(restored.videoTypeFilter)
    ) {
      sidebarState.setVideoTypeFilter(restored.videoTypeFilter);
    }
    if (restored.acknowledgedFilter) {
      sidebarState.setAcknowledgedFilter(restored.acknowledgedFilter);
    }
    if (restored.channelSortMode) {
      sidebarState.setChannelSortMode(restored.channelSortMode);
    }
    if (Array.isArray(restored.channelOrder)) {
      sidebarState.setChannelOrder(restored.channelOrder);
    }

    const url =
      typeof window !== "undefined" ? new URL(window.location.href) : null;
    const videoInUrl = Boolean(url?.searchParams.get("video")?.trim());

    if (sidebarState.selectedVideoId) {
      const showVideoPanel = !mobileViewportMq || videoInUrl;
      mobileBrowseOpen = !showVideoPanel;
    } else {
      mobileBrowseOpen = true;
    }
  }

  function replaceWorkspaceUrl(href: string) {
    if (!shallowUrlSyncReady || typeof window === "undefined") return;
    const nextUrl = new URL(href, window.location.origin);
    if (
      nextUrl.pathname === window.location.pathname &&
      nextUrl.search === window.location.search
    ) {
      return;
    }
    replacePageState(
      `${nextUrl.pathname}${nextUrl.search}${nextUrl.hash}`,
      history.state,
    );
  }

  function persistViewUrl() {
    if (
      !viewUrlHydrated ||
      !shallowUrlSyncReady ||
      typeof window === "undefined"
    )
      return;
    const omitVideoFromUrl = mobileViewportMq && mobileBrowseOpen;
    const nextHref = buildWorkspaceViewHref({
      selectedChannelId: sidebarState.selectedChannelId,
      selectedVideoId: omitVideoFromUrl ? null : sidebarState.selectedVideoId,
      contentMode,
      videoTypeFilter: sidebarState.videoTypeFilter,
      acknowledgedFilter: sidebarState.acknowledgedFilter,
    });
    replaceWorkspaceUrl(nextHref);
  }

  function persistWorkspaceState() {
    if (!workspaceStateHydrated || typeof localStorage === "undefined") return;
    const snapshot: WorkspaceStateSnapshot = {
      selectedChannelId: sidebarState.selectedChannelId,
      selectedVideoId: sidebarState.selectedVideoId,
      contentMode,
      videoTypeFilter: sidebarState.videoTypeFilter,
      acknowledgedFilter: sidebarState.acknowledgedFilter,
      channelOrder: sidebarState.channelOrder,
      channelSortMode: sidebarState.channelSortMode,
    };
    saveWorkspaceState(localStorage, snapshot, workspaceStorageKey);
    // Debounce-persist channel order + sort mode to the backend so it survives
    // across devices/browsers. 1 s delay avoids bursting on rapid reorders.
    // Anonymous sessions use localStorage only; preferences API requires sign-in.
    if (!preferencesHydrated) return;
    if (authState.current.authState !== "authenticated") return;
    if (preferencesSaveTimer) clearTimeout(preferencesSaveTimer);
    preferencesSaveTimer = setTimeout(() => {
      if (authState.current.authState !== "authenticated") {
        preferencesSaveTimer = null;
        return;
      }
      void savePreferences({
        channel_order: sidebarState.channelOrder,
        channel_sort_mode: sidebarState.channelSortMode,
        vocabulary_replacements: vocabularyReplacements,
      }).catch((err) => {
        if (presentAuthRequiredNoticeIfNeeded(err)) return;
        throw err;
      });
      preferencesSaveTimer = null;
    }, 1000);
  }

  $effect(() => {
    persistWorkspaceState();
  });

  $effect(() => {
    persistViewUrl();
  });

  // Tour State
  const tour = createGuideState(10);
  const tourSteps = createHomeTourSteps({
    get mobileBrowseOpen() {
      return mobileBrowseOpen;
    },
    set mobileBrowseOpen(v) {
      mobileBrowseOpen = v;
    },
    get selectedVideoId() {
      return sidebarState.selectedVideoId;
    },
    get selectedChannelId() {
      return sidebarState.selectedChannelId;
    },
    get videos() {
      return sidebarState.videos;
    },
    get contentMode() {
      return content.contentMode;
    },
    isAuthenticated: () => authState.current.authState === "authenticated",
    selectVideo: (id, from, force) => selectVideo(id, from, force),
    setMode: (m) => {
      content.contentMode = m as WorkspaceContentMode;
    },
    tick,
  });

  const guideOpen = $derived(tour.isOpen);
  const guideStep = $derived(tour.step);

  onMount(() => {
    const mq = window.matchMedia("(max-width: 1023px)");
    mobileViewportMq = mq.matches;
    const onViewportChange = () => {
      mobileViewportMq = mq.matches;
    };
    mq.addEventListener("change", onViewportChange);

    restoreWorkspaceState();
    const unsubBrowseIntent = mobileWorkspaceBrowseIntent.subscribe(
      (wantsBrowse) => {
        if (!wantsBrowse) return;
        mobileBrowseOpen = true;
        mobileWorkspaceBrowseIntent.set(false);
      },
    );
    workspaceStateHydrated = true;
    setTimeout(() => {
      shallowUrlSyncReady = true;
      persistViewUrl();
    }, 0);
    void (async () => {
      try {
        const selectedChannelIdAtMount = selectedChannelId;
        const selectedVideoIdAtMount = selectedVideoId;
        const acknowledgedAtMount =
          resolveAcknowledgedParam(acknowledgedFilter);

        const [bootstrapResult, apiPreferences] = await Promise.all([
          resolveBootstrapOnMount({
            serverBootstrap: page.data.bootstrap ?? null,
            selectedChannelId: selectedChannelIdAtMount,
            workspaceCacheScopeKey,
            viewSnapshotCacheKey: sidebarState.selectedChannelId
              ? buildWorkspaceSnapshotCacheKey(
                  sidebarState.selectedChannelId,
                  sidebarState.videoTypeFilter,
                  acknowledgedAtMount,
                )
              : null,
          }),
          getPreferences().catch(() => null),
        ]);

        if (apiPreferences && authState.current.authState === "authenticated") {
          if (apiPreferences.channel_order.length > 0) {
            sidebarState.setChannelOrder(apiPreferences.channel_order);
          }
          sidebarState.setChannelSortMode(
            apiPreferences.channel_sort_mode as ChannelSortMode,
          );
          vocabularyReplacements = apiPreferences.vocabulary_replacements ?? [];
        }

        const hasInitialData = Boolean(
          bootstrapResult.channels && bootstrapResult.channels.length > 0,
        );

        if (bootstrapResult.channels && bootstrapResult.channels.length > 0) {
          sidebarState.setChannels(
            applySavedChannelOrder(
              bootstrapResult.channels,
              sidebarState.channelOrder,
            ),
          );
          syncChannelOrderFromList();
        }

        if (bootstrapResult.aiAvailable !== null) {
          aiAvailable = bootstrapResult.aiAvailable;
        }
        if (bootstrapResult.aiStatus !== null) {
          aiStatus = bootstrapResult.aiStatus;
        }
        if (bootstrapResult.searchStatus !== null) {
          searchStatus = bootstrapResult.searchStatus;
        }

        if (
          bootstrapResult.snapshot &&
          selectedChannelIdAtMount &&
          bootstrapResult.snapshot.channel_id === selectedChannelIdAtMount
        ) {
          await applyChannelSnapshot(
            selectedChannelIdAtMount,
            bootstrapResult.snapshot,
            selectedVideoIdAtMount,
            true,
          );
        }

        void loadBootstrapRefresh({ silent: hasInitialData }).finally(() => {
          viewUrlHydrated = true;
        });
      } finally {
        preferencesHydrated = true;
      }
    })();

    tour.restoreFromUrl();

    void import("$lib/components/workspace/WorkspaceSearchBar.svelte").then(
      (m) => {
        WorkspaceSearchBarComponent = m.default;
      },
    );

    return () => {
      mq.removeEventListener("change", onViewportChange);
      unsubBrowseIntent();
      if (preferencesSaveTimer) {
        clearTimeout(preferencesSaveTimer);
        preferencesSaveTimer = null;
      }
    };
  });

  function buildWorkspaceSnapshotCacheKey(
    channelId: string,
    type: VideoTypeFilter,
    acknowledged: boolean | undefined,
  ) {
    const acknowledgedKey =
      acknowledged === undefined ? "all" : acknowledged ? "ack" : "unack";
    return `workspace:${channelId}:type=${type}:ack=${acknowledgedKey}:limit=${limit}`;
  }

  async function handleSearchResultSelection(
    result: SearchResult,
    targetMode: "transcript" | "summary",
  ) {
    if (selectedChannelId !== result.channel_id) {
      await selectChannel(result.channel_id, result.video_id, true);
    } else {
      await selectVideo(result.video_id, true);
    }

    if (contentMode !== targetMode) {
      content.contentMode = targetMode;
      await content.loadContent();
    }
  }

  async function loadBootstrapRefresh(options?: { silent?: boolean }) {
    const silent = options?.silent ?? false;
    const previousSelectedChannelId = selectedChannelId;

    if (!silent) {
      sidebarState.setLoadingChannels(true);
      errorMessage = null;
    }

    try {
      const bootstrap = await getWorkspaceBootstrapWhenAvailable({
        selectedChannelId: previousSelectedChannelId,
        videoType: sidebarState.videoTypeFilter,
        acknowledged: resolveAcknowledgedParam(sidebarState.acknowledgedFilter),
        limit,
        retryDelayMs: 500,
      });

      sidebarState.setChannels(
        applySavedChannelOrder(bootstrap.channels, sidebarState.channelOrder),
      );
      syncChannelOrderFromList();
      void putCachedChannels(sidebarState.channels, workspaceCacheScopeKey);

      aiAvailable = bootstrap.ai_available;
      aiStatus = bootstrap.ai_status;
      searchStatus = bootstrap.search_status;
      void putCachedBootstrapMeta(
        {
          ai_available: bootstrap.ai_available,
          ai_status: bootstrap.ai_status,
          search_status: bootstrap.search_status,
        },
        workspaceCacheScopeKey,
      );

      const selectionChannelId = sidebarState.selectedChannelId;
      const selectionVideoId = sidebarState.selectedVideoId;

      const initialChannelId = resolveInitialChannelSelection(
        bootstrap.channels,
        selectionChannelId ?? previousSelectedChannelId,
        selectionChannelId,
      );

      if (!initialChannelId) {
        sidebarState.setSelectedChannelId(null);
        mobileBrowseOpen = true;
        clearSelectedVideoState();
        sidebarState.setVideos([]);
        sidebarState.setSyncDepth(null);
        sidebarState.setOffset(0);
        sidebarState.setHasMore(true);
        sidebarState.setHistoryExhausted(false);
        sidebarState.setBackfillingHistory(false);
        allowLoadedVideoSyncDepthOverride = false;
      } else {
        const preferredVideoId =
          initialChannelId === selectionChannelId ? selectionVideoId : null;
        const canReuseRenderedSnapshot =
          initialChannelId === selectionChannelId &&
          sidebarState.videos.length > 0;

        sidebarState.setSelectedChannelId(initialChannelId);

        if (!silent || preferredVideoId !== selectionVideoId) {
          content.resetSummaryQuality();
          content.videoInfo = null;
          content.editing = false;
          content.clearFormattingFeedback();
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
          sidebarState.setSelectedVideoId(preferredVideoId);
          sidebarState.setVideos([]);
          sidebarState.setOffset(0);
          sidebarState.setHasMore(true);
          sidebarState.setHistoryExhausted(false);
          sidebarState.setBackfillingHistory(false);
          allowLoadedVideoSyncDepthOverride = false;
          sidebarState.setSyncDepth(null);
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
        // Modal only
      } else if (!silent || !errorMessage) {
        errorMessage = (error as Error).message;
      }
    } finally {
      if (!silent) {
        sidebarState.setLoadingChannels(false);
        sidebarState.setLoadingVideos(false);
      }
    }
  }

  async function handleDeleteChannel(channelId: string) {
    if (!canManageLibrary) {
      showDeleteAccessPrompt = true;
      return;
    }

    sidebarState.setChannelIdToDelete(channelId);
    sidebarState.setShowDeleteConfirmation(true);
  }

  async function confirmDeleteChannel() {
    if (!sidebarState.channelIdToDelete || !canManageLibrary) return;
    const channelId = sidebarState.channelIdToDelete;
    const channelViewKey = getChannelViewKey(channelId);
    sidebarState.setShowDeleteConfirmation(false);
    sidebarState.setChannelIdToDelete(null);

    const previousChannels = [...sidebarState.channels];
    const previousSelectedChannelId = sidebarState.selectedChannelId;

    sidebarState.removeChannel(channelId);

    if (selectedChannelId === channelId) {
      const nextChannelId = resolveNextChannelSelection(
        previousChannels,
        channelId,
      );
      if (nextChannelId) {
        await selectChannel(nextChannelId);
      } else {
        sidebarState.setSelectedChannelId(null);
        sidebarState.setSelectedVideoId(null);
        mobileBrowseOpen = true;
        sidebarState.setVideos([]);
        content.contentText = "";
        content.draft = "";
      }
    }

    try {
      await deleteChannel(channelId);
      void removeCachedChannel(channelId, workspaceCacheScopeKey);
      channelVideoStateCache.delete(channelViewKey);
    } catch (error) {
      sidebarState.setChannels(previousChannels);
      sidebarState.setSelectedChannelId(previousSelectedChannelId);
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    }
  }

  async function selectChannel(
    channelId: string | null,
    videoId: string | null = null,
    _scroll = true,
  ) {
    if (sidebarState.selectedChannelId === channelId && !videoId) return;

    sidebarState.setSelectedChannelId(channelId);
    if (!channelId) return;

    if (!videoId) {
      clearSelectedVideoState();
    }

    const channelViewKey = getChannelViewKey(channelId);
    const cachedChannelVideoState = channelVideoStateCache.get(channelViewKey);
    const hasCachedChannelVideoState =
      !!cachedChannelVideoState && cachedChannelVideoState.videos.length > 0;

    content.clearFormattingFeedback();
    if (hasCachedChannelVideoState && cachedChannelVideoState) {
      restoreCachedChannelVideoState(cachedChannelVideoState);
      sidebarState.setLoadingVideos(false);
      void refreshAndLoadVideos(channelId, false, videoId, true);
      return;
    }

    sidebarState.setVideos([]);
    sidebarState.setOffset(0);
    sidebarState.setHasMore(true);
    sidebarState.historyExhausted = false;
    sidebarState.backfillingHistory = false;
    allowLoadedVideoSyncDepthOverride = false;
    await refreshAndLoadVideos(channelId, false, videoId);
  }

  async function refreshAndLoadVideos(
    channelId: string,
    bypassTtl = false,
    preferredVideoId: string | null = selectedVideoId,
    silentInitialSnapshot = false,
  ) {
    const isAck = resolveAcknowledgedParam(acknowledgedFilter);
    await loadChannelSnapshotWithRefresh({
      channelId,
      refreshedAtByChannel: channelLastRefreshedAt,
      ttlMs: CHANNEL_REFRESH_TTL_MS,
      bypassTtl,
      initialSilent: silentInitialSnapshot,
      getMutationEpoch: () => sidebarState.getVideoListMutationEpoch(),
      loadSnapshot: () =>
        getChannelSnapshot(channelId, {
          limit,
          offset,
          videoType: videoTypeFilter,
          acknowledged: isAck,
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
        if (!errorMessage) {
          errorMessage = message;
        }
      },
    });
  }

  async function loadVideos(reset = false, silent = false) {
    if (!selectedChannelId) return;
    if (!sidebarState.selectedChannelId) return;
    if (sidebarState.loadingVideos && !silent) return;

    if (!silent) sidebarState.setLoadingVideos(true);
    if (!silent) errorMessage = null;

    try {
      const isAck = resolveAcknowledgedParam(sidebarState.acknowledgedFilter);
      const list = await listVideos(
        sidebarState.selectedChannelId,
        limit,
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

      if (reset) {
        sidebarState.setVideos(list.videos);
        sidebarState.setOffset(list.next_offset ?? list.videos.length);
      } else {
        sidebarState.setVideos([...sidebarState.videos, ...list.videos]);
        sidebarState.setOffset(
          list.next_offset ?? sidebarState.offset + list.videos.length,
        );
      }
      sidebarState.setHasMore(list.has_more);
      if (reset) {
        allowLoadedVideoSyncDepthOverride = false;
      }

      if (reset) {
        await hydrateSelectedVideo(sidebarState.selectedVideoId, isAck);
      }
    } catch (error) {
      if (presentAuthRequiredNoticeIfNeeded(error)) {
        // Modal only
      } else if (!silent || !errorMessage) {
        errorMessage = (error as Error).message;
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
    )
      return;

    if (sidebarState.hasMore) {
      await loadVideos(false);
      allowLoadedVideoSyncDepthOverride = true;
      return;
    }

    sidebarState.setBackfillingHistory(true);
    errorMessage = null;

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
        sidebarState.setHistoryExhausted(true);
      }

      await loadVideos(false);
      await loadSyncDepth();
      allowLoadedVideoSyncDepthOverride = true;
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    } finally {
      sidebarState.setBackfillingHistory(false);
    }
  }

  async function loadAllVideosForMobileBrowse(isAborted: () => boolean) {
    const channelId = sidebarState.selectedChannelId;
    if (!channelId || !mobileBrowseOpen || !mobileViewportMq) return;

    let bootWait = 0;
    while (bootWait++ < 100 && !isAborted()) {
      if (
        !mobileBrowseOpen ||
        sidebarState.selectedChannelId !== channelId ||
        !mobileViewportMq
      ) {
        return;
      }
      const hasList = sidebarState.videos.length > 0 || !sidebarState.hasMore;
      if (hasList) break;
      await tick();
      await new Promise((r) => setTimeout(r, 40));
    }

    let safety = 0;
    while (safety++ < 2000 && !isAborted()) {
      if (
        !mobileBrowseOpen ||
        sidebarState.selectedChannelId !== channelId ||
        !mobileViewportMq
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
        await new Promise((r) => setTimeout(r, 30));
      }
      if (isAborted()) return;
      if (!mobileBrowseOpen || sidebarState.selectedChannelId !== channelId) {
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
    if (!channelId || !mobileBrowseOpen || !mobileViewportMq) return;

    let safety = 0;
    while (sidebarState.hasMore && safety++ < 500) {
      if (
        sidebarState.selectedChannelId !== channelId ||
        !mobileBrowseOpen ||
        !mobileViewportMq
      ) {
        return;
      }
      while (sidebarState.loadingVideos || sidebarState.backfillingHistory) {
        await tick();
        await new Promise((r) => setTimeout(r, 30));
      }
      if (
        sidebarState.selectedChannelId !== channelId ||
        !mobileBrowseOpen ||
        !mobileViewportMq
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
    if (fromUserInteraction) mobileBrowseOpen = false;
    if (contentMode === "summary" && selectedVideoId) {
      closeSummarySession();
    }
    sidebarState.setSelectedVideoId(videoId);
    if (videoId && selectedChannelId) {
      track({
        event: "video_opened",
        video_id: videoId,
        channel_id: selectedChannelId,
      });
    }
    content.contentText = "";
    content.draft = "";
    const cachedHighlights = videoId ? videoHighlightsByVideoId[videoId] : null;
    if (videoId && !cachedHighlights) {
      void hydrateVideoHighlights(videoId);
    }
    // Do not gate on `selectedVideoId` ($derived) vs `videoId`: the derived value can
    // lag immediately after setSelectedVideoId, skipping loadContent and leaving the
    // main panel empty while the sidebar and URL still show a video.
    content.resetSummaryQuality();
    content.videoInfo = null;
    content.editing = false;
    content.clearFormattingFeedback();
    await content.loadContent();
  }

  async function setMode(mode: WorkspaceContentMode) {
    if (contentMode === mode) return;
    const previousMode = contentMode;
    if (previousMode === "summary" && selectedVideoId) {
      closeSummarySession();
    }
    content.contentMode = mode;
    if (selectedVideoId && selectedChannelId) {
      track({
        event: "content_mode_changed",
        video_id: selectedVideoId,
        channel_id: selectedChannelId,
        from_mode: previousMode,
        to_mode: mode,
      });
    }
    content.resetSummaryQuality();
    content.videoInfo = null;
    content.editing = false;
    content.clearFormattingFeedback();
    await content.loadContent();
  }

  onMount(() => {
    const handler = (event: Event) => {
      const e = event as CustomEvent<{ mode?: unknown }>;
      const mode = e.detail?.mode;
      if (!isWorkspaceContentMode(mode)) return;
      void setMode(mode);
    };
    window.addEventListener(DASTILL_SET_WORKSPACE_CONTENT_MODE_EVENT, handler);
    return () =>
      window.removeEventListener(
        DASTILL_SET_WORKSPACE_CONTENT_MODE_EVENT,
        handler,
      );
  });

  const canManageLibrary = $derived(
    authState.current.authState === "authenticated",
  );
  const aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  const contentHtml = $derived(renderMarkdown(content.contentText));

  function presentAddSourceFeedback(next: AddSourceFeedback) {
    addSourceFeedback = next;
    addSourceFeedbackDismissed = false;
  }

  function dismissAddSourceFeedback() {
    addSourceFeedbackDismissed = true;
    if (addSourceFeedback?.status !== "loading") {
      addSourceFeedback = null;
    }
  }

  async function trackAddedVideo(result: AddVideoResult) {
    const sequence = ++addSourceFeedbackPollSequence;
    let nextResult = result;

    presentAddSourceFeedback(
      buildVideoAddFeedback(
        nextResult,
        resolveAddedVideoStatus(nextResult.video),
      ),
    );

    while (sequence === addSourceFeedbackPollSequence) {
      const currentStatus = resolveAddedVideoStatus(nextResult.video);
      if (currentStatus !== "loading") {
        return;
      }

      await new Promise((resolve) => window.setTimeout(resolve, 4000));
      if (sequence !== addSourceFeedbackPollSequence) {
        return;
      }

      try {
        const refreshedVideo = await getVideo(nextResult.video.id, true);
        nextResult = { ...nextResult, video: refreshedVideo };
        presentAddSourceFeedback(
          buildVideoAddFeedback(
            nextResult,
            resolveAddedVideoStatus(refreshedVideo),
          ),
        );
      } catch {
        // Keep polling quietly; the initial acceptance feedback already surfaced.
      }
    }
  }

  async function trackAddedChannel(channel: Channel) {
    const sequence = ++addSourceFeedbackPollSequence;
    presentAddSourceFeedback(buildChannelAddFeedback(channel, "loading"));

    while (sequence === addSourceFeedbackPollSequence) {
      await new Promise((resolve) => window.setTimeout(resolve, 4000));
      if (sequence !== addSourceFeedbackPollSequence) {
        return;
      }

      try {
        const videos = await listVideos(
          channel.id,
          1,
          0,
          "all",
          undefined,
          false,
          undefined,
          true,
        );
        const status = resolveAddedChannelStatus(videos.videos);
        presentAddSourceFeedback(buildChannelAddFeedback(channel, status));
        if (status === "ready") {
          return;
        }
      } catch {
        // Keep polling quietly; the initial acceptance feedback already surfaced.
      }
    }
  }

  async function openAddSourceFeedbackTarget() {
    const current = addSourceFeedback;
    if (!current) {
      return;
    }

    addSourceFeedbackPollSequence += 1;
    addSourceFeedback = null;
    addSourceFeedbackDismissed = false;

    if (current.kind === "video") {
      await sidebarState.selectChannel(
        current.targetChannelId,
        current.videoId,
        true,
      );
      await selectVideo(current.videoId, true, true);
      return;
    }

    mobileBrowseOpen = true;
    await sidebarState.selectChannel(current.channelId, null, true);
  }

  onMount(() => {
    return () => {
      addSourceFeedbackPollSequence += 1;
    };
  });

  $effect(() => {
    if (mobileBrowseOpen) {
      mobileBottomBar.set({ kind: "hidden" });
      return () => {
        mobileBottomBar.set({ kind: "sections" });
      };
    }

    const inVideoDetail =
      !mobileBrowseOpen && Boolean(selectedVideoId) && !editing;
    if (!inVideoDetail) {
      mobileBottomBar.set({ kind: "sections" });
    } else {
      mobileBottomBar.set({
        kind: "videoActions",
        youtubeUrl: selectedVideoYoutubeUrl,
        showRegenerate: contentMode === "summary",
        regenerating: selectedVideoId
          ? content.regeneratingSummaryVideoIds.includes(selectedVideoId)
          : false,
        aiAvailable: aiAvailable ?? false,
        onRegenerate: content.regenerateSummaryContent,
        showFormatAction: contentMode === "transcript",
        formatting:
          content.formattingContent &&
          content.formattingVideoId === selectedVideoId,
        onFormat: content.cleanFormatting,
        showRevertAction: hasUpdatedTranscript,
        reverting:
          content.revertingContent &&
          content.revertingVideoId === selectedVideoId,
        canRevert: canRevertTranscript,
        onRevert: content.revertToOriginalTranscript,
        busy: loadingContent,
        onRequestResetVideo: () => {
          showResetVideoConfirmation = true;
        },
        resetting:
          content.resettingVideo &&
          content.resettingVideoId === selectedVideoId,
        showAcknowledgeToggle: true,
        acknowledged: selectedVideo?.acknowledged ?? false,
        onAcknowledgeToggle: toggleAcknowledge,
        showEditAction:
          contentMode === "transcript" || contentMode === "summary",
        onEdit: content.startEdit,
      });
    }
    return () => {
      mobileBottomBar.set({ kind: "sections" });
    };
  });

  $effect(() => {
    if (
      !mobileViewportMq ||
      !mobileBrowseOpen ||
      !sidebarState.selectedChannelId
    ) {
      return;
    }

    let cancelled = false;
    void (async () => {
      await tick();
      await loadAllVideosForMobileBrowse(() => cancelled);
    })();

    return () => {
      cancelled = true;
    };
  });

  async function toggleAcknowledge() {
    if (!sidebarState.selectedVideoId) return;
    const targetVideoId = sidebarState.selectedVideoId;
    const resolved = resolveVideoForAcknowledgeToggle(
      sidebarState.videos,
      targetVideoId,
      pendingSelectedVideo,
    );
    if (!resolved) return;
    const { video, videoFromList } = resolved;

    errorMessage = null;

    const previousVideos = [...sidebarState.videos];
    const previousPendingSelectedVideo = pendingSelectedVideo;
    const previousSelectedVideoId = sidebarState.selectedVideoId;
    const newAcknowledged = !video.acknowledged;

    sidebarState.bumpVideoListMutationEpoch();

    const optimisticVideo = { ...video, acknowledged: newAcknowledged };
    const optimisticList = buildOptimisticAcknowledgeSidebarList(
      videoFromList,
      previousVideos,
      sidebarState.videos,
      targetVideoId,
      newAcknowledged,
      sidebarState.acknowledgedFilter,
    );
    if (videoFromList) {
      sidebarState.setVideos(optimisticList);
    } else {
      pendingSelectedVideo = optimisticVideo;
    }
    videoAcknowledgeSeq += 1;
    videoAcknowledgeSync = {
      seq: videoAcknowledgeSeq,
      video: optimisticVideo,
      confirmed: false,
    };

    const selectionDroppedFromFilter =
      selectionDroppedAfterAcknowledgeOptimistic(
        videoFromList,
        optimisticList,
        previousSelectedVideoId,
        optimisticVideo,
        sidebarState.acknowledgedFilter,
      );
    if (selectionDroppedFromFilter) {
      content.editing = false;
      content.clearFormattingFeedback();
      if (videoFromList) {
        if (optimisticList.length === 0) {
          sidebarState.setSelectedVideoId(null);
          content.contentText = "";
          content.draft = "";
        } else {
          await selectVideo(optimisticList[0].id);
        }
      } else {
        sidebarState.setSelectedVideoId(null);
        pendingSelectedVideo = null;
        content.contentText = "";
        content.draft = "";
      }
    }

    try {
      const updated = await updateAcknowledged(targetVideoId, newAcknowledged);
      if (videoFromList) {
        sidebarState.setVideos(
          sidebarState.videos
            .map((v) => (v.id === updated.id ? updated : v))
            .filter((v) =>
              matchesAcknowledgedFilterVideo(
                v,
                sidebarState.acknowledgedFilter,
              ),
            ),
        );
      } else if (!selectionDroppedFromFilter) {
        pendingSelectedVideo = updated;
      }
      if (selectedChannelId) {
        track({
          event: "video_acknowledged_changed",
          video_id: targetVideoId,
          channel_id: selectedChannelId,
          acknowledged: newAcknowledged,
        });
      }

      videoAcknowledgeSeq += 1;
      videoAcknowledgeSync = {
        seq: videoAcknowledgeSeq,
        video: updated,
        confirmed: true,
      };

      const stillSelected = isStillSelectedAfterAcknowledgeSuccess(
        sidebarState.selectedVideoId,
        sidebarState.videos,
        pendingSelectedVideo,
      );
      if (!stillSelected) {
        content.editing = false;
        content.clearFormattingFeedback();
        if (sidebarState.videos.length === 0) {
          sidebarState.setSelectedVideoId(null);
          content.contentText = "";
          content.draft = "";
        } else {
          await selectVideo(sidebarState.videos[0].id);
        }
      }
    } catch (error) {
      sidebarState.setVideos(previousVideos);
      sidebarState.setSelectedVideoId(previousSelectedVideoId);
      pendingSelectedVideo = previousPendingSelectedVideo;
      const reverted = resolveRevertedVideoForAcknowledge(
        previousVideos,
        targetVideoId,
        previousPendingSelectedVideo,
      );
      if (reverted) {
        videoAcknowledgeSeq += 1;
        videoAcknowledgeSync = {
          seq: videoAcknowledgeSeq,
          video: reverted,
          confirmed: true,
        };
      }
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    }
  }

  async function refreshSummaryQuality() {
    if (
      !selectedVideoId ||
      contentMode !== "summary" ||
      editing ||
      loadingContent
    )
      return;
    const targetVideoId = selectedVideoId;
    try {
      const { getSummary } = await import("$lib/api");
      const summary = await getSummary(targetVideoId);
      if (
        selectedVideoId !== targetVideoId ||
        contentMode !== "summary" ||
        editing
      )
        return;
      if (!content.contentText.trim()) {
        content.cacheLoadedSummary(summary, targetVideoId);
        content.draft = content.contentText;
        content.videoInfo = null;
        if (videoHighlightsByVideoId[targetVideoId] === undefined) {
          void hydrateVideoHighlights(targetVideoId);
        }
      }
      content.applySummaryQuality(summary);
    } catch {
      // Keep previous quality state if background refresh fails.
    }
  }

  $effect(() =>
    createAiStatusPoller({
      onStatus: (status) => {
        aiAvailable = status.available;
        aiStatus = status.status;
      },
    }),
  );

  $effect(() => {
    if (
      contentMode !== "summary" ||
      !selectedVideoId ||
      editing ||
      loadingContent ||
      content.summaryQualityScore !== null ||
      content.summaryQualityNote !== null
    ) {
      return;
    }

    const needsReadySummaryRetry = shouldRetryReadySummaryLoad({
      contentMode,
      selectedVideo,
      contentText: content.contentText,
      loadingContent,
      editing,
    });
    const intervalMs = needsReadySummaryRetry ? 2000 : 7000;
    if (needsReadySummaryRetry) {
      void refreshSummaryQuality();
    }
    const timer = setInterval(() => {
      void refreshSummaryQuality();
    }, intervalMs);
    return () => clearInterval(timer);
  });

  async function clearBrowseVideoFilters() {
    const actions = sidebarState.videoActions;
    if (actions.onClearAllFilters) {
      await actions.onClearAllFilters();
    } else {
      await actions.onVideoTypeFilterChange("all");
      await actions.onAcknowledgedFilterChange("all");
    }
    if (mobileBrowseOpen && mobileViewportMq) {
      await loadDbPagesOnlyForMobileBrowse();
    }
  }

  const browseFilterDisabled = $derived(
    !sidebarState.selectedChannelId || sidebarState.videoState.loadingVideos,
  );

  const workspaceContentSelection = $derived({
    mobileVisible: true,
    mobileBackInTopBar: !mobileBrowseOpen && Boolean(selectedVideoId),
    selectedChannel,
    selectedVideo,
    selectedVideoId,
    contentMode,
  });

  /** From `cite` query param; scroll runs in TranscriptView when content matches `video`. */
  const citationScrollText = $derived.by(() => {
    const url = page.url;
    const cite = url.searchParams.get("cite")?.trim();
    if (!cite) {
      return null;
    }
    if (loadingContent) {
      return null;
    }
    const videoParam = url.searchParams.get("video")?.trim();
    if (videoParam && selectedVideoId && videoParam !== selectedVideoId) {
      return null;
    }
    return cite;
  });

  function onCitationScrollConsumed() {
    const url = new URL(page.url.href);
    if (!url.searchParams.has("cite") && !url.searchParams.has("chunk")) {
      return;
    }
    url.searchParams.delete("cite");
    url.searchParams.delete("chunk");
    replaceWorkspaceUrl(`${url.pathname}${url.search}${url.hash}`);
  }

  const workspaceContentState = $derived({
    loadingContent,
    editing,
    aiAvailable: aiAvailable ?? false,
    summaryQualityScore: content.summaryQualityScore,
    summaryQualityNote: content.summaryQualityNote,
    summaryModelUsed: content.summaryModelUsed,
    summaryQualityModelUsed: content.summaryQualityModelUsed,
    videoInfo,
    contentHtml,
    contentText,
    transcriptRenderMode,
    contentHighlights,
    selectedVideoHighlights,
    selectedVideoYoutubeUrl,
    draft: content.draft,
    formattingContent: content.formattingContent,
    formattingVideoId: content.formattingVideoId,
    regeneratingSummaryVideoIds: content.regeneratingSummaryVideoIds,
    revertingContent: content.revertingContent,
    revertingVideoId: content.revertingVideoId,
    resettingVideo: content.resettingVideo,
    resettingVideoId: content.resettingVideoId,
    creatingHighlight,
    creatingHighlightVideoId,
    creatingVocabularyReplacement,
    deletingHighlightId,
    canRevertTranscript,
    showRevertTranscriptAction: hasUpdatedTranscript,
    formattingNotice: content.formattingNotice,
    formattingNoticeVideoId: content.formattingNoticeVideoId,
    formattingNoticeTone: content.formattingNoticeTone,
    citationScrollText,
    canPersistHighlights: true,
  });
  const workspaceContentActions = $derived.by(() => ({
    onBack: () => {
      mobileBrowseOpen = true;
    },
    onSetMode: setMode,
    onStartEdit: content.startEdit,
    onCancelEdit: content.cancelEdit,
    onSaveEdit: content.saveEdit,
    onCleanFormatting: content.cleanFormatting,
    onRegenerateSummary: content.regenerateSummaryContent,
    onRevertTranscript: content.revertToOriginalTranscript,
    onResetVideo: content.resetVideoContent,
    onDraftChange: (value: string) => {
      content.draft = value;
    },
    onToggleAcknowledge: toggleAcknowledge,
    onCreateHighlight: saveSelectionHighlight,
    onCreateVocabularyReplacement: saveVocabularyReplacement,
    onDeleteHighlight: deleteExistingHighlight,
    onShowChannels: () => {
      mobileBrowseOpen = true;
    },
    onShowVideos: () => {
      mobileBrowseOpen = true;
    },
    onCitationScrollConsumed,
  }));
  const workspaceOverlaysState = $derived({
    errorMessage,
    showDeleteConfirmation: sidebarState.showDeleteConfirmation,
    showDeleteAccessPrompt,
    showAddSourceFeedback: !!addSourceFeedback && !addSourceFeedbackDismissed,
    showResetVideoConfirmation,
  });
  const workspaceOverlaysActions = {
    onDismissError: () => {
      errorMessage = null;
    },
    onConfirmDelete: confirmDeleteChannel,
    onCancelDelete: () => sidebarState.setShowDeleteConfirmation(false),
    onConfirmAccessPrompt: async () => {
      showDeleteAccessPrompt = false;
      const redirectTo = `${page.url.pathname}${page.url.search}`;
      await goto(`/login?redirectTo=${encodeURIComponent(redirectTo)}`);
    },
    onCancelAccessPrompt: () => {
      showDeleteAccessPrompt = false;
    },
    onConfirmResetVideo: async () => {
      showResetVideoConfirmation = false;
      await content.resetVideoContent();
    },
    onCancelResetVideo: () => {
      showResetVideoConfirmation = false;
    },
  };

  return {
    page,
    DOCS_URL,
    get aiIndicator() {
      return aiIndicator;
    },
    openGuide,
    closeGuide,
    setGuideStep,
    sidebarState,
    get errorMessage() {
      return errorMessage;
    },
    set errorMessage(v) {
      errorMessage = v;
    },
    get videoAcknowledgeSync() {
      return videoAcknowledgeSync;
    },
    handleChannelSyncDateSaved,
    handleDeleteChannel,
    get showDeleteAccessPrompt() {
      return showDeleteAccessPrompt;
    },
    set showDeleteAccessPrompt(v) {
      showDeleteAccessPrompt = v;
    },
    get mobileBrowseOpen() {
      return mobileBrowseOpen;
    },
    set mobileBrowseOpen(v) {
      mobileBrowseOpen = v;
    },
    get selectedVideoId() {
      return selectedVideoId;
    },
    get browseFilterDisabled() {
      return browseFilterDisabled;
    },
    onBrowseVideoTypeFilterChange,
    onBrowseAcknowledgedFilterChange,
    clearBrowseVideoFilters,
    get contentMode() {
      return contentMode;
    },
    setMode,
    get loadingContent() {
      return loadingContent;
    },
    get editing() {
      return editing;
    },
    get hasUpdatedTranscript() {
      return hasUpdatedTranscript;
    },
    get formattingContent() {
      return formattingContent;
    },
    get formattingVideoId() {
      return formattingVideoId;
    },
    get regeneratingSummaryVideoIds() {
      return regeneratingSummaryVideoIds;
    },
    get revertingContent() {
      return revertingContent;
    },
    get revertingVideoId() {
      return revertingVideoId;
    },
    get resettingVideo() {
      return resettingVideo;
    },
    get resettingVideoId() {
      return resettingVideoId;
    },
    get aiAvailable() {
      return aiAvailable;
    },
    get canRevertTranscript() {
      return canRevertTranscript;
    },
    get selectedVideoYoutubeUrl() {
      return selectedVideoYoutubeUrl;
    },
    get draft() {
      return draft;
    },
    get selectedVideo() {
      return selectedVideo;
    },
    content,
    get showResetVideoConfirmation() {
      return showResetVideoConfirmation;
    },
    set showResetVideoConfirmation(v) {
      showResetVideoConfirmation = v;
    },
    toggleAcknowledge,
    get WorkspaceSearchBarComponent() {
      return WorkspaceSearchBarComponent;
    },
    get searchStatus() {
      return searchStatus;
    },
    handleSearchResultSelection,
    loadMoreVideos,
    get canManageLibrary() {
      return canManageLibrary;
    },
    get workspaceContentSelection() {
      return workspaceContentSelection;
    },
    get workspaceContentState() {
      return workspaceContentState;
    },
    get workspaceContentActions() {
      return workspaceContentActions;
    },
    get workspaceOverlaysState() {
      return workspaceOverlaysState;
    },
    workspaceOverlaysActions,
    get addSourceFeedback() {
      return addSourceFeedback;
    },
    get addSourceFeedbackDismissed() {
      return addSourceFeedbackDismissed;
    },
    dismissAddSourceFeedback,
    openAddSourceFeedbackTarget,
    get guideOpen() {
      return guideOpen;
    },
    get guideStep() {
      return guideStep;
    },
    tourSteps,
    get vocabularyModalSource() {
      return vocabularyModalSource;
    },
    get vocabularyModalValue() {
      return vocabularyModalValue;
    },
    set vocabularyModalValue(v) {
      vocabularyModalValue = v;
    },
    get creatingVocabularyReplacement() {
      return creatingVocabularyReplacement;
    },
    confirmVocabularyReplacement,
    closeVocabularyModal,
  };
}
