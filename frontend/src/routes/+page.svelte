<script lang="ts">
  import { goto, replaceState as replacePageState } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount, tick } from "svelte";
  import { SvelteMap } from "svelte/reactivity";
  import type { Component } from "svelte";
  import {
    addChannel,
    backfillChannelVideos,
    type BackfillChannelVideosResponse,
    cleanTranscriptFormatting,
    createHighlight,
    deleteHighlight,
    deleteChannel,
    ensureTranscript,
    ensureVideoInfo,
    getChannelSnapshot,
    getChannelSyncDepth,
    getPreferences,
    getVideo,
    getVideoHighlights,
    getSummary,
    getWorkspaceBootstrap,
    getWorkspaceBootstrapWhenAvailable,
    listChannels,
    listVideos,
    refreshChannel,
    regenerateSummary,
    resetVideo,
    savePreferences,
    updateSummary,
    updateChannel,
    updateTranscript,
    updateAcknowledged,
    RateLimitedError,
  } from "$lib/api";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import { DOCS_URL } from "$lib/app-config";
  import type { TourStep } from "$lib/components/FeatureGuide.svelte";
  import WorkspaceContentPanel from "$lib/components/workspace/WorkspaceContentPanel.svelte";
  import MobileYouTubeTopNav from "$lib/components/mobile/MobileYouTubeTopNav.svelte";
  import MobileHomeBrowseOverlay from "$lib/components/mobile/MobileHomeBrowseOverlay.svelte";
  import WorkspaceDesktopTopBar from "$lib/components/workspace/WorkspaceDesktopTopBar.svelte";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import WorkspaceSidebar from "$lib/components/workspace/WorkspaceSidebar.svelte";
  import WorkspaceSidebarVideoFilterControl from "$lib/components/workspace/WorkspaceSidebarVideoFilterControl.svelte";

  // Lazy-loaded dynamic components (create Vite code-split boundaries)
  // WorkspaceSearchBar: eagerly loaded on mount (visible immediately, split for bundle size)
  // FeatureGuide: truly lazy-loaded only when the guide is first opened
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let WorkspaceSearchBarComponent = $state<Component<any> | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let FeatureGuideComponent = $state<Component<any> | null>(null);
  import type {
    AiStatus,
    Channel,
    ChannelSnapshot,
    CreateHighlightRequest,
    Highlight,
    HighlightSource,
    SearchResult,
    SearchStatus,
    Summary as SummaryPayload,
    TranscriptRenderMode,
    VideoInfo as VideoInfoPayload,
    Video,
    VideoTypeFilter,
  } from "$lib/types";
  import {
    applySavedChannelOrder,
    loadWorkspaceState,
    restoreWorkspaceSnapshot,
    resolveInitialChannelSelection,
    saveWorkspaceState,
    type WorkspaceStateSnapshot,
  } from "$lib/channel-workspace";
  import {
    normalizeTranscriptForRender,
    renderMarkdown,
  } from "$lib/utils/markdown";
  import {
    buildOptimisticHighlight,
    mergeHighlightIntoList,
    reconcileOptimisticHighlight,
  } from "$lib/utils/highlights";
  import {
    putCachedBootstrapMeta,
    putCachedChannels,
    putCachedViewSnapshot,
    removeCachedChannel,
  } from "$lib/workspace-cache";
  import { resolveBootstrapOnMount } from "$lib/ssr-bootstrap";
  import { resolveOldestLoadedReadyVideoDate } from "$lib/sync-depth";
  import { createAiStatusPoller } from "$lib/utils/ai-poller";
  import {
    resolveGuideStepFromUrl,
    writeGuideStepToUrl,
  } from "$lib/utils/guide";
  import { mobileWorkspaceBrowseIntent } from "$lib/mobile-navigation/mobileWorkspaceBrowseIntent";
  import {
    buildWorkspaceViewHref,
    mergeWorkspaceViewState,
    parseWorkspaceViewUrlState,
  } from "$lib/view-url";
  import { track } from "$lib/analytics/tracker";
  import {
    closeSummarySession,
    openSummarySession,
    isSummarySessionOpen,
  } from "$lib/analytics/read-time";
  import {
    buildChannelViewCacheKey,
    cloneSyncDepthState,
    cloneVideos,
    createChannelViewCache,
    type ChannelSyncDepthState,
    type ChannelViewCacheKeyPart,
  } from "$lib/channel-view-cache";
  import { channelOrderFromList } from "$lib/workspace/channels";
  import {
    applyOptimisticAcknowledge,
    buildOptimisticChannel,
    removeChannelFromCollection,
    removeChannelId,
    replaceOptimisticChannel,
    replaceOptimisticChannelId,
  } from "$lib/workspace/channel-actions";
  import {
    filterVideosByAcknowledged,
    filterVideosByType,
    loadChannelSnapshotWithRefresh,
    resolveNextChannelSelection,
  } from "$lib/workspace/route-helpers";
  import {
    resolveSummaryQualityPresentation,
    resolveTranscriptPresentation,
    shouldRetryReadySummaryLoad,
    stripContentPrefix,
  } from "$lib/workspace/content";
  import {
    buildFormattingAttemptSummary,
    clearFormattingFeedbackState,
    resetSummaryQualityState,
  } from "$lib/workspace/formatting";
  import {
    mergeVideoHighlights,
    removeVideoHighlightFromState,
  } from "$lib/workspace/highlight-actions";
  import { createSidebarState } from "$lib/workspace/sidebar-state.svelte";
  import { deriveSummaryTrackingId } from "$lib/workspace/summary-tracking-id";
  import { mobileBottomBar } from "$lib/mobile-navigation/mobileBottomBar";
  import {
    type AcknowledgedFilter,
    type ChannelSortMode,
    isWorkspaceContentMode,
    isWorkspaceVideoTypeFilter,
    resolveAcknowledgedParam,
    type WorkspaceContentMode,
  } from "$lib/workspace/types";
  const FORMAT_MAX_TURNS = 5;
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

  // -- Guide tour (URL-driven: ?guide=0, ?guide=1, ...) --
  let guideOpen = $state(false);
  let guideStep = $state(0);

  function openGuide() {
    guideStep = 0;
    guideOpen = true;
    writeGuideStepToUrl(0);
  }

  function closeGuide() {
    guideOpen = false;
    writeGuideStepToUrl(null);
  }

  function setGuideStep(s: number) {
    guideStep = s;
    writeGuideStepToUrl(s);
  }

  let aiAvailable = $state<boolean | null>(null);
  let aiStatus = $state<AiStatus | null>(null);
  let searchStatus = $state<SearchStatus | null>(null);
  let loadingContent = $state(false);

  let errorMessage = $state<string | null>(null);
  let showDeleteConfirmation = $state(false);
  let showDeleteAccessPrompt = $state(false);
  let showResetVideoConfirmation = $state(false);
  let channelIdToDelete = $state<string | null>(null);
  let summaryQualityScore = $state<number | null>(null);
  let summaryQualityNote = $state<string | null>(null);
  let summaryModelUsed = $state<string | null>(null);
  let summaryQualityModelUsed = $state<string | null>(null);
  let videoInfo = $state<VideoInfoPayload | null>(null);

  let historyExhausted = $state(false);
  let backfillingHistory = $state(false);
  let allowLoadedVideoSyncDepthOverride = $state(false);
  /**
   * Default backend expensive limit is 120/min per client (`EXPENSIVE_RATE_LIMIT_PER_MINUTE`).
   * Space POST /backfill calls so mobile auto-load does not burst 429s.
   */
  const MIN_BACKFILL_INTERVAL_MS = 2100;
  let lastBackfillRequestAtMs = 0;

  let contentMode = $state<WorkspaceContentMode>("info");
  /** Matches Tailwind `lg` (mobile-only UI). Used to avoid racing desktop channel snapshot loads. */
  let mobileViewportMq = $state(false);
  let mobileBrowseOpen = $state(true);
  let contentText = $state("");
  let transcriptRenderMode = $state<TranscriptRenderMode>("plain_text");
  let draftTranscriptRenderMode = $state<TranscriptRenderMode>("plain_text");
  let contentRenderText = $derived(
    contentMode === "transcript"
      ? transcriptRenderMode === "markdown"
        ? normalizeTranscriptForRender(contentText)
        : contentText
      : contentText,
  );
  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  let isOperator = $derived(Boolean($page.data.isOperator));
  let contentHtml = $derived(renderMarkdown(contentRenderText));
  let editing = $state(false);
  let draft = $state("");
  let formattingContent = $state(false);
  let formattingVideoId = $state<string | null>(null);
  let regeneratingSummaryVideoIds = $state<string[]>([]);
  let revertingContent = $state(false);
  let revertingVideoId = $state<string | null>(null);
  let resettingVideo = $state(false);
  let resettingVideoId = $state<string | null>(null);
  let videoHighlightsByVideoId = $state<Record<string, Highlight[]>>({});
  let nextOptimisticHighlightId = -1;
  let creatingHighlight = $state(false);
  let creatingHighlightVideoId = $state<string | null>(null);
  let deletingHighlightId = $state<number | null>(null);
  // (Moved further down to ensure dependencies are initialized)
  let originalTranscriptByVideoId = $state<Record<string, string>>({});
  const contentCache = new SvelteMap<
    string,
    {
      transcript?: {
        text: string;
        renderMode: TranscriptRenderMode;
      };
      summary?: {
        text: string;
        quality: SummaryPayload;
        trackingId: string;
      };
      info?: VideoInfoPayload;
    }
  >();
  let formattingNotice = $state<string | null>(null);
  let formattingNoticeVideoId = $state<string | null>(null);
  let formattingNoticeTone = $state<"info" | "success" | "warning">("info");
  let formattingAttemptsUsed = $state<number | null>(null);
  let formattingAttemptsMax = $state<number | null>(null);
  let formattingAttemptsVideoId = $state<string | null>(null);
  let formattingRequestSeq = 0;
  let activeFormattingRequest = $state(0);
  let contentRequestSeq = 0;
  let activeContentRequestId = 0;

  let workspaceStateHydrated = $state(false);
  /** SvelteKit's replaceState throws until the client router has started; sidebar restore runs in onMount before that. */
  let shallowUrlSyncReady = $state(false);
  let viewUrlHydrated = $state(false);
  let pendingSelectedVideo = $state<Video | null>(null);
  // Sidebar State (using unified composable)
  const sidebarState = createSidebarState({
    initialChannelId: $page.data.selectedChannelId,
    initialVideoId: $page.data.selectedVideoId,
    initialVideoTypeFilter: $page.data.videoTypeFilter ?? "all",
    initialAcknowledgedFilter: $page.data.acknowledgedFilter ?? "all",
    onSelectVideo: (videoId: string, context?: { forceReload?: boolean }) => {
      return selectVideo(videoId, true, context?.forceReload ?? false);
    },
    onChannelSelected: (channelId: string) => {
      if (!sidebarState.selectedVideoId) {
        clearSelectedVideoState();
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
    onAcknowledgedFilterChange: (value: boolean | undefined) => {
      const ack: AcknowledgedFilter =
        value === true ? "ack" : value === false ? "unack" : "all";
      const href = buildWorkspaceViewHref({
        selectedChannelId: sidebarState.selectedChannelId,
        selectedVideoId,
        contentMode,
        videoTypeFilter: sidebarState.videoTypeFilter,
        acknowledgedFilter: ack,
      });
      replaceWorkspaceUrl(href);
    },
    onOpenChannelOverview: async (channelId: string) => {
      await goto(`/channels/${encodeURIComponent(channelId)}`);
    },
    onVideoListReset: () => {
      historyExhausted = false;
      backfillingHistory = false;
    },
  });

  let videoAcknowledgeSeq = 0;
  let videoAcknowledgeSync = $state<{
    seq: number;
    video: Video;
    /** When false, only merge into per-channel lists; do not refetch (refetch would hit stale GET cache before PUT invalidates). */
    confirmed: boolean;
  } | null>(null);

  // Backward compatibility aliases for existing UI logic
  // (We use getters to stay reactive to the sidebar state)
  const channels = $derived(sidebarState.channels);
  const selectedChannelId = $derived(sidebarState.selectedChannelId);
  const selectedChannel = $derived(sidebarState.selectedChannel);
  const loadingChannels = $derived(sidebarState.loadingChannels);
  const addingChannel = $derived(sidebarState.addingChannel);
  const videos = $derived(sidebarState.videos);
  const loadingVideos = $derived(sidebarState.loadingVideos);
  const refreshingChannel = $derived(sidebarState.refreshingChannel);
  const backfillingVideos = $derived(sidebarState.backfillingHistory);
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

  // Legacy state being moved/removed:
  // let loadingVideos = $state(false); // REMOVED
  // REMOVED
  // ... (others removed similarly)

  // REMOVED DUPLICATE let selectedVideoId declaration
  // selectedVideo derived moved up

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
    historyExhausted = state.historyExhausted;
    backfillingHistory = state.backfillingHistory;
    allowLoadedVideoSyncDepthOverride = state.allowLoadedVideoSyncDepthOverride;
    sidebarState.setSyncDepth(cloneSyncDepthState(state.syncDepth));
  }

  $effect(() => {
    if (!selectedChannelId) return;

    channelVideoStateCache.set(getChannelViewKey(selectedChannelId), {
      videos: cloneVideos(videos),
      offset,
      hasMore,
      historyExhausted,
      backfillingHistory,
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

  function clearSelectedVideoState() {
    sidebarState.setSelectedVideoId(null);
    pendingSelectedVideo = null;
    contentText = "";
    transcriptRenderMode = "plain_text";
    draft = "";
    draftTranscriptRenderMode = "plain_text";
    resetSummaryQuality();
    resetVideoInfo();
  }

  async function resolvePendingSelectedVideo(
    videoId: string,
    channelId: string,
  ) {
    try {
      const video = await getVideo(videoId);
      if (selectedChannelId !== channelId || selectedVideoId !== videoId) {
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
    let hasSelectedVideo = videos.some(
      (video) => video.id === preferredVideoId,
    );
    let scannedPages = 0;
    const targetChannelId = selectedChannelId;
    const pendingSelectedVideoRequest =
      hasSelectedVideo || !targetChannelId
        ? Promise.resolve(null)
        : resolvePendingSelectedVideo(preferredVideoId, targetChannelId);

    if (!loadingContent) {
      // Keep selected row and rendered content in lockstep after refresh/rehydration.
      // Even when contentText is non-empty, it may belong to the previously selected
      // video, so always reload for the restored selectedVideoId.
      void loadContent();
    }

    while (
      !hasSelectedVideo &&
      sidebarState.hasMore &&
      scannedPages < SELECTED_VIDEO_SCAN_PAGE_LIMIT &&
      targetChannelId &&
      sidebarState.selectedChannelId === targetChannelId &&
      selectedVideoId === preferredVideoId
    ) {
      const next = await listVideos(
        targetChannelId,
        sidebarState.limit,
        sidebarState.offset,
        sidebarState.videoTypeFilter,
        acknowledged,
      );
      scannedPages += 1;
      if (next.length === 0) {
        sidebarState.setHasMore(false);
        break;
      }

      sidebarState.setVideos([...videos, ...next]);
      sidebarState.setOffset(offset + next.length);
      sidebarState.setHasMore(next.length === limit);
      hasSelectedVideo = videos.some((video) => video.id === preferredVideoId);
    }

    if (!hasSelectedVideo) {
      const restoredVideo = await pendingSelectedVideoRequest;
      if (
        restoredVideo &&
        selectedChannelId === targetChannelId &&
        selectedVideoId === preferredVideoId
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
        video_count: snapshot.channel_video_count,
      });
      void putCachedViewSnapshot(
        buildWorkspaceSnapshotCacheKey(channelId, videoTypeFilter, isAck),
        snapshot,
      );
      await hydrateSelectedVideo(preferredVideoId, isAck);
    } catch (error) {
      if (!silent || !errorMessage) {
        errorMessage = (error as Error).message;
      }
    } finally {
      if (!silent) {
        sidebarState.setLoadingVideos(false);
      }
    }
  }

  async function syncEarliestDateFromLoadedVideos() {
    if (!selectedChannelId || !selectedChannel) return;
    if (selectedChannel.earliest_sync_date_user_set) return;
    const oldest = resolveOldestLoadedReadyVideoDate(videos);
    if (!oldest) return;

    const currentEarliest = selectedChannel.earliest_sync_date
      ? new Date(selectedChannel.earliest_sync_date)
      : null;
    const shouldPushBack =
      !currentEarliest ||
      Number.isNaN(currentEarliest.getTime()) ||
      oldest < currentEarliest;
    if (!shouldPushBack) return;

    const updated = await updateChannel(sidebarState.selectedChannelId!, {
      earliest_sync_date: oldest.toISOString(),
    });
    sidebarState.setChannels(
      sidebarState.channels.map((channel) =>
        channel.id === sidebarState.selectedChannelId ? updated : channel,
      ),
    );
    void loadSyncDepth();
  }
  const selectedVideoYoutubeUrl = $derived(
    selectedVideoId
      ? `https://www.youtube.com/watch?v=${selectedVideoId}`
      : null,
  );
  const selectedOriginalTranscript = $derived(
    selectedVideoId
      ? (originalTranscriptByVideoId[selectedVideoId] ?? null)
      : null,
  );
  const hasUpdatedTranscript = $derived(
    contentMode === "transcript" &&
      selectedOriginalTranscript !== null &&
      contentText !== selectedOriginalTranscript,
  );
  const canRevertTranscript = $derived(
    contentMode === "transcript" &&
      selectedOriginalTranscript !== null &&
      (editing
        ? draft !== selectedOriginalTranscript
        : contentText !== selectedOriginalTranscript),
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

  async function hydrateVideoHighlights(
    videoId: string,
    options: { showError?: boolean } = {},
  ) {
    try {
      const highlights = await getVideoHighlights(videoId);
      storeVideoHighlights(videoId, highlights);
      return highlights;
    } catch (error) {
      if (options.showError) {
        errorMessage = (error as Error).message;
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
      errorMessage = (error as Error).message;
    } finally {
      creatingHighlight = false;
      creatingHighlightVideoId = null;
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

    try {
      await deleteHighlight(highlightId);
      removeVideoHighlight(targetVideoId, highlightId);
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      deletingHighlightId = null;
    }
  }

  function syncChannelOrderFromList() {
    sidebarState.setChannelOrder(channelOrderFromList(sidebarState.channels));
  }

  function applySummaryQuality(summary: SummaryPayload) {
    const presentation = resolveSummaryQualityPresentation(summary);
    summaryQualityScore = presentation.score;
    summaryQualityNote = presentation.note;
    summaryModelUsed = presentation.modelUsed;
    summaryQualityModelUsed = presentation.qualityModelUsed;
  }

  function syncSummaryTrackingSession(
    summary: SummaryPayload,
    videoId: string,
    channelId: string,
  ) {
    const trackingId = deriveSummaryTrackingId(summary);
    if (isSummarySessionOpen(videoId, trackingId)) {
      return trackingId;
    }

    closeSummarySession();
    openSummarySession({
      video_id: videoId,
      channel_id: channelId,
      summary_id: trackingId,
    });
    return trackingId;
  }

  function cacheLoadedSummary(summary: SummaryPayload, videoId: string) {
    const summaryText = stripContentPrefix(
      summary.content || "Summary unavailable.",
    );
    const trackingId = selectedChannelId
      ? syncSummaryTrackingSession(summary, videoId, selectedChannelId)
      : deriveSummaryTrackingId(summary);
    const prev = contentCache.get(videoId);
    contentCache.set(videoId, {
      ...prev,
      summary: {
        text: summaryText,
        quality: summary,
        trackingId,
      },
    });
    return summaryText;
  }

  function resetSummaryQuality() {
    const nextState = resetSummaryQualityState();
    summaryQualityScore = nextState.score;
    summaryQualityNote = nextState.note;
    summaryModelUsed = nextState.modelUsed;
    summaryQualityModelUsed = nextState.qualityModelUsed;
  }

  function resetVideoInfo() {
    videoInfo = null;
  }

  function clearFormattingFeedback() {
    const nextState = clearFormattingFeedbackState();
    formattingNotice = nextState.formattingNotice;
    formattingNoticeVideoId = nextState.formattingNoticeVideoId;
    formattingAttemptsUsed = nextState.formattingAttemptsUsed;
    formattingAttemptsMax = nextState.formattingAttemptsMax;
    formattingAttemptsVideoId = nextState.formattingAttemptsVideoId;
  }

  function isCurrentContentRequest(
    requestId: number,
    targetVideoId: string,
    targetMode: "transcript" | "summary" | "highlights" | "info",
  ) {
    return (
      activeContentRequestId === requestId &&
      selectedVideoId === targetVideoId &&
      contentMode === targetMode
    );
  }

  function restoreWorkspaceState() {
    const restored = mergeWorkspaceViewState(
      restoreWorkspaceSnapshot(
        typeof localStorage === "undefined"
          ? null
          : loadWorkspaceState(localStorage),
        {
          includeSelectedVideoId: true,
          includeContentMode: true,
          includeVideoTypeFilter: true,
          includeAcknowledgedFilter: true,
          includeChannelSortMode: true,
        },
      ),
      typeof window === "undefined"
        ? {}
        : parseWorkspaceViewUrlState(new URL(window.location.href)),
    );

    if ("selectedChannelId" in restored) {
      sidebarState.setSelectedChannelId(restored.selectedChannelId ?? null);
    }
    if ("selectedVideoId" in restored) {
      sidebarState.setSelectedVideoId(restored.selectedVideoId ?? null);
    }
    if (restored.contentMode && isWorkspaceContentMode(restored.contentMode)) {
      contentMode = restored.contentMode;
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
    saveWorkspaceState(localStorage, snapshot);
    // Debounce-persist channel order + sort mode to the backend so it survives
    // across devices/browsers. 1 s delay avoids bursting on rapid reorders.
    if (!preferencesHydrated) return;
    if (preferencesSaveTimer) clearTimeout(preferencesSaveTimer);
    preferencesSaveTimer = setTimeout(() => {
      void savePreferences({
        channel_order: sidebarState.channelOrder,
        channel_sort_mode: sidebarState.channelSortMode,
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

        // Resolve initial state from server bootstrap (SSR) + IndexedDB warm-start.
        // Fetch preferences in parallel so the API channel order is available before
        // channels are rendered (VAL-CROSS-004).
        const [bootstrapResult, apiPreferences] = await Promise.all([
          resolveBootstrapOnMount({
            serverBootstrap: $page.data.bootstrap ?? null,
            selectedChannelId: selectedChannelIdAtMount,
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

        // Apply API preferences — override localStorage channel order when the
        // backend has a non-empty saved order (cross-device persistence).
        if (apiPreferences) {
          if (apiPreferences.channel_order.length > 0) {
            sidebarState.setChannelOrder(apiPreferences.channel_order);
          }
          sidebarState.setChannelSortMode(
            apiPreferences.channel_sort_mode as ChannelSortMode,
          );
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

        // Background refresh via consolidated bootstrap endpoint (1 API call for
        // channels + AI status + search status + snapshot — satisfies VAL-DATA-002).
        void loadBootstrapRefresh({ silent: hasInitialData }).finally(() => {
          viewUrlHydrated = true;
        });
      } finally {
        preferencesHydrated = true;
      }
    })();

    const restoredGuideStep = resolveGuideStepFromUrl(
      new URL(window.location.href),
      tourSteps.length,
    );
    if (restoredGuideStep !== null) {
      guideStep = restoredGuideStep;
      guideOpen = true;
    }

    // Eagerly load WorkspaceSearchBar on mount (creates a Vite code-split boundary
    // while ensuring the search bar is available as soon as the component hydrates).
    void import("$lib/components/workspace/WorkspaceSearchBar.svelte").then(
      (m) => {
        WorkspaceSearchBarComponent = m.default;
      },
    );

    return () => {
      mq.removeEventListener("change", onViewportChange);
      unsubBrowseIntent();
    };
  });

  // Lazily load FeatureGuide only when the guide tour is first opened.
  // This creates a true on-demand code-split boundary since the tour is optional.
  $effect(() => {
    if (guideOpen && !FeatureGuideComponent) {
      void import("$lib/components/FeatureGuide.svelte").then((m) => {
        FeatureGuideComponent = m.default;
      });
    }
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
      await setMode(targetMode);
    }
  }

  /**
   * Background workspace refresh using the consolidated bootstrap endpoint.
   *
   * Fetches channels, AI status, search status, and the selected channel
   * snapshot in a single request (VAL-DATA-002). Replaces the previous
   * pattern of separate listChannels + refreshWorkspaceMeta + getChannelSnapshot
   * calls that totalled 3-4 separate API requests.
   */
  async function loadBootstrapRefresh(options?: { silent?: boolean }) {
    const silent = options?.silent ?? false;
    const previousSelectedChannelId = selectedChannelId;
    const previousSelectedVideoId = selectedVideoId;

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
      void putCachedChannels(sidebarState.channels);

      // Apply AI/search status from the bootstrap response
      aiAvailable = bootstrap.ai_available;
      aiStatus = bootstrap.ai_status;
      searchStatus = bootstrap.search_status;
      void putCachedBootstrapMeta({
        ai_available: bootstrap.ai_available,
        ai_status: bootstrap.ai_status,
        search_status: bootstrap.search_status,
      });

      // Selection may have changed while the bootstrap request was in flight (e.g.
      // user clicked a video in another channel). Resolve channel/video from
      // current state so we do not overwrite the new selection with stale data.
      const selectionChannelId = sidebarState.selectedChannelId;
      const selectionVideoId = sidebarState.selectedVideoId;

      const initialChannelId = resolveInitialChannelSelection(
        bootstrap.channels,
        selectionChannelId ?? previousSelectedChannelId,
        null,
      );

      if (!initialChannelId) {
        sidebarState.setSelectedChannelId(null);
        mobileBrowseOpen = true;
        clearSelectedVideoState();
        sidebarState.setVideos([]);
        sidebarState.setSyncDepth(null);
        sidebarState.setOffset(0);
        sidebarState.setHasMore(true);
        historyExhausted = false;
        backfillingHistory = false;
        allowLoadedVideoSyncDepthOverride = false;
      } else {
        const preferredVideoId =
          initialChannelId === selectionChannelId ? selectionVideoId : null;
        const canReuseRenderedSnapshot =
          initialChannelId === selectionChannelId &&
          sidebarState.videos.length > 0;

        sidebarState.setSelectedChannelId(initialChannelId);
        resetSummaryQuality();
        resetVideoInfo();
        editing = false;
        clearFormattingFeedback();

        if (
          bootstrap.snapshot &&
          bootstrap.selected_channel_id === initialChannelId
        ) {
          // Bootstrap includes a snapshot for this channel — apply it directly,
          // avoiding an additional getChannelSnapshot API call.
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
          historyExhausted = false;
          backfillingHistory = false;
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
      if (!silent || !errorMessage) {
        errorMessage = (error as Error).message;
      }
    } finally {
      if (!silent) {
        sidebarState.setLoadingChannels(false);
        sidebarState.setLoadingVideos(false);
      }
    }
  }

  async function initChannels() {
    sidebarState.setLoadingChannels(true);
    try {
      const bootstrap = await getWorkspaceBootstrap();
      if (bootstrap) {
        sidebarState.setChannels(bootstrap.channels);
      }
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      sidebarState.setLoadingChannels(false);
    }
  }
  async function handleAddChannel(input: string) {
    if (!input.trim()) return false;

    const { optimisticChannel, tempId, trimmedInput } =
      buildOptimisticChannel(input);
    sidebarState.setAddingChannel(false);
    errorMessage = null;

    const previousChannels = [...channels];
    const previousSelectedId = selectedChannelId;

    sidebarState.addOptimisticChannel(optimisticChannel);

    try {
      const channel = await addChannel(trimmedInput);

      sidebarState.setChannels(
        replaceOptimisticChannel(sidebarState.channels, tempId, channel),
      );
      sidebarState.replaceOptimisticChannelId(tempId, channel.id);
      sidebarState.setSelectedChannelId(channel.id);

      await selectChannel(channel.id);
      return true;
    } catch (error) {
      sidebarState.setChannels(previousChannels);
      sidebarState.setSelectedChannelId(previousSelectedId);
      errorMessage = (error as Error).message;
      return false;
    } finally {
      sidebarState.setAddingChannel(false);
    }
  }

  async function handleDeleteChannel(channelId: string) {
    if (!isOperator) {
      showDeleteAccessPrompt = true;
      return;
    }

    sidebarState.setChannelIdToDelete(channelId);
    sidebarState.setShowDeleteConfirmation(true);
  }

  async function confirmDeleteChannel() {
    if (!sidebarState.channelIdToDelete || !isOperator) return;
    const channelId = sidebarState.channelIdToDelete;
    const channelViewKey = getChannelViewKey(channelId);
    sidebarState.setShowDeleteConfirmation(false);
    sidebarState.setChannelIdToDelete(null);

    // Optimistic removal — snapshot state, remove immediately, revert on error
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
        contentText = "";
        draft = "";
      }
    }

    try {
      await deleteChannel(channelId);
      void removeCachedChannel(channelId);
      channelVideoStateCache.delete(channelViewKey);
    } catch (error) {
      // Revert optimistic removal on failure
      sidebarState.setChannels(previousChannels);
      sidebarState.setSelectedChannelId(previousSelectedChannelId);
      errorMessage = (error as Error).message;
    }
  }

  function cancelDeleteChannel() {
    sidebarState.setShowDeleteConfirmation(false);
    sidebarState.setChannelIdToDelete(null);
  }

  function cancelDeleteAccessPrompt() {
    showDeleteAccessPrompt = false;
  }

  async function confirmDeleteAccessPrompt() {
    showDeleteAccessPrompt = false;
    const redirectTo = `${$page.url.pathname}${$page.url.search}`;
    await goto(`/login?redirectTo=${encodeURIComponent(redirectTo)}`);
  }

  $effect(() => {
    if (!sidebarState.selectedChannelId) {
      sidebarState.setSyncDepth(null);
    }
  });

  async function selectChannel(
    channelId: string | null,
    videoId: string | null = null,
    scroll = true,
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

    clearFormattingFeedback();
    if (hasCachedChannelVideoState && cachedChannelVideoState) {
      restoreCachedChannelVideoState(cachedChannelVideoState);
      sidebarState.setLoadingVideos(false);
      void refreshAndLoadVideos(channelId, false, videoId, true);
      return;
    }

    sidebarState.setVideos([]);
    sidebarState.setOffset(0);
    sidebarState.setHasMore(true);
    historyExhausted = false;
    backfillingHistory = false;
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
        sidebarState.setVideos(list);
        sidebarState.setOffset(list.length);
      } else {
        sidebarState.setVideos([...sidebarState.videos, ...list]);
        sidebarState.setOffset(sidebarState.offset + list.length);
      }
      sidebarState.setHasMore(list.length === limit);
      if (reset) {
        allowLoadedVideoSyncDepthOverride = false;
      }

      if (reset) {
        await hydrateSelectedVideo(sidebarState.selectedVideoId, isAck);
      }
    } catch (error) {
      if (!silent || !errorMessage) {
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
      backfillingHistory
    )
      return;

    if (sidebarState.hasMore) {
      await loadVideos(false);
      allowLoadedVideoSyncDepthOverride = true;
      await syncEarliestDateFromLoadedVideos();
      return;
    }

    backfillingHistory = true;
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
        historyExhausted = true;
      }

      await loadVideos(false);
      await loadSyncDepth();
      allowLoadedVideoSyncDepthOverride = true;
      await syncEarliestDateFromLoadedVideos();
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      backfillingHistory = false;
    }
  }

  /**
   * While mobile browse is open, paginate and backfill until every video that
   * matches the active filters is loaded (no manual Load More / Load History).
   */
  async function loadAllVideosForMobileBrowse(isAborted: () => boolean) {
    const channelId = sidebarState.selectedChannelId;
    if (!channelId || !mobileBrowseOpen || !mobileViewportMq) return;

    // Wait for the initial channel snapshot (often silent) so we do not race
    // listVideos with applyChannelSnapshot.
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
          backfillingHistory ||
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
      if (historyExhausted) {
        break;
      }
      await loadMoreVideos();
    }
  }

  /**
   * After a filter change on mobile browse, fetch remaining pages from the API
   * only (no YouTube backfill). Keeps the list complete without the long
   * full-history pass that `loadAllVideosForMobileBrowse` runs on channel open.
   */
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
      while (sidebarState.loadingVideos || backfillingHistory) {
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
    forceReload = false,
  ) {
    if (fromUserInteraction) mobileBrowseOpen = false;
    if (!forceReload && videoId === selectedVideoId) return;
    // Close any open summary session when switching videos.
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
    contentText = "";
    draft = "";
    const video = videoId ? videos.find((v) => v.id === videoId) : null;
    const cachedHighlights = videoId ? videoHighlightsByVideoId[videoId] : null;
    const highlights =
      videoId && cachedHighlights
        ? cachedHighlights
        : ((videoId ? await hydrateVideoHighlights(videoId) : []) ?? []);
    if (selectedVideoId !== videoId) return;
    resetSummaryQuality();
    resetVideoInfo();
    editing = false;
    clearFormattingFeedback();
    await loadContent();
  }

  async function setMode(
    mode: "transcript" | "summary" | "highlights" | "info",
  ) {
    if (contentMode === mode) return;
    const previousMode = contentMode;
    if (previousMode === "summary" && selectedVideoId) {
      closeSummarySession();
    }
    contentMode = mode;
    if (selectedVideoId && selectedChannelId) {
      track({
        event: "content_mode_changed",
        video_id: selectedVideoId,
        channel_id: selectedChannelId,
        from_mode: previousMode,
        to_mode: mode,
      });
    }
    resetSummaryQuality();
    resetVideoInfo();
    editing = false;
    clearFormattingFeedback();
    await loadContent();
  }

  async function tourPrepareFirstVideoIfNeeded() {
    mobileBrowseOpen = false;
    await tick();
    if (!selectedVideoId && selectedChannelId && videos.length > 0) {
      await selectVideo(videos[0].id, false, false);
    }
    await tick();
  }

  async function tourPrepareOpenAddChannel() {
    mobileBrowseOpen = true;
    await tick();
    document.getElementById("tour-add-channel")?.click();
    await tick();
    await tick();
  }

  const TAB_STRIP_TOUR = [
    "#workspace-tabs-mobile",
    "#workspace-tabs-desktop",
    "#content-view",
  ] as const;

  const tourSteps: TourStep[] = [
    {
      selector: "#workspace",
      title: "Welcome to dAstIll",
      body:
        "dAstIll helps you keep up with YouTube without the doom-scrolling. " +
        "It pulls transcripts from your favorite channels and creates AI summaries, " +
        "so you can quickly decide which videos are worth your time. " +
        "Note: This is a showcase app. It's not intended to be a production-ready multi-user application. " +
        "In fact, the business model of YouTube prevent this from ever becoming that. I'm just having fun with it.",
      placement: "right",
      prepare: () => {
        mobileBrowseOpen = true;
      },
    },
    {
      selector: "#channel-input",
      title: "Add a Channel",
      body: "Paste a URL or handle here to subscribe to a channel. New uploads are tracked automatically.",
      placement: "bottom",
      prepare: () => {
        void tourPrepareOpenAddChannel();
      },
      fallbackSelectors: ["#tour-add-channel", "#tour-library-tools"],
    },
    {
      selector: "#workspace-tabs-mobile",
      title: "Read the Transcript",
      body: "Every video's spoken content is available as a full transcript you can read at your own pace.",
      placement: "bottom",
      prepare: async () => {
        await tourPrepareFirstVideoIfNeeded();
        if (contentMode !== "transcript") {
          await setMode("transcript");
        }
      },
      fallbackSelectors: [...TAB_STRIP_TOUR],
    },
    {
      selector: "#workspace-tabs-mobile",
      title: "AI Summary",
      body: "The Summary tab shows the distilled version so you can decide if the full video is still worth watching.",
      placement: "bottom",
      prepare: async () => {
        await tourPrepareFirstVideoIfNeeded();
        if (contentMode !== "summary") {
          await setMode("summary");
        }
      },
      fallbackSelectors: [...TAB_STRIP_TOUR],
    },
    {
      selector: '[data-tour-target="nav-chat"]',
      title: "AI Chat",
      body: "Chat with your library. Our agentic RAG-based LLM system let's you ask questions about specific videos and will even do deep research for you.",
      placement: "right",
      prepare: () => {
        mobileBrowseOpen = true;
      },
      fallbackSelectors: [
        "#nav-chat-link",
        "#mobile-nav-chat-link",
        "#app-section-nav-rail a[href='/chat']",
        "#app-section-nav-mobile a[href='/chat']",
      ],
    },
    {
      selector: "#workspace",
      title: "Other features",
      body: "Search, sort, and filter videos. Set earliest date to sync from and load more videos to go further back in time.",
      placement: "bottom",
      prepare: () => {
        mobileBrowseOpen = true;
      },
      fallbackSelectors: ["#tour-library-tools"],
    },
    {
      selector: "#mark-read-toggle",
      title: "Mark as read",
      body: "Tip: Use it with the read filter in the library to get that sweet dopamine feeling of progress.",
      placement: "bottom",
      prepare: async () => {
        if (contentMode === "info" || contentMode === "highlights") {
          await setMode("transcript");
        }
        await tourPrepareFirstVideoIfNeeded();
      },
      fallbackSelectors: [
        "#content-actions",
        "#workspace-tabs-mobile",
        "#workspace-tabs-desktop",
        "#content-view",
      ],
    },
    {
      selector: "#workspace-tabs-mobile",
      title: "Your Highlights",
      body: "Found something worth remembering? Select any text in the transcript or summary and save it as a highlight. All your saved passages for this video appear here.",
      placement: "bottom",
      prepare: async () => {
        await tourPrepareFirstVideoIfNeeded();
        if (contentMode !== "highlights") {
          await setMode("highlights");
        }
      },
      fallbackSelectors: [...TAB_STRIP_TOUR],
    },
    {
      selector: "#ai-status-pill",
      title: "AI availability",
      body: "This dot beside the logo shows whether the AI backend is reachable for summaries and chat. Reading still works without it.",
      placement: "bottom",
      prepare: () => {
        mobileBrowseOpen = true;
      },
      fallbackSelectors: [
        "a[aria-label='Go to dAstIll home']",
        "#nav-workspace-link",
        "#mobile-nav-workspace-link",
      ],
    },
    {
      selector: "#guide-trigger",
      title: "Come back to this guide any time",
      body: "Reopen this guide at any time.",
      placement: "right",
      prepare: () => {
        mobileBrowseOpen = true;
      },
      fallbackSelectors: ["#workspace"],
    },
  ];

  function invalidateContentCache(
    videoId: string,
    mode?: "transcript" | "summary" | "info",
  ) {
    if (!mode) {
      contentCache.delete(videoId);
      return;
    }
    const entry = contentCache.get(videoId);
    if (entry) {
      delete entry[mode];
    }
  }

  async function loadContent() {
    if (!selectedVideoId) return;
    const targetVideoId = selectedVideoId;
    const targetMode = contentMode;
    const requestId = ++contentRequestSeq;
    activeContentRequestId = requestId;

    // Check cache first
    if (
      targetMode === "highlights" &&
      videoHighlightsByVideoId[targetVideoId] !== undefined
    ) {
      contentText = "";
      resetSummaryQuality();
      resetVideoInfo();
      draft = "";
      loadingContent = false;
      activeContentRequestId = 0;
      return;
    }

    const cached = contentCache.get(targetVideoId);
    if (cached) {
      if (targetMode === "transcript" && cached.transcript !== undefined) {
        contentText = cached.transcript.text;
        transcriptRenderMode = cached.transcript.renderMode;
        draft = contentText;
        draftTranscriptRenderMode = transcriptRenderMode;
        resetSummaryQuality();
        resetVideoInfo();
        if (videoHighlightsByVideoId[targetVideoId] === undefined) {
          void hydrateVideoHighlights(targetVideoId);
        }
        loadingContent = false;
        activeContentRequestId = 0;
        return;
      }
      if (targetMode === "summary" && cached.summary) {
        contentText = cached.summary.text;
        applySummaryQuality(cached.summary.quality);
        if (selectedChannelId) {
          syncSummaryTrackingSession(
            cached.summary.quality,
            targetVideoId,
            selectedChannelId,
          );
        }
        resetVideoInfo();
        draft = contentText;
        if (videoHighlightsByVideoId[targetVideoId] === undefined) {
          void hydrateVideoHighlights(targetVideoId);
        }
        loadingContent = false;
        activeContentRequestId = 0;
        return;
      }
      if (targetMode === "info" && cached.info) {
        videoInfo = cached.info;
        contentText = "";
        resetSummaryQuality();
        draft = contentText;
        loadingContent = false;
        activeContentRequestId = 0;
        return;
      }
    }

    loadingContent = true;
    errorMessage = null;

    try {
      if (targetMode === "transcript") {
        if (selectedChannelId) {
          track({
            event: "transcript_ensure_requested",
            video_id: targetVideoId,
            channel_id: selectedChannelId,
          });
        }
        let transcriptSuccess = false;
        let transcript;
        try {
          transcript = await ensureTranscript(targetVideoId);
          transcriptSuccess = true;
        } finally {
          if (selectedChannelId) {
            track({
              event: "transcript_ensure_completed",
              video_id: targetVideoId,
              channel_id: selectedChannelId,
              success: transcriptSuccess,
            });
          }
        }
        if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
          return;
        const presentation = resolveTranscriptPresentation(transcript!);
        const originalTranscript = presentation.originalText;
        contentText = presentation.content;
        transcriptRenderMode = presentation.renderMode;
        draftTranscriptRenderMode = presentation.renderMode;
        if (!(targetVideoId in originalTranscriptByVideoId)) {
          originalTranscriptByVideoId[targetVideoId] = originalTranscript;
        }
        // Cache the transcript
        const entry = contentCache.get(targetVideoId) ?? {};
        entry.transcript = {
          text: presentation.content,
          renderMode: presentation.renderMode,
        };
        contentCache.set(targetVideoId, entry);
        resetSummaryQuality();
        resetVideoInfo();
        void hydrateVideoHighlights(targetVideoId);
      } else {
        if (targetMode === "summary") {
          try {
            const summary = await getSummary(targetVideoId);
            if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
              return;
            contentText = cacheLoadedSummary(summary, targetVideoId);
            applySummaryQuality(summary);
            resetVideoInfo();
            void hydrateVideoHighlights(targetVideoId);
          } catch (error) {
            if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
              return;
            const message = (error as Error).message || "";
            // Queue owns summary generation; navigation stays read-only.
            if (message.includes("Summary not found")) {
              contentText = "";
              resetSummaryQuality();
              resetVideoInfo();
            } else {
              throw error;
            }
          }
        } else if (targetMode === "highlights") {
          const highlights = await hydrateVideoHighlights(targetVideoId, {
            showError: true,
          });
          if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
            return;
          if (!highlights) {
            return;
          }
          contentText = "";
          resetSummaryQuality();
          resetVideoInfo();
        } else {
          const info = await ensureVideoInfo(targetVideoId);
          if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
            return;
          videoInfo = info;
          contentText = "";
          // Cache the info
          const entry = contentCache.get(targetVideoId) ?? {};
          entry.info = info;
          contentCache.set(targetVideoId, entry);
          resetSummaryQuality();
        }
      }
      if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
        return;
      draft = contentText;
      if (targetMode === "transcript") {
        draftTranscriptRenderMode = transcriptRenderMode;
      }
    } catch (error) {
      if (activeContentRequestId === requestId) {
        contentText = "";
        draft = "";
        errorMessage = (error as Error).message;
      }
    } finally {
      if (activeContentRequestId === requestId) {
        loadingContent = false;
        activeContentRequestId = 0;
      }
    }
  }

  function startEdit() {
    editing = true;
    draft = contentText;
    draftTranscriptRenderMode = transcriptRenderMode;
  }

  function cancelEdit() {
    editing = false;
    draft = contentText;
    draftTranscriptRenderMode = transcriptRenderMode;
  }

  async function saveEdit() {
    if (!selectedVideoId) return;
    if (contentMode === "info" || contentMode === "highlights") return;
    const targetVideoId = selectedVideoId;

    loadingContent = true;
    errorMessage = null;

    try {
      if (contentMode === "transcript") {
        const transcript = await updateTranscript(
          targetVideoId,
          draft,
          draftTranscriptRenderMode,
        );
        const presentation = resolveTranscriptPresentation(transcript);
        contentText = presentation.content;
        transcriptRenderMode = presentation.renderMode;
        draftTranscriptRenderMode = presentation.renderMode;
        invalidateContentCache(targetVideoId, "transcript");
        resetSummaryQuality();
        resetVideoInfo();
      } else {
        const summary = await updateSummary(targetVideoId, draft);
        contentText = stripContentPrefix(
          summary.content || "Summary unavailable.",
        );
        invalidateContentCache(targetVideoId, "summary");
        applySummaryQuality(summary);
        if (selectedChannelId && contentMode === "summary") {
          syncSummaryTrackingSession(summary, targetVideoId, selectedChannelId);
        }
        resetVideoInfo();
      }
      editing = false;
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      loadingContent = false;
    }
  }

  async function regenerateSummaryContent() {
    if (!selectedVideoId || contentMode !== "summary") return;
    const targetVideoId = selectedVideoId;

    regeneratingSummaryVideoIds = [
      ...regeneratingSummaryVideoIds.filter((id) => id !== targetVideoId),
      targetVideoId,
    ];
    errorMessage = null;
    sidebarState.setVideos(
      sidebarState.videos.map((v) =>
        v.id === targetVideoId
          ? { ...v, summary_status: "loading" as const }
          : v,
      ),
    );

    try {
      const summary = await regenerateSummary(targetVideoId);
      invalidateContentCache(targetVideoId, "summary");
      sidebarState.setVideos(
        sidebarState.videos.map((v) =>
          v.id === targetVideoId
            ? { ...v, summary_status: "ready" as const }
            : v,
        ),
      );
      if (
        sidebarState.selectedVideoId === targetVideoId &&
        contentMode === "summary"
      ) {
        contentText = stripContentPrefix(
          summary.content || "Summary unavailable.",
        );
        applySummaryQuality(summary);
        if (selectedChannelId) {
          syncSummaryTrackingSession(summary, targetVideoId, selectedChannelId);
        }
        draft = contentText;
      }
    } catch (error) {
      errorMessage = (error as Error).message;
      sidebarState.setVideos(
        sidebarState.videos.map((v) =>
          v.id === targetVideoId
            ? { ...v, summary_status: "failed" as const }
            : v,
        ),
      );
    } finally {
      regeneratingSummaryVideoIds = regeneratingSummaryVideoIds.filter(
        (id) => id !== targetVideoId,
      );
    }
  }

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
          ? regeneratingSummaryVideoIds.includes(selectedVideoId)
          : false,
        aiAvailable: aiAvailable ?? false,
        onRegenerate: regenerateSummaryContent,
        showFormatAction: contentMode === "transcript",
        formatting: formattingContent && formattingVideoId === selectedVideoId,
        onFormat: cleanFormatting,
        showRevertAction: hasUpdatedTranscript,
        reverting: revertingContent && revertingVideoId === selectedVideoId,
        canRevert: canRevertTranscript,
        onRevert: revertToOriginalTranscript,
        busy: loadingContent,
        onRequestResetVideo: () => {
          showResetVideoConfirmation = true;
        },
        resetting: resettingVideo && resettingVideoId === selectedVideoId,
        showAcknowledgeToggle: true,
        acknowledged: selectedVideo?.acknowledged ?? false,
        onAcknowledgeToggle: toggleAcknowledge,
        showEditAction:
          contentMode === "transcript" || contentMode === "summary",
        onEdit: startEdit,
      });
    }
    return () => {
      mobileBottomBar.set({ kind: "sections" });
    };
  });

  // Full paginate + backfill only when browse opens or channel changes — not on
  // every filter tweak (that used to restart a long backfill and felt broken).
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

  async function resetVideoContent() {
    if (!selectedVideoId) return;
    const targetVideoId = selectedVideoId;

    resettingVideo = true;
    resettingVideoId = targetVideoId;
    errorMessage = null;

    sidebarState.setVideos(
      sidebarState.videos.map((v) =>
        v.id === targetVideoId
          ? {
              ...v,
              transcript_status: "pending" as const,
              summary_status: "pending" as const,
            }
          : v,
      ),
    );
    invalidateContentCache(targetVideoId, "transcript");
    invalidateContentCache(targetVideoId, "summary");
    contentText = "";
    draft = "";

    try {
      await resetVideo(targetVideoId);
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      resettingVideo = false;
      resettingVideoId = null;
    }
  }

  async function cleanFormatting() {
    if (!selectedVideoId || contentMode !== "transcript") return;
    const targetVideoId = selectedVideoId;
    const startedInEditMode = editing;
    const source = startedInEditMode ? draft : contentText;
    const requestId = ++formattingRequestSeq;

    activeFormattingRequest = requestId;
    formattingContent = true;
    formattingVideoId = targetVideoId;
    errorMessage = null;
    formattingNotice = "Formatting transcript with Ollama…";
    formattingNoticeVideoId = targetVideoId;
    formattingNoticeTone = "info";
    formattingAttemptsUsed = 0;
    formattingAttemptsMax = FORMAT_MAX_TURNS;
    formattingAttemptsVideoId = targetVideoId;

    try {
      const result = await cleanTranscriptFormatting(targetVideoId, source);
      const attemptsSummary = buildFormattingAttemptSummary(result);
      formattingAttemptsUsed = result.attempts_used;
      formattingAttemptsMax = result.max_attempts;
      formattingAttemptsVideoId = targetVideoId;
      if (startedInEditMode) {
        if (
          activeFormattingRequest === requestId &&
          sidebarState.selectedVideoId === targetVideoId &&
          editing
        ) {
          draft = result.content;
          if (result.content !== source) {
            draftTranscriptRenderMode = "markdown";
          }
        }
        formattingNotice =
          result.content === source
            ? `No formatting changes. ${attemptsSummary}`
            : `Formatting applied to draft. Save to persist. ${attemptsSummary}`;
        formattingNoticeVideoId = targetVideoId;
      } else {
        if (result.content !== source) {
          const transcript = await updateTranscript(
            targetVideoId,
            result.content,
            "markdown",
          );
          invalidateContentCache(targetVideoId, "transcript");
          if (
            activeFormattingRequest === requestId &&
            selectedVideoId === targetVideoId &&
            !editing
          ) {
            const presentation = resolveTranscriptPresentation(transcript);
            contentText = presentation.content;
            transcriptRenderMode = presentation.renderMode;
            draftTranscriptRenderMode = presentation.renderMode;
            draft = contentText;
          }
        }
        formattingNotice =
          result.content === source
            ? `No formatting changes. ${attemptsSummary}`
            : `Formatting applied and saved. ${attemptsSummary}`;
        formattingNoticeVideoId = targetVideoId;
      }
      formattingNoticeTone = "success";
      if (result.timed_out) {
        formattingNotice = `Formatting reached the time limit. Current transcript was kept. ${attemptsSummary}`;
        formattingNoticeVideoId = targetVideoId;
        formattingNoticeTone = "warning";
      } else if (!result.preserved_text) {
        errorMessage =
          "Formatting changed transcript words. Original transcript text was kept.";
        formattingNotice = `Safety guard kept original wording. Only spacing changes are allowed. ${attemptsSummary}`;
        formattingNoticeVideoId = targetVideoId;
        formattingNoticeTone = "warning";
      }
    } catch (error) {
      errorMessage = (error as Error).message;
      clearFormattingFeedback();
    } finally {
      if (activeFormattingRequest === requestId) {
        formattingContent = false;
        formattingVideoId = null;
      }
    }
  }

  async function revertToOriginalTranscript() {
    if (!selectedVideoId || contentMode !== "transcript") return;
    const targetVideoId = selectedVideoId;
    const original = originalTranscriptByVideoId[targetVideoId];
    if (!original) return;

    const startedInEditMode = editing;
    const source = startedInEditMode ? draft : contentText;
    if (source === original) {
      formattingNotice = "Already showing the original transcript.";
      formattingNoticeVideoId = targetVideoId;
      formattingNoticeTone = "info";
      formattingAttemptsUsed = null;
      formattingAttemptsMax = null;
      formattingAttemptsVideoId = null;
      return;
    }

    revertingContent = true;
    revertingVideoId = targetVideoId;
    errorMessage = null;
    formattingNotice = startedInEditMode
      ? "Reverting draft to original transcript…"
      : "Reverting transcript to original…";
    formattingNoticeVideoId = targetVideoId;
    formattingNoticeTone = "info";

    try {
      if (startedInEditMode) {
        if (selectedVideoId === targetVideoId && editing) {
          draft = original;
          draftTranscriptRenderMode = "plain_text";
        }
        formattingNotice =
          "Draft reset to original transcript. Save to persist.";
      } else {
        const transcript = await updateTranscript(
          targetVideoId,
          original,
          "plain_text",
        );
        invalidateContentCache(targetVideoId, "transcript");
        if (selectedVideoId === targetVideoId && !editing) {
          const presentation = resolveTranscriptPresentation(transcript);
          contentText = presentation.content;
          transcriptRenderMode = presentation.renderMode;
          draftTranscriptRenderMode = presentation.renderMode;
          draft = contentText;
        }
        formattingNotice = "Original transcript restored.";
      }
      formattingNoticeVideoId = targetVideoId;
      formattingNoticeTone = "success";
    } catch (error) {
      errorMessage = (error as Error).message;
      clearFormattingFeedback();
    } finally {
      revertingContent = false;
      revertingVideoId = null;
    }
  }

  async function setVideoTypeFilter(nextValue: VideoTypeFilter) {
    if (sidebarState.videoTypeFilter === nextValue) return;
    sidebarState.setVideoTypeFilter(nextValue);
    sidebarState.setVideos(filterVideosByType(sidebarState.videos, nextValue));
    await loadVideos(true, true);
  }

  async function setAcknowledgedFilter(nextValue: AcknowledgedFilter) {
    if (sidebarState.acknowledgedFilter === nextValue) return;
    sidebarState.setAcknowledgedFilter(nextValue);
    sidebarState.setVideos(
      filterVideosByAcknowledged(sidebarState.videos, nextValue),
    );
    await loadVideos(true, true);
  }

  function matchesAcknowledgedFilter(video: Video) {
    if (sidebarState.acknowledgedFilter === "ack") return video.acknowledged;
    if (sidebarState.acknowledgedFilter === "unack") return !video.acknowledged;
    return true;
  }

  async function toggleAcknowledge() {
    if (!sidebarState.selectedVideoId) return;
    const targetVideoId = sidebarState.selectedVideoId;
    const videoFromList = sidebarState.videos.find(
      (v) => v.id === targetVideoId,
    );
    const video =
      videoFromList ??
      (pendingSelectedVideo?.id === targetVideoId
        ? pendingSelectedVideo
        : null);
    if (!video) return;

    errorMessage = null;

    const previousVideos = [...sidebarState.videos];
    const previousPendingSelectedVideo = pendingSelectedVideo;
    const previousSelectedVideoId = sidebarState.selectedVideoId;
    const newAcknowledged = !video.acknowledged;

    // Invalidate in-flight snapshot applies so they cannot overwrite this toggle.
    sidebarState.bumpVideoListMutationEpoch();

    // Optimistic update — flip state immediately, no loading indicator
    const optimisticVideo = { ...video, acknowledged: newAcknowledged };
    const optimisticList = videoFromList
      ? applyOptimisticAcknowledge(
          sidebarState.videos,
          targetVideoId,
          newAcknowledged,
        ).filter(matchesAcknowledgedFilter)
      : previousVideos;
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

    const selectionDroppedFromFilter = Boolean(
      previousSelectedVideoId &&
      (videoFromList
        ? !optimisticList.some((v) => v.id === previousSelectedVideoId)
        : !matchesAcknowledgedFilter(optimisticVideo)),
    );
    if (selectionDroppedFromFilter) {
      editing = false;
      clearFormattingFeedbackState();
      if (videoFromList) {
        if (optimisticList.length === 0) {
          sidebarState.setSelectedVideoId(null);
          contentText = "";
          draft = "";
        } else {
          await selectVideo(optimisticList[0].id);
        }
      } else {
        sidebarState.setSelectedVideoId(null);
        pendingSelectedVideo = null;
        contentText = "";
        draft = "";
      }
    }

    try {
      const updated = await updateAcknowledged(targetVideoId, newAcknowledged);
      if (videoFromList) {
        sidebarState.setVideos(
          sidebarState.videos
            .map((v) => (v.id === updated.id ? updated : v))
            .filter(matchesAcknowledgedFilter),
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

      const stillSelected =
        sidebarState.selectedVideoId != null &&
        (sidebarState.videos.some(
          (v) => v.id === sidebarState.selectedVideoId,
        ) ||
          pendingSelectedVideo?.id === sidebarState.selectedVideoId);
      if (!stillSelected) {
        editing = false;
        clearFormattingFeedbackState();
        if (sidebarState.videos.length === 0) {
          sidebarState.setSelectedVideoId(null);
          contentText = "";
          draft = "";
        } else {
          await selectVideo(sidebarState.videos[0].id);
        }
      }
    } catch (error) {
      sidebarState.setVideos(previousVideos);
      sidebarState.setSelectedVideoId(previousSelectedVideoId);
      pendingSelectedVideo = previousPendingSelectedVideo;
      const reverted =
        previousVideos.find((v) => v.id === targetVideoId) ??
        (previousPendingSelectedVideo?.id === targetVideoId
          ? previousPendingSelectedVideo
          : null);
      if (reverted) {
        videoAcknowledgeSeq += 1;
        videoAcknowledgeSync = {
          seq: videoAcknowledgeSeq,
          video: reverted,
          confirmed: true,
        };
      }
      errorMessage = (error as Error).message;
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
      const summary = await getSummary(targetVideoId);
      if (
        selectedVideoId !== targetVideoId ||
        contentMode !== "summary" ||
        editing
      )
        return;
      if (!contentText.trim()) {
        contentText = cacheLoadedSummary(summary, targetVideoId);
        draft = contentText;
        resetVideoInfo();
        if (videoHighlightsByVideoId[targetVideoId] === undefined) {
          void hydrateVideoHighlights(targetVideoId);
        }
      }
      applySummaryQuality(summary);
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
      summaryQualityScore !== null ||
      summaryQualityNote !== null
    ) {
      return;
    }

    const needsReadySummaryRetry = shouldRetryReadySummaryLoad({
      contentMode,
      selectedVideo,
      contentText,
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

  function mergeUpdatedChannel(updatedChannel: Channel) {
    sidebarState.setChannels(
      sidebarState.channels.map((channel) =>
        channel.id === updatedChannel.id ? updatedChannel : channel,
      ),
    );
  }

  async function openChannelVideo(
    channelId: string,
    videoId: string,
    switchToContent = false,
  ) {
    await sidebarState.selectChannel(channelId, videoId);
    if (switchToContent) {
      mobileBrowseOpen = false;
    }
  }

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
    const url = $page.url;
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
    const url = new URL($page.url.href);
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
    summaryQualityScore,
    summaryQualityNote,
    summaryModelUsed,
    summaryQualityModelUsed,
    videoInfo,
    contentHtml,
    contentText,
    transcriptRenderMode,
    contentHighlights,
    selectedVideoHighlights,
    selectedVideoYoutubeUrl,
    draft,
    formattingContent,
    formattingVideoId,
    regeneratingSummaryVideoIds,
    revertingContent,
    revertingVideoId,
    resettingVideo,
    resettingVideoId,
    creatingHighlight,
    creatingHighlightVideoId,
    deletingHighlightId,
    canRevertTranscript,
    showRevertTranscriptAction: hasUpdatedTranscript,
    formattingNotice,
    formattingNoticeVideoId,
    formattingNoticeTone,
    citationScrollText,
  });
  const workspaceContentActions = $derived.by(() => ({
    onBack: () => {
      mobileBrowseOpen = true;
    },
    onSetMode: setMode,
    onStartEdit: startEdit,
    onCancelEdit: cancelEdit,
    onSaveEdit: saveEdit,
    onCleanFormatting: cleanFormatting,
    onRegenerateSummary: regenerateSummaryContent,
    onRevertTranscript: revertToOriginalTranscript,
    onResetVideo: resetVideoContent,
    onDraftChange: (value: string) => {
      draft = value;
    },
    onToggleAcknowledge: toggleAcknowledge,
    onCreateHighlight: saveSelectionHighlight,
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
    showDeleteConfirmation,
    showDeleteAccessPrompt,
    showResetVideoConfirmation,
  });
  const workspaceOverlaysActions = {
    onDismissError: () => {
      errorMessage = null;
    },
    onConfirmDelete: confirmDeleteChannel,
    onCancelDelete: cancelDeleteChannel,
    onConfirmAccessPrompt: confirmDeleteAccessPrompt,
    onCancelAccessPrompt: cancelDeleteAccessPrompt,
    onConfirmResetVideo: async () => {
      showResetVideoConfirmation = false;
      await resetVideoContent();
    },
    onCancelResetVideo: () => {
      showResetVideoConfirmation = false;
    },
  };
</script>

<WorkspaceShell
  currentSection="workspace"
  {aiIndicator}
  onOpenGuide={openGuide}
>
  {#snippet sidebar(shell)}
    <WorkspaceSidebar
      videoListMode="per_channel_preview"
      addSourceErrorMessage={errorMessage}
      initialChannelPreviews={$page.data.channelPreviews ?? {}}
      initialChannelPreviewsFilterKey={$page.data.channelPreviewsFilterKey ??
        "all:all:default"}
      shell={{
        collapsed: shell.collapsed,
        width: shell.width,
        mobileVisible: shell.mobileVisible ?? false,
        onToggleCollapse: shell.toggle,
      }}
      channelState={{
        ...sidebarState.channelState,
        canDeleteChannels: isOperator,
      }}
      channelActions={{
        ...sidebarState.channelActions,
        onDeleteChannel: handleDeleteChannel,
        onDeleteAccessRequired: () => {
          showDeleteAccessPrompt = true;
        },
      }}
      videoState={sidebarState.videoState}
      videoActions={sidebarState.videoActions}
      {videoAcknowledgeSync}
      onChannelSyncDateSaved={handleChannelSyncDateSaved}
    />
  {/snippet}
  {#snippet mobileTopBar()}
    <MobileYouTubeTopNav
      showBackInsteadOfMenu={!mobileBrowseOpen && Boolean(selectedVideoId)}
      onBack={() => {
        mobileBrowseOpen = true;
      }}
    >
      {#snippet trailing()}
        {#if mobileBrowseOpen}
          <div
            class="flex min-w-0 shrink-0 items-center justify-end"
            aria-label="Video filters"
          >
            <WorkspaceSidebarVideoFilterControl
              videoTypeFilter={sidebarState.videoState.videoTypeFilter}
              acknowledgedFilter={sidebarState.videoState.acknowledgedFilter}
              disabled={browseFilterDisabled}
              onSelectVideoType={onBrowseVideoTypeFilterChange}
              onSelectAcknowledged={onBrowseAcknowledgedFilterChange}
              onClearAllFilters={clearBrowseVideoFilters}
            />
          </div>
        {:else}
          <div class="w-10 shrink-0" aria-hidden="true"></div>
        {/if}
      {/snippet}
    </MobileYouTubeTopNav>
  {/snippet}
  {#snippet topBar()}
    <WorkspaceDesktopTopBar
      {contentMode}
      onSetMode={setMode}
      {selectedVideoId}
      {loadingContent}
      {editing}
      {hasUpdatedTranscript}
      {formattingContent}
      {formattingVideoId}
      {regeneratingSummaryVideoIds}
      {revertingContent}
      {revertingVideoId}
      {resettingVideo}
      {resettingVideoId}
      aiAvailable={aiAvailable ?? false}
      {canRevertTranscript}
      {selectedVideoYoutubeUrl}
      {draft}
      selectedVideoAcknowledged={selectedVideo?.acknowledged ?? false}
      onEdit={startEdit}
      onCancel={cancelEdit}
      onSave={saveEdit}
      onFormat={cleanFormatting}
      onRegenerate={regenerateSummaryContent}
      onRevert={revertToOriginalTranscript}
      onRequestResetVideo={() => {
        showResetVideoConfirmation = true;
      }}
      onDraftChange={(value) => {
        draft = value;
      }}
      onAcknowledgeToggle={toggleAcknowledge}
    >
      {#snippet searchBar()}
        {#if WorkspaceSearchBarComponent}
          <WorkspaceSearchBarComponent
            initialSearchStatus={searchStatus}
            onSearchResultSelect={handleSearchResultSelection}
          />
        {/if}
      {/snippet}
    </WorkspaceDesktopTopBar>
  {/snippet}

  <MobileHomeBrowseOverlay
    open={mobileBrowseOpen}
    channels={sidebarState.channels}
    selectedChannelId={sidebarState.selectedChannelId}
    onSelectChannel={(channelId) => {
      void sidebarState.selectChannel(channelId);
    }}
    onClose={() => {
      mobileBrowseOpen = false;
    }}
    channelState={{
      ...sidebarState.channelState,
      canDeleteChannels: isOperator,
    }}
    channelActions={{
      ...sidebarState.channelActions,
      onDeleteChannel: handleDeleteChannel,
      onDeleteAccessRequired: () => {
        showDeleteAccessPrompt = true;
      },
    }}
    videoState={{
      ...sidebarState.videoState,
      historyExhausted,
      backfillingHistory,
    }}
    videoActions={{
      ...sidebarState.videoActions,
      onLoadMoreVideos: loadMoreVideos,
    }}
    canDeleteChannels={isOperator}
    addSourceErrorMessage={errorMessage}
    onChannelSyncDateSaved={handleChannelSyncDateSaved}
  />

  <WorkspaceContentPanel
    selection={workspaceContentSelection}
    content={workspaceContentState}
    actions={workspaceContentActions}
    overlays={workspaceOverlaysState}
    overlayActions={workspaceOverlaysActions}
  />

  {#if FeatureGuideComponent}
    <FeatureGuideComponent
      open={guideOpen}
      step={guideStep}
      steps={tourSteps}
      docsUrl={DOCS_URL}
      onClose={closeGuide}
      onStep={setGuideStep}
    />
  {/if}
</WorkspaceShell>
