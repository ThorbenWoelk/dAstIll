<script lang="ts">
  import { browser } from "$app/environment";
  import { goto, replaceState as replacePageState } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { authState } from "$lib/auth-state.svelte";
  import {
    getAuthStorageScopeKey,
    getScopedStorageKey,
  } from "$lib/auth-storage";
  import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
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
  import ErrorToast from "$lib/components/ErrorToast.svelte";
  import MobileChannelGallery from "$lib/components/mobile/MobileChannelGallery.svelte";
  import MobileYouTubeTopNav from "$lib/components/mobile/MobileYouTubeTopNav.svelte";
  import QueueContentPanel from "$lib/components/queue/QueueContentPanel.svelte";
  import {
    buildQueueGalleryChannelPreviews,
    deriveEarliestSyncDateInput,
    deriveEffectiveEarliestSyncDate,
    deriveQueueRefreshCadence,
    deriveQueueStats,
  } from "$lib/queue/route-state";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import WorkspaceSidebar from "$lib/components/workspace/WorkspaceSidebar.svelte";
  import { resolveBootstrapOnMount } from "$lib/ssr-bootstrap";
  import {
    putCachedBootstrapMeta,
    putCachedChannels,
    putCachedViewSnapshot,
  } from "$lib/workspace-cache";
  import {
    applySavedChannelOrder,
    loadWorkspaceState,
    resolveInitialChannelSelection,
    restoreWorkspaceSnapshot,
    saveWorkspaceState,
  } from "$lib/channel-workspace";
  import type {
    AiStatus,
    Channel,
    ChannelSnapshot,
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
    isAcknowledgedFilter,
    isWorkspaceVideoTypeFilter,
    resolveAcknowledgedParam,
    type AcknowledgedFilter,
    type ChannelSortMode,
  } from "$lib/workspace/types";
  import { createAiStatusPoller, refreshAiStatus } from "$lib/utils/ai-poller";
  import { createSidebarState } from "$lib/workspace/sidebar-state.svelte";
  import { mobileBottomBar } from "$lib/mobile-navigation/mobileBottomBar";

  const sidebar = createSidebarState({
    limit: 20,
    getViewCacheScopeKey: () => "queue",
    onSelectVideo: openQueuedVideo,
    onChannelSelected: (_id) => {},
    onChannelDeselected: () => {},
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
        getAuthStorageScopeKey(authState.current),
      );
    },
    onError: (msg: string) => {
      errorMessage = msg;
    },
    onChannelAdded: async (channel) => {
      await sidebar.selectChannel(channel.id, null, true);
    },
    onChannelDeleted: (id) => {
      // no-op: handled by the component
    },
    onPersistWorkspaceState: (state) => {
      if (typeof localStorage === "undefined") return;
      saveWorkspaceState(localStorage, state, workspaceStorageKey);
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
        void putCachedChannels(sidebar.channels, workspaceCacheScopeKey);

        const initialChannelId = resolveInitialChannelSelection(
          sidebar.channels,
          sidebar.selectedChannelId,
          sidebar.channelOrder[0], // Pass a single string (the first channel ID) as the preference
        );

        if (!initialChannelId) {
          sidebar.setSelectedChannelId(null);
          sidebar.setVideos([]);
          sidebar.setSyncDepth(null);
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
        if (presentAuthRequiredNoticeIfNeeded(error)) {
          // Modal only
        } else if (!silent || !errorMessage) {
          errorMessage = (error as Error).message;
        }
      }
    },
    onLoadChannelSnapshot: async (channelId, snapshotOptions, silent) => {
      return getChannelSnapshot(channelId, {
        ...snapshotOptions,
        queueOnly: true,
      });
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
      _includeOptimistic,
    ) => {
      return listVideos(
        channelId,
        limit,
        offset,
        videoTypeFilter,
        acknowledgedFilter,
        true,
        undefined,
      );
    },
    onOpenChannelOverview: async (channelId: string) => {
      // Switch UI immediately; load selected channel queue data in background.
      sidebar.setSelectedChannelId(channelId);
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
    return `queue:${channelId}:type=${sidebar.videoTypeFilter}:ack=${acknowledgedKey}:limit=${sidebar.limit}`;
  }

  let aiStatus = $state<AiStatus | null>(null);
  let errorMessage = $state<string | null>(null);
  let workspaceStorageKey = $derived(
    getScopedStorageKey(
      "dastill.workspace.state.v1",
      getAuthStorageScopeKey(authState.current),
    ),
  );
  let workspaceCacheScopeKey = $derived(
    getAuthStorageScopeKey(authState.current),
  );
  let workspaceStateHydrated = $state(false);
  let viewUrlHydrated = $state(false);
  /** Mirrors workspace: replaceState is unsafe until after the client router is ready. */
  let queueUrlSyncReady = $state(false);
  // let lastSyncedAt = $state<Date | null>(null); // No longer needed
  const earliestSyncDateInput = $derived(
    deriveEarliestSyncDateInput(sidebar.selectedChannel, sidebar.syncDepth),
  );
  let savingSyncDate = $state(false);
  let retryingTranscriptVideoId = $state<string | null>(null);
  /** Bumped after each silent list refresh so desktop per-channel queue lists reload. */
  let queueVideoRefreshTick = $state(0);

  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  const effectiveEarliestSyncDate = $derived(
    deriveEffectiveEarliestSyncDate(sidebar.selectedChannel, sidebar.syncDepth),
  );

  const queueStats = $derived(deriveQueueStats(sidebar.videos));

  const failedTranscriptVideos = $derived(
    sidebar.videos.filter((video) => video.transcript_status === "failed"),
  );

  const queueRefreshCadence = $derived.by(
    (): "off" | "fast" | "slow" | "idle" => {
      return deriveQueueRefreshCadence({
        browser,
        selectedChannelId: sidebar.selectedChannelId,
        loadingVideos: sidebar.loadingVideos,
        videos: sidebar.videos,
      });
    },
  );

  const queueRefreshIntervalMs = $derived(
    queueRefreshCadence === "fast"
      ? 3000
      : queueRefreshCadence === "slow"
        ? 8000
        : queueRefreshCadence === "idle"
          ? 12000
          : 0,
  );

  $effect(() => {
    const ms = queueRefreshIntervalMs;
    if (ms === 0) return;

    const timer = window.setInterval(() => {
      void (async () => {
        await sidebar.loadVideos(true, true);
        queueVideoRefreshTick += 1;
        const sel = sidebar.selectedVideoId;
        if (sel && !sidebar.videos.some((v) => v.id === sel)) {
          sidebar.setSelectedVideoId(null);
        }
      })();
    }, ms);

    return () => window.clearInterval(timer);
  });

  const galleryChannelPreviews = $derived.by(() => {
    return buildQueueGalleryChannelPreviews({
      basePreviews: ($page.data.channelPreviews ?? {}) as Record<
        string,
        ChannelSnapshot
      >,
      selectedChannelId: sidebar.selectedChannelId,
      syncDepth: sidebar.syncDepth,
      videos: sidebar.videos,
      hasMore: sidebar.hasMore,
      offset: sidebar.offset,
    });
  });

  $effect(() =>
    createAiStatusPoller({
      onStatus: (status) => {
        aiStatus = status.status;
      },
    }),
  );

  let previousQueueChannelId = $state<string | null>(null);
  $effect(() => {
    const id = sidebar.selectedChannelId;
    if (
      previousQueueChannelId !== null &&
      id !== null &&
      previousQueueChannelId !== id
    ) {
      sidebar.setSelectedVideoId(null);
    }
    previousQueueChannelId = id;
  });

  function replaceQueueUrl(href: string) {
    if (!queueUrlSyncReady || typeof window === "undefined") return;
    const nextUrl = new URL(href, window.location.origin);
    if (
      nextUrl.pathname === window.location.pathname &&
      nextUrl.search === window.location.search
    ) {
      return;
    }
    replacePageState(
      `${nextUrl.pathname}${nextUrl.search}${nextUrl.hash}`,
      window.history.state,
    );
  }

  function persistQueueViewUrl() {
    if (
      !viewUrlHydrated ||
      !queueUrlSyncReady ||
      typeof window === "undefined"
    ) {
      return;
    }
    const nextHref = buildQueueViewHref({
      selectedChannelId: sidebar.selectedChannelId,
      selectedVideoId: sidebar.selectedVideoId,
      videoTypeFilter: sidebar.videoTypeFilter,
      acknowledgedFilter: sidebar.acknowledgedFilter,
    });
    replaceQueueUrl(nextHref);
  }

  $effect(() => {
    persistQueueViewUrl();
  });

  onMount(() => {
    const guideParam = new URL(window.location.href).searchParams.get("guide");
    if (guideParam !== null) {
      void goto(`/?guide=${guideParam}`, { replaceState: true });
      return;
    }

    restoreQueueState();
    workspaceStateHydrated = true;
    setTimeout(() => {
      queueUrlSyncReady = true;
      persistQueueViewUrl();
    }, 0);

    void (async () => {
      try {
        const selectedChannelIdAtMount = sidebar.selectedChannelId;

        const bootstrapResult = await resolveBootstrapOnMount({
          serverBootstrap: $page.data.bootstrap ?? null,
          selectedChannelId: selectedChannelIdAtMount,
          workspaceCacheScopeKey,
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
          void putCachedBootstrapMeta(
            {
              ai_available: bootstrapResult.aiAvailable,
              ai_status: bootstrapResult.aiStatus,
              search_status: bootstrapResult.searchStatus,
            },
            workspaceCacheScopeKey,
          );
        }

        if (
          bootstrapResult.snapshot &&
          selectedChannelIdAtMount &&
          bootstrapResult.snapshot.channel_id === selectedChannelIdAtMount
        ) {
          sidebar.setSyncDepth(bootstrapResult.snapshot.sync_depth);
          sidebar.setVideos(bootstrapResult.snapshot.videos);
          sidebar.setOffset(
            bootstrapResult.snapshot.next_offset ??
              bootstrapResult.snapshot.videos.length,
          );
          sidebar.setHasMore(bootstrapResult.snapshot.has_more);
          void putCachedViewSnapshot(
            buildQueueSnapshotCacheKey(selectedChannelIdAtMount),
            {
              channel_id: selectedChannelIdAtMount,
              channel_video_count: bootstrapResult.snapshot.channel_video_count,
              has_more: bootstrapResult.snapshot.has_more,
              next_offset: bootstrapResult.snapshot.next_offset,
              videos: bootstrapResult.snapshot.videos,
              sync_depth: bootstrapResult.snapshot.sync_depth,
            },
            workspaceCacheScopeKey,
          );
        }

        await sidebar.loadInitial({
          silent: hasInitialData,
        });
      } finally {
        viewUrlHydrated = true;
      }
    })();
  });

  function openGuide() {
    void goto("/?guide=0");
  }

  function restoreQueueState() {
    const restored = mergeQueueViewState(
      restoreWorkspaceSnapshot(
        typeof localStorage === "undefined"
          ? null
          : loadWorkspaceState(localStorage, workspaceStorageKey),
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
    if (typeof restored.selectedVideoId === "string") {
      sidebar.setSelectedVideoId(restored.selectedVideoId);
    }
    if (
      restored.videoTypeFilter &&
      isWorkspaceVideoTypeFilter(restored.videoTypeFilter)
    ) {
      sidebar.setVideoTypeFilter(restored.videoTypeFilter);
    }
    if (
      restored.acknowledgedFilter &&
      isAcknowledgedFilter(restored.acknowledgedFilter)
    ) {
      sidebar.setAcknowledgedFilter(restored.acknowledgedFilter);
    }
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
      void putCachedChannels(sidebar.channels, workspaceCacheScopeKey);

      // Reload videos with the new sync boundary
      sidebar.setVideos([]);
      sidebar.setOffset(0);
      sidebar.setHasMore(true);
      await sidebar.refreshAndLoadVideos(sidebar.selectedChannelId);
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
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
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    } finally {
      retryingTranscriptVideoId = null;
    }
  }

  async function openVideoTranscriptInWorkspace(video: Video) {
    if (typeof localStorage !== "undefined") {
      saveWorkspaceState(
        localStorage,
        {
          selectedChannelId: video.channel_id,
          selectedVideoId: video.id,
          contentMode: "info",
          videoTypeFilter: "all",
          acknowledgedFilter: "all",
        },
        workspaceStorageKey,
      );
    }

    await goto(
      buildWorkspaceViewHref({
        selectedChannelId: video.channel_id,
        selectedVideoId: video.id,
        contentMode: "info",
        videoTypeFilter: "all",
        acknowledgedFilter: "all",
      }),
    );
  }

  function openQueuedVideo(videoId: string) {
    if (!sidebar.selectedChannelId) return;
    sidebar.setSelectedVideoId(videoId);
  }

  async function handleSearchResultSelection(
    result: SearchResult,
    targetMode: "transcript" | "summary",
  ) {
    if (typeof localStorage !== "undefined") {
      saveWorkspaceState(
        localStorage,
        {
          selectedChannelId: result.channel_id,
          selectedVideoId: result.video_id,
          contentMode: targetMode,
          videoTypeFilter: "all",
          acknowledgedFilter: "all",
        },
        workspaceStorageKey,
      );
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

  const selectedQueueVideo = $derived(
    sidebar.selectedVideoId
      ? (sidebar.videos.find((v) => v.id === sidebar.selectedVideoId) ?? null)
      : null,
  );

  const queueContentPanelState = $derived({
    mobileVisible: true,
    selectedChannel: sidebar.selectedChannel,
    selectedChannelId: sidebar.selectedChannelId,
    selectedVideoId: sidebar.selectedVideoId,
    selectedQueueVideo,
    queueStats,
    failedTranscriptVideos,
    retryingTranscriptVideoId,
    effectiveEarliestSyncDate,
    earliestSyncDateInput,
    savingSyncDate,
    refreshingChannel: sidebar.refreshingChannel,
  });
  const queueContentPanelActions = {
    onBack: () => {},
    onSaveSyncDate: saveEarliestSyncDate,
    onRetryTranscript: retryTranscriptDownload,
    onClearSelectedVideo: () => sidebar.setSelectedVideoId(null),
    onOpenVideoInWorkspace: openVideoTranscriptInWorkspace,
  };

  const shellCollapsed = $derived(sidebar.sidebarCollapsed);
  const shellWidth = $derived(sidebar.sidebarWidth);
  const shellToggleSidebar = sidebar.toggleSidebar;
  const queueSidebar = sidebar;

  async function clearQueueBrowseVideoFilters() {
    const actions = queueSidebar.videoActions;
    if (actions.onClearAllFilters) {
      await actions.onClearAllFilters();
    } else {
      await actions.onVideoTypeFilterChange("all");
      await actions.onAcknowledgedFilterChange("all");
    }
  }

  // Use primitive sidebar fields, not `videoState`: the derived `videoState` object
  // changes on every video list update and would retrigger this effect constantly,
  // spamming `mobileBottomBar.set` and breaking taps on section nav (mobile).
  const queueBrowseFilterDisabled = $derived(
    !queueSidebar.selectedChannelId || queueSidebar.loadingVideos,
  );

  $effect(() => {
    if (sidebar.selectedChannelId) {
      mobileBottomBar.set({
        kind: "sectionsWithVideoFilter",
        filter: {
          videoTypeFilter: sidebar.videoTypeFilter,
          acknowledgedFilter: sidebar.acknowledgedFilter,
          disabled: queueBrowseFilterDisabled,
          onSelectVideoType: sidebar.videoActions.onVideoTypeFilterChange,
          onSelectAcknowledged: sidebar.videoActions.onAcknowledgedFilterChange,
          onClearAllFilters: clearQueueBrowseVideoFilters,
        },
      });
    } else {
      mobileBottomBar.set({ kind: "sections" });
    }
    return () => {
      mobileBottomBar.set({ kind: "sections" });
    };
  });
</script>

<WorkspaceShell currentSection="queue" {aiIndicator} onOpenGuide={openGuide}>
  {#snippet mobileTopBar()}
    <MobileYouTubeTopNav />
  {/snippet}
  {#snippet sidebar({
    collapsed: shellCollapsed,
    toggle: shellToggleSidebar,
    width: shellWidth,
  })}
    <WorkspaceSidebar
      videoListMode="per_channel_preview"
      previewSessionKey="download-queue-sidebar-navigation"
      addSourceErrorMessage={errorMessage}
      initialChannelPreviews={$page.data.channelPreviews ?? {}}
      initialChannelPreviewsFilterKey={$page.data.channelPreviewsFilterKey ??
        "all:all:unified"}
      previewScope={{ kind: "unified" }}
      {queueVideoRefreshTick}
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

  <div
    class="flex h-full min-h-0 flex-col lg:flex-row"
    aria-label="Download queue"
  >
    <div class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden lg:hidden">
      <MobileChannelGallery
        channels={queueSidebar.channels}
        selectedChannelId={queueSidebar.selectedChannelId}
        onSelectChannel={(channelId) => {
          void queueSidebar.selectChannel(channelId);
        }}
        channelPreviews={galleryChannelPreviews}
        queueUnifiedSummary={true}
      />
      <div
        class="min-h-0 flex-1 overflow-hidden border-t border-[var(--border-soft)]/50"
      >
        <WorkspaceSidebar
          videoListMode="selected_channel"
          addSourceErrorMessage={errorMessage}
          initialChannelPreviews={$page.data.channelPreviews ?? {}}
          initialChannelPreviewsFilterKey={$page.data
            .channelPreviewsFilterKey ?? "all:all:unified"}
          previewScope={{ kind: "unified" }}
          {queueVideoRefreshTick}
          readOnly={true}
          shell={{
            collapsed: false,
            width: undefined,
            mobileVisible: true,
            onToggleCollapse: () => {},
          }}
          channelState={{
            ...queueSidebar.channelState,
            channels: queueSidebar.channels,
            selectedChannelId: queueSidebar.selectedChannelId,
            canDeleteChannels: false,
          }}
          channelActions={queueSidebar.channelActions}
          videoState={queueSidebar.videoState}
          videoActions={queueSidebar.videoActions}
          hideChannelUi
        />
      </div>
    </div>

    <div
      class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden lg:min-w-0"
    >
      <QueueContentPanel
        hideMobileBack
        readOnly={true}
        state={queueContentPanelState}
        actions={queueContentPanelActions}
      />
    </div>
  </div>

  {#if errorMessage}
    <ErrorToast
      message={errorMessage}
      onDismiss={() => (errorMessage = null)}
    />
  {/if}
</WorkspaceShell>
