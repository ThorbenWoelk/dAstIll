<script lang="ts">
  import { goto, replaceState as replacePageState } from "$app/navigation";
  import { onMount } from "svelte";
  import {
    getChannelSnapshot,
    getChannelSyncDepth,
    isAiAvailable,
    listChannelsWhenAvailable,
    listVideos,
    refreshChannel,
    updateChannel,
  } from "$lib/api";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import AiStatusIndicator from "$lib/components/AiStatusIndicator.svelte";
  import FeatureGuide, {
    type TourStep,
  } from "$lib/components/FeatureGuide.svelte";
  import {
    applySavedChannelOrder,
    beginChannelDrag,
    buildQueueSnapshotOptions,
    completeChannelDrop,
    finishChannelDrag,
    loadWorkspaceState,
    markChannelRefreshed,
    reorderChannels as reorderChannelList,
    restoreWorkspaceSnapshot,
    resolveInitialChannelSelection,
    saveWorkspaceState,
    shouldRefreshChannel,
    updateChannelDragOver,
    type WorkspaceStateSnapshot,
  } from "$lib/channel-workspace";
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
  import ChannelCard from "$lib/components/ChannelCard.svelte";
  import { DOCS_URL } from "$lib/app-config";
  import Footer from "$lib/components/Footer.svelte";
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import SectionNavigation from "$lib/components/SectionNavigation.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import type {
    AiStatus,
    Channel,
    ChannelSnapshot,
    ContentStatus,
    QueueTab,
    Video,
  } from "$lib/types";
  import {
    buildQueueViewHref,
    buildWorkspaceViewHref,
    mergeQueueViewState,
    parseQueueViewUrlState,
  } from "$lib/view-url";

  const dateFormatter = new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
  const syncTimeFormatter = new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
    second: "2-digit",
  });
  let channels = $state<Channel[]>([]);
  let channelOrder = $state<string[]>([]);
  let videos = $state<Video[]>([]);
  let selectedChannelId = $state<string | null>(null);
  let draggedChannelId = $state<string | null>(null);
  let dragOverChannelId = $state<string | null>(null);
  let loadingChannels = $state(false);
  let aiStatus = $state<AiStatus | null>(null);

  // -- Guide tour (URL-driven: ?guide=0, ?guide=1, ...) --
  let guideOpen = $state(false);
  let guideStep = $state(0);

  const tourSteps: TourStep[] = [
    {
      selector: "#ai-status-pill",
      title: "AI Status",
      body: "The colored dot shows AI engine availability. In showcase mode, AI is disabled but all browsing features work fully.",
      placement: "bottom",
    },
    {
      selector: "nav[aria-label='Queue tabs']",
      title: "Queue Tabs",
      body: "Switch between Transcripts, Summaries, and Evaluations to monitor the processing pipeline for each channel.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "details";
      },
    },
    {
      selector: ".footer-link",
      title: "Open Source",
      body: "dAstIll is fully open source. Check out the GitHub repository, star the project, or contribute to improve it!",
      placement: "top",
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
  let loadingVideos = $state(false);

  let errorMessage = $state<string | null>(null);
  let showDeleteConfirmation = $state(false);
  let channelIdToDelete = $state<string | null>(null);
  let workspaceStateHydrated = $state(false);
  let viewUrlHydrated = $state(false);

  let offset = $state(0);
  const limit = 20;
  let hasMore = $state(true);
  let lastSyncedAt = $state<Date | null>(null);
  let queueDeltaSinceLastSync = $state<number | null>(null);
  let previousQueuedTotal = $state<number | null>(null);
  let earliestSyncDateInput = $state("");
  let savingSyncDate = $state(false);
  let syncDepth = $state<{
    earliest_sync_date: string | null;
    earliest_sync_date_user_set: boolean;
    derived_earliest_ready_date: string | null;
  } | null>(null);
  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );

  const MAX_RETRIES = 3;
  const CHANNEL_REFRESH_TTL_MS = 5 * 60 * 1000;
  const channelLastRefreshedAt = new Map<string, number>();

  $effect(() => {
    const timer = setInterval(() => {
      void isAiAvailable()
        .then((status) => {
          aiStatus = status.status;
        })
        .catch(() => {
          aiStatus = "offline";
        });
    }, 30000);
    return () => clearInterval(timer);
  });

  let mobileTab = $state<"channels" | "details">("details");
  let manageChannels = $state(false);
  let queueTab = $state<QueueTab>("transcripts");

  function reorderChannels(dragId: string, targetId: string) {
    const reordered = reorderChannelList(channels, dragId, targetId);
    if (!reordered) return;
    channels = reordered.channels;
    channelOrder = reordered.channelOrder;
  }

  function handleChannelDragStart(channelId: string, event: DragEvent) {
    const dragState = beginChannelDrag(channelId, event.dataTransfer);
    draggedChannelId = dragState.draggedChannelId;
    dragOverChannelId = dragState.dragOverChannelId;
  }

  function handleChannelDragOver(channelId: string, event: DragEvent) {
    event.preventDefault();
    dragOverChannelId = updateChannelDragOver(dragOverChannelId, channelId);
  }

  function handleChannelDrop(channelId: string, event: DragEvent) {
    event.preventDefault();
    const { sourceId, dragState } = completeChannelDrop(
      channelId,
      draggedChannelId,
      event.dataTransfer?.getData("text/plain") || null,
    );
    if (sourceId) {
      reorderChannels(sourceId, channelId);
    }
    draggedChannelId = dragState.draggedChannelId;
    dragOverChannelId = dragState.dragOverChannelId;
  }

  function handleChannelDragEnd() {
    const dragState = finishChannelDrag();
    draggedChannelId = dragState.draggedChannelId;
    dragOverChannelId = dragState.dragOverChannelId;
  }

  const selectedChannel = $derived(
    channels.find((channel) => channel.id === selectedChannelId) ?? null,
  );

  const effectiveEarliestSyncDate = $derived(
    selectedChannel?.earliest_sync_date_user_set
      ? selectedChannel.earliest_sync_date
      : (syncDepth?.derived_earliest_ready_date ??
          selectedChannel?.earliest_sync_date),
  );

  const queuedVideos = $derived(videos);

  function getQueueState(video: Video): Exclude<ContentStatus, "ready"> {
    if (
      video.transcript_status === "failed" ||
      video.summary_status === "failed"
    ) {
      return "failed";
    }

    if (
      video.transcript_status === "loading" ||
      video.summary_status === "loading"
    ) {
      return "loading";
    }

    return "pending";
  }

  type DistillationStatusKind = "processing" | "queued" | "failed";

  interface DistillationStatusCopy {
    kind: DistillationStatusKind;
    label: string;
    detail: string;
  }

  function getDistillationStatusCopy(video: Video): DistillationStatusCopy {
    const retries = video.retry_count ?? 0;
    const permanentlyFailed = retries >= MAX_RETRIES;

    if (video.transcript_status !== "ready") {
      if (video.transcript_status === "loading") {
        return {
          kind: "processing",
          label: "PROCESSING TRANSCRIPT",
          detail: "Transcript extraction is running now.",
        };
      }

      if (video.transcript_status === "failed") {
        return {
          kind: "failed",
          label: permanentlyFailed
            ? "TRANSCRIPT FAILED (PERMANENT)"
            : "TRANSCRIPT FAILED (RETRYING)",
          detail: permanentlyFailed
            ? "Automatic retries are exhausted."
            : "Transcript extraction failed. Automatic retry is queued.",
        };
      }

      return {
        kind: "queued",
        label: "QUEUED FOR TRANSCRIPT",
        detail: "Waiting in queue to start transcript extraction.",
      };
    }

    if (video.summary_status !== "ready") {
      if (video.summary_status === "loading") {
        return {
          kind: "processing",
          label: "PROCESSING SUMMARY",
          detail: "Summary generation is running now.",
        };
      }

      if (video.summary_status === "failed") {
        return {
          kind: "failed",
          label: permanentlyFailed
            ? "SUMMARY FAILED (PERMANENT)"
            : "SUMMARY FAILED (RETRYING)",
          detail: permanentlyFailed
            ? "Automatic retries are exhausted."
            : "Summary generation failed. Automatic retry is queued.",
        };
      }

      return {
        kind: "queued",
        label: "QUEUED FOR SUMMARY",
        detail: "Transcript is ready. Waiting in queue for summary generation.",
      };
    }

    // Both ready but no quality score - pending evaluation
    return {
      kind: "queued",
      label: "PENDING EVALUATION",
      detail: "Summary is ready. Waiting for quality evaluation.",
    };
  }

  const queueStats = $derived({
    total: queuedVideos.length,
    loading: queuedVideos.filter((video) => {
      if (queueTab === "transcripts")
        return video.transcript_status === "loading";
      if (queueTab === "summaries") return video.summary_status === "loading";
      return false;
    }).length,
    pending: queuedVideos.filter((video) => {
      if (queueTab === "transcripts")
        return video.transcript_status === "pending";
      if (queueTab === "summaries") return video.summary_status === "pending";
      return true; // Evaluations are all pending
    }).length,
    failed: queuedVideos.filter((video) => {
      if (queueTab === "transcripts")
        return video.transcript_status === "failed";
      if (queueTab === "summaries") return video.summary_status === "failed";
      return false;
    }).length,
  });
  const queuedVideosWithDistillationStatus = $derived(
    queuedVideos.map((video) => ({
      video,
      distillationStatus: getDistillationStatusCopy(video),
    })),
  );

  function formatDate(value: string) {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return "Date unavailable";
    return dateFormatter.format(date);
  }

  function formatSyncDate(value: string | null | undefined) {
    if (!value) return "Not set";
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return "Not set";
    return dateFormatter.format(date);
  }

  function setSyncSnapshot(snapshot: Video[]) {
    const queuedCount = snapshot.length;

    if (previousQueuedTotal === null) {
      queueDeltaSinceLastSync = null;
    } else {
      queueDeltaSinceLastSync = queuedCount - previousQueuedTotal;
    }

    previousQueuedTotal = queuedCount;
    lastSyncedAt = new Date();
  }

  async function applyChannelSnapshot(
    channelId: string,
    snapshot: ChannelSnapshot,
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

      syncDepth = snapshot.sync_depth;
      videos = snapshot.videos;
      offset = snapshot.videos.length;
      hasMore = snapshot.videos.length === limit;
      setSyncSnapshot(snapshot.videos);
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

  function restoreWorkspaceState() {
    const restored = mergeQueueViewState(
      restoreWorkspaceSnapshot(
        typeof localStorage === "undefined"
          ? null
          : loadWorkspaceState(localStorage),
      ),
      typeof window === "undefined"
        ? {}
        : parseQueueViewUrlState(new URL(window.location.href)),
    );

    if ("selectedChannelId" in restored) {
      selectedChannelId = restored.selectedChannelId ?? null;
    }
    if (restored.channelOrder) {
      channelOrder = restored.channelOrder;
    }
    if (restored.queueTab) {
      queueTab = restored.queueTab;
    }
  }

  function persistWorkspaceState() {
    if (!workspaceStateHydrated || typeof localStorage === "undefined") return;
    saveWorkspaceState(localStorage, {
      selectedChannelId,
      channelOrder,
    });
  }

  async function openVideoTranscriptInWorkspace(video: Video) {
    if (typeof localStorage !== "undefined") {
      saveWorkspaceState(localStorage, {
        selectedChannelId: video.channel_id,
        selectedVideoId: video.id,
        contentMode: "transcript",
        videoTypeFilter: "all",
        acknowledgedFilter: "all",
      });
    }

    await goto(
      buildWorkspaceViewHref({
        selectedChannelId: video.channel_id,
        selectedVideoId: video.id,
        contentMode: "transcript",
        videoTypeFilter: "all",
        acknowledgedFilter: "all",
      }),
    );
  }

  function persistViewUrl() {
    if (!viewUrlHydrated || typeof window === "undefined") return;
    const nextHref = buildQueueViewHref({
      selectedChannelId,
      queueTab,
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

  $effect(() => {
    persistWorkspaceState();
  });

  $effect(() => {
    persistViewUrl();
  });

  $effect(() => {
    if (!selectedChannel) {
      earliestSyncDateInput = "";
      return;
    }
    const effective = selectedChannel.earliest_sync_date_user_set
      ? selectedChannel.earliest_sync_date
      : (syncDepth?.derived_earliest_ready_date ??
        selectedChannel.earliest_sync_date);
    if (effective) {
      earliestSyncDateInput = new Date(effective).toISOString().split("T")[0];
    } else {
      earliestSyncDateInput = "";
    }
  });

  $effect(() => {
    if (!selectedChannelId) {
      syncDepth = null;
    }
  });

  // Reload videos when queue tab changes
  let previousQueueTab = $state<QueueTab>("transcripts");
  $effect(() => {
    const currentTab = queueTab;
    if (currentTab !== previousQueueTab) {
      previousQueueTab = currentTab;
      if (selectedChannelId) {
        videos = [];
        offset = 0;
        hasMore = true;
        previousQueuedTotal = null;
        queueDeltaSinceLastSync = null;
        void refreshAndLoadVideos(selectedChannelId);
      }
    }
  });

  onMount(() => {
    restoreWorkspaceState();
    previousQueueTab = queueTab;
    workspaceStateHydrated = true;
    void (async () => {
      try {
        await loadInitial();
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

  async function loadInitial() {
    loadingChannels = true;
    errorMessage = null;

    try {
      // Phase 1: Get channels as fast as possible
      const channelList = await listChannelsWhenAvailable({
        retryDelayMs: 500,
      });
      channels = applySavedChannelOrder(channelList, channelOrder);
      channelOrder = channels.map((c) => c.id);
      loadingChannels = false;

      // Resolve initial channel selection
      const initialChannelId = resolveInitialChannelSelection(
        channels,
        selectedChannelId,
        null,
      );

      if (!initialChannelId) {
        selectedChannelId = null;
        videos = [];
        syncDepth = null;
      } else {
        selectedChannelId = initialChannelId;
        // Phase 2: Load snapshot for the selected channel (non-blocking)
        void refreshAndLoadVideos(initialChannelId);
      }

      // Phase 3: Check AI status in background
      void isAiAvailable()
        .then((status) => {
          aiStatus = status.status;
        })
        .catch(() => {
          aiStatus = "offline";
        });
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      loadingChannels = false;
    }
  }

  async function selectChannel(channelId: string) {
    selectedChannelId = channelId;
    mobileTab = "details";
    videos = [];
    offset = 0;
    hasMore = true;
    lastSyncedAt = null;
    queueDeltaSinceLastSync = null;
    previousQueuedTotal = null;
    await refreshAndLoadVideos(channelId);
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
      const { deleteChannel } = await import("$lib/api");
      await deleteChannel(channelId);
      channels = channels.filter((c) => c.id !== channelId);
      channelOrder = channelOrder.filter((id) => id !== channelId);
      if (selectedChannelId === channelId) {
        const nextChannel = channels.length > 0 ? channels[0] : null;
        if (nextChannel) {
          await selectChannel(nextChannel.id);
        } else {
          selectedChannelId = null;
          videos = [];
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

  let refreshingChannel = $state(false);

  async function refreshAndLoadVideos(channelId: string) {
    const snapshotOptions = buildQueueSnapshotOptions(queueTab, limit);
    const snapshot = await getChannelSnapshot(channelId, snapshotOptions);
    await applyChannelSnapshot(channelId, snapshot);

    // Skip YouTube refresh if channel was refreshed recently
    if (
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
      // After refresh, silently reload the queue snapshot.
      if (selectedChannelId === channelId) {
        const refreshedSnapshot = await getChannelSnapshot(
          channelId,
          snapshotOptions,
        );
        await applyChannelSnapshot(channelId, refreshedSnapshot, true);
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
      const list = await listVideos(
        selectedChannelId,
        limit,
        reset ? 0 : offset,
        "all",
        undefined,
        false,
        queueTab,
      );
      videos = reset ? list : [...videos, ...list];
      offset = (reset ? 0 : offset) + list.length;
      hasMore = list.length === limit;
      if (reset) {
        setSyncSnapshot(list);
      }
    } catch (error) {
      if (!silent || !errorMessage) errorMessage = (error as Error).message;
    } finally {
      if (!silent) loadingVideos = false;
    }
  }

  async function saveEarliestSyncDate() {
    if (!selectedChannelId || !earliestSyncDateInput || savingSyncDate) return;
    errorMessage = null;
    savingSyncDate = true;
    try {
      const updated = await updateChannel(selectedChannelId, {
        earliest_sync_date: new Date(earliestSyncDateInput).toISOString(),
        earliest_sync_date_user_set: true,
      });
      channels = channels.map((channel) =>
        channel.id === selectedChannelId ? updated : channel,
      );
      syncDepth = await getChannelSyncDepth(selectedChannelId);
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      savingSyncDate = false;
    }
  }
</script>

<div
  class="page-shell page-shell--panel-mobile-shell page-shell--with-mobile-nav min-h-screen px-4 pb-12 pt-8 sm:px-8 max-lg:px-0"
>
  <a
    href="#main-content"
    class="skip-link absolute left-4 top-4 z-50 rounded-full bg-[var(--accent)] px-4 py-2 text-sm font-semibold text-white shadow-lg shadow-[var(--accent)]/20"
  >
    Skip to Main Content
  </a>

  <header
    class="mx-auto flex w-full max-w-[1440px] min-w-0 flex-wrap items-start gap-3 px-4 pb-2 sm:px-2 lg:items-center"
  >
    <div class="flex min-w-0 flex-1 items-center gap-3">
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
      <button
        type="button"
        id="guide-trigger"
        class="inline-flex h-7 w-7 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-40 transition-all hover:opacity-80 hover:bg-[var(--muted)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
        onclick={openGuide}
        aria-label="Feature guide"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="10"></circle>
          <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
          <line x1="12" y1="17" x2="12.01" y2="17"></line>
        </svg>
      </button>
    </div>

    <div class="ml-auto flex shrink-0 items-center gap-2">
      <ThemeToggle />
      <SectionNavigation
        currentSection="queue"
        docsUrl={DOCS_URL}
        mobileMode="inline"
      />
    </div>
  </header>

  <main
    id="main-content"
    class="panel-shell-main mx-auto mt-4 grid w-full max-w-[1440px] gap-0 lg:grid-cols-[260px_minmax(0,1fr)] xl:grid-cols-[280px_minmax(0,1fr)] items-start max-lg:mt-0"
  >
    <aside
      class="flex min-h-0 flex-col gap-4 border-0 lg:h-fit lg:pr-5 lg:border-r lg:border-[var(--border-soft)] lg:pl-2 lg:sticky lg:top-4 fade-in stagger-1 {mobileTab !==
      'channels'
        ? 'hidden lg:flex'
        : 'h-full p-4 gap-4'}"
    >
      <div class="flex items-center justify-between gap-2">
        <h2
          class="text-base font-bold tracking-tight text-[var(--soft-foreground)]"
        >
          Channels
        </h2>
        <button
          type="button"
          class="inline-flex h-7 w-7 items-center justify-center rounded-full transition-colors {manageChannels
            ? 'text-[var(--danger)]'
            : 'text-[var(--soft-foreground)] opacity-40 hover:opacity-80'}"
          data-tooltip={manageChannels ? "Exit manage mode" : "Manage channels"}
          onclick={() => {
            manageChannels = !manageChannels;
          }}
          aria-label={manageChannels ? "Exit manage mode" : "Manage channels"}
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
            ></path><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path></svg
          >
        </button>
      </div>

      <div
        class="custom-scrollbar mobile-bottom-stack-padding flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto pr-1 lg:max-h-[60vh] lg:pb-0"
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
            No channels followed.
          </p>
        {:else}
          {#each channels as channel}
            <ChannelCard
              {channel}
              active={selectedChannelId === channel.id}
              showDelete={manageChannels}
              draggableEnabled
              loading={channel.id.startsWith("temp-")}
              dragging={draggedChannelId === channel.id}
              dragOver={dragOverChannelId === channel.id &&
                draggedChannelId !== channel.id}
              onSelect={() => selectChannel(channel.id)}
              onDragStart={(event) => handleChannelDragStart(channel.id, event)}
              onDragOver={(event) => handleChannelDragOver(channel.id, event)}
              onDrop={(event) => handleChannelDrop(channel.id, event)}
              onDragEnd={handleChannelDragEnd}
              onDelete={() => handleDeleteChannel(channel.id)}
            />
          {/each}
        {/if}
      </div>
    </aside>

    <section
      class="flex min-h-0 min-w-0 flex-col gap-6 overflow-hidden border-0 lg:pl-6 fade-in stagger-2 {mobileTab !==
      'details'
        ? 'hidden lg:flex'
        : 'h-full p-4 pt-4'}"
    >
      <div
        class="flex flex-wrap items-center justify-between gap-4 pb-3 border-b border-[var(--border-soft)]"
      >
        <div class="flex items-center gap-3 min-w-0">
          <button
            onclick={() => (mobileTab = "channels")}
            class="lg:hidden inline-flex items-center gap-2 text-[13px] font-semibold text-[var(--foreground)] transition-transform active:scale-95"
          >
            <div
              class="h-6 w-6 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]"
            >
              <img
                src={selectedChannel?.thumbnail_url || defaultChannelIcon}
                alt=""
                class="h-full w-full object-cover"
              />
            </div>
            <span class="truncate"
              >{selectedChannel ? selectedChannel.name : "None"}</span
            >
            <svg
              class="h-3 w-3 opacity-30 shrink-0"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="3"
              stroke-linecap="round"
              stroke-linejoin="round"><path d="m6 9 6 6 6-6" /></svg
            >
          </button>
          <span
            class="max-lg:hidden flex items-center gap-2 text-[13px] font-semibold text-[var(--foreground)]"
          >
            <div
              class="h-6 w-6 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]"
            >
              <img
                src={selectedChannel?.thumbnail_url || defaultChannelIcon}
                alt=""
                class="h-full w-full object-cover"
              />
            </div>
            {selectedChannel ? selectedChannel.name : "None"}
          </span>
          <span
            class="hidden sm:block text-[11px] text-[var(--soft-foreground)] opacity-40"
          >
            {#if lastSyncedAt}
              {syncTimeFormatter.format(lastSyncedAt)}
            {/if}
          </span>
        </div>
        <div class="flex items-center gap-4 text-[11px] font-bold tabular-nums">
          <span
            class="text-[var(--soft-foreground)] opacity-60"
            data-tooltip="Total">{queueStats.total} items</span
          >
          {#if queueStats.loading > 0}
            <span class="text-amber-600 flex items-center gap-1.5">
              <span class="h-1.5 w-1.5 rounded-full bg-amber-500 animate-pulse"
              ></span>
              {queueStats.loading} active
            </span>
          {/if}
          {#if queueStats.failed > 0}
            <span class="text-[var(--danger-foreground)]"
              >{queueStats.failed} failed</span
            >
          {/if}
        </div>
      </div>

      <nav
        class="flex gap-0 border-b border-[var(--border-soft)]"
        aria-label="Queue tabs"
      >
        <button
          type="button"
          class="px-4 py-2.5 text-[11px] font-bold uppercase tracking-[0.1em] transition-all border-b-2 {queueTab ===
          'transcripts'
            ? 'text-[var(--foreground)] border-[var(--foreground)]'
            : 'text-[var(--soft-foreground)] opacity-50 border-transparent hover:opacity-80'}"
          onclick={() => (queueTab = "transcripts")}
          aria-current={queueTab === "transcripts" ? "page" : undefined}
        >
          Transcripts
        </button>
        <button
          type="button"
          class="px-4 py-2.5 text-[11px] font-bold uppercase tracking-[0.1em] transition-all border-b-2 {queueTab ===
          'summaries'
            ? 'text-[var(--foreground)] border-[var(--foreground)]'
            : 'text-[var(--soft-foreground)] opacity-50 border-transparent hover:opacity-80'}"
          onclick={() => (queueTab = "summaries")}
          aria-current={queueTab === "summaries" ? "page" : undefined}
        >
          Summaries
        </button>
        <button
          type="button"
          class="px-4 py-2.5 text-[11px] font-bold uppercase tracking-[0.1em] transition-all border-b-2 {queueTab ===
          'evaluations'
            ? 'text-[var(--foreground)] border-[var(--foreground)]'
            : 'text-[var(--soft-foreground)] opacity-50 border-transparent hover:opacity-80'}"
          onclick={() => (queueTab = "evaluations")}
          aria-current={queueTab === "evaluations" ? "page" : undefined}
        >
          Evaluations
        </button>
      </nav>

      {#if selectedChannel}
        <div class="flex flex-wrap items-center gap-4 py-2">
          <div class="flex items-center gap-2 min-w-0">
            <p
              class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50"
            >
              Sync from
            </p>
            <p class="text-[13px] font-semibold text-[var(--foreground)]">
              {formatSyncDate(effectiveEarliestSyncDate)}
            </p>
          </div>
          <div class="flex items-center gap-2 ml-auto">
            <input
              type="date"
              class="rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--surface)] px-2.5 py-1.5 text-[12px] font-medium focus:outline-none focus:border-[var(--accent)]/40 transition-colors"
              bind:value={earliestSyncDateInput}
              disabled={savingSyncDate}
            />
            <button
              type="button"
              class="rounded-[var(--radius-sm)] bg-[var(--foreground)] px-3 py-1.5 text-[10px] font-bold uppercase tracking-[0.08em] text-white transition-all hover:bg-[var(--accent-strong)] disabled:opacity-30"
              onclick={saveEarliestSyncDate}
              disabled={!earliestSyncDateInput || savingSyncDate}
            >
              {savingSyncDate ? "..." : "Set"}
            </button>
          </div>
        </div>
      {/if}

      <div
        class="custom-scrollbar mobile-bottom-stack-padding flex-1 overflow-y-auto lg:pb-0"
      >
        {#if !selectedChannelId}
          <div
            class="flex flex-col items-center justify-center py-20 text-center"
          >
            <p class="text-[15px] text-[var(--soft-foreground)] opacity-30">
              Select a channel to view its queue.
            </p>
          </div>
        {:else if loadingVideos && videos.length === 0}
          <div class="space-y-4 mt-4" role="status" aria-live="polite">
            {#each Array.from({ length: 4 }) as _, index (index)}
              <div
                class="animate-pulse rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--background)] p-6"
              >
                <div
                  class="h-4 w-3/4 rounded-full bg-[var(--muted)] opacity-60"
                ></div>
                <div
                  class="mt-4 h-3 w-1/4 rounded-full bg-[var(--muted)] opacity-40"
                ></div>
              </div>
            {/each}
          </div>
        {:else if queueStats.total === 0}
          <div
            class="flex flex-col items-center justify-center py-20 text-center"
          >
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="text-emerald-500 mb-3"
              ><polyline points="20 6 9 17 4 12" /></svg
            >
            <p class="text-[14px] text-[var(--soft-foreground)] opacity-50">
              Queue is clear.
            </p>
          </div>
        {:else}
          <ul class="mt-2 flex flex-col">
            {#each queuedVideosWithDistillationStatus as item}
              {@const video = item.video}
              <li class="border-b border-[var(--border-soft)] last:border-b-0">
                <button
                  type="button"
                  class="group w-full cursor-pointer py-4 px-1 text-left focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 max-lg:py-3"
                  onclick={() => openVideoTranscriptInWorkspace(video)}
                  aria-label={`Open transcript workspace for ${video.title}`}
                >
                  <p
                    class="line-clamp-2 text-[14px] font-semibold text-[var(--foreground)] leading-[1.4] tracking-tight group-hover:text-[var(--accent-strong)] transition-colors"
                  >
                    {video.title}
                  </p>
                  <div class="mt-2 flex flex-wrap items-center gap-3">
                    <span
                      class="text-[11px] font-medium text-[var(--soft-foreground)] opacity-50"
                    >
                      {formatDate(video.published_at)}
                    </span>
                    {#if item.distillationStatus.kind === "processing"}
                      <span
                        class="inline-flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--accent)]"
                      >
                        <span class="relative flex h-1.5 w-1.5">
                          <span
                            class="animate-ping absolute inline-flex h-full w-full rounded-full bg-[var(--accent)] opacity-75"
                          ></span>
                          <span
                            class="relative inline-flex rounded-full h-1.5 w-1.5 bg-[var(--accent)]"
                          ></span>
                        </span>
                        Processing
                      </span>
                    {:else if item.distillationStatus.kind === "failed"}
                      <span
                        class="inline-flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--danger-foreground)]"
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
                          ><circle cx="12" cy="12" r="10" /><line
                            x1="12"
                            y1="8"
                            x2="12"
                            y2="12"
                          /><line x1="12" y1="16" x2="12.01" y2="16" /></svg
                        >
                        Failed{#if video.retry_count !== undefined && video.retry_count > 0}
                          ({video.retry_count}/{MAX_RETRIES}){/if}
                      </span>
                    {:else}
                      <span
                        class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-40"
                      >
                        Queued
                      </span>
                    {/if}
                  </div>
                </button>
              </li>
            {/each}
          </ul>
        {/if}

        {#if hasMore && selectedChannelId}
          <div class="mt-4 flex justify-center">
            <button
              type="button"
              class="w-full rounded-[var(--radius-sm)] border border-[var(--border-soft)] py-2.5 text-[11px] font-bold uppercase tracking-[0.15em] text-[var(--soft-foreground)] transition-all hover:border-[var(--accent)]/40 hover:text-[var(--foreground)] disabled:opacity-30"
              onclick={() => loadVideos(false)}
              disabled={loadingVideos}
            >
              {loadingVideos ? "Loading..." : "Load More"}
            </button>
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
      class="mobile-tab-item {mobileTab === 'details'
        ? 'mobile-tab-item--active'
        : ''}"
      onclick={() => (mobileTab = "details")}
      aria-current={mobileTab === "details" ? "page" : undefined}
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
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="16" x2="12" y2="12" />
        <line x1="12" y1="8" x2="12.01" y2="8" />
      </svg>
      <span>Details</span>
    </button>
  </nav>

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

  <FeatureGuide
    open={guideOpen}
    step={guideStep}
    steps={tourSteps}
    onClose={closeGuide}
    onStep={setGuideStep}
  />

  <Footer showMobile />
</div>
