<script lang="ts">
  import { goto, replaceState as replacePageState } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount, tick } from "svelte";
  import {
    addChannel,
    backfillChannelVideos,
    cleanTranscriptFormatting,
    createHighlight,
    deleteHighlight,
    deleteChannel,
    ensureSummary,
    ensureTranscript,
    ensureVideoInfo,
    getChannelSnapshot,
    getChannelSyncDepth,
    getVideo,
    getVideoHighlights,
    getSummary,
    getWorkspaceBootstrapWhenAvailable,
    listVideos,
    refreshChannel,
    regenerateSummary,
    updateSummary,
    updateTranscript,
    updateAcknowledged,
    updateChannel,
  } from "$lib/api";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import { DOCS_URL } from "$lib/app-config";
  import FeatureGuide from "$lib/components/FeatureGuide.svelte";
  import type { TourStep } from "$lib/components/FeatureGuide.svelte";
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import WorkspaceContentPanel from "$lib/components/workspace/WorkspaceContentPanel.svelte";
  import WorkspaceSearchBar from "$lib/components/workspace/WorkspaceSearchBar.svelte";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import WorkspaceSidebar from "$lib/components/workspace/WorkspaceSidebar.svelte";
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
  import {
    buildWorkspaceViewHref,
    mergeWorkspaceViewState,
    parseWorkspaceViewUrlState,
  } from "$lib/view-url";
  import {
    buildChannelViewCacheKey,
    cloneSyncDepthState,
    cloneVideos,
    createChannelViewCache,
    type ChannelSyncDepthState,
  } from "$lib/channel-view-cache";
  import { channelOrderFromList } from "$lib/workspace/channels";
  import {
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
  import {
    resolveDefaultContentMode,
    WORKSPACE_CONTENT_MODE_ORDER,
  } from "$lib/workspace/navigation";
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

  let channels = $state<Channel[]>([]);
  let channelOrder = $state<string[]>([]);
  let videos = $state<Video[]>([]);
  let selectedChannelId = $state<string | null>(null);
  let selectedVideoId = $state<string | null>(null);
  let channelSortMode = $state<ChannelSortMode>("custom");

  // -- Guide tour (URL-driven: ?guide=0, ?guide=1, ...) --
  let guideOpen = $state(false);
  let guideStep = $state(0);

  const tourSteps: TourStep[] = [
    {
      selector: "#workspace",
      title: "Welcome to dAstIll",
      body: "dAstIll helps you keep up with YouTube without the doom-scrolling. It pulls transcripts from your favorite channels and creates AI summaries, so you can quickly decide which videos are worth your time.",
      placement: "right",
      prepare: () => {
        mobileTab = "browse";
      },
    },
    {
      selector: "#channel-input",
      title: "Add a Channel",
      body: "Paste any YouTube channel URL or handle here and press Enter. dAstIll will fetch the channel and start pulling in its videos automatically.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "browse";
      },
    },
    {
      selector: "#workspace",
      title: "Your Channels",
      body: "All the channels you follow show up here. Click any channel to see its videos. You can search, sort, and drag to reorder them.",
      placement: "right",
      prepare: () => {
        mobileTab = "browse";
      },
    },
    {
      selector: "#videos",
      title: "Video List",
      body: "These are the videos from the selected channel, newest first. Older videos keep loading in the background. Click any video to read its content.",
      placement: "right",
      prepare: () => {
        mobileTab = "browse";
      },
    },
    {
      selector: "#video-filter-button",
      title: "Filter Videos",
      body: "Narrow the list by type (full videos or Shorts) or by read status (all, unread, read). Useful when a channel has hundreds of uploads.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "browse";
      },
    },
    {
      selector: "#content-mode-tabs",
      title: "Read the Transcript",
      body: "Every video's spoken content is available as a full transcript you can read at your own pace - much faster than watching the whole video.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "content";
        if (contentMode !== "transcript") {
          void setMode("transcript");
        }
      },
    },
    {
      selector: "#content-mode-tabs",
      title: "AI Summary",
      body: "Don't have time for the full transcript? The AI summary gives you the key points in a fraction of the time. This is the fastest way to decide if a video is worth watching.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "content";
        if (contentMode !== "summary") {
          void setMode("summary");
        }
      },
    },
    {
      selector: "#content-mode-tabs",
      title: "Your Highlights",
      body: "Found something worth remembering? Select any text in the transcript or summary and save it as a highlight. All your saved passages for this video appear here.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "content";
        if (contentMode !== "highlights") {
          void setMode("highlights");
        }
      },
    },
    {
      selector: "#content-mode-tabs",
      title: "Video Details",
      body: "See the publish date, duration, description, and thumbnail for any video - all without leaving the app.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "content";
        if (contentMode !== "info") {
          void setMode("info");
        }
      },
    },
    {
      selector: "#content-actions",
      title: "Actions",
      body: "Use these buttons to edit text, clean up formatting with AI, regenerate a summary, or jump straight to the video on YouTube.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "content";
        if (contentMode === "info" || contentMode === "highlights") {
          void setMode("transcript");
        }
      },
    },
    {
      selector: "#content-actions",
      title: "Track What You've Read",
      body: "Mark videos as read once you've reviewed them, then use the filter to show only unread videos. That way you always know what's new.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "content";
        if (contentMode === "info" || contentMode === "highlights") {
          void setMode("transcript");
        }
      },
    },
    {
      selector: "#ai-status-pill",
      title: "AI Availability",
      body: "This indicator shows whether AI features like summaries and chat are online. Browsing, reading, and highlights work even when AI is offline.",
      placement: "bottom",
    },
    {
      selector: "#nav-docs-link",
      title: "Learn More",
      body: "Want to go deeper? The documentation covers everything from setup to advanced features.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "browse";
      },
      fallbackSelector: "#mobile-nav-docs-link",
    },
  ];

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

  let loadingChannels = $state(false);
  let aiAvailable = $state<boolean | null>(null);
  let aiStatus = $state<AiStatus | null>(null);
  let searchStatus = $state<SearchStatus | null>(null);
  let loadingVideos = $state(false);
  let loadingContent = $state(false);

  let addingChannel = $state(false);
  let errorMessage = $state<string | null>(null);
  let showDeleteConfirmation = $state(false);
  let showDeleteAccessPrompt = $state(false);
  let channelIdToDelete = $state<string | null>(null);
  let summaryQualityScore = $state<number | null>(null);
  let summaryQualityNote = $state<string | null>(null);
  let summaryModelUsed = $state<string | null>(null);
  let summaryQualityModelUsed = $state<string | null>(null);
  let videoInfo = $state<VideoInfoPayload | null>(null);
  let syncDepth = $state<ChannelSyncDepthState | null>(null);

  let contentMode = $state<WorkspaceContentMode>("transcript");
  let mobileTab = $state<"browse" | "content">("browse");
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
  let regeneratingSummary = $state(false);
  let regeneratingVideoId = $state<string | null>(null);
  let revertingContent = $state(false);
  let revertingVideoId = $state<string | null>(null);
  let videoHighlightsByVideoId = $state<Record<string, Highlight[]>>({});
  let nextOptimisticHighlightId = -1;
  let creatingHighlight = $state(false);
  let creatingHighlightVideoId = $state<string | null>(null);
  let deletingHighlightId = $state<number | null>(null);
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
  let originalTranscriptByVideoId = $state<Record<string, string>>({});
  const contentCache = new Map<
    string,
    {
      transcript?: {
        text: string;
        renderMode: TranscriptRenderMode;
      };
      summary?: { text: string; quality: SummaryPayload };
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

  let offset = $state(0);
  const limit = 20;
  let hasMore = $state(true);
  let historyExhausted = $state(false);
  let backfillingHistory = $state(false);
  let allowLoadedVideoSyncDepthOverride = $state(false);

  let videoTypeFilter = $state<VideoTypeFilter>("all");
  let acknowledgedFilter = $state<AcknowledgedFilter>("all");
  let workspaceStateHydrated = $state(false);
  let viewUrlHydrated = $state(false);
  let pendingSelectedVideo = $state<Video | null>(null);

  const selectedChannel = $derived(
    channels.find((channel) => channel.id === selectedChannelId) ?? null,
  );
  const selectedVideo = $derived(
    videos.find((video) => video.id === selectedVideoId) ??
      (pendingSelectedVideo?.id === selectedVideoId
        ? pendingSelectedVideo
        : null),
  );

  function getChannelViewKey(channelId: string) {
    return buildChannelViewCacheKey(
      channelId,
      videoTypeFilter,
      acknowledgedFilter,
    );
  }

  function restoreCachedChannelVideoState(state: CachedChannelVideoState) {
    videos = cloneVideos(state.videos);
    offset = state.offset;
    hasMore = state.hasMore;
    historyExhausted = state.historyExhausted;
    backfillingHistory = state.backfillingHistory;
    allowLoadedVideoSyncDepthOverride = state.allowLoadedVideoSyncDepthOverride;
    syncDepth = cloneSyncDepthState(state.syncDepth);
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
      syncDepth = null;
      return;
    }
    try {
      syncDepth = await getChannelSyncDepth(selectedChannelId);
    } catch {
      syncDepth = null;
    }
  }

  function clearSelectedVideoState() {
    selectedVideoId = null;
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

    selectedVideoId = preferredVideoId;
    let hasSelectedVideo = videos.some(
      (video) => video.id === preferredVideoId,
    );
    let scannedPages = 0;
    const targetChannelId = selectedChannelId;
    const pendingSelectedVideoRequest =
      hasSelectedVideo || !targetChannelId
        ? Promise.resolve(null)
        : resolvePendingSelectedVideo(preferredVideoId, targetChannelId);

    if (!loadingContent && contentText.trim().length === 0) {
      void loadContent();
    }

    while (
      !hasSelectedVideo &&
      hasMore &&
      scannedPages < SELECTED_VIDEO_SCAN_PAGE_LIMIT &&
      targetChannelId &&
      selectedChannelId === targetChannelId &&
      selectedVideoId === preferredVideoId
    ) {
      const next = await listVideos(
        targetChannelId,
        limit,
        offset,
        videoTypeFilter,
        acknowledged,
      );
      scannedPages += 1;
      if (next.length === 0) {
        hasMore = false;
        break;
      }

      videos = [...videos, ...next];
      offset += next.length;
      hasMore = next.length === limit;
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
      loadingVideos = true;
      errorMessage = null;
    }

    try {
      if (selectedChannelId !== channelId) {
        return;
      }

      const isAck = resolveAcknowledgedParam(acknowledgedFilter);

      syncDepth = snapshot.sync_depth;
      allowLoadedVideoSyncDepthOverride = false;
      videos = snapshot.videos;
      offset = snapshot.videos.length;
      hasMore = snapshot.videos.length === limit;
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
        loadingVideos = false;
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

    const updated = await updateChannel(selectedChannelId, {
      earliest_sync_date: oldest.toISOString(),
    });
    channels = channels.map((channel) =>
      channel.id === selectedChannelId ? updated : channel,
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
    } catch (error) {
      removeVideoHighlight(targetVideoId, optimisticHighlight.id);
      errorMessage = (error as Error).message;
    } finally {
      creatingHighlight = false;
      creatingHighlightVideoId = null;
    }
  }

  async function deleteExistingHighlight(highlightId: number) {
    if (!selectedVideoId || !isOperator) {
      return;
    }

    const targetVideoId = selectedVideoId;
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
    channelOrder = channelOrderFromList(channels);
  }

  function applySummaryQuality(summary: SummaryPayload) {
    const presentation = resolveSummaryQualityPresentation(summary);
    summaryQualityScore = presentation.score;
    summaryQualityNote = presentation.note;
    summaryModelUsed = presentation.modelUsed;
    summaryQualityModelUsed = presentation.qualityModelUsed;
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
      selectedChannelId = restored.selectedChannelId ?? null;
    }
    if ("selectedVideoId" in restored) {
      selectedVideoId = restored.selectedVideoId ?? null;
    }
    if (restored.contentMode && isWorkspaceContentMode(restored.contentMode)) {
      contentMode = restored.contentMode;
    }
    if (
      restored.videoTypeFilter &&
      isWorkspaceVideoTypeFilter(restored.videoTypeFilter)
    ) {
      videoTypeFilter = restored.videoTypeFilter;
    }
    if (restored.acknowledgedFilter) {
      acknowledgedFilter = restored.acknowledgedFilter;
    }
    if (restored.channelOrder) {
      channelOrder = restored.channelOrder;
    }
    if (restored.channelSortMode) {
      channelSortMode = restored.channelSortMode;
    }

    if (selectedVideoId) {
      mobileTab = "content";
    } else if (selectedChannelId) {
      mobileTab = "browse";
    } else {
      mobileTab = "browse";
    }
  }

  function persistViewUrl() {
    if (!viewUrlHydrated || typeof window === "undefined") return;
    const nextHref = buildWorkspaceViewHref({
      selectedChannelId,
      selectedVideoId,
      contentMode,
      videoTypeFilter,
      acknowledgedFilter,
    });
    const nextUrl = new URL(nextHref, window.location.origin);
    if (
      nextUrl.pathname === window.location.pathname &&
      nextUrl.search === window.location.search
    ) {
      return;
    }
    replacePageState(nextUrl, window.history.state);
  }

  function persistWorkspaceState() {
    if (!workspaceStateHydrated || typeof localStorage === "undefined") return;
    const snapshot: WorkspaceStateSnapshot = {
      selectedChannelId,
      selectedVideoId,
      contentMode,
      videoTypeFilter,
      acknowledgedFilter,
      channelOrder,
      channelSortMode,
    };
    saveWorkspaceState(localStorage, snapshot);
  }

  $effect(() => {
    persistWorkspaceState();
  });

  $effect(() => {
    persistViewUrl();
  });

  onMount(() => {
    restoreWorkspaceState();
    workspaceStateHydrated = true;
    void (async () => {
      const selectedChannelIdAtMount = selectedChannelId;
      const selectedVideoIdAtMount = selectedVideoId;
      const acknowledgedAtMount = resolveAcknowledgedParam(acknowledgedFilter);

      // Resolve initial state from server bootstrap (SSR) + IndexedDB warm-start.
      // IndexedDB is always read here before any network API call (VAL-CROSS-004).
      const bootstrapResult = await resolveBootstrapOnMount({
        serverBootstrap: $page.data.bootstrap ?? null,
        selectedChannelId: selectedChannelIdAtMount,
        viewSnapshotCacheKey: selectedChannelIdAtMount
          ? buildWorkspaceSnapshotCacheKey(
              selectedChannelIdAtMount,
              videoTypeFilter,
              acknowledgedAtMount,
            )
          : null,
      });

      const hasInitialData = Boolean(
        bootstrapResult.channels && bootstrapResult.channels.length > 0,
      );

      if (bootstrapResult.channels && bootstrapResult.channels.length > 0) {
        channels = applySavedChannelOrder(
          bootstrapResult.channels,
          channelOrder,
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
    })();

    const restoredGuideStep = resolveGuideStepFromUrl(
      new URL(window.location.href),
      tourSteps.length,
    );
    if (restoredGuideStep !== null) {
      guideStep = restoredGuideStep;
      guideOpen = true;
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
      loadingChannels = true;
      errorMessage = null;
    }

    try {
      const bootstrap = await getWorkspaceBootstrapWhenAvailable({
        selectedChannelId: previousSelectedChannelId,
        videoType: videoTypeFilter,
        acknowledged: resolveAcknowledgedParam(acknowledgedFilter),
        limit,
        retryDelayMs: 500,
      });

      channels = applySavedChannelOrder(bootstrap.channels, channelOrder);
      syncChannelOrderFromList();
      void putCachedChannels(channels);

      // Apply AI/search status from the bootstrap response
      aiAvailable = bootstrap.ai_available;
      aiStatus = bootstrap.ai_status;
      searchStatus = bootstrap.search_status;
      void putCachedBootstrapMeta({
        ai_available: bootstrap.ai_available,
        ai_status: bootstrap.ai_status,
        search_status: bootstrap.search_status,
      });

      const initialChannelId = resolveInitialChannelSelection(
        bootstrap.channels,
        previousSelectedChannelId,
        null,
      );

      if (!initialChannelId) {
        selectedChannelId = null;
        mobileTab = "browse";
        clearSelectedVideoState();
        videos = [];
        syncDepth = null;
        offset = 0;
        hasMore = true;
        historyExhausted = false;
        backfillingHistory = false;
        allowLoadedVideoSyncDepthOverride = false;
      } else {
        const preferredVideoId =
          initialChannelId === previousSelectedChannelId
            ? previousSelectedVideoId
            : null;
        const canReuseRenderedSnapshot =
          initialChannelId === previousSelectedChannelId && videos.length > 0;

        selectedChannelId = initialChannelId;
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
          selectedVideoId = preferredVideoId;
          videos = [];
          offset = 0;
          hasMore = true;
          historyExhausted = false;
          backfillingHistory = false;
          allowLoadedVideoSyncDepthOverride = false;
          syncDepth = null;
          if (!silent) {
            loadingVideos = true;
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
        loadingChannels = false;
        loadingVideos = false;
      }
    }
  }

  function reorderChannels(nextOrder: string[]) {
    channels = applySavedChannelOrder(channels, nextOrder);
    channelOrder = nextOrder;
  }
  async function handleAddChannel(input: string) {
    if (!input.trim()) return false;

    const { optimisticChannel, tempId, trimmedInput } =
      buildOptimisticChannel(input);
    addingChannel = true;
    errorMessage = null;

    const previousChannels = [...channels];
    const previousSelectedId = selectedChannelId;

    channels = [optimisticChannel, ...channels];
    channelOrder = [tempId, ...channelOrder];

    try {
      const channel = await addChannel(trimmedInput);

      channels = replaceOptimisticChannel(channels, tempId, channel);
      channelOrder = replaceOptimisticChannelId(
        channelOrder,
        tempId,
        channel.id,
      );
      selectedChannelId = channel.id;
      void putCachedChannels(channels);

      await selectChannel(channel.id);
      return true;
    } catch (error) {
      channels = previousChannels;
      selectedChannelId = previousSelectedId;
      syncChannelOrderFromList();
      errorMessage = (error as Error).message;
      return false;
    } finally {
      addingChannel = false;
    }
  }

  async function handleDeleteChannel(channelId: string) {
    if (!isOperator) {
      showDeleteAccessPrompt = true;
      return;
    }

    channelIdToDelete = channelId;
    showDeleteConfirmation = true;
  }

  async function confirmDeleteChannel() {
    if (!channelIdToDelete || !isOperator) return;
    const channelId = channelIdToDelete;
    showDeleteConfirmation = false;
    channelIdToDelete = null;

    try {
      await deleteChannel(channelId);
      void removeCachedChannel(channelId);
      channels = removeChannelFromCollection(channels, channelId);
      channelOrder = removeChannelId(channelOrder, channelId);
      if (selectedChannelId === channelId) {
        const nextChannelId = resolveNextChannelSelection(channels, channelId);
        if (nextChannelId) {
          await selectChannel(nextChannelId);
        } else {
          selectedChannelId = null;
          selectedVideoId = null;
          mobileTab = "browse";
          videos = [];
          contentText = "";
          draft = "";
        }
      }
    } catch (error) {
      errorMessage = (error as Error).message;
    }
  }

  function cancelDeleteChannel() {
    showDeleteConfirmation = false;
    channelIdToDelete = null;
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
    if (!selectedChannelId) {
      syncDepth = null;
    }
  });

  async function selectChannel(
    channelId: string,
    preferredVideoId: string | null = null,
    fromUserInteraction = false,
  ) {
    const channelViewKey = getChannelViewKey(channelId);
    const cachedChannelVideoState = channelVideoStateCache.get(channelViewKey);
    const hasCachedChannelVideoState =
      !!cachedChannelVideoState && cachedChannelVideoState.videos.length > 0;

    if (fromUserInteraction) mobileTab = "browse";
    selectedChannelId = channelId;
    clearSelectedVideoState();
    selectedVideoId = preferredVideoId;
    resetSummaryQuality();
    resetVideoInfo();
    editing = false;
    clearFormattingFeedback();
    if (hasCachedChannelVideoState && cachedChannelVideoState) {
      restoreCachedChannelVideoState(cachedChannelVideoState);
      loadingVideos = false;
      void refreshAndLoadVideos(channelId, false, preferredVideoId, true);
      return;
    }

    videos = [];
    offset = 0;
    hasMore = true;
    historyExhausted = false;
    backfillingHistory = false;
    allowLoadedVideoSyncDepthOverride = false;
    await refreshAndLoadVideos(channelId, false, preferredVideoId);
  }

  let refreshingChannel = $state(false);

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
      loadSnapshot: () =>
        getChannelSnapshot(channelId, {
          limit,
          offset: 0,
          videoType: videoTypeFilter,
          acknowledged: isAck,
        }),
      applySnapshot: (snapshot, silent = false) =>
        applyChannelSnapshot(channelId, snapshot, preferredVideoId, silent),
      refreshChannel: () => refreshChannel(channelId),
      shouldReloadAfterRefresh: () => selectedChannelId === channelId,
      onRefreshingChange: (refreshing) => {
        refreshingChannel = refreshing;
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
    if (loadingVideos && !silent) return;

    if (!silent) loadingVideos = true;
    if (!silent) errorMessage = null;

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
      if (reset) {
        allowLoadedVideoSyncDepthOverride = false;
      }

      if (reset) {
        await hydrateSelectedVideo(selectedVideoId, isAck);
      }
    } catch (error) {
      if (!silent || !errorMessage) errorMessage = (error as Error).message;
    } finally {
      if (!silent) loadingVideos = false;
    }
  }

  async function loadMoreVideos() {
    if (!selectedChannelId || loadingVideos || backfillingHistory) return;

    if (hasMore) {
      await loadVideos(false);
      allowLoadedVideoSyncDepthOverride = true;
      await syncEarliestDateFromLoadedVideos();
      return;
    }

    backfillingHistory = true;
    errorMessage = null;

    try {
      // Try to backfill a batch of 50
      const result = await backfillChannelVideos(selectedChannelId, 50);

      // Use the explicit flag from backend to know if we hit the actual end of YouTube results
      if (result.exhausted) {
        historyExhausted = true;
      }

      // Load the newly added videos (if any) or just try to see if we can find more older ones
      await loadVideos(false);
      allowLoadedVideoSyncDepthOverride = true;
      await syncEarliestDateFromLoadedVideos();
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      backfillingHistory = false;
    }
  }

  async function selectVideo(videoId: string, fromUserInteraction = false) {
    if (fromUserInteraction) mobileTab = "content";
    if (videoId === selectedVideoId) return;
    selectedVideoId = videoId;
    contentText = "";
    draft = "";
    const video = videos.find((v) => v.id === videoId);
    const cachedHighlights = videoHighlightsByVideoId[videoId];
    const highlights =
      cachedHighlights ?? (await hydrateVideoHighlights(videoId)) ?? [];
    if (selectedVideoId !== videoId) return;
    contentMode = resolveDefaultContentMode({
      hasHighlights: highlights.length > 0,
      hasSummary: video?.summary_status === "ready",
      hasTranscript: video?.transcript_status === "ready",
    });
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
    contentMode = mode;
    resetSummaryQuality();
    resetVideoInfo();
    editing = false;
    clearFormattingFeedback();
    await loadContent();
  }

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
        const transcript = await ensureTranscript(targetVideoId);
        if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
          return;
        const presentation = resolveTranscriptPresentation(transcript);
        const originalTranscript = presentation.originalText;
        contentText = presentation.content;
        transcriptRenderMode = presentation.renderMode;
        draftTranscriptRenderMode = presentation.renderMode;
        if (!(targetVideoId in originalTranscriptByVideoId)) {
          originalTranscriptByVideoId = {
            ...originalTranscriptByVideoId,
            [targetVideoId]: originalTranscript,
          };
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
          const summary = await ensureSummary(targetVideoId);
          if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
            return;
          contentText = stripContentPrefix(
            summary.content || "Summary unavailable.",
          );
          applySummaryQuality(summary);
          // Cache the summary
          const entry = contentCache.get(targetVideoId) ?? {};
          entry.summary = { text: contentText, quality: summary };
          contentCache.set(targetVideoId, entry);
          resetVideoInfo();
          void hydrateVideoHighlights(targetVideoId);
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

    regeneratingSummary = true;
    regeneratingVideoId = targetVideoId;
    errorMessage = null;
    videos = videos.map((v) =>
      v.id === targetVideoId ? { ...v, summary_status: "loading" as const } : v,
    );

    try {
      const summary = await regenerateSummary(targetVideoId);
      invalidateContentCache(targetVideoId, "summary");
      videos = videos.map((v) =>
        v.id === targetVideoId ? { ...v, summary_status: "ready" as const } : v,
      );
      if (selectedVideoId === targetVideoId && contentMode === "summary") {
        contentText = stripContentPrefix(
          summary.content || "Summary unavailable.",
        );
        applySummaryQuality(summary);
        draft = contentText;
      }
    } catch (error) {
      errorMessage = (error as Error).message;
      videos = videos.map((v) =>
        v.id === targetVideoId
          ? { ...v, summary_status: "failed" as const }
          : v,
      );
    } finally {
      regeneratingSummary = false;
      regeneratingVideoId = null;
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
          selectedVideoId === targetVideoId &&
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
    if (videoTypeFilter === nextValue) return;
    videoTypeFilter = nextValue;
    videos = filterVideosByType(videos, nextValue);
    await loadVideos(true, true);
  }

  async function setAcknowledgedFilter(nextValue: AcknowledgedFilter) {
    if (acknowledgedFilter === nextValue) return;
    acknowledgedFilter = nextValue;
    videos = filterVideosByAcknowledged(videos, nextValue);
    await loadVideos(true, true);
  }

  function matchesAcknowledgedFilter(video: Video) {
    if (acknowledgedFilter === "ack") return video.acknowledged;
    if (acknowledgedFilter === "unack") return !video.acknowledged;
    return true;
  }

  async function toggleAcknowledge() {
    if (!selectedVideoId) return;
    const video = videos.find((v) => v.id === selectedVideoId);
    if (!video) return;

    loadingContent = true;
    errorMessage = null;

    try {
      const updated = await updateAcknowledged(
        selectedVideoId,
        !video.acknowledged,
      );
      videos = videos
        .map((v) => (v.id === updated.id ? updated : v))
        .filter(matchesAcknowledgedFilter);

      const stillSelected = videos.some((v) => v.id === selectedVideoId);
      if (!stillSelected) {
        editing = false;
        clearFormattingFeedback();
        if (videos.length === 0) {
          selectedVideoId = null;
          contentText = "";
          draft = "";
        } else {
          await selectVideo(videos[0].id);
        }
      }
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      loadingContent = false;
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

    const timer = setInterval(() => {
      void refreshSummaryQuality();
    }, 7000);
    return () => clearInterval(timer);
  });

  function mergeUpdatedChannel(updatedChannel: Channel) {
    channels = channels.map((channel) =>
      channel.id === updatedChannel.id ? updatedChannel : channel,
    );
  }

  async function openChannelOverview(channelId: string) {
    await goto(`/channels/${encodeURIComponent(channelId)}`);
  }

  async function openChannelVideo(
    channelId: string,
    videoId: string,
    switchToContent = false,
  ) {
    await selectChannel(channelId, videoId, false);
    if (switchToContent) {
      mobileTab = "content";
    }
  }

  const workspaceSidebarChannelState = $derived({
    channels,
    selectedChannelId,
    loadingChannels,
    addingChannel,
    channelSortMode,
    canDeleteChannels: isOperator,
  });
  const workspaceSidebarVideoState = $derived({
    videos,
    pendingSelectedVideo,
    selectedVideoId,
    selectedChannel,
    loadingVideos,
    refreshingChannel,
    hasMore,
    historyExhausted,
    backfillingHistory,
    videoTypeFilter,
    acknowledgedFilter,
    syncDepth,
    allowLoadedVideoSyncDepthOverride,
  });
  const workspaceSidebarChannelActions = {
    onChannelSortModeChange: (nextValue: ChannelSortMode) => {
      channelSortMode = nextValue;
    },
    onAddChannel: handleAddChannel,
    onSelectChannel: (channelId: string) => {
      if (channelId === selectedChannelId) {
        selectedChannelId = null;
        clearSelectedVideoState();
        return;
      }
      void selectChannel(channelId, null, true);
    },
    onOpenChannelOverview: openChannelOverview,
    onDeleteChannel: handleDeleteChannel,
    onDeleteAccessRequired: () => {
      showDeleteAccessPrompt = true;
    },
    onReorderChannels: reorderChannels,
    onChannelUpdated: mergeUpdatedChannel,
  };
  const workspaceSidebarVideoActions = {
    onSelectVideo: (videoId: string) => void selectVideo(videoId, true),
    onSelectChannelVideo: (channelId: string, videoId: string) =>
      void openChannelVideo(channelId, videoId),
    onLoadMoreVideos: loadMoreVideos,
    onVideoTypeFilterChange: setVideoTypeFilter,
    onAcknowledgedFilterChange: setAcknowledgedFilter,
  };
  const mobileWorkspaceSidebarVideoActions = {
    ...workspaceSidebarVideoActions,
    onSelectVideo: (videoId: string) => {
      mobileTab = "content";
      void selectVideo(videoId, true);
    },
    onSelectChannelVideo: (channelId: string, videoId: string) =>
      void openChannelVideo(channelId, videoId, true),
  };
  const workspaceContentSelection = $derived({
    mobileVisible: true,
    selectedChannel,
    selectedVideo,
    selectedVideoId,
    contentMode,
  });
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
    regeneratingSummary,
    regeneratingVideoId,
    revertingContent,
    revertingVideoId,
    creatingHighlight,
    creatingHighlightVideoId,
    deletingHighlightId,
    canRevertTranscript,
    showRevertTranscriptAction: hasUpdatedTranscript,
    formattingNotice,
    formattingNoticeVideoId,
    formattingNoticeTone,
  });
  const workspaceContentActions = $derived.by(() => ({
    onBack: () => {
      mobileTab = "browse";
    },
    onSetMode: setMode,
    onStartEdit: startEdit,
    onCancelEdit: cancelEdit,
    onSaveEdit: saveEdit,
    onCleanFormatting: cleanFormatting,
    onRegenerateSummary: regenerateSummaryContent,
    onRevertTranscript: revertToOriginalTranscript,
    onDraftChange: (value: string) => {
      draft = value;
    },
    onToggleAcknowledge: toggleAcknowledge,
    onCreateHighlight: saveSelectionHighlight,
    onDeleteHighlight: isOperator ? deleteExistingHighlight : undefined,
    onShowChannels: () => {
      mobileTab = "browse";
    },
    onShowVideos: () => {
      mobileTab = "browse";
    },
  }));
  const workspaceOverlaysState = $derived({
    errorMessage,
    showDeleteConfirmation,
    showDeleteAccessPrompt,
  });
  const workspaceOverlaysActions = {
    onDismissError: () => {
      errorMessage = null;
    },
    onConfirmDelete: confirmDeleteChannel,
    onCancelDelete: cancelDeleteChannel,
    onConfirmAccessPrompt: confirmDeleteAccessPrompt,
    onCancelAccessPrompt: cancelDeleteAccessPrompt,
  };
</script>

<WorkspaceShell
  currentSection="workspace"
  {aiIndicator}
  onOpenGuide={openGuide}
>
  {#snippet sidebar({
    collapsed: sidebarCollapsed,
    toggle: toggleSidebar,
    width: sidebarWidth,
  })}
    <WorkspaceSidebar
      videoListMode="per_channel_preview"
      initialChannelPreviews={$page.data.channelPreviews ?? {}}
      initialChannelPreviewsFilterKey={$page.data.channelPreviewsFilterKey ??
        "all:all"}
      shell={{
        collapsed: sidebarCollapsed,
        width: sidebarWidth,
        mobileVisible: mobileTab === "browse",
        onToggleCollapse: toggleSidebar,
      }}
      channelState={workspaceSidebarChannelState}
      channelActions={workspaceSidebarChannelActions}
      videoState={workspaceSidebarVideoState}
      videoActions={workspaceSidebarVideoActions}
    />
  {/snippet}

  {#snippet topBar()}
    <div class="flex items-center gap-6" id="content-mode-tabs">
      {#each WORKSPACE_CONTENT_MODE_ORDER as mode}
        <button
          type="button"
          class={`-mb-px border-b-2 py-3.5 text-[11px] font-bold uppercase tracking-[0.12em] transition-colors ${
            contentMode === mode
              ? "border-[var(--accent)] text-[var(--accent-strong)]"
              : "border-transparent text-[var(--soft-foreground)] opacity-75 hover:text-[var(--foreground)] hover:opacity-100"
          }`}
          aria-pressed={contentMode === mode}
          onclick={() => void setMode(mode)}
        >
          {mode === "transcript"
            ? "Transcript"
            : mode === "summary"
              ? "Summary"
              : mode === "highlights"
                ? "Highlights"
                : "Info"}
        </button>
      {/each}

      {#if selectedVideoId && !loadingContent && !editing}
        <div
          class="ml-2 border-l border-[var(--border-soft)] pl-4"
          id="content-actions"
        >
          <ContentEditor
            editing={false}
            busy={loadingContent}
            aiAvailable={aiAvailable ?? false}
            formatting={formattingContent &&
              formattingVideoId === selectedVideoId}
            regenerating={regeneratingSummary &&
              regeneratingVideoId === selectedVideoId}
            reverting={revertingContent && revertingVideoId === selectedVideoId}
            showFormatAction={contentMode === "transcript"}
            showRegenerateAction={contentMode === "summary"}
            showRevertAction={hasUpdatedTranscript}
            showEditAction={contentMode === "transcript" ||
              contentMode === "summary"}
            canRevert={canRevertTranscript}
            youtubeUrl={selectedVideoYoutubeUrl}
            value={draft}
            acknowledged={selectedVideo?.acknowledged ?? false}
            onEdit={startEdit}
            onCancel={cancelEdit}
            onSave={saveEdit}
            onFormat={cleanFormatting}
            onRegenerate={regenerateSummaryContent}
            onRevert={revertToOriginalTranscript}
            onChange={(value) => {
              draft = value;
            }}
            onAcknowledgeToggle={toggleAcknowledge}
          />
        </div>
      {/if}
    </div>

    <div class="flex min-w-0 flex-1 items-center justify-end gap-3">
      <WorkspaceSearchBar
        initialSearchStatus={searchStatus}
        onSearchResultSelect={handleSearchResultSelection}
      />
    </div>
  {/snippet}

  {#if mobileTab === "browse"}
    <div
      class="fixed inset-0 z-[80] lg:hidden"
      role="dialog"
      aria-modal="true"
      aria-label="Browse channels"
    >
      <button
        type="button"
        class="absolute inset-0 bg-[var(--overlay)]"
        onclick={() => {
          mobileTab = "content";
        }}
        aria-label="Close sidebar"
      ></button>
      <div
        class="relative z-10 h-full w-[min(85vw,20rem)] overflow-hidden border-r border-[var(--accent-border-soft)] bg-[var(--surface-strong)] shadow-2xl"
      >
        <WorkspaceSidebar
          videoListMode="per_channel_preview"
          initialChannelPreviews={$page.data.channelPreviews ?? {}}
          initialChannelPreviewsFilterKey={$page.data
            .channelPreviewsFilterKey ?? "all:all"}
          shell={{
            collapsed: false,
            width: undefined,
            mobileVisible: true,
            onToggleCollapse: () => {},
          }}
          channelState={workspaceSidebarChannelState}
          channelActions={workspaceSidebarChannelActions}
          videoState={workspaceSidebarVideoState}
          videoActions={mobileWorkspaceSidebarVideoActions}
        />
      </div>
    </div>
  {/if}

  <WorkspaceContentPanel
    selection={workspaceContentSelection}
    content={workspaceContentState}
    actions={workspaceContentActions}
    overlays={workspaceOverlaysState}
    overlayActions={workspaceOverlaysActions}
  />

  <FeatureGuide
    open={guideOpen}
    step={guideStep}
    steps={tourSteps}
    docsUrl={DOCS_URL}
    onClose={closeGuide}
    onStep={setGuideStep}
  />
</WorkspaceShell>
