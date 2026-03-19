<script lang="ts">
  import { goto, replaceState as replacePageState } from "$app/navigation";
  import { onMount } from "svelte";
  import {
    addChannel,
    deleteChannel,
    getChannelSnapshot,
    getChannelSyncDepth,
    listChannelsWhenAvailable,
    listVideos,
    refreshChannel,
    updateChannel,
  } from "$lib/api";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import { DOCS_URL } from "$lib/app-config";
  import FeatureGuide, {
    type TourStep,
  } from "$lib/components/FeatureGuide.svelte";
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import ErrorToast from "$lib/components/ErrorToast.svelte";
  import QueueContentPanel from "$lib/components/queue/QueueContentPanel.svelte";
  import QueueVideoSidebar from "$lib/components/queue/QueueVideoSidebar.svelte";
  import WorkspaceChannelSidebar from "$lib/components/workspace/WorkspaceChannelSidebar.svelte";
  import WorkspaceHeader from "$lib/components/workspace/WorkspaceHeader.svelte";
  import WorkspaceMobileTabBar from "$lib/components/workspace/WorkspaceMobileTabBar.svelte";
  import {
    applySavedChannelOrder,
    buildQueueSnapshotOptions,
    loadWorkspaceState,
    markChannelRefreshed,
    resolveInitialChannelSelection,
    restoreWorkspaceSnapshot,
    saveWorkspaceState,
    shouldRefreshChannel,
  } from "$lib/channel-workspace";
  import type {
    AiStatus,
    Channel,
    ChannelSnapshot,
    QueueTab,
    SearchResult,
    Video,
  } from "$lib/types";
  import {
    buildQueueViewHref,
    buildWorkspaceViewHref,
    mergeQueueViewState,
    parseQueueViewUrlState,
  } from "$lib/view-url";
  import { channelOrderFromList } from "$lib/workspace/channels";
  import {
    buildOptimisticChannel,
    removeChannelFromCollection,
    removeChannelId,
    replaceOptimisticChannel,
    replaceOptimisticChannelId,
  } from "$lib/workspace/channel-actions";
  import type {
    ChannelSortMode,
    DistillationStatusCopy,
  } from "$lib/workspace/types";
  import { createAiStatusPoller, refreshAiStatus } from "$lib/utils/ai-poller";
  import {
    resolveGuideStepFromUrl,
    writeGuideStepToUrl,
  } from "$lib/utils/guide";

  const CHANNEL_REFRESH_TTL_MS = 5 * 60 * 1000;
  const MAX_RETRIES = 3;
  const limit = 20;
  const channelLastRefreshedAt = new Map<string, number>();

  type QueueMobileTab = "channels" | "videos" | "content";

  let channels = $state<Channel[]>([]);
  let channelOrder = $state<string[]>([]);
  let videos = $state<Video[]>([]);
  let selectedChannelId = $state<string | null>(null);
  let loadingChannels = $state(false);
  let loadingVideos = $state(false);
  let addingChannel = $state(false);
  let channelSortMode = $state<ChannelSortMode>("custom");
  let aiStatus = $state<AiStatus | null>(null);
  let mobileTab = $state<QueueMobileTab>("channels");
  let queueTab = $state<QueueTab>("transcripts");
  let errorMessage = $state<string | null>(null);
  let showDeleteConfirmation = $state(false);
  let channelIdToDelete = $state<string | null>(null);
  let workspaceStateHydrated = $state(false);
  let viewUrlHydrated = $state(false);
  let offset = $state(0);
  let hasMore = $state(true);
  let lastSyncedAt = $state<Date | null>(null);
  let earliestSyncDateInput = $state("");
  let savingSyncDate = $state(false);
  let refreshingChannel = $state(false);
  let syncDepth = $state<{
    earliest_sync_date: string | null;
    earliest_sync_date_user_set: boolean;
    derived_earliest_ready_date: string | null;
  } | null>(null);
  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );

  let guideOpen = $state(false);
  let guideStep = $state(0);
  let previousQueueTab = $state<QueueTab>("transcripts");

  const tourSteps: TourStep[] = [
    {
      selector: "#workspace",
      title: "Channel Sidebar",
      body: "Queue now uses the same channel navigation shell as Workspace, including search, sort, add, and reorder controls.",
      placement: "right",
      prepare: () => {
        mobileTab = "channels";
      },
    },
    {
      selector: "#videos",
      title: "Queue List",
      body: "The middle pane mirrors Workspace navigation while keeping queue-specific tabs for transcripts, summaries, and evaluations.",
      placement: "right",
      prepare: () => {
        mobileTab = "videos";
      },
    },
    {
      selector: "#content-view",
      title: "Queue Insights",
      body: "The right pane gives you channel-level queue health, sync depth controls, and stage-specific context without leaving Queue.",
      placement: "left",
      prepare: () => {
        mobileTab = "content";
      },
    },
    {
      selector: "nav[aria-label='Workspace sections']",
      title: "Navigate the app",
      body: "Queue now shares the same shell and section navigation patterns as Workspace.",
      placement: "bottom",
      prepare: () => {
        mobileTab = selectedChannelId ? "videos" : "channels";
      },
    },
  ];

  const selectedChannel = $derived(
    channels.find((channel) => channel.id === selectedChannelId) ?? null,
  );
  const queuedVideos = $derived(videos);
  const effectiveEarliestSyncDate = $derived(
    selectedChannel?.earliest_sync_date_user_set
      ? selectedChannel.earliest_sync_date
      : (syncDepth?.derived_earliest_ready_date ??
          selectedChannel?.earliest_sync_date),
  );
  const queueStats = $derived({
    total: queuedVideos.length,
    loading: queuedVideos.filter((video) => {
      if (queueTab === "transcripts") {
        return video.transcript_status === "loading";
      }
      if (queueTab === "summaries") {
        return video.summary_status === "loading";
      }
      return false;
    }).length,
    pending: queuedVideos.filter((video) => {
      if (queueTab === "transcripts") {
        return video.transcript_status === "pending";
      }
      if (queueTab === "summaries") {
        return video.summary_status === "pending";
      }
      return true;
    }).length,
    failed: queuedVideos.filter((video) => {
      if (queueTab === "transcripts") {
        return video.transcript_status === "failed";
      }
      if (queueTab === "summaries") {
        return video.summary_status === "failed";
      }
      return false;
    }).length,
  });
  const queuedVideosWithDistillationStatus = $derived(
    queuedVideos.map((video) => ({
      video,
      distillationStatus: getDistillationStatusCopy(video),
    })),
  );

  $effect(() =>
    createAiStatusPoller({
      onStatus: (status) => {
        aiStatus = status.status;
      },
    }),
  );

  $effect(() => {
    if (!selectedChannel) {
      earliestSyncDateInput = "";
      return;
    }

    const effective = selectedChannel.earliest_sync_date_user_set
      ? selectedChannel.earliest_sync_date
      : (syncDepth?.derived_earliest_ready_date ??
        selectedChannel.earliest_sync_date);

    earliestSyncDateInput = effective
      ? new Date(effective).toISOString().split("T")[0]
      : "";
  });

  $effect(() => {
    if (!selectedChannelId) {
      syncDepth = null;
      if (mobileTab !== "channels") {
        mobileTab = "channels";
      }
    }
  });

  $effect(() => {
    persistWorkspaceState();
  });

  $effect(() => {
    persistViewUrl();
  });

  $effect(() => {
    const currentTab = queueTab;
    if (currentTab !== previousQueueTab) {
      previousQueueTab = currentTab;
      if (selectedChannelId) {
        videos = [];
        offset = 0;
        hasMore = true;
        void refreshAndLoadVideos(selectedChannelId);
      }
    }
  });

  onMount(() => {
    restoreQueueState();
    previousQueueTab = queueTab;
    workspaceStateHydrated = true;

    void (async () => {
      try {
        await loadInitial();
      } finally {
        viewUrlHydrated = true;
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
  });

  function openGuide() {
    guideStep = 0;
    guideOpen = true;
    writeGuideStepToUrl(0);
  }

  function closeGuide() {
    guideOpen = false;
    writeGuideStepToUrl(null);
  }

  function setGuideStep(step: number) {
    guideStep = step;
    writeGuideStepToUrl(step);
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

    return {
      kind: "queued",
      label: "PENDING EVALUATION",
      detail: "Summary is ready. Waiting for quality evaluation.",
    };
  }

  function setSyncSnapshot() {
    lastSyncedAt = new Date();
  }

  function syncChannelOrderFromList() {
    channelOrder = channelOrderFromList(channels);
  }

  function restoreQueueState() {
    const restored = mergeQueueViewState(
      restoreWorkspaceSnapshot(
        typeof localStorage === "undefined"
          ? null
          : loadWorkspaceState(localStorage),
        {
          includeChannelSortMode: true,
        },
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
    if (restored.channelSortMode) {
      channelSortMode = restored.channelSortMode;
    }
    if (restored.queueTab) {
      queueTab = restored.queueTab;
    }

    mobileTab = selectedChannelId ? "videos" : "channels";
  }

  function persistWorkspaceState() {
    if (!workspaceStateHydrated || typeof localStorage === "undefined") return;
    saveWorkspaceState(localStorage, {
      selectedChannelId,
      channelOrder,
      channelSortMode,
    });
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
      setSyncSnapshot();
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

  async function loadInitial() {
    loadingChannels = true;
    errorMessage = null;

    try {
      const channelList = await listChannelsWhenAvailable({
        retryDelayMs: 500,
      });
      channels = applySavedChannelOrder(channelList, channelOrder);
      syncChannelOrderFromList();
      loadingChannels = false;

      const initialChannelId = resolveInitialChannelSelection(
        channels,
        selectedChannelId,
        null,
      );

      if (!initialChannelId) {
        selectedChannelId = null;
        videos = [];
        syncDepth = null;
        mobileTab = "channels";
      } else {
        selectedChannelId = initialChannelId;
        if (mobileTab === "channels") {
          mobileTab = "videos";
        }
        void refreshAndLoadVideos(initialChannelId);
      }

      void refreshAiStatus((status) => {
        aiStatus = status.status;
      }).catch(() => {
        aiStatus = "offline";
      });
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      loadingChannels = false;
    }
  }

  async function selectChannel(channelId: string, fromUserInteraction = false) {
    selectedChannelId = channelId;
    if (fromUserInteraction) {
      mobileTab = "videos";
    }
    videos = [];
    offset = 0;
    hasMore = true;
    lastSyncedAt = null;
    await refreshAndLoadVideos(channelId);
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
    selectedChannelId = tempId;
    mobileTab = "videos";

    try {
      const channel = await addChannel(trimmedInput);
      channels = replaceOptimisticChannel(channels, tempId, channel);
      channelOrder = replaceOptimisticChannelId(
        channelOrder,
        tempId,
        channel.id,
      );
      selectedChannelId = channel.id;
      await selectChannel(channel.id, true);
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
      channels = removeChannelFromCollection(channels, channelId);
      channelOrder = removeChannelId(channelOrder, channelId);
      if (selectedChannelId === channelId) {
        const nextChannel = channels[0] ?? null;
        if (nextChannel) {
          await selectChannel(nextChannel.id);
        } else {
          selectedChannelId = null;
          videos = [];
          syncDepth = null;
          mobileTab = "channels";
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

  function reorderChannels(nextOrder: string[]) {
    channels = applySavedChannelOrder(channels, nextOrder);
    channelOrder = nextOrder;
  }

  async function refreshAndLoadVideos(channelId: string) {
    const snapshotOptions = buildQueueSnapshotOptions(queueTab, limit);
    const snapshot = await getChannelSnapshot(channelId, snapshotOptions);
    await applyChannelSnapshot(channelId, snapshot);

    if (
      !shouldRefreshChannel(
        channelLastRefreshedAt,
        channelId,
        CHANNEL_REFRESH_TTL_MS,
      )
    ) {
      return;
    }

    refreshingChannel = true;
    try {
      await refreshChannel(channelId);
      markChannelRefreshed(channelLastRefreshedAt, channelId);
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
        setSyncSnapshot();
      }
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

  async function saveEarliestSyncDate(value: string) {
    if (!selectedChannelId || !value || savingSyncDate) return;
    errorMessage = null;
    savingSyncDate = true;
    try {
      const updated = await updateChannel(selectedChannelId, {
        earliest_sync_date: new Date(value).toISOString(),
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

  async function handleSearchResultSelection(
    result: SearchResult,
    targetMode: "transcript" | "summary",
  ) {
    if (typeof localStorage !== "undefined") {
      saveWorkspaceState(localStorage, {
        selectedChannelId: result.channel_id,
        selectedVideoId: result.video_id,
        contentMode: targetMode,
        videoTypeFilter: "all",
        acknowledgedFilter: "all",
      });
    }

    await goto(
      buildWorkspaceViewHref({
        selectedChannelId: result.channel_id,
        selectedVideoId: result.video_id,
        contentMode: targetMode,
        videoTypeFilter: "all",
        acknowledgedFilter: "all",
      }),
    );
  }
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
    currentSection="queue"
    {aiIndicator}
    onOpenGuide={openGuide}
    onSearchResultSelect={handleSearchResultSelection}
  />

  <WorkspaceMobileTabBar
    activeTab={mobileTab}
    onTabChange={(tab) => {
      mobileTab = tab as QueueMobileTab;
    }}
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
      onSelectChannel={(channelId) => selectChannel(channelId, true)}
      onDeleteChannel={handleDeleteChannel}
      onReorderChannels={reorderChannels}
    />

    <QueueVideoSidebar
      mobileVisible={mobileTab === "videos"}
      {selectedChannelId}
      {selectedChannel}
      {queueTab}
      {loadingVideos}
      {refreshingChannel}
      {hasMore}
      {lastSyncedAt}
      {queueStats}
      items={queuedVideosWithDistillationStatus}
      onBack={() => {
        mobileTab = "channels";
      }}
      onQueueTabChange={(value) => {
        queueTab = value;
      }}
      onOpenVideo={openVideoTranscriptInWorkspace}
      onLoadMoreVideos={() => loadVideos(false)}
    />

    <QueueContentPanel
      mobileVisible={mobileTab === "content"}
      {selectedChannel}
      {selectedChannelId}
      {queueTab}
      {queueStats}
      {effectiveEarliestSyncDate}
      {earliestSyncDateInput}
      {savingSyncDate}
      {refreshingChannel}
      onBack={() => {
        mobileTab = "videos";
      }}
      onSaveSyncDate={saveEarliestSyncDate}
    />
  </main>

  {#if errorMessage}
    <ErrorToast
      message={errorMessage}
      onDismiss={() => (errorMessage = null)}
    />
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
