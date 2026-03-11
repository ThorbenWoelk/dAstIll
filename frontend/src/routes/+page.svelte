<script lang="ts">
  import { onMount, tick } from "svelte";
  import {
    addChannel,
    backfillChannelVideos,
    cleanTranscriptFormatting,
    deleteChannel,
    getChannelSnapshot,
    getChannelSyncDepth,
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
  import AiStatusIndicator from "$lib/components/AiStatusIndicator.svelte";
  import ChannelCard from "$lib/components/ChannelCard.svelte";
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
  import TranscriptView from "$lib/components/TranscriptView.svelte";
  import VideoCard from "$lib/components/VideoCard.svelte";
  import type {
    AiStatus,
    Channel,
    ChannelSnapshot,
    Summary as SummaryPayload,
    VideoInfo as VideoInfoPayload,
    Video,
    VideoTypeFilter,
  } from "$lib/types";
  import {
    applySavedChannelOrder,
    prioritizeChannelOrder,
    resolveInitialChannelSelection,
    WORKSPACE_STATE_KEY,
    type WorkspaceStateSnapshot,
  } from "$lib/channel-workspace";
  import {
    normalizeTranscriptForRender,
    renderMarkdown,
  } from "$lib/utils/markdown";
  import {
    resolveDisplayedSyncDepthIso,
    resolveOldestLoadedReadyVideoDate,
  } from "$lib/sync-depth";

  const secondaryButtonClass =
    "inline-flex items-center justify-center rounded-full border border-[var(--border)] px-5 py-3 text-xs font-semibold uppercase tracking-[0.2em] text-[var(--foreground)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]";

  const channelSubmitButtonClass =
    "inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-[var(--border)] bg-[var(--surface)] text-xl leading-none text-[var(--accent)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent-strong)] disabled:cursor-not-allowed disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]";
  const FORMAT_MAX_TURNS = 5;
  const FORMAT_HARD_TIMEOUT_MINUTES = 5;
  const CHANNEL_REFRESH_TTL_MS = 5 * 60 * 1000;
  const SELECTED_VIDEO_SCAN_PAGE_LIMIT = 8;
  const channelLastRefreshedAt = new Map<string, number>();

  type AcknowledgedFilter = "all" | "unack" | "ack";

  let channels = $state<Channel[]>([]);
  let channelOrder = $state<string[]>([]);
  let videos = $state<Video[]>([]);
  let selectedChannelId = $state<string | null>(null);
  let selectedVideoId = $state<string | null>(null);
  let draggedChannelId = $state<string | null>(null);
  let dragOverChannelId = $state<string | null>(null);
  let channelSearchQuery = $state("");
  let channelSortMode = $state<"custom" | "alpha" | "newest">("custom");
  let channelSearchOpen = $state(false);
  let manageChannels = $state(false);

  let channelInput = $state("");
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
  let videoInfo = $state<VideoInfoPayload | null>(null);
  let syncDepth = $state<{
    earliest_sync_date: string | null;
    earliest_sync_date_user_set: boolean;
    derived_earliest_ready_date: string | null;
  } | null>(null);

  let contentMode = $state<"transcript" | "summary" | "info">("transcript");
  let mobileTab = $state<"channels" | "videos" | "content">("channels");
  let contentText = $state("");
  let contentRenderText = $derived(
    contentMode === "transcript"
      ? normalizeTranscriptForRender(contentText)
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
  let originalTranscriptByVideoId = $state<Record<string, string>>({});
  const contentCache = new Map<
    string,
    {
      transcript?: string;
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
  let filterMenuOpen = $state(false);
  let filterMenuContainer = $state<HTMLDivElement | null>(null);
  let videoListContainer = $state<HTMLDivElement | null>(null);
  let atVideoListBottom = $state(false);
  let filterMenuLabel = $derived(
    videoTypeFilter === "all"
      ? "Open video filter menu."
      : `Video type filter set to ${videoTypeFilter}. Open filter menu.`,
  );

  const selectedChannel = $derived(
    channels.find((channel) => channel.id === selectedChannelId) ?? null,
  );

  const filteredChannels = $derived.by(() => {
    let result = channels;
    if (channelSearchQuery.trim()) {
      const query = channelSearchQuery.trim().toLowerCase();
      result = result.filter(
        (channel) =>
          channel.name?.toLowerCase().includes(query) ||
          channel.handle?.toLowerCase().includes(query),
      );
    }
    if (channelSortMode === "alpha") {
      result = [...result].sort((a, b) =>
        (a.name ?? "").localeCompare(b.name ?? ""),
      );
    } else if (channelSortMode === "newest") {
      result = [...result].sort((a, b) =>
        (b.added_at ?? "").localeCompare(a.added_at ?? ""),
      );
    }
    return result;
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
    contentText = "";
    draft = "";
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

  function isContentMode(
    value: unknown,
  ): value is "transcript" | "summary" | "info" {
    return value === "transcript" || value === "summary" || value === "info";
  }

  function isVideoTypeFilter(value: unknown): value is VideoTypeFilter {
    return value === "all" || value === "long" || value === "short";
  }

  function stripPrefix(text: string): string {
    return text.replace(/^(?:Transcript|Summary):\s*/i, "").trimStart();
  }

  function syncChannelOrderFromList() {
    channelOrder = channels.map((channel) => channel.id);
  }

  function applySummaryQuality(summary: SummaryPayload) {
    summaryQualityScore =
      typeof summary.quality_score === "number"
        ? Math.max(0, Math.min(10, Math.round(summary.quality_score)))
        : null;
    summaryQualityNote = summary.quality_note?.trim() || null;
    summaryModelUsed = summary.model_used ?? null;
  }

  function resetSummaryQuality() {
    summaryQualityScore = null;
    summaryQualityNote = null;
    summaryModelUsed = null;
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
    targetMode: "transcript" | "summary" | "info",
  ) {
    return (
      activeContentRequestId === requestId &&
      selectedVideoId === targetVideoId &&
      contentMode === targetMode
    );
  }

  function formatPublishedAt(value: string | null | undefined) {
    if (!value) return "Unknown";
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;
    return new Intl.DateTimeFormat(undefined, {
      dateStyle: "long",
      timeStyle: "short",
    }).format(date);
  }

  function formatSyncDate(value: string | null | undefined) {
    if (!value) return "Unknown";
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return "Unknown";
    return new Intl.DateTimeFormat(undefined, {
      dateStyle: "long",
    }).format(date);
  }

  function formatCount(value: number | null | undefined) {
    if (value === null || value === undefined) return "Unknown";
    return new Intl.NumberFormat().format(value);
  }

  function formatDuration(
    seconds: number | null | undefined,
    iso8601: string | null | undefined,
  ) {
    if (seconds !== null && seconds !== undefined && seconds >= 0) {
      const hrs = Math.floor(seconds / 3600);
      const mins = Math.floor((seconds % 3600) / 60);
      const secs = seconds % 60;
      if (hrs > 0) {
        return `${hrs}h ${mins}m ${secs}s`;
      }
      return `${mins}m ${secs}s`;
    }
    if (iso8601) return iso8601;
    return "Unknown";
  }

  function restoreWorkspaceState() {
    if (typeof localStorage === "undefined") return;
    const raw = localStorage.getItem(WORKSPACE_STATE_KEY);
    if (!raw) return;

    try {
      const snapshot = JSON.parse(raw) as Partial<WorkspaceStateSnapshot>;
      if (
        typeof snapshot.selectedChannelId === "string" ||
        snapshot.selectedChannelId === null
      ) {
        selectedChannelId = snapshot.selectedChannelId;
      }
      if (
        typeof snapshot.selectedVideoId === "string" ||
        snapshot.selectedVideoId === null
      ) {
        selectedVideoId = snapshot.selectedVideoId;
      }
      if (isContentMode(snapshot.contentMode)) {
        contentMode = snapshot.contentMode;
      }
      if (isVideoTypeFilter(snapshot.videoTypeFilter)) {
        videoTypeFilter = snapshot.videoTypeFilter;
      } else if (typeof snapshot.hideShorts === "boolean") {
        videoTypeFilter = snapshot.hideShorts ? "long" : "all";
      }
      if (
        snapshot.acknowledgedFilter &&
        ["all", "unack", "ack"].includes(snapshot.acknowledgedFilter)
      ) {
        acknowledgedFilter = snapshot.acknowledgedFilter;
      }
      if (Array.isArray(snapshot.channelOrder)) {
        channelOrder = snapshot.channelOrder.filter(
          (id): id is string => typeof id === "string",
        );
      }
      if (
        snapshot.channelSortMode &&
        ["custom", "alpha", "newest"].includes(snapshot.channelSortMode)
      ) {
        channelSortMode = snapshot.channelSortMode;
      }
    } catch {
      localStorage.removeItem(WORKSPACE_STATE_KEY);
    }
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
    localStorage.setItem(WORKSPACE_STATE_KEY, JSON.stringify(snapshot));
  }

  $effect(() => {
    persistWorkspaceState();
  });

  onMount(() => {
    restoreWorkspaceState();
    workspaceStateHydrated = true;
    void loadChannels(null, true);

    const handlePointerDown = (event: PointerEvent) => {
      if (!filterMenuOpen || !filterMenuContainer) return;
      if (!filterMenuContainer.contains(event.target as Node)) {
        filterMenuOpen = false;
      }
    };

    document.addEventListener("pointerdown", handlePointerDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
    };
  });

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

  function reorderChannels(dragId: string, targetId: string) {
    if (dragId === targetId) return;
    const ids = channels.map((channel) => channel.id);
    const fromIndex = ids.indexOf(dragId);
    const toIndex = ids.indexOf(targetId);
    if (fromIndex < 0 || toIndex < 0) return;

    ids.splice(fromIndex, 1);
    ids.splice(toIndex, 0, dragId);
    const byId = new Map(channels.map((channel) => [channel.id, channel]));
    channels = ids
      .map((id) => byId.get(id))
      .filter((channel): channel is Channel => !!channel);
    channelOrder = ids;
  }

  function handleChannelDragStart(channelId: string, event: DragEvent) {
    draggedChannelId = channelId;
    dragOverChannelId = channelId;
    if (!event.dataTransfer) return;
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", channelId);
  }

  function handleChannelDragOver(channelId: string, event: DragEvent) {
    event.preventDefault();
    if (dragOverChannelId !== channelId) {
      dragOverChannelId = channelId;
    }
  }

  function handleChannelDrop(channelId: string, event: DragEvent) {
    event.preventDefault();
    const fallbackId = event.dataTransfer?.getData("text/plain") || null;
    const sourceId = draggedChannelId || fallbackId;
    if (sourceId) {
      reorderChannels(sourceId, channelId);
    }
    draggedChannelId = null;
    dragOverChannelId = null;
  }

  function handleChannelDragEnd() {
    draggedChannelId = null;
    dragOverChannelId = null;
  }

  async function handleAddChannel(input: string) {
    if (!input.trim()) return;

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
    channelInput = "";

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
    } catch (error) {
      // Rollback on error
      channels = previousChannels;
      selectedChannelId = previousSelectedId;
      syncChannelOrderFromList();
      errorMessage = (error as Error).message;
      channelInput = trimmedInput; // Restore input on error
    } finally {
      addingChannel = false;
    }
  }

  function handleChannelSubmit(event: SubmitEvent) {
    event.preventDefault();
    handleAddChannel(channelInput);
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
    const lastRefresh = channelLastRefreshedAt.get(channelId);
    if (
      !bypassTtl &&
      lastRefresh &&
      Date.now() - lastRefresh < CHANNEL_REFRESH_TTL_MS
    ) {
      return;
    }

    // Lazy load/refresh the channel in the background
    refreshingChannel = true;
    try {
      await refreshChannel(channelId);
      channelLastRefreshedAt.set(channelId, Date.now());
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
    if (contentMode === "summary" && video && video.summary_status !== "ready") {
      contentMode = "transcript";
    }
    resetSummaryQuality();
    resetVideoInfo();
    editing = false;
    clearFormattingFeedback();
    await loadContent();
  }

  async function setMode(mode: "transcript" | "summary" | "info") {
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
    const cached = contentCache.get(targetVideoId);
    if (cached) {
      if (targetMode === "transcript" && cached.transcript !== undefined) {
        contentText = cached.transcript;
        draft = contentText;
        resetSummaryQuality();
        resetVideoInfo();
        loadingContent = false;
        activeContentRequestId = 0;
        return;
      }
      if (targetMode === "summary" && cached.summary) {
        contentText = cached.summary.text;
        applySummaryQuality(cached.summary.quality);
        resetVideoInfo();
        draft = contentText;
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
        const originalTranscript = stripPrefix(
          transcript.raw_text ||
            transcript.formatted_markdown ||
            "Transcript unavailable.",
        );
        contentText = stripPrefix(
          transcript.formatted_markdown ||
            transcript.raw_text ||
            "Transcript unavailable.",
        );
        if (!(targetVideoId in originalTranscriptByVideoId)) {
          originalTranscriptByVideoId = {
            ...originalTranscriptByVideoId,
            [targetVideoId]: originalTranscript,
          };
        }
        // Cache the transcript
        const entry = contentCache.get(targetVideoId) ?? {};
        entry.transcript = contentText;
        contentCache.set(targetVideoId, entry);
        resetSummaryQuality();
        resetVideoInfo();
      } else {
        if (targetMode === "summary") {
          const summary = await getSummary(targetVideoId);
          if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
            return;
          contentText = stripPrefix(summary.content || "Summary unavailable.");
          applySummaryQuality(summary);
          // Cache the summary
          const entry = contentCache.get(targetVideoId) ?? {};
          entry.summary = { text: contentText, quality: summary };
          contentCache.set(targetVideoId, entry);
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
  }

  function cancelEdit() {
    editing = false;
    draft = contentText;
  }

  async function saveEdit() {
    if (!selectedVideoId) return;
    if (contentMode === "info") return;
    const targetVideoId = selectedVideoId;

    loadingContent = true;
    errorMessage = null;

    try {
      if (contentMode === "transcript") {
        const transcript = await updateTranscript(targetVideoId, draft);
        contentText = stripPrefix(
          transcript.formatted_markdown ||
            transcript.raw_text ||
            "Transcript unavailable.",
        );
        invalidateContentCache(targetVideoId, "transcript");
        resetSummaryQuality();
        resetVideoInfo();
      } else {
        const summary = await updateSummary(targetVideoId, draft);
        contentText = stripPrefix(summary.content || "Summary unavailable.");
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
        contentText = stripPrefix(summary.content || "Summary unavailable.");
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
          );
          invalidateContentCache(targetVideoId, "transcript");
          if (
            activeFormattingRequest === requestId &&
            selectedVideoId === targetVideoId &&
            !editing
          ) {
            contentText = stripPrefix(
              transcript.formatted_markdown ||
                transcript.raw_text ||
                "Transcript unavailable.",
            );
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
        }
        formattingNotice =
          "Draft reset to original transcript. Save to persist.";
      } else {
        const transcript = await updateTranscript(targetVideoId, original);
        invalidateContentCache(targetVideoId, "transcript");
        if (selectedVideoId === targetVideoId && !editing) {
          contentText = stripPrefix(
            transcript.formatted_markdown ||
              transcript.raw_text ||
              "Transcript unavailable.",
          );
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
    filterMenuOpen = false;
    if (videoTypeFilter === nextValue) return;
    videoTypeFilter = nextValue;
    await loadVideos(true);
  }

  async function setAcknowledgedFilter(nextValue: AcknowledgedFilter) {
    filterMenuOpen = false;
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

  function toggleFilterMenu() {
    filterMenuOpen = !filterMenuOpen;
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      filterMenuOpen = false;
    }
  }

  function updateVideoListBottomState() {
    if (!videoListContainer) {
      atVideoListBottom = false;
      return;
    }
    const thresholdPx = 12;
    atVideoListBottom =
      videoListContainer.scrollTop + videoListContainer.clientHeight >=
      videoListContainer.scrollHeight - thresholdPx;
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

  $effect(() => {
    selectedChannelId;
    videos.length;
    loadingVideos;
    void tick().then(updateVideoListBottomState);
  });
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="page-shell min-h-screen px-3 py-4 max-lg:px-0 lg:px-6">
  <a
    href="#main-content"
    class="skip-link absolute left-4 top-4 z-50 rounded-full bg-[var(--accent)] px-4 py-2 text-sm font-semibold text-white"
  >
    Skip to Main Content
  </a>

  <header
    class="mx-auto flex w-full max-w-[1440px] items-center justify-between gap-3 px-4 sm:px-2 pb-2 mb-0"
  >
    <div class="flex items-center gap-3">
      <a
        href="/"
        class="text-xl sm:text-2xl font-bold tracking-tighter text-[var(--foreground)] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
        aria-label="Go to dAstIll home"
      >
        DASTILL
      </a>
      {#if aiIndicator}
        <AiStatusIndicator
          detail={aiIndicator.detail}
          dotClass={aiIndicator.dotClass}
          title={aiIndicator.title}
        />
      {/if}
    </div>

    <nav class="flex items-center gap-0.5" aria-label="Workspace sections">
      <a
        href="/"
        class="rounded-full px-3.5 py-1.5 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)] bg-[var(--muted)] transition-all"
      >
        Workspace
      </a>
      <a
        href="/download-queue"
        class="rounded-full px-3.5 py-1.5 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50 transition-all hover:opacity-100"
      >
        Queue
      </a>
    </nav>
  </header>

    <main
      id="main-content"
      class="mx-auto mt-0 grid w-full max-w-[1440px] items-start lg:mt-4 lg:gap-0 lg:grid-cols-[260px_300px_minmax(0,1fr)] xl:grid-cols-[280px_340px_minmax(0,1fr)]"
    >
      <aside
        class="flex min-w-0 flex-col border-0 lg:gap-3 lg:pr-5 lg:border-r lg:border-[var(--border-soft)] lg:pl-2 lg:sticky lg:top-4 lg:h-[calc(100vh-4rem)] fade-in stagger-1 {mobileTab !==
        'channels'
          ? 'hidden lg:flex'
          : 'h-[calc(100dvh-10rem)] p-3 gap-4'}"
        id="workspace"
      >
        <div class="flex items-center justify-between gap-2">
          <h2
            class="text-base font-bold tracking-tight text-[var(--soft-foreground)]"
          >
            Channels
          </h2>
          <div class="flex items-center gap-0.5">
            <button
              type="button"
              class="inline-flex h-7 w-7 items-center justify-center rounded-full transition-colors {manageChannels
                ? 'text-red-500'
                : 'text-[var(--soft-foreground)] opacity-40 hover:opacity-80'}"
              data-tooltip={manageChannels
                ? "Exit manage mode"
                : "Manage channels"}
              onclick={() => {
                manageChannels = !manageChannels;
              }}
              aria-label={manageChannels
                ? "Exit manage mode"
                : "Manage channels"}
            >
              <svg
                width="13"
                height="13"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                stroke-linejoin="round"
                ><path d="M3 6h18"></path><path
                  d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"
                ></path><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"
                ></path></svg
              >
            </button>
            <button
              type="button"
              class="inline-flex h-7 w-7 items-center justify-center rounded-full transition-colors {channelSearchOpen
                ? 'text-[var(--accent)]'
                : 'text-[var(--soft-foreground)] opacity-40 hover:opacity-80'}"
              data-tooltip={channelSearchOpen
                ? "Close search"
                : "Search channels"}
              onclick={() => {
                channelSearchOpen = !channelSearchOpen;
                if (!channelSearchOpen) channelSearchQuery = "";
              }}
              aria-label={channelSearchOpen
                ? "Close search"
                : "Search channels"}
            >
              <svg
                width="13"
                height="13"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                stroke-linejoin="round"
                ><circle cx="11" cy="11" r="8"></circle><line
                  x1="21"
                  y1="21"
                  x2="16.65"
                  y2="16.65"
                ></line></svg
              >
            </button>
            <button
              type="button"
              class="inline-flex h-7 w-7 items-center justify-center rounded-full transition-colors {channelSortMode !==
              'custom'
                ? 'text-[var(--accent)]'
                : 'text-[var(--soft-foreground)] opacity-40 hover:opacity-80'}"
              data-tooltip={channelSortMode === "custom"
                ? "Sort: Custom"
                : channelSortMode === "alpha"
                  ? "Sort: A-Z"
                  : "Sort: Newest"}
              onclick={() => {
                channelSortMode =
                  channelSortMode === "custom"
                    ? "alpha"
                    : channelSortMode === "alpha"
                      ? "newest"
                      : "custom";
              }}
              aria-label="Cycle sort mode"
            >
              {#if channelSortMode === "alpha"}
                <svg
                  width="13"
                  height="13"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  ><path d="M3 6h8"></path><path d="M3 12h5"></path><path
                    d="M3 18h3"
                  ></path><path d="M18 6v12"></path><path d="m14 18 4 4 4-4"
                  ></path></svg
                >
              {:else if channelSortMode === "newest"}
                <svg
                  width="13"
                  height="13"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  ><path d="M3 6h3"></path><path d="M3 12h5"></path><path
                    d="M3 18h8"
                  ></path><path d="M18 18V6"></path><path d="m14 6 4-4 4 4"
                  ></path></svg
                >
              {:else}
                <svg
                  width="13"
                  height="13"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  ><path d="M3 6h8"></path><path d="M3 12h5"></path><path
                    d="M3 18h3"
                  ></path><path d="M18 6v12"></path></svg
                >
              {/if}
            </button>
          </div>
        </div>
        {#if channelSearchOpen}
          <div
            class="flex items-center gap-2 border-b border-[var(--border-soft)] px-1 pb-2 transition-all"
          >
            <svg
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="shrink-0 text-[var(--soft-foreground)] opacity-30"
              ><circle cx="11" cy="11" r="8"></circle><line
                x1="21"
                y1="21"
                x2="16.65"
                y2="16.65"
              ></line></svg
            >
            <input
              type="text"
              class="min-w-0 flex-1 bg-transparent text-[13px] placeholder:text-[var(--soft-foreground)] placeholder:opacity-40 focus-visible:outline-none"
              placeholder="Filter..."
              bind:value={channelSearchQuery}
            />
            {#if channelSearchQuery}
              <button
                type="button"
                class="inline-flex h-5 w-5 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-40 hover:opacity-80 transition-opacity"
                onclick={() => {
                  channelSearchQuery = "";
                }}
                aria-label="Clear search"
              >
                <svg
                  width="10"
                  height="10"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="3"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  ><line x1="18" y1="6" x2="6" y2="18"></line><line
                    x1="6"
                    y1="6"
                    x2="18"
                    y2="18"
                  ></line></svg
                >
              </button>
            {/if}
          </div>
        {/if}

        <form
          class="grid"
          onsubmit={handleChannelSubmit}
          aria-label="Follow channel"
        >
          <div
            class="flex min-w-0 items-center gap-2 border-b border-[var(--border-soft)] pb-1 transition-all focus-within:border-[var(--accent)]/40"
          >
            <label for="channel-input" class="sr-only">Add Channel</label>
            <input
              id="channel-input"
              name="channel"
              autocomplete="off"
              spellcheck={false}
              class="min-w-0 flex-1 bg-transparent py-2 text-[13px] font-medium placeholder:text-[var(--soft-foreground)] placeholder:opacity-40 focus-visible:outline-none"
              placeholder="Follow a channel..."
              bind:value={channelInput}
            />
            <button
              type="submit"
              class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-[var(--foreground)] text-white transition-all hover:bg-[var(--accent-strong)] disabled:opacity-15"
              disabled={!channelInput.trim() || addingChannel}
              aria-label="Follow channel"
            >
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="3"
                stroke-linecap="round"
                stroke-linejoin="round"
                ><line x1="12" y1="5" x2="12" y2="19"></line><line
                  x1="5"
                  y1="12"
                  x2="19"
                  y2="12"
                ></line></svg
              >
            </button>
          </div>
        </form>

        <div
          class="flex flex-1 min-h-0 flex-col gap-1.5 overflow-y-auto pr-1 custom-scrollbar"
          aria-busy={loadingChannels}
        >
          {#if loadingChannels}
            <div class="space-y-4" role="status" aria-live="polite">
              {#each Array.from({ length: 4 }) as _, index (index)}
                <div class="flex animate-pulse items-center gap-4 px-3 py-3">
                  <div
                    class="h-10 w-10 shrink-0 rounded-full bg-[var(--muted)] opacity-60"
                  ></div>
                  <div class="min-w-0 flex-1 space-y-2">
                    <div
                      class="h-3 w-3/4 rounded-full bg-[var(--muted)] opacity-60"
                    ></div>
                    <div
                      class="h-2 w-1/2 rounded-full bg-[var(--muted)] opacity-40"
                    ></div>
                  </div>
                </div>
              {/each}
            </div>
          {:else if channels.length === 0}
            <p
              class="px-1 text-[14px] font-medium text-[var(--soft-foreground)] opacity-50 italic"
            >
              Start by following a channel.
            </p>
          {:else if filteredChannels.length === 0}
            <p
              class="px-1 text-[14px] font-medium text-[var(--soft-foreground)] opacity-50 italic"
            >
              No channels match your search.
            </p>
          {:else}
            {#each filteredChannels as channel}
              <ChannelCard
                {channel}
                active={selectedChannelId === channel.id}
                showDelete={manageChannels}
                draggableEnabled={channelSortMode === "custom" &&
                  !channelSearchQuery.trim()}
                loading={channel.id.startsWith("temp-")}
                dragging={draggedChannelId === channel.id}
                dragOver={dragOverChannelId === channel.id &&
                  draggedChannelId !== channel.id}
                onSelect={() => selectChannel(channel.id, null, true)}
                onDragStart={(event) =>
                  handleChannelDragStart(channel.id, event)}
                onDragOver={(event) => handleChannelDragOver(channel.id, event)}
                onDrop={(event) => handleChannelDrop(channel.id, event)}
                onDragEnd={handleChannelDragEnd}
                onDelete={() => handleDeleteChannel(channel.id)}
              />
            {/each}
          {/if}
        </div>
      </aside>

      <aside
        class="flex min-w-0 flex-col border-0 lg:gap-3 lg:px-5 lg:border-r lg:border-[var(--border-soft)] lg:sticky lg:top-4 lg:h-[calc(100vh-4rem)] fade-in stagger-2 {mobileTab !==
        'videos'
          ? 'hidden lg:flex'
          : 'h-[calc(100dvh-4rem)] p-3 gap-4'}"
        id="videos"
      >
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-2 min-w-0">
            <h2
              class="text-base font-bold tracking-tight text-[var(--soft-foreground)]"
            >
              Videos
            </h2>
            {#if refreshingChannel}
              <span
                class="h-3 w-3 animate-spin rounded-full border-[1.5px] border-[var(--border)] border-t-[var(--accent)]"
                role="status"
                aria-label="Syncing"
              ></span>
            {/if}
          </div>
          <div class="relative" bind:this={filterMenuContainer}>
            <button
              type="button"
              class={`group flex h-7 w-7 items-center justify-center rounded-full transition-all duration-200 ${videoTypeFilter !== "all" || acknowledgedFilter !== "all" || filterMenuOpen ? "bg-[var(--accent)] text-white" : "text-[var(--soft-foreground)] opacity-40 hover:opacity-80"} focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 disabled:opacity-20`}
              onclick={toggleFilterMenu}
              disabled={!selectedChannelId || loadingVideos}
              aria-label={filterMenuLabel}
              aria-haspopup="menu"
              aria-expanded={filterMenuOpen}
              aria-controls="video-filter-menu"
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <line x1="3" y1="6" x2="21" y2="6"></line>
                <line x1="7" y1="12" x2="17" y2="12"></line>
                <line x1="10" y1="18" x2="14" y2="18"></line>
              </svg>
            </button>
            {#if filterMenuOpen}
              <div
                id="video-filter-menu"
                role="menu"
                aria-label="Video filters"
                class="absolute right-0 top-full z-20 mt-2 w-56 overflow-hidden rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-white shadow-xl fade-in"
              >
                <div class="p-2 space-y-4">
                  <div class="grid gap-1">
                    <p
                      class="px-2 pb-1 text-[10px] font-bold text-[var(--soft-foreground)] opacity-50"
                    >
                      TYPE
                    </p>
                    <button
                      type="button"
                      role="menuitemradio"
                      aria-checked={videoTypeFilter === "all"}
                      class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === "all" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
                      onclick={() => setVideoTypeFilter("all")}
                    >
                      <span>All Content</span>
                      {#if videoTypeFilter === "all"}
                        <svg
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="3"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><polyline points="20 6 9 17 4 12" /></svg
                        >
                      {/if}
                    </button>
                    <button
                      type="button"
                      role="menuitemradio"
                      aria-checked={videoTypeFilter === "long"}
                      class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === "long" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
                      onclick={() => setVideoTypeFilter("long")}
                    >
                      <span>Full Videos</span>
                      {#if videoTypeFilter === "long"}
                        <svg
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="3"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><polyline points="20 6 9 17 4 12" /></svg
                        >
                      {/if}
                    </button>
                    <button
                      type="button"
                      role="menuitemradio"
                      aria-checked={videoTypeFilter === "short"}
                      class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === "short" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
                      onclick={() => setVideoTypeFilter("short")}
                    >
                      <span>Shorts</span>
                      {#if videoTypeFilter === "short"}
                        <svg
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="3"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><polyline points="20 6 9 17 4 12" /></svg
                        >
                      {/if}
                    </button>
                  </div>

                  <div class="grid gap-1">
                    <p
                      class="px-2 pb-1 text-[10px] font-bold text-[var(--soft-foreground)] opacity-50"
                    >
                      STATUS
                    </p>
                    <button
                      type="button"
                      role="menuitemradio"
                      aria-checked={acknowledgedFilter === "all"}
                      class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === "all" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
                      onclick={() => setAcknowledgedFilter("all")}
                    >
                      <span>All Statuses</span>
                      {#if acknowledgedFilter === "all"}
                        <svg
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="3"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><polyline points="20 6 9 17 4 12" /></svg
                        >
                      {/if}
                    </button>
                    <button
                      type="button"
                      role="menuitemradio"
                      aria-checked={acknowledgedFilter === "unack"}
                      class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === "unack" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
                      onclick={() => setAcknowledgedFilter("unack")}
                    >
                      <span>Unread</span>
                      {#if acknowledgedFilter === "unack"}
                        <svg
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="3"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><polyline points="20 6 9 17 4 12" /></svg
                        >
                      {/if}
                    </button>
                    <button
                      type="button"
                      role="menuitemradio"
                      aria-checked={acknowledgedFilter === "ack"}
                      class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === "ack" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
                      onclick={() => setAcknowledgedFilter("ack")}
                    >
                      <span>Read</span>
                      {#if acknowledgedFilter === "ack"}
                        <svg
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="3"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><polyline points="20 6 9 17 4 12" /></svg
                        >
                      {/if}
                    </button>
                  </div>
                </div>
              </div>
            {/if}
          </div>
        </div>

        <div
          class="grid flex-1 min-h-0 gap-4 overflow-y-auto pr-1 custom-scrollbar"
          bind:this={videoListContainer}
          onscroll={updateVideoListBottomState}
          aria-busy={loadingVideos}
        >
          {#if loadingVideos && videos.length === 0}
            {#each Array.from({ length: 3 }) as _, index (index)}
              <article
                class="flex min-h-[14rem] flex-col gap-4 rounded-[var(--radius-md)] p-4 animate-pulse bg-[var(--muted)]/30"
              >
                <div
                  class="aspect-video rounded-[var(--radius-sm)] bg-[var(--muted)] opacity-60"
                ></div>
                <div
                  class="h-4 w-11/12 rounded-full bg-[var(--muted)] opacity-60"
                ></div>
                <div
                  class="h-3 w-2/5 rounded-full bg-[var(--muted)] opacity-40"
                ></div>
              </article>
            {/each}
          {:else if videos.length === 0}
            <p
              class="px-1 text-[14px] font-medium text-[var(--soft-foreground)] opacity-50 italic"
            >
              Waiting for the library to fill.
            </p>
          {:else}
            {#each videos as video}
              <VideoCard
                {video}
                active={selectedVideoId === video.id}
                onSelect={() => selectVideo(video.id, true)}
              />
            {/each}
          {/if}
        </div>

        {#if selectedChannelId}
          <div class="flex flex-col gap-3 pt-3">
            {#if hasMore || !historyExhausted}
              <button
                type="button"
                class="w-full rounded-[var(--radius-sm)] border border-[var(--border-soft)] py-2.5 text-[11px] font-bold uppercase tracking-[0.15em] text-[var(--soft-foreground)] transition-all hover:border-[var(--accent)]/40 hover:text-[var(--foreground)] disabled:opacity-30"
                onclick={loadMoreVideos}
                disabled={loadingVideos || backfillingHistory}
              >
                {#if loadingVideos || backfillingHistory}
                  Loading...
                {:else if hasMore}
                  More
                {:else}
                  Explore History
                {/if}
              </button>
            {/if}

            {#if atVideoListBottom && videos.length > 0}
              <p
                class="text-[11px] text-[var(--soft-foreground)] opacity-40 px-0.5"
              >
                Synced to {formatSyncDate(
                  resolveDisplayedSyncDepthIso({
                    videos,
                    selectedChannel,
                    syncDepth,
                    allowLoadedVideoOverride: allowLoadedVideoSyncDepthOverride,
                  }),
                )}
              </p>
            {/if}
          </div>
        {/if}
      </aside>

      <section
        class="flex min-w-0 flex-col overflow-hidden border-0 lg:gap-4 lg:py-6 lg:pl-6 lg:sticky lg:top-4 lg:h-[calc(100vh-4rem)] fade-in stagger-3 {mobileTab !==
        'content'
          ? 'hidden lg:flex'
          : 'h-[calc(100dvh-4rem)]'}"
        id="content-view"
      >
        <div
          class="flex flex-wrap items-center justify-between gap-3 px-4 sm:px-6 lg:px-0 max-lg:pt-3 max-lg:pb-1"
        >
          <div class="flex items-center gap-3 sm:gap-4">
            <h2 class="sr-only">Display Content</h2>
            <Toggle
              options={["transcript", "summary", "info"]}
              value={contentMode}
              onChange={(value) =>
                setMode(value as "transcript" | "summary" | "info")}
            />
          </div>

          {#if selectedVideoId && !loadingContent && !editing && contentMode !== "info"}
            <div class="flex items-center justify-end h-10">
              <ContentEditor
                editing={false}
                busy={loadingContent}
                aiAvailable={aiAvailable ?? false}
                formatting={formattingContent &&
                  formattingVideoId === selectedVideoId}
                regenerating={regeneratingSummary &&
                  regeneratingVideoId === selectedVideoId}
                reverting={revertingContent &&
                  revertingVideoId === selectedVideoId}
                showFormatAction={contentMode === "transcript"}
                showRegenerateAction={contentMode === "summary"}
                showRevertAction={contentMode === "transcript"}
                canRevert={canRevertTranscript}
                youtubeUrl={contentMode === "transcript"
                  ? selectedVideoYoutubeUrl
                  : null}
                value={draft}
                acknowledged={videos.find((v) => v.id === selectedVideoId)
                  ?.acknowledged ?? false}
                onEdit={startEdit}
                onCancel={cancelEdit}
                onSave={saveEdit}
                onFormat={cleanFormatting}
                onRegenerate={regenerateSummaryContent}
                onRevert={revertToOriginalTranscript}
                onChange={(value) => (draft = value)}
                onAcknowledgeToggle={toggleAcknowledge}
              />
            </div>
          {/if}
        </div>

        <div
          class="w-full min-h-0 flex-1 overflow-y-auto px-4 sm:px-6 lg:px-0 lg:pr-4 max-lg:pt-4 custom-scrollbar"
        >
          {#if selectedVideoId && !loadingContent}
            {@const selectedVideo = videos.find(
              (v) => v.id === selectedVideoId,
            )}
            {#if selectedVideo}
              <nav class="mb-3 sm:mb-4 flex flex-wrap items-center gap-x-1.5 gap-y-0.5 text-[12px] text-[var(--soft-foreground)] opacity-60" aria-label="Breadcrumb">
                {#if selectedChannel}
                  <button type="button" class="shrink-0 hover:text-[var(--foreground)] transition-colors" onclick={() => { mobileTab = 'channels'; }}>{selectedChannel.name}</button>
                  <svg class="shrink-0" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"/></svg>
                {/if}
                <button type="button" class="font-medium text-[var(--foreground)] opacity-80 hover:opacity-100 transition-opacity text-left" onclick={() => { mobileTab = 'videos'; }}>{selectedVideo.title}</button>
              </nav>
            {/if}
          {/if}
          {#if contentMode === "transcript" && selectedVideoId && ((formattingContent && formattingVideoId === selectedVideoId) || (formattingNotice && formattingNoticeVideoId === selectedVideoId))}
            <div
              class={`mb-4 sm:mb-8 p-4 rounded-[var(--radius-md)] border flex flex-wrap items-center gap-3 transition-all duration-500 ${
                formattingNoticeTone === "warning"
                  ? "border-[var(--accent)]/20 bg-[var(--accent-soft)]/50 text-[var(--accent-strong)]"
                  : "border-[var(--border-soft)] bg-[var(--muted)]/30 text-[var(--soft-foreground)]"
              }`}
              role="status"
              aria-live="polite"
            >
              {#if formattingContent && formattingVideoId === selectedVideoId}
                <span class="relative flex h-2 w-2">
                  <span
                    class="animate-ping absolute inline-flex h-full w-full rounded-full bg-current opacity-75"
                  ></span>
                  <span
                    class="relative inline-flex rounded-full h-2 w-2 bg-current"
                  ></span>
                </span>
              {:else}
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="3"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  ><circle cx="12" cy="12" r="10" /><polyline
                    points="12 6 12 12 16 14"
                  /></svg
                >
              {/if}
              <p class="text-[12px] font-bold tracking-wide uppercase">
                {formattingContent && formattingVideoId === selectedVideoId
                  ? formattingNotice || "Refining transcript with Ollama…"
                  : formattingNotice}
              </p>
            </div>
          {/if}
          {#if contentMode === "summary" && selectedVideoId && !loadingContent}
            <div
              class="mb-2 flex flex-wrap items-center gap-2 text-[11px] text-[var(--soft-foreground)] opacity-40"
              role="status"
              aria-live="polite"
            >
              <svg
                width="11"
                height="11"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="3"
                stroke-linecap="round"
                stroke-linejoin="round"
                ><polygon
                  points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"
                /></svg
              >
              <span class="font-bold uppercase tracking-[0.08em]">
                {#if summaryQualityScore !== null}
                  Quality Analysis: {summaryQualityScore}/10
                {:else}
                  Evaluating quality…
                {/if}
              </span>
              {#if summaryQualityNote}
                <span class="italic">"{summaryQualityNote}"</span>
              {/if}
              <span class="hidden sm:inline"
                >Distilled by {summaryModelUsed ?? "unknown model"}</span
              >
            </div>
          {/if}

          {#if !selectedVideoId}
            <div
              class="flex flex-col items-center justify-center h-full text-center py-20"
            >
              <p class="text-[15px] text-[var(--soft-foreground)] opacity-30">
                Select a video to view its content.
              </p>
            </div>
          {:else if loadingContent}
            <div
              class="space-y-8 animate-pulse mt-4"
              role="status"
              aria-live="polite"
            >
              <div
                class="h-10 w-3/5 rounded-[var(--radius-sm)] bg-[var(--muted)]/60"
              ></div>
              <div class="space-y-4 pt-4">
                <div class="h-4 w-full rounded-full bg-[var(--muted)]/50"></div>
                <div
                  class="h-4 w-11/12 rounded-full bg-[var(--muted)]/50"
                ></div>
                <div
                  class="h-4 w-10/12 rounded-full bg-[var(--muted)]/50"
                ></div>
                <div class="h-4 w-full rounded-full bg-[var(--muted)]/50"></div>
                <div class="h-4 w-3/4 rounded-full bg-[var(--muted)]/50"></div>
              </div>
              <p
                class="pt-10 text-[10px] font-bold uppercase tracking-[0.4em] text-[var(--accent)] text-center"
              >
                Processing {contentMode}…
              </p>
            </div>
          {:else if contentMode === "info"}
            <div class="space-y-8 text-[15px] leading-relaxed pb-20">
              <h3
                class="text-[20px] font-bold font-serif leading-tight text-[var(--foreground)]"
              >
                {videoInfo?.title || "Untitled"}
              </h3>

              <div class="grid gap-x-6 gap-y-4 grid-cols-2 lg:grid-cols-4">
                <div>
                  <p
                    class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50 mb-1"
                  >
                    Published
                  </p>
                  <p class="font-semibold text-[13px]">
                    {formatPublishedAt(videoInfo?.published_at)}
                  </p>
                </div>
                <div>
                  <p
                    class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50 mb-1"
                  >
                    Views
                  </p>
                  <p class="font-semibold text-[13px]">
                    {formatCount(videoInfo?.view_count)}
                  </p>
                </div>
                <div>
                  <p
                    class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50 mb-1"
                  >
                    Duration
                  </p>
                  <p class="font-semibold text-[13px]">
                    {formatDuration(
                      videoInfo?.duration_seconds,
                      videoInfo?.duration_iso8601,
                    )}
                  </p>
                </div>
                <div>
                  <p
                    class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50 mb-1"
                  >
                    Channel
                  </p>
                  <p class="font-semibold text-[13px] truncate">
                    {videoInfo?.channel_name || "Unknown"}
                  </p>
                </div>
              </div>

              {#if videoInfo?.watch_url}
                <a
                  href={videoInfo.watch_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="inline-flex items-center gap-2 group text-[13px] font-semibold text-[var(--accent)] hover:text-[var(--accent-strong)]"
                >
                  <span>Open on YouTube</span>
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5"
                    ><line x1="7" y1="17" x2="17" y2="7" /><polyline
                      points="7 7 17 7 17 17"
                    /></svg
                  >
                </a>
              {/if}

              {#if videoInfo?.description}
                <div>
                  <p
                    class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50 mb-3"
                  >
                    Description
                  </p>
                  <p
                    class="whitespace-pre-wrap text-[14px] font-medium leading-relaxed text-[var(--foreground)] opacity-70"
                  >
                    {videoInfo.description}
                  </p>
                </div>
              {/if}
            </div>
          {:else if editing}
            <div class="pb-20">
              <ContentEditor
                editing
                busy={loadingContent}
                aiAvailable={aiAvailable ?? false}
                formatting={formattingContent &&
                  formattingVideoId === selectedVideoId}
                regenerating={regeneratingSummary &&
                  regeneratingVideoId === selectedVideoId}
                reverting={revertingContent &&
                  revertingVideoId === selectedVideoId}
                showFormatAction={contentMode === "transcript"}
                showRegenerateAction={contentMode === "summary"}
                showRevertAction={contentMode === "transcript"}
                canRevert={canRevertTranscript}
                youtubeUrl={contentMode === "transcript"
                  ? selectedVideoYoutubeUrl
                  : null}
                value={draft}
                acknowledged={videos.find((v) => v.id === selectedVideoId)
                  ?.acknowledged ?? false}
                onEdit={startEdit}
                onCancel={cancelEdit}
                onSave={saveEdit}
                onFormat={cleanFormatting}
                onRegenerate={regenerateSummaryContent}
                onRevert={revertToOriginalTranscript}
                onChange={(value) => (draft = value)}
                onAcknowledgeToggle={toggleAcknowledge}
              />
            </div>
          {:else}
            <div class="max-lg:pb-32">
              <TranscriptView
                html={contentHtml}
                formatting={contentMode === "transcript" &&
                  formattingContent &&
                  formattingVideoId === selectedVideoId}
              />
            </div>
          {/if}
        </div>
      </section>
    </main>

    <nav class="mobile-tab-bar lg:hidden" aria-label="Panel navigation">
      <button
        type="button"
        class="mobile-tab-item {mobileTab === 'channels'
          ? 'mobile-tab-item--active'
          : ''}"
        onclick={() => (mobileTab = "channels")}
        aria-current={mobileTab === "channels" ? "page" : undefined}
      >
        <svg
          class="h-5 w-5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="3" y="3" width="6" height="18" rx="1.5" />
          <rect x="15" y="3" width="6" height="18" rx="1.5" />
          <rect x="9" y="3" width="6" height="18" rx="1.5" />
        </svg>
        <span>Channels</span>
      </button>
      <button
        type="button"
        class="mobile-tab-item {mobileTab === 'videos'
          ? 'mobile-tab-item--active'
          : ''}"
        onclick={() => (mobileTab = "videos")}
        aria-current={mobileTab === "videos" ? "page" : undefined}
      >
        <svg
          class="h-5 w-5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polygon points="6,3 20,12 6,21" />
        </svg>
        <span>Videos</span>
      </button>
      <button
        type="button"
        class="mobile-tab-item {mobileTab === 'content'
          ? 'mobile-tab-item--active'
          : ''}"
        onclick={() => (mobileTab = "content")}
        aria-current={mobileTab === "content" ? "page" : undefined}
      >
        <svg
          class="h-5 w-5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
          <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
        </svg>
        <span>Content</span>
      </button>
    </nav>

  {#if errorMessage}
    <div
      class="fixed bottom-6 max-lg:bottom-[calc(3.5rem+env(safe-area-inset-bottom)+1rem)] left-1/2 z-50 w-[min(90vw,420px)] -translate-x-1/2 rounded-[var(--radius-md)] bg-white border border-rose-200 px-4 py-3 shadow-lg fade-in"
      role="status"
      aria-live="polite"
    >
      <div class="flex items-start gap-3">
        <p class="text-[13px] font-medium text-rose-600 flex-1">
          {errorMessage}
        </p>
        <button
          onclick={() => (errorMessage = null)}
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
            ><line x1="18" y1="6" x2="6" y2="18"></line><line
              x1="6"
              y1="6"
              x2="18"
              y2="18"
            ></line></svg
          >
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
</div>
