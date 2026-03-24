<script lang="ts">
  import { goto, replaceState as replacePageState } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import {
    addChannel,
    deleteChannel,
    ensureTranscript,
    getChannelSnapshot,
    getChannelSyncDepth,
    listChannelsWhenAvailable,
    listVideos,
    refreshChannel,
    updateChannel,
  } from "$lib/api";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import { DOCS_URL } from "$lib/app-config";
  import FeatureGuide from "$lib/components/FeatureGuide.svelte";
  import type { TourStep } from "$lib/components/FeatureGuide.svelte";
  import ErrorToast from "$lib/components/ErrorToast.svelte";
  import QueueContentPanel from "$lib/components/queue/QueueContentPanel.svelte";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import WorkspaceMobileTabBar from "$lib/components/workspace/WorkspaceMobileTabBar.svelte";
  import WorkspaceSidebar from "$lib/components/workspace/WorkspaceSidebar.svelte";
  import { resolveBootstrapOnMount } from "$lib/ssr-bootstrap";
  import {
    putCachedBootstrapMeta,
    putCachedChannels,
    putCachedViewSnapshot,
  } from "$lib/workspace-cache";
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
  import { createSidebarState } from "$lib/workspace/sidebar-state.svelte";

  const queueMobileTabs = [
    { value: "browse", label: "Browse" },
    { value: "content", label: "Content" },
  ] as const;

  type QueueMobileTab = (typeof queueMobileTabs)[number]["value"];
  const queueMobileTabItems: Array<{ value: string; label: string }> =
    queueMobileTabs.map((tab) => ({ ...tab }));

  function resolveQueueMobileTab(value: string): QueueMobileTab {
    return queueMobileTabs.some((tab) => tab.value === value)
      ? (value as QueueMobileTab)
      : "browse";
  }

  const sidebar = createSidebarState({
    limit: 20,
    onSelectVideo: openQueuedVideo,
    onChannelSelected: (id) => {
      mobileTab = "browse";
    },
    onChannelDeselected: () => {
      mobileTab = "browse";
    },
    onVideoListReset: () => {
      // no-op: historyExhausted and others are constants here
    },
    onVideosLoaded: (res) => {
      if (res.reset) {
        setSyncSnapshot();
      }
      void putCachedViewSnapshot(
        buildQueueSnapshotCacheKey(sidebar.selectedChannelId!),
        {
          channel_id: sidebar.selectedChannelId!,
          channel_video_count: res.videos.length,
          videos: res.videos,
          sync_depth: sidebar.syncDepth,
        } as ChannelSnapshot,
      );
    },
    onError: (msg: string) => {
      errorMessage = msg;
    },
    onChannelAdded: async (channel) => {
      mobileTab = "browse";
      await sidebar.selectChannel(channel.id, null, true);
    },
    onChannelDeleted: (id) => {
      // no-op: handled by the component
    },
    onPersistWorkspaceState: (state) => {
      if (typeof localStorage === "undefined") return;
      saveWorkspaceState(localStorage, state);
    },
    onPersistViewUrl: (state) => {
      if (typeof window === "undefined") return;
      const nextHref = buildQueueViewHref({
        ...state,
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
    },
    onLoadInitial: async (options) => {
      const silent = options?.silent ?? false;
      try {
        const channelList = await listChannelsWhenAvailable({
          retryDelayMs: 500,
        });
        sidebar.setChannels(
          applySavedChannelOrder(channelList, sidebar.channelOrder),
        );
        sidebar.syncChannelOrderFromList();
        void putCachedChannels(sidebar.channels);

        const initialChannelId = resolveInitialChannelSelection(
          sidebar.channels,
          sidebar.selectedChannelId,
          sidebar.channelOrder[0], // Pass a single string (the first channel ID) as the preference
        );

        if (!initialChannelId) {
          sidebar.setSelectedChannelId(null);
          sidebar.setVideos([]);
          sidebar.setSyncDepth(null);
          mobileTab = "browse";
        } else {
          sidebar.setSelectedChannelId(initialChannelId);
          await sidebar.refreshAndLoadVideos(initialChannelId, silent);
        }

        void refreshAiStatus((status) => {
          aiStatus = status.status;
        }).catch(() => {
          aiStatus = "offline";
        });
      } catch (error) {
        if (!silent || !errorMessage) {
          errorMessage = (error as Error).message;
        }
      }
    },
    onLoadChannelSnapshot: async (channelId, snapshotOptions, silent) => {
      return getChannelSnapshot(channelId, snapshotOptions);
    },
    onRefreshChannel: async (channelId) => {
      return refreshChannel(channelId);
    },
    onListVideos: async (
      channelId,
      limit,
      offset,
      videoTypeFilter,
      acknowledgedFilter,
      includeOptimistic,
    ) => {
      return listVideos(
        channelId,
        limit,
        offset,
        videoTypeFilter,
        acknowledgedFilter,
        includeOptimistic,
        queueTab,
      );
    },
    onOpenChannelOverview: async (channelId: string) => {
      // Switch UI immediately; load selected channel queue data in background.
      sidebar.setSelectedChannelId(channelId);
      mobileTab = "content";
      void sidebar.selectChannel(channelId, null, true);
    },
  });

  function setSyncSnapshot() {
    // lastSyncedAt = new Date(); // No longer needed, handled by sidebar composable
  }

  function buildQueueSnapshotCacheKey(channelId: string) {
    const acknowledged = resolveAcknowledgedParam(sidebar.acknowledgedFilter);
    const acknowledgedKey =
      acknowledged === undefined ? "all" : acknowledged ? "ack" : "unack";
    return `queue:${channelId}:tab=${queueTab}:type=${sidebar.videoTypeFilter}:ack=${acknowledgedKey}:limit=${sidebar.limit}`;
  }

  let aiStatus = $state<AiStatus | null>(null);
  let mobileTab = $state<QueueMobileTab>("browse");
  let queueTab = $state<QueueTab>("transcripts");
  let errorMessage = $state<string | null>(null);
  let workspaceStateHydrated = $state(false);
  let viewUrlHydrated = $state(false);
  // let lastSyncedAt = $state<Date | null>(null); // No longer needed
  let earliestSyncDateInput = $state("");
  let savingSyncDate = $state(false);
  let retryingTranscriptVideoId = $state<string | null>(null);

  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  let guideOpen = $state(false);
  let guideStep = $state(0);
  let previousQueueTab = $state<QueueTab>("transcripts");

  const tourSteps: TourStep[] = [
    {
      selector: "#workspace",
      title: "Pick a Channel",
      body: "Select a channel from the sidebar to see what's being processed. This is the same channel list as the main workspace.",
      placement: "right",
      prepare: () => {
        mobileTab = "browse";
      },
    },
    {
      selector: "#queue-stage-tabs",
      title: "Processing Stages",
      body: "Videos go through three stages: transcript download, AI summary generation, and quality check. Switch tabs to see the backlog at each stage.",
      placement: "bottom",
      prepare: () => {
        mobileTab = "content";
      },
    },
    {
      selector: "#content-view",
      title: "Queue Status",
      body: "See how many videos are waiting at each stage and how far back the history goes. This is where you monitor the progress of your library.",
      placement: "left",
      prepare: () => {
        mobileTab = "content";
      },
    },
  ];

  const effectiveEarliestSyncDate = $derived(
    sidebar.selectedChannel?.earliest_sync_date_user_set
      ? sidebar.selectedChannel.earliest_sync_date
      : (sidebar.syncDepth?.derived_earliest_ready_date ??
          sidebar.selectedChannel?.earliest_sync_date),
  );

  const queueStats = $derived({
    total: sidebar.videos.length,
    loading: sidebar.videos.filter((video) => {
      if (queueTab === "transcripts") {
        return video.transcript_status === "loading";
      }
      if (queueTab === "summaries") {
        return video.summary_status === "loading";
      }
      return false;
    }).length,
    pending: sidebar.videos.filter((video) => {
      if (queueTab === "transcripts") {
        return video.transcript_status === "pending";
      }
      if (queueTab === "summaries") {
        return video.summary_status === "pending";
      }
      return true;
    }).length,
    failed: sidebar.videos.filter((video) => {
      if (queueTab === "transcripts") {
        return video.transcript_status === "failed";
      }
      if (queueTab === "summaries") {
        return video.summary_status === "failed";
      }
      return false;
    }).length,
  });

  const failedTranscriptVideos = $derived(
    sidebar.videos.filter((video) => video.transcript_status === "failed"),
  );

  $effect(() =>
    createAiStatusPoller({
      onStatus: (status) => {
        aiStatus = status.status;
      },
    }),
  );

  $effect(() => {
    if (!sidebar.selectedChannel) {
      earliestSyncDateInput = "";
      return;
    }

    const effective = sidebar.selectedChannel.earliest_sync_date_user_set
      ? sidebar.selectedChannel.earliest_sync_date
      : (sidebar.syncDepth?.derived_earliest_ready_date ??
        sidebar.selectedChannel.earliest_sync_date);

    earliestSyncDateInput = effective
      ? new Date(effective).toISOString().split("T")[0]
      : "";
  });

  $effect(() => {
    if (!sidebar.selectedChannelId) {
      if (mobileTab !== "browse") {
        mobileTab = "browse";
      }
    }
  });

  $effect(() => {
    const currentTab = queueTab;
    if (currentTab !== previousQueueTab) {
      previousQueueTab = currentTab;
      if (sidebar.selectedChannelId) {
        sidebar.setVideos([]);
        sidebar.setOffset(0);
        sidebar.setHasMore(true);
        void sidebar.refreshAndLoadVideos(sidebar.selectedChannelId);
      }
    }
  });

  onMount(() => {
    restoreQueueState();
    workspaceStateHydrated = true;

    void (async () => {
      try {
        const selectedChannelIdAtMount = sidebar.selectedChannelId;

        const bootstrapResult = await resolveBootstrapOnMount({
          serverBootstrap: $page.data.bootstrap ?? null,
          selectedChannelId: selectedChannelIdAtMount,
          viewSnapshotCacheKey: selectedChannelIdAtMount
            ? buildQueueSnapshotCacheKey(selectedChannelIdAtMount)
            : null,
        });

        const hasInitialData = Boolean(
          bootstrapResult.channels && bootstrapResult.channels.length > 0,
        );

        if (bootstrapResult.channels && bootstrapResult.channels.length > 0) {
          sidebar.setChannels(
            applySavedChannelOrder(
              bootstrapResult.channels,
              sidebar.channelOrder,
            ),
          );
          sidebar.syncChannelOrderFromList();
        }

        if (bootstrapResult.aiStatus !== null) {
          aiStatus = bootstrapResult.aiStatus;
        }
        if (
          bootstrapResult.aiAvailable !== null &&
          bootstrapResult.aiStatus !== null &&
          bootstrapResult.searchStatus !== null
        ) {
          void putCachedBootstrapMeta({
            ai_available: bootstrapResult.aiAvailable,
            ai_status: bootstrapResult.aiStatus,
            search_status: bootstrapResult.searchStatus,
          });
        }

        if (
          bootstrapResult.snapshot &&
          selectedChannelIdAtMount &&
          bootstrapResult.snapshot.channel_id === selectedChannelIdAtMount
        ) {
          sidebar.setSyncDepth(bootstrapResult.snapshot.sync_depth);
          sidebar.setVideos(bootstrapResult.snapshot.videos);
          sidebar.setOffset(bootstrapResult.snapshot.videos.length);
          sidebar.setHasMore(
            bootstrapResult.snapshot.videos.length === sidebar.limit,
          );
          void putCachedViewSnapshot(
            buildQueueSnapshotCacheKey(selectedChannelIdAtMount),
            {
              channel_id: selectedChannelIdAtMount,
              channel_video_count: bootstrapResult.snapshot.channel_video_count,
              videos: bootstrapResult.snapshot.videos,
              sync_depth: bootstrapResult.snapshot.sync_depth,
            },
          );
        }

        await sidebar.loadInitial({
          silent: hasInitialData,
        });
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
      sidebar.setSelectedChannelId(restored.selectedChannelId ?? null);
    }
    if (restored.channelOrder) {
      sidebar.setChannelOrder(restored.channelOrder);
    }
    if (restored.channelSortMode) {
      sidebar.setChannelSortMode(restored.channelSortMode);
    }
    if (restored.queueTab) {
      queueTab = restored.queueTab;
    }

    mobileTab = "browse";
  }

  async function saveEarliestSyncDate(value: string) {
    if (!sidebar.selectedChannelId) return;
    savingSyncDate = true;
    errorMessage = null;

    try {
      const channel = await updateChannel(sidebar.selectedChannelId, {
        earliest_sync_date: value ? new Date(value).toISOString() : null,
        earliest_sync_date_user_set: true,
      });

      sidebar.updateChannel(channel);
      void putCachedChannels(sidebar.channels);

      // Reload videos with the new sync boundary
      sidebar.setVideos([]);
      sidebar.setOffset(0);
      sidebar.setHasMore(true);
      await sidebar.refreshAndLoadVideos(sidebar.selectedChannelId);
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      savingSyncDate = false;
    }
  }

  async function retryTranscriptDownload(videoId: string) {
    retryingTranscriptVideoId = videoId;
    errorMessage = null;

    try {
      await ensureTranscript(videoId);
      // Wait a bit for the backend to start the job
      await new Promise((resolve) => setTimeout(resolve, 500));
      if (sidebar.selectedChannelId) {
        await sidebar.loadVideos(true, true);
      }
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      retryingTranscriptVideoId = null;
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
    if (!sidebar.selectedChannelId) return;
    sidebar.setSelectedVideoId(videoId);
    const href = buildWorkspaceViewHref({
      selectedChannelId: sidebar.selectedChannelId,
      selectedVideoId: videoId,
      contentMode: "transcript",
      videoTypeFilter: "all",
      acknowledgedFilter: "all",
    });
    await goto(href);
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

  const queueContentPanelState = $derived({
    mobileVisible: true,
    selectedChannel: sidebar.selectedChannel,
    selectedChannelId: sidebar.selectedChannelId,
    queueTab,
    queueStats,
    failedTranscriptVideos,
    retryingTranscriptVideoId,
    effectiveEarliestSyncDate,
    earliestSyncDateInput,
    savingSyncDate,
    refreshingChannel: sidebar.refreshingChannel,
  });
  const queueContentPanelActions = {
    onBack: () => {
      mobileTab = "browse";
    },
    onQueueTabChange: (value: QueueTab) => {
      queueTab = value;
    },
    onSaveSyncDate: saveEarliestSyncDate,
    onRetryTranscript: retryTranscriptDownload,
  };

  const shellCollapsed = $derived(sidebar.sidebarCollapsed);
  const shellWidth = $derived(sidebar.sidebarWidth);
  const shellToggleSidebar = sidebar.toggleSidebar;
  const queueSidebar = sidebar;
</script>

<WorkspaceShell currentSection="queue" {aiIndicator} onOpenGuide={openGuide}>
  {#snippet sidebar({
    collapsed: shellCollapsed,
    toggle: shellToggleSidebar,
    width: shellWidth,
  })}
    <WorkspaceSidebar
      videoListMode="per_channel_preview"
        addSourceErrorMessage={errorMessage}
      initialChannelPreviews={$page.data.channelPreviews ?? {}}
      initialChannelPreviewsFilterKey={$page.data.channelPreviewsFilterKey ??
        `all:all:${queueTab}`}
      channelSnapshotQueueTab={queueTab}
      readOnly={true}
      shell={{
        collapsed: shellCollapsed,
        width: shellWidth,
        mobileVisible: false,
        onToggleCollapse: shellToggleSidebar,
      }}
      channelState={queueSidebar.channelState}
      channelActions={queueSidebar.channelActions}
      videoState={queueSidebar.videoState}
      videoActions={queueSidebar.videoActions}
    />
  {/snippet}

  <WorkspaceMobileTabBar
    tabs={queueMobileTabItems}
    activeTab={mobileTab}
    onTabChange={(tab) => {
      mobileTab = resolveQueueMobileTab(tab);
    }}
  />

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
            addSourceErrorMessage={errorMessage}
          initialChannelPreviews={$page.data.channelPreviews ?? {}}
          initialChannelPreviewsFilterKey={$page.data
            .channelPreviewsFilterKey ?? `all:all:${queueTab}`}
          channelSnapshotQueueTab={queueTab}
          readOnly={true}
          shell={{
            collapsed: false,
            width: undefined,
            mobileVisible: true,
            onToggleCollapse: () => {},
          }}
          channelState={queueSidebar.channelState}
          channelActions={queueSidebar.channelActions}
          videoState={queueSidebar.videoState}
          videoActions={{
            ...queueSidebar.videoActions,
            onSelectVideo: (videoId) => {
              mobileTab = "content";
              void openQueuedVideo(videoId);
            },
          }}
        />
      </div>
    </div>
  {/if}

  <QueueContentPanel
    readOnly={true}
    state={queueContentPanelState}
    actions={queueContentPanelActions}
  />

  {#if errorMessage}
    <ErrorToast
      message={errorMessage}
      onDismiss={() => (errorMessage = null)}
    />
  {/if}

  <FeatureGuide
    open={guideOpen}
    step={guideStep}
    steps={tourSteps}
    docsUrl={DOCS_URL}
    onClose={closeGuide}
    onStep={setGuideStep}
  />
</WorkspaceShell>
