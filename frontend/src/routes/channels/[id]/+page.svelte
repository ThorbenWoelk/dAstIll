<script lang="ts">
  import { goto, preloadData } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
  import {
    addChannel,
    addVideo,
    deleteChannel as deleteChannelRequest,
    getChannelSyncDepth,
    getWorkspaceBootstrap,
    getVideo,
    listVideos,
    listChannels,
    refreshChannel,
    updateChannel,
  } from "$lib/api";
  import AddSourceFeedbackToast from "$lib/components/AddSourceFeedbackToast.svelte";
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import ErrorToast from "$lib/components/ErrorToast.svelte";
  import MobileYouTubeTopNav from "$lib/components/mobile/MobileYouTubeTopNav.svelte";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import WorkspaceSidebar from "$lib/components/workspace/WorkspaceSidebar.svelte";
  import {
    applySavedChannelOrder,
    finalizeAddedChannelOrder,
    loadWorkspaceState,
    restoreWorkspaceSnapshot,
    saveWorkspaceState,
  } from "$lib/channel-workspace";
  import type {
    AiStatus,
    AddVideoResult,
    Channel,
    ChannelSnapshot,
    SyncDepth,
    Video,
    VideoTypeFilter,
    WorkspaceBootstrap,
  } from "$lib/types";
  import { looksLikeYouTubeVideoInput } from "$lib/utils/youtube-input";
  import { createAiStatusPoller } from "$lib/utils/ai-poller";
  import { formatShortDate } from "$lib/utils/date";
  import { buildWorkspaceViewHref } from "$lib/view-url";
  import {
    buildChannelAddFeedback,
    buildVideoAddFeedback,
    type AddSourceFeedback,
    resolveAddedChannelStatus,
    resolveAddedVideoStatus,
  } from "$lib/workspace/add-source-feedback";
  import { channelOrderFromList } from "$lib/workspace/channels";
  import { formatSyncDate } from "$lib/workspace/content";
  import { mobileBottomBar } from "$lib/mobile-navigation/mobileBottomBar";
  import type {
    AcknowledgedFilter,
    ChannelSortMode,
  } from "$lib/workspace/types";

  const initialBootstrap = ($page.data.bootstrap ??
    null) as WorkspaceBootstrap | null;
  const initialSelectedSnapshot = initialBootstrap?.snapshot ?? null;

  let channels = $state<Channel[]>(initialBootstrap?.channels ?? []);
  let syncDepth = $state<SyncDepth | null>(
    initialSelectedSnapshot?.sync_depth ?? null,
  );
  let earliestSyncDateInput = $state("");
  let loadingChannels = $state(initialBootstrap === null);
  let loadingOverview = $state(initialBootstrap === null);
  let addingChannel = $state(false);
  let savingSyncDate = $state(false);
  let errorMessage = $state<string | null>(null);
  let addSourceFeedback = $state<AddSourceFeedback | null>(null);
  let addSourceFeedbackDismissed = $state(false);
  let addSourceFeedbackPollSequence = 0;
  let showDeleteConfirmation = $state(false);
  let showDeleteAccessPrompt = $state(false);
  let channelIdToDelete = $state<string | null>(null);
  let mobileChannelsDrawerOpen = $state(false);
  let workspaceStateHydrated = $state(false);

  $effect(() => {
    if (mobileChannelsDrawerOpen) {
      mobileBottomBar.set({ kind: "sections" });
    } else {
      mobileBottomBar.set({ kind: "hidden" });
    }
    return () => {
      mobileBottomBar.set({ kind: "sections" });
    };
  });
  let channelOrder = $state<string[]>([]);
  let channelSortMode = $state<ChannelSortMode>("custom");
  let videoTypeFilter = $state<VideoTypeFilter>("all");
  let acknowledgedFilter = $state<AcknowledgedFilter>("all");
  let aiStatus = $state<AiStatus | null>(null);
  let activeOverviewRequest = 0;
  let lastOverviewLoadKey = $state<string | null>(null);
  let seededChannelPreviews = $state<Record<string, ChannelSnapshot>>(
    (($page.data.channelPreviews ?? {}) as Record<string, ChannelSnapshot>) ??
      {},
  );
  let seededChannelPreviewsFilterKey = $state<string>(
    (($page.data.channelPreviewsFilterKey ?? "all:all:default") as string) ??
      "all:all:default",
  );

  let selectedChannelId = $derived($page.params.id ?? null);
  let selectedChannel = $derived(
    channels.find((item) => item.id === selectedChannelId) ?? null,
  );
  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  let isOperator = $derived(Boolean($page.data.isOperator));
  let missingChannelMessage = $derived.by(() => {
    if (loadingOverview) {
      return null;
    }

    if (channels.length === 0) {
      return "Follow a channel to start shaping your workspace.";
    }

    return selectedChannel ? null : "Channel not found.";
  });

  function resolveEffectiveSyncDate(
    currentChannel: Channel | null,
    currentSyncDepth: SyncDepth | null,
  ) {
    if (!currentChannel) {
      return null;
    }

    return currentChannel.earliest_sync_date_user_set
      ? currentChannel.earliest_sync_date
      : (currentSyncDepth?.derived_earliest_ready_date ??
          currentChannel.earliest_sync_date ??
          null);
  }

  function syncInputValue(
    currentChannel: Channel | null = selectedChannel,
    currentSyncDepth: SyncDepth | null = syncDepth,
  ) {
    const effective = resolveEffectiveSyncDate(
      currentChannel,
      currentSyncDepth,
    );
    earliestSyncDateInput = effective
      ? new Date(effective).toISOString().split("T")[0]
      : "";
  }

  function applyChannelPreferences(nextChannels: Channel[]) {
    return applySavedChannelOrder(nextChannels, channelOrder);
  }

  function resolvePreviewFilterKey(
    currentVideoType: VideoTypeFilter,
    currentAcknowledgedFilter: AcknowledgedFilter,
  ) {
    return `${currentVideoType}:${currentAcknowledgedFilter}:default`;
  }

  function buildOverviewLoadKey(
    channelId: string | null,
    currentVideoType: VideoTypeFilter,
    currentAcknowledgedFilter: AcknowledgedFilter,
  ) {
    return `${channelId ?? "__none__"}:${currentVideoType}:${currentAcknowledgedFilter}`;
  }

  function applyBootstrapState(
    bootstrap: WorkspaceBootstrap,
    filterKey: string,
    options?: { replaceChannels?: boolean },
  ) {
    const replaceChannels = options?.replaceChannels ?? true;
    const nextChannels = applyChannelPreferences(bootstrap.channels);
    if (replaceChannels) {
      channels = nextChannels;
      if (channelOrder.length === 0) {
        channelOrder = channelOrderFromList(nextChannels);
      }
    }

    const snapshot = bootstrap.snapshot;
    if (
      snapshot &&
      bootstrap.selected_channel_id &&
      snapshot.channel_id === bootstrap.selected_channel_id
    ) {
      seededChannelPreviews = {
        [snapshot.channel_id]: snapshot,
      };
      seededChannelPreviewsFilterKey = filterKey;
    }

    if (!bootstrap.selected_channel_id) {
      syncDepth = null;
      earliestSyncDateInput = "";
      return;
    }

    const currentChannel =
      nextChannels.find((item) => item.id === bootstrap.selected_channel_id) ??
      null;
    syncDepth = snapshot?.sync_depth ?? null;
    syncInputValue(currentChannel, snapshot?.sync_depth ?? null);
  }

  function mergeUpdatedChannel(updatedChannel: Channel) {
    channels = channels.map((channel) =>
      channel.id === updatedChannel.id ? updatedChannel : channel,
    );
  }

  async function refreshSelectedChannelDepth(
    channelId: string,
    currentChannel: Channel | null = selectedChannel,
  ) {
    const nextSyncDepth = await getChannelSyncDepth(channelId);
    if (selectedChannelId !== channelId) {
      return;
    }

    syncDepth = nextSyncDepth;
    syncInputValue(currentChannel, nextSyncDepth);
  }

  async function loadChannelOverview(channelId: string | null) {
    const shouldReloadChannels = channels.length === 0;
    return loadChannelOverviewState(channelId, { shouldReloadChannels });
  }

  async function loadChannelOverviewState(
    channelId: string | null,
    options?: { shouldReloadChannels?: boolean },
  ) {
    const requestId = ++activeOverviewRequest;
    loadingOverview = true;
    if (options?.shouldReloadChannels ?? false) {
      loadingChannels = true;
    }
    errorMessage = null;

    try {
      let nextChannels = channels;
      if (options?.shouldReloadChannels ?? false) {
        const bootstrap = await getWorkspaceBootstrap({
          selectedChannelId: channelId,
          videoType: videoTypeFilter,
          acknowledged:
            acknowledgedFilter === "all"
              ? undefined
              : acknowledgedFilter === "ack",
        });
        if (requestId !== activeOverviewRequest) {
          return;
        }

        applyBootstrapState(
          bootstrap,
          resolvePreviewFilterKey(videoTypeFilter, acknowledgedFilter),
        );
        nextChannels = channels;
      }

      if (!channelId) {
        syncDepth = null;
        earliestSyncDateInput = "";
        return;
      }

      const currentChannel =
        nextChannels.find((item) => item.id === channelId) ?? null;

      if (!currentChannel) {
        syncDepth = null;
        earliestSyncDateInput = "";
        return;
      }

      if (
        options?.shouldReloadChannels &&
        nextChannels.some((item) => item.id === channelId)
      ) {
        return;
      }

      const nextSyncDepth = await getChannelSyncDepth(channelId);
      if (requestId !== activeOverviewRequest) {
        return;
      }

      syncDepth = nextSyncDepth;
      syncInputValue(currentChannel, nextSyncDepth);
    } catch (error) {
      if (requestId !== activeOverviewRequest) {
        return;
      }

      errorMessage = (error as Error).message;
      syncDepth = null;
      earliestSyncDateInput = "";
    } finally {
      if (requestId === activeOverviewRequest) {
        if (options?.shouldReloadChannels ?? false) {
          loadingChannels = false;
        }
        loadingOverview = false;
      }
    }
  }

  async function saveSyncDate() {
    if (!selectedChannelId || !earliestSyncDateInput || savingSyncDate) {
      return;
    }

    savingSyncDate = true;
    errorMessage = null;

    try {
      const updatedChannel = await updateChannel(selectedChannelId, {
        earliest_sync_date: new Date(earliestSyncDateInput).toISOString(),
        earliest_sync_date_user_set: true,
      });
      mergeUpdatedChannel(updatedChannel);
      await refreshChannel(selectedChannelId);
      await refreshSelectedChannelDepth(selectedChannelId, updatedChannel);
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      savingSyncDate = false;
    }
  }

  async function handleAddChannel(input: string) {
    addingChannel = true;
    errorMessage = null;
    const submittedInput = input.trim();

    try {
      if (looksLikeYouTubeVideoInput(submittedInput)) {
        const result = await addVideo(submittedInput);
        const nextChannels = applySavedChannelOrder(
          await listChannels(),
          channelOrder,
        );
        channels = nextChannels;
        if (channelOrder.length === 0) {
          channelOrder = channelOrderFromList(nextChannels);
        }
        mobileChannelsDrawerOpen = false;
        void trackAddedVideo(result);
        return true;
      }

      const addedChannel = await addChannel(submittedInput);
      const nextOrder = finalizeAddedChannelOrder(
        channelOrder,
        addedChannel.id,
      );
      const nextChannels = applySavedChannelOrder(
        await listChannels(),
        nextOrder,
      );
      channelOrder = nextOrder;
      channels = nextChannels;
      if (channelOrder.length === 0) {
        channelOrder = channelOrderFromList(nextChannels);
      }
      mobileChannelsDrawerOpen = false;
      void trackAddedChannel(addedChannel);
      return true;
    } catch (error) {
      errorMessage = (error as Error).message;
      return false;
    } finally {
      addingChannel = false;
    }
  }

  async function openChannelOverview(channelId: string) {
    mobileChannelsDrawerOpen = false;
    if (channelId === selectedChannelId) {
      return;
    }

    await goto(`/channels/${encodeURIComponent(channelId)}`);
  }

  async function openVideoInWorkspace(channelId: string, videoId: string) {
    const href = buildWorkspaceViewHref({
      selectedChannelId: channelId,
      selectedVideoId: videoId,
      contentMode: "info",
      videoTypeFilter,
      acknowledgedFilter,
    });

    if (typeof localStorage !== "undefined") {
      saveWorkspaceState(localStorage, {
        selectedChannelId: channelId,
        selectedVideoId: videoId,
        contentMode: "info",
        videoTypeFilter,
        acknowledgedFilter,
        channelOrder,
        channelSortMode,
      });
    }

    await preloadData(href);
    await goto(href, { keepFocus: true, noScroll: true });
  }

  function reorderChannels(nextOrder: string[]) {
    channelOrder = nextOrder;
    channels = applySavedChannelOrder(channels, nextOrder);
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
    if (!channelIdToDelete || !isOperator) {
      return;
    }

    const deletedChannelId = channelIdToDelete;
    showDeleteConfirmation = false;
    channelIdToDelete = null;
    errorMessage = null;

    try {
      await deleteChannelRequest(deletedChannelId);
      const remainingChannels = channels.filter(
        (channel) => channel.id !== deletedChannelId,
      );
      channels = remainingChannels;
      channelOrder = channelOrder.filter((id) => id !== deletedChannelId);

      if (selectedChannelId === deletedChannelId) {
        syncDepth = null;
        earliestSyncDateInput = "";
        const nextChannelId = remainingChannels[0]?.id ?? null;

        if (nextChannelId) {
          await goto(`/channels/${encodeURIComponent(nextChannelId)}`);
        } else {
          await goto("/");
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
      await goto(
        buildWorkspaceViewHref({
          selectedChannelId: current.targetChannelId,
          selectedVideoId: current.videoId,
          contentMode: "info",
          videoTypeFilter,
          acknowledgedFilter,
        }),
      );
      return;
    }

    await goto(`/channels/${encodeURIComponent(current.channelId)}`);
  }

  onMount(() => {
    if (typeof localStorage !== "undefined") {
      const restored = restoreWorkspaceSnapshot(
        loadWorkspaceState(localStorage),
        {
          includeVideoTypeFilter: true,
          includeAcknowledgedFilter: true,
          includeChannelSortMode: true,
        },
      );

      channelOrder = restored.channelOrder ?? [];
      channelSortMode = restored.channelSortMode ?? "custom";
      videoTypeFilter = restored.videoTypeFilter ?? "all";
      acknowledgedFilter = restored.acknowledgedFilter ?? "all";
    }

    if (initialBootstrap) {
      applyBootstrapState(initialBootstrap, seededChannelPreviewsFilterKey, {
        replaceChannels: true,
      });
    } else {
      loadingChannels = true;
      loadingOverview = true;
    }

    const hasSeededSelectedSnapshot =
      Boolean(initialBootstrap?.snapshot) &&
      initialBootstrap?.selected_channel_id === selectedChannelId;

    workspaceStateHydrated = true;
    if (!hasSeededSelectedSnapshot || channels.length === 0) {
      lastOverviewLoadKey = buildOverviewLoadKey(
        selectedChannelId,
        videoTypeFilter,
        acknowledgedFilter,
      );
      void loadChannelOverviewState(selectedChannelId, {
        shouldReloadChannels: channels.length === 0,
      });
    }

    return () => {
      addSourceFeedbackPollSequence += 1;
    };
  });

  $effect(() => {
    if (!workspaceStateHydrated) {
      return;
    }

    const nextLoadKey = buildOverviewLoadKey(
      selectedChannelId,
      videoTypeFilter,
      acknowledgedFilter,
    );
    if (nextLoadKey === lastOverviewLoadKey) {
      return;
    }

    lastOverviewLoadKey = nextLoadKey;
    void loadChannelOverviewState(selectedChannelId, {
      shouldReloadChannels: false,
    });
  });

  $effect(() => {
    if (!workspaceStateHydrated || typeof localStorage === "undefined") {
      return;
    }

    saveWorkspaceState(localStorage, {
      selectedChannelId,
      videoTypeFilter,
      acknowledgedFilter,
      channelOrder,
      channelSortMode,
    });
  });

  $effect(() =>
    createAiStatusPoller({
      onStatus: (status) => {
        aiStatus = status.status;
      },
    }),
  );

  const overviewSidebarChannelState = $derived({
    channels,
    selectedChannelId,
    loadingChannels,
    addingChannel,
    channelSortMode,
    canDeleteChannels: isOperator,
  });
  const overviewSidebarVideoState = $derived({
    videos: [] as Video[],
    selectedVideoId: null,
    selectedChannel,
    loadingVideos: false,
    refreshingChannel: false,
    hasMore: false,
    historyExhausted: false,
    backfillingHistory: false,
    videoTypeFilter,
    acknowledgedFilter,
    syncDepth,
    offset: 0,
    allowLoadedVideoSyncDepthOverride: false,
  });
  const overviewSidebarChannelActions = {
    onChannelSortModeChange: (nextValue: ChannelSortMode) => {
      channelSortMode = nextValue;
    },
    onAddChannel: handleAddChannel,
    onSelectChannel: openChannelOverview,
    onOpenChannelOverview: openChannelOverview,
    onDeleteChannel: handleDeleteChannel,
    onDeleteAccessRequired: () => {
      showDeleteAccessPrompt = true;
    },
    onReorderChannels: reorderChannels,
    onChannelUpdated: (updatedChannel: Channel) => {
      mergeUpdatedChannel(updatedChannel);
      if (updatedChannel.id === selectedChannelId) {
        void refreshSelectedChannelDepth(
          updatedChannel.id,
          updatedChannel,
        ).catch((error) => {
          errorMessage = (error as Error).message;
        });
      }
    },
  };
  const overviewSidebarVideoActions = {
    onSelectVideo: async () => {},
    onSelectChannelVideo: openVideoInWorkspace,
    onLoadMoreVideos: async () => {},
    onVideoTypeFilterChange: async (value: VideoTypeFilter) => {
      videoTypeFilter = value;
    },
    onAcknowledgedFilterChange: async (value: AcknowledgedFilter) => {
      acknowledgedFilter = value;
    },
  };
</script>

<WorkspaceShell currentSection="workspace" {aiIndicator}>
  {#snippet mobileTopBar()}
    <MobileYouTubeTopNav />
  {/snippet}
  {#snippet sidebar({ collapsed, toggle, width })}
    <WorkspaceSidebar
      videoListMode="per_channel_preview"
      previewSessionKey="workspace-sidebar-navigation"
      initialChannelPreviews={seededChannelPreviews}
      initialChannelPreviewsFilterKey={seededChannelPreviewsFilterKey}
      addSourceErrorMessage={errorMessage}
      shell={{
        collapsed,
        width,
        mobileVisible: false,
        onToggleCollapse: toggle,
      }}
      channelState={overviewSidebarChannelState}
      channelActions={overviewSidebarChannelActions}
      videoState={overviewSidebarVideoState}
      videoActions={overviewSidebarVideoActions}
    />
  {/snippet}

  {#if mobileChannelsDrawerOpen}
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
          mobileChannelsDrawerOpen = false;
        }}
        aria-label="Close sidebar"
      ></button>
      <div
        class="relative z-10 h-full w-[min(85vw,20rem)] overflow-hidden border-r border-[var(--accent-border-soft)] bg-[var(--surface-strong)] shadow-2xl"
      >
        <WorkspaceSidebar
          videoListMode="per_channel_preview"
          previewSessionKey="workspace-sidebar-navigation"
          initialChannelPreviews={seededChannelPreviews}
          initialChannelPreviewsFilterKey={seededChannelPreviewsFilterKey}
          addSourceErrorMessage={errorMessage}
          shell={{
            collapsed: false,
            width: undefined,
            mobileVisible: true,
            onToggleCollapse: () => {},
          }}
          channelState={overviewSidebarChannelState}
          channelActions={overviewSidebarChannelActions}
          videoState={overviewSidebarVideoState}
          videoActions={overviewSidebarVideoActions}
        />
      </div>
    </div>
  {/if}

  <section
    id="content-view"
    class="fade-in stagger-3 relative z-10 flex h-full min-h-0 min-w-0 flex-col overflow-hidden lg:gap-4 lg:px-8 lg:pt-6 lg:pb-6"
  >
    <div
      class="flex flex-wrap items-center justify-between gap-4 border-b border-[var(--accent-border-soft)] px-4 py-4 sm:px-6 lg:px-0"
    >
      <div class="flex min-w-0 items-center gap-2 sm:gap-4">
        <button
          type="button"
          class="inline-flex h-10 w-10 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-70 transition hover:bg-[var(--accent-wash)] hover:opacity-100 lg:hidden"
          aria-label="Back to workspace"
          onclick={() => void goto("/")}
        >
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path d="M15 18l-6-6 6-6" />
          </svg>
        </button>
        <button
          type="button"
          class="inline-flex h-10 w-10 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-70 transition hover:bg-[var(--accent-wash)] hover:opacity-100 lg:hidden"
          aria-label="Open channel list"
          onclick={() => {
            mobileChannelsDrawerOpen = true;
          }}
        >
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path d="M4 7h16" />
            <path d="M4 12h16" />
            <path d="M4 17h16" />
          </svg>
        </button>
        <div
          class="h-14 w-14 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]"
        >
          {#if selectedChannel}
            <img
              src={selectedChannel.thumbnail_url || defaultChannelIcon}
              alt={selectedChannel.name}
              class="h-full w-full object-cover"
              referrerpolicy="no-referrer"
            />
          {/if}
        </div>

        <div class="min-w-0">
          <p
            class="hidden text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-50 sm:block"
          >
            Workspace
          </p>
          <h1
            class="mt-1 font-serif text-[22px] font-bold tracking-tight text-[var(--foreground)] sm:mt-2 sm:text-[32px]"
          >
            {selectedChannel ? selectedChannel.name : "Channel overview"}
          </h1>
          <p
            class="mt-1 text-[13px] text-[var(--soft-foreground)] sm:mt-2 sm:text-[14px]"
          >
            {#if selectedChannel}
              {selectedChannel.handle ?? selectedChannel.id}
            {:else}
              Follow channels and tune sync boundaries from the shared app view.
            {/if}
          </p>
        </div>
      </div>

      {#if selectedChannel}
        <div class="text-left sm:text-right">
          <p
            class="text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-50"
          >
            Sync boundary
          </p>
          <p class="mt-2 text-[15px] font-semibold text-[var(--foreground)]">
            {formatSyncDate(
              resolveEffectiveSyncDate(selectedChannel, syncDepth),
            )}
          </p>
        </div>
      {/if}
    </div>

    <div
      class="custom-scrollbar mobile-bottom-stack-padding min-h-0 flex-1 overflow-y-auto px-4 py-4 sm:px-6 lg:px-0 lg:pr-4 lg:pb-0"
    >
      {#if loadingOverview}
        <div
          class="grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(18rem,0.8fr)]"
        >
          <div class="space-y-4">
            {#each Array.from({ length: 2 }) as _, index (index)}
              <div
                class="animate-pulse rounded-[var(--radius-lg)] bg-[var(--panel-surface)] p-5 shadow-sm"
              >
                <div
                  class="h-4 w-28 rounded-full bg-[var(--border)] opacity-60"
                ></div>
                <div
                  class="mt-4 h-10 w-3/4 rounded-full bg-[var(--border)] opacity-35"
                ></div>
                <div
                  class="mt-3 h-3 w-full rounded-full bg-[var(--border)] opacity-25"
                ></div>
                <div
                  class="mt-2 h-3 w-2/3 rounded-full bg-[var(--border)] opacity-20"
                ></div>
              </div>
            {/each}
          </div>
          <div class="space-y-4">
            <div
              class="animate-pulse rounded-[var(--radius-lg)] bg-[var(--surface)] p-5 shadow-sm"
            >
              <div
                class="h-4 w-24 rounded-full bg-[var(--border)] opacity-60"
              ></div>
              <div
                class="mt-4 h-3 w-1/2 rounded-full bg-[var(--border)] opacity-25"
              ></div>
              <div
                class="mt-2 h-3 w-2/3 rounded-full bg-[var(--border)] opacity-20"
              ></div>
            </div>
          </div>
        </div>
      {:else if missingChannelMessage}
        <div
          class="rounded-[var(--radius-lg)] bg-[var(--panel-surface)] p-6 shadow-sm"
        >
          <p
            class="text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-50"
          >
            Channel overview
          </p>
          <p class="mt-3 text-[16px] font-semibold text-[var(--foreground)]">
            {missingChannelMessage}
          </p>
        </div>
      {:else if selectedChannel}
        <div
          class="grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(18rem,0.8fr)]"
        >
          <div class="space-y-4">
            <section
              class="rounded-[var(--radius-lg)] bg-[var(--panel-surface)] p-5 shadow-sm sm:p-6"
            >
              <p
                class="text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
              >
                Current sync boundary
              </p>
              <p
                class="mt-3 text-[22px] font-semibold tracking-tight text-[var(--foreground)]"
              >
                {formatSyncDate(
                  resolveEffectiveSyncDate(selectedChannel, syncDepth),
                )}
              </p>
              <p
                class="mt-3 max-w-2xl text-[14px] leading-6 text-[var(--soft-foreground)]"
              >
                Control how far back this channel should sync inside the shared
                workspace. Newer videos stay surfaced automatically once
                transcripts are ready.
              </p>
            </section>

            <section
              class="rounded-[var(--radius-lg)] bg-[var(--surface)] p-5 shadow-sm sm:p-6"
            >
              <p
                class="text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
              >
                Adjust boundary
              </p>
              <div class="mt-4 flex flex-col gap-3 sm:flex-row sm:items-center">
                <input
                  type="date"
                  class="min-w-0 flex-1 rounded-full border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-4 py-2 text-[14px] font-medium transition-colors focus:border-[var(--accent)]/40 focus:outline-none"
                  bind:value={earliestSyncDateInput}
                  disabled={savingSyncDate}
                />
                <button
                  type="button"
                  class="inline-flex items-center justify-center rounded-full bg-[var(--foreground)] px-4 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--background)] transition-all hover:bg-[var(--accent-strong)] disabled:opacity-40"
                  onclick={() => void saveSyncDate()}
                  disabled={!earliestSyncDateInput || savingSyncDate}
                >
                  {savingSyncDate ? "Saving" : "Save"}
                </button>
              </div>
            </section>
          </div>

          <aside class="space-y-4">
            <section
              class="rounded-[var(--radius-lg)] bg-[var(--surface)] p-5 shadow-sm"
            >
              <p
                class="text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
              >
                Details
              </p>

              <dl class="mt-4 space-y-4 text-[14px]">
                <div>
                  <dt class="text-[var(--soft-foreground)] opacity-70">
                    Handle
                  </dt>
                  <dd class="mt-1 font-medium text-[var(--foreground)]">
                    {selectedChannel.handle ?? "Not provided"}
                  </dd>
                </div>

                <div>
                  <dt class="text-[var(--soft-foreground)] opacity-70">
                    Channel ID
                  </dt>
                  <dd
                    class="mt-1 break-all font-medium text-[var(--foreground)]"
                  >
                    {selectedChannel.id}
                  </dd>
                </div>

                <div>
                  <dt class="text-[var(--soft-foreground)] opacity-70">
                    Added
                  </dt>
                  <dd class="mt-1 font-medium text-[var(--foreground)]">
                    {formatShortDate(selectedChannel.added_at)}
                  </dd>
                </div>

                <div>
                  <dt class="text-[var(--soft-foreground)] opacity-70">
                    Boundary source
                  </dt>
                  <dd class="mt-1 font-medium text-[var(--foreground)]">
                    {selectedChannel.earliest_sync_date_user_set
                      ? "Manual override"
                      : "Derived from ready transcripts"}
                  </dd>
                </div>
              </dl>
            </section>
          </aside>
        </div>
      {/if}
    </div>
  </section>

  {#if errorMessage}
    <ErrorToast
      message={errorMessage}
      onDismiss={() => (errorMessage = null)}
    />
  {/if}

  {#if addSourceFeedback && !addSourceFeedbackDismissed}
    <AddSourceFeedbackToast
      feedback={addSourceFeedback}
      onDismiss={dismissAddSourceFeedback}
      onAction={openAddSourceFeedbackTarget}
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

  <ConfirmationModal
    show={showDeleteAccessPrompt}
    title="Admin sign-in required"
    message="Deleting channels is restricted to admins. Sign in to unlock channel management."
    confirmLabel="Sign in"
    cancelLabel="Not now"
    tone="info"
    onConfirm={confirmDeleteAccessPrompt}
    onCancel={cancelDeleteAccessPrompt}
  />
</WorkspaceShell>
