<script lang="ts">
  import { replaceState as replacePageState } from "$app/navigation";
  import { onMount } from "svelte";
  import {
    addChannel,
    backfillChannelVideos,
    cleanTranscriptFormatting,
    createHighlight,
    deleteHighlight,
    deleteChannel,
    getChannelSnapshot,
    getChannelSyncDepth,
    getVideoHighlights,
    getVideoInfo,
    getWorkspaceBootstrapWhenAvailable,
    getSummary,
    getTranscript,
    isAiAvailable,
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
  import FeatureGuide, {
    type TourStep,
  } from "$lib/components/FeatureGuide.svelte";
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import WorkspaceChannelSidebar from "$lib/components/workspace/WorkspaceChannelSidebar.svelte";
  import WorkspaceContentPanel from "$lib/components/workspace/WorkspaceContentPanel.svelte";
  import WorkspaceHeader from "$lib/components/workspace/WorkspaceHeader.svelte";
  import WorkspaceMobileTabBar from "$lib/components/workspace/WorkspaceMobileTabBar.svelte";
  import WorkspaceVideoSidebar from "$lib/components/workspace/WorkspaceVideoSidebar.svelte";
  import type {
    AiStatus,
    Channel,
    ChannelSnapshot,
    CreateHighlightRequest,
    Highlight,
    HighlightSource,
    SearchResult,
    Summary as SummaryPayload,
    Transcript as TranscriptPayload,
    TranscriptRenderMode,
    VideoInfo as VideoInfoPayload,
    Video,
    VideoTypeFilter,
  } from "$lib/types";
  import {
    applySavedChannelOrder,
    loadWorkspaceState,
    markChannelRefreshed,
    restoreWorkspaceSnapshot,
    resolveInitialChannelSelection,
    saveWorkspaceState,
    shouldRefreshChannel,
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
  import { resolveOldestLoadedReadyVideoDate } from "$lib/sync-depth";
  import {
    buildWorkspaceViewHref,
    mergeWorkspaceViewState,
    parseWorkspaceViewUrlState,
  } from "$lib/view-url";
  import { channelOrderFromList } from "$lib/workspace/channels";
  import {
    resolveSummaryQualityPresentation,
    resolveTranscriptPresentation,
    stripContentPrefix,
  } from "$lib/workspace/content";
  import {
    type AcknowledgedFilter,
    type ChannelSortMode,
    isWorkspaceContentMode,
    isWorkspaceVideoTypeFilter,
    type WorkspaceContentMode,
  } from "$lib/workspace/types";
  const FORMAT_MAX_TURNS = 5;
  const FORMAT_HARD_TIMEOUT_MINUTES = 5;
  const CHANNEL_REFRESH_TTL_MS = 5 * 60 * 1000;
  const SELECTED_VIDEO_SCAN_PAGE_LIMIT = 8;
  const channelLastRefreshedAt = new Map<string, number>();

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
      title: "Channel Sidebar",
      body: "All your followed YouTube channels live here. Drag to reorder, search to filter, and use the trash icon to manage.",
      placement: "right",
      prepare: () => {
        mobileTab = "channels";
      },
    },
    {
      selector: "#channel-input",
      title: "Follow a Channel",
      body: "Paste any YouTube channel URL, handle, or channel ID and press enter. The channel and its videos are fetched automatically.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "channels";
      },
    },
    {
      selector: "#videos",
      title: "Video Library",
      body: "Every video from your channels appears here. Use the filter button to show only long-form, shorts, or unread videos.",
      placement: "right",
      prepare: () => {
        mobileTab = "videos";
      },
    },
    {
      selector: "#content-mode-tabs",
      title: "Transcript View",
      body: "Each video's transcript is automatically extracted and displayed. This is the raw text of everything said in the video.",
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
      title: "Summary View",
      body: "Switch to Summary to see an AI-generated distillation of the video's key points. The model name and a quality score are shown below the text.",
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
      title: "Info View",
      body: "The Info tab shows video metadata - publish date, duration, description, and thumbnail - pulled directly from YouTube.",
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
      title: "Content Actions",
      body: "The action bar gives you quick access to: edit the text inline, reformat the transcript with AI, regenerate the summary, jump to the video on YouTube, and mark a video as read.",
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
      title: "AI Status",
      body: "The colored dot shows AI engine availability. In showcase mode, AI features like formatting and summary generation are disabled, but browsing and reading work fully.",
      placement: "bottom",
    },
    {
      selector: "#guide-trigger",
      title: "Documentation",
      body: "Find detailed guides and references in the docs — everything you need to get the most out of dAstIll.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "channels";
      },
    },
  ];

  function openGuide() {
    guideStep = 0;
    guideOpen = true;
    syncGuideToUrl(0);
  }

  function closeGuide() {
    guideOpen = false;
    removeGuideFromUrl();
  }

  function setGuideStep(s: number) {
    guideStep = s;
    syncGuideToUrl(s);
  }

  function syncGuideToUrl(s: number) {
    if (typeof window === "undefined") return;
    const url = new URL(window.location.href);
    url.searchParams.set("guide", String(s));
    window.history.replaceState(window.history.state, "", url);
  }

  function removeGuideFromUrl() {
    if (typeof window === "undefined") return;
    const url = new URL(window.location.href);
    url.searchParams.delete("guide");
    window.history.replaceState(window.history.state, "", url);
  }

  let loadingChannels = $state(false);
  let aiAvailable = $state<boolean | null>(null);
  let aiStatus = $state<AiStatus | null>(null);
  let loadingVideos = $state(false);
  let loadingContent = $state(false);

  let addingChannel = $state(false);
  let errorMessage = $state<string | null>(null);
  let showDeleteConfirmation = $state(false);
  let channelIdToDelete = $state<string | null>(null);
  let summaryQualityScore = $state<number | null>(null);
  let summaryQualityNote = $state<string | null>(null);
  let summaryModelUsed = $state<string | null>(null);
  let summaryQualityModelUsed = $state<string | null>(null);
  let videoInfo = $state<VideoInfoPayload | null>(null);
  let syncDepth = $state<{
    earliest_sync_date: string | null;
    earliest_sync_date_user_set: boolean;
    derived_earliest_ready_date: string | null;
  } | null>(null);

  let contentMode = $state<WorkspaceContentMode>("transcript");
  let mobileTab = $state<"channels" | "videos" | "content">("channels");
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

  const selectedChannel = $derived(
    channels.find((channel) => channel.id === selectedChannelId) ?? null,
  );
  const selectedVideo = $derived(
    videos.find((video) => video.id === selectedVideoId) ?? null,
  );

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
    contentText = "";
    transcriptRenderMode = "plain_text";
    draft = "";
    draftTranscriptRenderMode = "plain_text";
    resetSummaryQuality();
    resetVideoInfo();
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
      void selectVideo(videos[0].id);
      return;
    }

    selectedVideoId = preferredVideoId;
    let hasSelectedVideo = videos.some(
      (video) => video.id === preferredVideoId,
    );
    let scannedPages = 0;
    const targetChannelId = selectedChannelId;

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
      void selectVideo(videos[0].id);
      return;
    }

    if (!loadingContent && contentText.trim().length === 0) {
      void loadContent();
    }
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

      const isAck =
        acknowledgedFilter === "ack"
          ? true
          : acknowledgedFilter === "unack"
            ? false
            : undefined;

      syncDepth = snapshot.sync_depth;
      allowLoadedVideoSyncDepthOverride = false;
      videos = snapshot.videos;
      offset = snapshot.videos.length;
      hasMore = snapshot.videos.length === limit;
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
    const current = videoHighlightsByVideoId[videoId] ?? [];
    storeVideoHighlights(videoId, mergeHighlightIntoList(current, highlight));
  }

  function removeVideoHighlight(videoId: string, highlightId: number) {
    const current = videoHighlightsByVideoId[videoId] ?? [];
    storeVideoHighlights(
      videoId,
      current.filter((item) => item.id !== highlightId),
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
    if (!selectedVideoId) {
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
    summaryQualityScore = null;
    summaryQualityNote = null;
    summaryModelUsed = null;
    summaryQualityModelUsed = null;
  }

  function resetVideoInfo() {
    videoInfo = null;
  }

  function clearFormattingFeedback() {
    formattingNotice = null;
    formattingNoticeVideoId = null;
    formattingAttemptsUsed = null;
    formattingAttemptsMax = null;
    formattingAttemptsVideoId = null;
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
      try {
        await loadChannels(null, true);
      } finally {
        viewUrlHydrated = true;
      }
    })();

    // Restore guide from URL
    const guideParam = new URL(window.location.href).searchParams.get("guide");
    if (guideParam !== null) {
      const parsed = parseInt(guideParam, 10);
      if (!Number.isNaN(parsed) && parsed >= 0 && parsed < tourSteps.length) {
        guideStep = parsed;
        guideOpen = true;
      }
    }
  });

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

  async function loadChannels(
    preferredChannelId: string | null = null,
    retryUntilBackendReachable = false,
  ) {
    loadingChannels = true;
    errorMessage = null;

    try {
      const isAck =
        acknowledgedFilter === "ack"
          ? true
          : acknowledgedFilter === "unack"
            ? false
            : undefined;
      const bootstrap = retryUntilBackendReachable
        ? await getWorkspaceBootstrapWhenAvailable({
            selectedChannelId: preferredChannelId ?? selectedChannelId,
            limit,
            offset: 0,
            videoType: videoTypeFilter,
            acknowledged: isAck,
            retryDelayMs: 500,
          })
        : await getWorkspaceBootstrapWhenAvailable({
            selectedChannelId: preferredChannelId ?? selectedChannelId,
            limit,
            offset: 0,
            videoType: videoTypeFilter,
            acknowledged: isAck,
            retryDelayMs: 0,
          });

      aiAvailable = bootstrap.ai_available;
      aiStatus = bootstrap.ai_status;
      channels = applySavedChannelOrder(bootstrap.channels, channelOrder);
      syncChannelOrderFromList();
      const initialChannelId = resolveInitialChannelSelection(
        channels,
        selectedChannelId,
        bootstrap.selected_channel_id ?? preferredChannelId,
      );
      if (!initialChannelId) {
        selectedChannelId = null;
        clearSelectedVideoState();
        syncDepth = null;
      } else {
        const preferredVideoId =
          initialChannelId === selectedChannelId ? selectedVideoId : null;
        selectedChannelId = initialChannelId;
        resetSummaryQuality();
        resetVideoInfo();
        editing = false;
        clearFormattingFeedback();
        if (
          bootstrap.snapshot &&
          bootstrap.snapshot.channel_id === initialChannelId
        ) {
          await applyChannelSnapshot(
            initialChannelId,
            bootstrap.snapshot,
            preferredVideoId,
          );
        } else {
          clearSelectedVideoState();
          syncDepth = null;
        }
      }
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      loadingChannels = false;
    }
  }

  function reorderChannels(nextOrder: string[]) {
    channels = applySavedChannelOrder(channels, nextOrder);
    channelOrder = nextOrder;
  }
  async function handleAddChannel(input: string) {
    if (!input.trim()) return false;

    const trimmedInput = input.trim();
    addingChannel = true;
    errorMessage = null;

    // Optimistic update
    const tempId = `temp-${Date.now()}`;
    const optimisticChannel: Channel = {
      id: tempId,
      name:
        trimmedInput.includes("youtube.com") ||
        trimmedInput.includes("youtu.be")
          ? "Fetching Channel..."
          : trimmedInput,
      added_at: new Date().toISOString(),
    };

    const previousChannels = [...channels];
    const previousSelectedId = selectedChannelId;

    channels = [optimisticChannel, ...channels];
    channelOrder = [tempId, ...channelOrder];
    selectedChannelId = tempId;

    try {
      const channel = await addChannel(trimmedInput);

      // Replace temp channel with real one locally
      channels = channels.map((c) => (c.id === tempId ? channel : c));
      channelOrder = channelOrder.map((id) =>
        id === tempId ? channel.id : id,
      );
      selectedChannelId = channel.id;

      // Load videos for the new channel (bypass TTL since it's brand new)
      await selectChannel(channel.id);
      return true;
    } catch (error) {
      // Rollback on error
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
    channelIdToDelete = channelId;
    showDeleteConfirmation = true;
  }

  async function confirmDeleteChannel() {
    if (!channelIdToDelete) return;
    const channelId = channelIdToDelete;
    showDeleteConfirmation = false;
    channelIdToDelete = null;

    try {
      await deleteChannel(channelId);
      channels = channels.filter((c) => c.id !== channelId);
      channelOrder = channelOrder.filter((id) => id !== channelId);
      if (selectedChannelId === channelId) {
        const nextChannel = channels.length > 0 ? channels[0] : null;
        if (nextChannel) {
          await selectChannel(nextChannel.id);
        } else {
          selectedChannelId = null;
          selectedVideoId = null;
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
    if (fromUserInteraction) mobileTab = "videos";
    selectedChannelId = channelId;
    clearSelectedVideoState();
    selectedVideoId = preferredVideoId;
    resetSummaryQuality();
    resetVideoInfo();
    editing = false;
    clearFormattingFeedback();
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
  ) {
    const isAck =
      acknowledgedFilter === "ack"
        ? true
        : acknowledgedFilter === "unack"
          ? false
          : undefined;

    const snapshot = await getChannelSnapshot(channelId, {
      limit,
      offset: 0,
      videoType: videoTypeFilter,
      acknowledged: isAck,
    });
    await applyChannelSnapshot(channelId, snapshot, preferredVideoId);

    // Skip YouTube refresh if channel was refreshed recently
    if (
      !bypassTtl &&
      !shouldRefreshChannel(
        channelLastRefreshedAt,
        channelId,
        CHANNEL_REFRESH_TTL_MS,
      )
    ) {
      return;
    }

    // Lazy load/refresh the channel in the background
    refreshingChannel = true;
    try {
      await refreshChannel(channelId);
      markChannelRefreshed(channelLastRefreshedAt, channelId);
      // After refresh, silently reload current channel data.
      if (selectedChannelId === channelId) {
        const refreshedSnapshot = await getChannelSnapshot(channelId, {
          limit,
          offset: 0,
          videoType: videoTypeFilter,
          acknowledged: isAck,
        });
        await applyChannelSnapshot(
          channelId,
          refreshedSnapshot,
          preferredVideoId,
          true,
        );
      }
    } catch (error) {
      if (!errorMessage) {
        errorMessage = (error as Error).message;
      }
    } finally {
      refreshingChannel = false;
    }
  }

  async function loadVideos(reset = false, silent = false) {
    if (!selectedChannelId) return;
    if (loadingVideos && !silent) return;

    if (!silent) loadingVideos = true;
    if (!silent) errorMessage = null;

    try {
      const isAck =
        acknowledgedFilter === "ack"
          ? true
          : acknowledgedFilter === "unack"
            ? false
            : undefined;
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
    const video = videos.find((v) => v.id === videoId);
    if (
      contentMode === "summary" &&
      video &&
      video.summary_status !== "ready"
    ) {
      contentMode = "transcript";
    }
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
        const transcript = await getTranscript(targetVideoId);
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
          const summary = await getSummary(targetVideoId);
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
          const info = await getVideoInfo(targetVideoId);
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

    try {
      const summary = await regenerateSummary(targetVideoId);
      invalidateContentCache(targetVideoId, "summary");
      if (selectedVideoId === targetVideoId && contentMode === "summary") {
        contentText = stripContentPrefix(
          summary.content || "Summary unavailable.",
        );
        applySummaryQuality(summary);
        draft = contentText;
      }
    } catch (error) {
      errorMessage = (error as Error).message;
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
    formattingNotice = `Formatting transcript with Ollama… (up to ${FORMAT_MAX_TURNS} tries, ${FORMAT_HARD_TIMEOUT_MINUTES} minute cutoff)`;
    formattingNoticeVideoId = targetVideoId;
    formattingNoticeTone = "info";
    formattingAttemptsUsed = 0;
    formattingAttemptsMax = FORMAT_MAX_TURNS;
    formattingAttemptsVideoId = targetVideoId;

    try {
      const result = await cleanTranscriptFormatting(targetVideoId, source);
      const attemptsSummary = `Attempts ${result.attempts_used}/${result.max_attempts}.`;
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
        formattingNotice = `Formatting stopped after ${FORMAT_HARD_TIMEOUT_MINUTES} minutes. Current transcript was kept. ${attemptsSummary}`;
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
    await loadVideos(true);
  }

  async function setAcknowledgedFilter(nextValue: AcknowledgedFilter) {
    if (acknowledgedFilter === nextValue) return;
    acknowledgedFilter = nextValue;
    await loadVideos(true);
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

  $effect(() => {
    const timer = setInterval(() => {
      void isAiAvailable()
        .then((status) => {
          aiAvailable = status.available;
          aiStatus = status.status;
        })
        .catch(() => {
          aiAvailable = false;
          aiStatus = "offline";
        });
    }, 30000);
    return () => clearInterval(timer);
  });

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
</script>

<div
  class="page-shell page-shell--panel-mobile-shell page-shell--with-mobile-nav min-h-screen px-3 py-4 max-lg:px-0 lg:px-6"
>
  <a
    href="#main-content"
    class="skip-link absolute left-4 top-4 z-50 rounded-full bg-[var(--accent)] px-4 py-2 text-sm font-semibold text-white"
  >
    Skip to Main Content
  </a>

  <WorkspaceHeader
    {aiIndicator}
    onOpenGuide={openGuide}
    onSearchResultSelect={handleSearchResultSelection}
  />

  <main
    id="main-content"
    class="panel-shell-main mx-auto mt-0 grid w-full max-w-[1440px] items-start lg:mt-4 lg:grid-cols-[260px_300px_minmax(0,1fr)] lg:gap-0 xl:grid-cols-[280px_340px_minmax(0,1fr)]"
  >
    <WorkspaceChannelSidebar
      mobileVisible={mobileTab === "channels"}
      {channels}
      {selectedChannelId}
      {loadingChannels}
      {addingChannel}
      {channelSortMode}
      onChannelSortModeChange={(nextValue) => {
        channelSortMode = nextValue;
      }}
      onAddChannel={handleAddChannel}
      onSelectChannel={(channelId) => selectChannel(channelId, null, true)}
      onDeleteChannel={handleDeleteChannel}
      onReorderChannels={reorderChannels}
    />

    <WorkspaceVideoSidebar
      mobileVisible={mobileTab === "videos"}
      {selectedChannelId}
      {selectedVideoId}
      {selectedChannel}
      {videos}
      {loadingVideos}
      {refreshingChannel}
      {hasMore}
      {historyExhausted}
      {backfillingHistory}
      {videoTypeFilter}
      {acknowledgedFilter}
      {syncDepth}
      {allowLoadedVideoSyncDepthOverride}
      onSelectVideo={(videoId) => selectVideo(videoId, true)}
      onLoadMoreVideos={loadMoreVideos}
      onVideoTypeFilterChange={setVideoTypeFilter}
      onAcknowledgedFilterChange={setAcknowledgedFilter}
    />

    <WorkspaceContentPanel
      mobileVisible={mobileTab === "content"}
      {selectedChannel}
      {selectedVideo}
      {selectedVideoId}
      {contentMode}
      {loadingContent}
      {editing}
      aiAvailable={aiAvailable ?? false}
      {summaryQualityScore}
      {summaryQualityNote}
      {summaryModelUsed}
      {summaryQualityModelUsed}
      {videoInfo}
      {contentHtml}
      {contentText}
      {transcriptRenderMode}
      {contentHighlights}
      {selectedVideoHighlights}
      {selectedVideoYoutubeUrl}
      {draft}
      {formattingContent}
      {formattingVideoId}
      {regeneratingSummary}
      {regeneratingVideoId}
      {revertingContent}
      {revertingVideoId}
      {creatingHighlight}
      {creatingHighlightVideoId}
      {deletingHighlightId}
      {canRevertTranscript}
      {formattingNotice}
      {formattingNoticeVideoId}
      {formattingNoticeTone}
      onSetMode={setMode}
      onStartEdit={startEdit}
      onCancelEdit={cancelEdit}
      onSaveEdit={saveEdit}
      onCleanFormatting={cleanFormatting}
      onRegenerateSummary={regenerateSummaryContent}
      onRevertTranscript={revertToOriginalTranscript}
      onDraftChange={(value) => {
        draft = value;
      }}
      onToggleAcknowledge={toggleAcknowledge}
      onCreateHighlight={saveSelectionHighlight}
      onDeleteHighlight={deleteExistingHighlight}
      onShowChannels={() => {
        mobileTab = "channels";
      }}
      onShowVideos={() => {
        mobileTab = "videos";
      }}
    />
  </main>

  <WorkspaceMobileTabBar
    activeTab={mobileTab}
    onTabChange={(tab) => {
      mobileTab = tab;
    }}
  />

  {#if errorMessage}
    <div
      class="mobile-bottom-stack-offset fixed bottom-6 left-1/2 z-50 w-[min(90vw,420px)] -translate-x-1/2 rounded-[var(--radius-md)] border border-[var(--danger-border)] bg-[var(--surface)] px-4 py-3 shadow-lg fade-in"
      role="status"
      aria-live="polite"
    >
      <div class="flex items-start gap-3">
        <p
          class="flex-1 text-[13px] font-medium text-[var(--danger-foreground)]"
        >
          {errorMessage}
        </p>
        <button
          onclick={() => {
            errorMessage = null;
          }}
          class="shrink-0 text-[var(--soft-foreground)] opacity-40 hover:opacity-80"
          aria-label="Dismiss"
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
    </div>
  {/if}

  <ConfirmationModal
    show={showDeleteConfirmation}
    title="Remove Channel?"
    message="Are you sure you want to remove this channel? All its downloaded transcripts and summaries will be permanently deleted."
    confirmLabel="Delete"
    cancelLabel="Keep"
    tone="danger"
    onConfirm={confirmDeleteChannel}
    onCancel={cancelDeleteChannel}
  />

  <FeatureGuide
    open={guideOpen}
    step={guideStep}
    steps={tourSteps}
    docsUrl={DOCS_URL}
    onClose={closeGuide}
    onStep={setGuideStep}
  />

</div>
