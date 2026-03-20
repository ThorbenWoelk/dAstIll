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
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import WorkspaceMobileTabBar from "$lib/components/workspace/WorkspaceMobileTabBar.svelte";
  import WorkspaceSidebar from "$lib/components/workspace/WorkspaceSidebar.svelte";
  import {
    applySavedChannelOrder,
    buildQueueSnapshotOptions,
    loadWorkspaceState,
    resolveInitialChannelSelection,
    restoreWorkspaceSnapshot,
    saveWorkspaceState,
  } from "$lib/channel-workspace";
  import type {
    AiStatus,
    Channel,
    ChannelSnapshot,
    QueueTab,
    SearchResult,
    VideoTypeFilter,
    Video,
  } from "$lib/types";
  import {
    buildQueueViewHref,
    buildWorkspaceViewHref,
    mergeQueueViewState,
    parseQueueViewUrlState,
  } from "$lib/view-url";
  import {
    buildChannelViewCacheKey,
    cloneDate,
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
    resolveAcknowledgedParam,
    type AcknowledgedFilter,
    type ChannelSortMode,
  } from "$lib/workspace/types";
  import { createAiStatusPoller, refreshAiStatus } from "$lib/utils/ai-poller";
  import {
    resolveGuideStepFromUrl,
    writeGuideStepToUrl,
  } from "$lib/utils/guide";

  const CHANNEL_REFRESH_TTL_MS = 5 * 60 * 1000;
  const limit = 20;
  const channelLastRefreshedAt = new Map<string, number>();

  type CachedQueueChannelState = {
    videos: Video[];
    offset: number;
    hasMore: boolean;
    lastSyncedAt: Date | null;
    syncDepth: ChannelSyncDepthState | null;
  };

  const queueChannelStateCache =
    createChannelViewCache<CachedQueueChannelState>((state) => ({
      ...state,
      videos: cloneVideos(state.videos),
      lastSyncedAt: cloneDate(state.lastSyncedAt),
      syncDepth: cloneSyncDepthState(state.syncDepth),
    }));

  const queueMobileTabs = [
    { value: "browse", label: "Browse" },
    { value: "content", label: "Content" },
  ] as const;

  type QueueMobileTab = (typeof queueMobileTabs)[number]["value"];

  let channels = $state<Channel[]>([]);
  let channelOrder = $state<string[]>([]);
  let videos = $state<Video[]>([]);
  let selectedChannelId = $state<string | null>(null);
  let loadingChannels = $state(false);
  let loadingVideos = $state(false);
  let addingChannel = $state(false);
  let channelSortMode = $state<ChannelSortMode>("custom");
  let aiStatus = $state<AiStatus | null>(null);
  let mobileTab = $state<QueueMobileTab>("browse");
  let queueTab = $state<QueueTab>("transcripts");
  let videoTypeFilter = $state<VideoTypeFilter>("all");
  let acknowledgedFilter = $state<AcknowledgedFilter>("all");
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
  let syncDepth = $state<ChannelSyncDepthState | null>(null);
  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  let guideOpen = $state(false);
  let guideStep = $state(0);
  let previousQueueTab = $state<QueueTab>("transcripts");
  const historyExhausted = true;
  const backfillingHistory = false;
  const allowLoadedVideoSyncDepthOverride = false;

  const tourSteps: TourStep[] = [
    {
      selector: "#workspace",
      title: "Channel Sidebar",
      body: "Queue now uses the same channel navigation shell as Workspace, including search, sort, add, and reorder controls.",
      placement: "right",
      prepare: () => {
        mobileTab = "browse";
      },
    },
    {
      selector: "#queue-stage-tabs",
      title: "Queue Stages",
      body: "Switch between transcript, summary, and evaluation queues while keeping the same workspace-style sidebar navigation.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "content";
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
        mobileTab = selectedChannelId ? "content" : "browse";
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

  function getQueueChannelViewKey(channelId: string) {
    return buildChannelViewCacheKey(
      channelId,
      queueTab,
      videoTypeFilter,
      acknowledgedFilter,
    );
  }

  function restoreCachedQueueChannelState(state: CachedQueueChannelState) {
    videos = cloneVideos(state.videos);
    offset = state.offset;
    hasMore = state.hasMore;
    lastSyncedAt = cloneDate(state.lastSyncedAt);
    syncDepth = cloneSyncDepthState(state.syncDepth);
  }

  $effect(() => {
    if (!selectedChannelId) return;

    queueChannelStateCache.set(getQueueChannelViewKey(selectedChannelId), {
      videos: cloneVideos(videos),
      offset,
      hasMore,
      lastSyncedAt: cloneDate(lastSyncedAt),
      syncDepth: cloneSyncDepthState(syncDepth),
    });
  });

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
      if (mobileTab !== "browse") {
        mobileTab = "browse";
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

    mobileTab = "browse";
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
        mobileTab = "browse";
      } else {
        selectedChannelId = initialChannelId;
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
    const channelViewKey = getQueueChannelViewKey(channelId);
    const cachedQueueChannelState = queueChannelStateCache.get(channelViewKey);
    const hasCachedQueueChannelState =
      !!cachedQueueChannelState && cachedQueueChannelState.videos.length > 0;

    selectedChannelId = channelId;
    if (fromUserInteraction) {
      mobileTab = "browse";
    }
    if (hasCachedQueueChannelState && cachedQueueChannelState) {
      restoreCachedQueueChannelState(cachedQueueChannelState);
      loadingVideos = false;
      void refreshAndLoadVideos(channelId, true);
      return;
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
    mobileTab = "browse";

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
        const nextChannelId = resolveNextChannelSelection(channels, channelId);
        if (nextChannelId) {
          await selectChannel(nextChannelId);
        } else {
          selectedChannelId = null;
          videos = [];
          syncDepth = null;
          mobileTab = "browse";
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

  async function refreshAndLoadVideos(
    channelId: string,
    silentInitialSnapshot = false,
  ) {
    const snapshotOptions = {
      ...buildQueueSnapshotOptions(queueTab, limit),
      videoType: videoTypeFilter,
      acknowledged: resolveAcknowledgedParam(acknowledgedFilter),
    };
    await loadChannelSnapshotWithRefresh({
      channelId,
      refreshedAtByChannel: channelLastRefreshedAt,
      ttlMs: CHANNEL_REFRESH_TTL_MS,
      loadSnapshot: () => getChannelSnapshot(channelId, snapshotOptions),
      applySnapshot: (snapshot, silent = false) =>
        applyChannelSnapshot(channelId, snapshot, silent),
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
      const list = await listVideos(
        selectedChannelId,
        limit,
        reset ? 0 : offset,
        videoTypeFilter,
        resolveAcknowledgedParam(acknowledgedFilter),
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

  async function openQueuedVideo(videoId: string) {
    const video = videos.find((item) => item.id === videoId);
    if (!video) return;
    await openVideoTranscriptInWorkspace(video);
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

  const queueSidebarChannelState = $derived({
    channels,
    selectedChannelId,
    loadingChannels,
    addingChannel,
    channelSortMode,
  });
  const queueSidebarVideoState = $derived({
    videos,
    selectedVideoId: null,
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
  const queueSidebarChannelActions = {
    onChannelSortModeChange: (nextValue: ChannelSortMode) => {
      channelSortMode = nextValue;
    },
    onAddChannel: handleAddChannel,
    onSelectChannel: (channelId: string) => {
      if (channelId === selectedChannelId) {
        selectedChannelId = null;
        syncDepth = null;
        return;
      }
      void selectChannel(channelId, true);
    },
    onDeleteChannel: handleDeleteChannel,
    onReorderChannels: reorderChannels,
  };
  const queueSidebarVideoActions = {
    onSelectVideo: openQueuedVideo,
    onLoadMoreVideos: () => loadVideos(false),
    onVideoTypeFilterChange: setVideoTypeFilter,
    onAcknowledgedFilterChange: setAcknowledgedFilter,
  };
  const queueContentPanelState = $derived({
    mobileVisible: true,
    selectedChannel,
    selectedChannelId,
    queueTab,
    queueStats,
    effectiveEarliestSyncDate,
    earliestSyncDateInput,
    savingSyncDate,
    refreshingChannel,
  });
  const queueContentPanelActions = {
    onBack: () => {
      mobileTab = "browse";
    },
    onQueueTabChange: (value: QueueTab) => {
      queueTab = value;
    },
    onSaveSyncDate: saveEarliestSyncDate,
  };
</script>

<WorkspaceShell currentSection="queue" {aiIndicator} onOpenGuide={openGuide}>
  {#snippet sidebar({ collapsed, toggle, width })}
    <WorkspaceSidebar
      shell={{
        collapsed,
        width,
        mobileVisible: false,
        onToggleCollapse: toggle,
      }}
      channelState={queueSidebarChannelState}
      channelActions={queueSidebarChannelActions}
      videoState={queueSidebarVideoState}
      videoActions={queueSidebarVideoActions}
    />
  {/snippet}

  <WorkspaceMobileTabBar
    tabs={queueMobileTabs as unknown as Array<{ value: string; label: string }>}
    activeTab={mobileTab}
    onTabChange={(tab) => {
      mobileTab = tab as QueueMobileTab;
    }}
  />

  {#if mobileTab === "browse"}
    <div
      class="fixed inset-0 z-[80] lg:hidden"
      role="dialog"
      aria-modal="true"
      aria-label="Browse queue channels"
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
          shell={{
            collapsed: false,
            width: undefined,
            mobileVisible: true,
            onToggleCollapse: () => {},
          }}
          channelState={queueSidebarChannelState}
          channelActions={queueSidebarChannelActions}
          videoState={queueSidebarVideoState}
          videoActions={queueSidebarVideoActions}
        />
      </div>
    </div>
  {/if}

  <QueueContentPanel
    state={queueContentPanelState}
    actions={queueContentPanelActions}
  />

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
</WorkspaceShell>
