<script lang="ts">
  import { goto, preloadData } from "$app/navigation";
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import type { Component } from "svelte";
  import { authState } from "$lib/auth-state.svelte";
  import {
    getAuthStorageScopeKey,
    getScopedStorageKey,
  } from "$lib/auth-storage";
  import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
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
    getSearchStatus,
    openSearchStatusStream,
  } from "$lib/api";
  import AddSourceFeedbackToast from "$lib/components/AddSourceFeedbackToast.svelte";
  import ChannelOverviewMainContent from "$lib/components/channels/ChannelOverviewMainContent.svelte";
  import ChannelOverviewMobileDrawer from "$lib/components/channels/ChannelOverviewMobileDrawer.svelte";
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import SignInRequiredModal from "$lib/components/SignInRequiredModal.svelte";
  import ErrorToast from "$lib/components/ErrorToast.svelte";
  import MobileYouTubeTopNav from "$lib/components/mobile/MobileYouTubeTopNav.svelte";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import WorkspaceSidebar from "$lib/components/workspace/WorkspaceSidebar.svelte";
  import WorkspaceMinimalTopBar from "$lib/components/workspace/WorkspaceMinimalTopBar.svelte";
  import type { AddSourceSubmission } from "$lib/workspace/component-props";
  import {
    applySavedChannelOrder,
    finalizeAddedChannelOrder,
    loadWorkspaceState,
    restoreWorkspaceSnapshot,
    saveWorkspaceState,
  } from "$lib/channel-workspace";
  import type {
    AiStatus,
    Channel,
    ChannelSnapshot,
    SearchResult,
    SearchStatus,
    SyncDepth,
    Video,
    VideoTypeFilter,
    WorkspaceBootstrap,
  } from "$lib/types";
  import { looksLikeYouTubeVideoInput } from "$lib/utils/youtube-input";
  import { createAiStatusPoller } from "$lib/utils/ai-poller";
  import { buildWorkspaceViewHref } from "$lib/view-url";
  import { type AddSourceFeedback } from "$lib/workspace/add-source-feedback";
  import { channelOrderFromList } from "$lib/workspace/channels";
  import { createChannelOverviewAddSourceFeedbackController } from "$lib/workspace/channel-overview-add-source-feedback.svelte";
  import { resolveSyncDateInputValue } from "$lib/workspace/sidebar-sync-date";
  import type {
    AcknowledgedFilter,
    ChannelSortMode,
  } from "$lib/workspace/types";
  const initialBootstrap = (page.data.bootstrap ??
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
  let showDeleteConfirmation = $state(false);
  let showDeleteAccessPrompt = $state(false);
  let channelIdToDelete = $state<string | null>(null);
  let mobileChannelsDrawerOpen = $state(false);
  let workspaceStateHydrated = $state(false);
  let channelOrder = $state<string[]>([]);
  let channelSortMode = $state<ChannelSortMode>("custom");
  let videoTypeFilter = $state<VideoTypeFilter>("all");
  let acknowledgedFilter = $state<AcknowledgedFilter>("all");
  let aiStatus = $state<AiStatus | null>(null);
  let activeOverviewRequest = 0;
  let lastOverviewLoadKey = $state<string | null>(null);
  let seededChannelPreviews = $state<Record<string, ChannelSnapshot>>(
    (page.data.channelPreviews ?? {}) as Record<string, ChannelSnapshot>,
  );
  let seededChannelPreviewsFilterKey = $state<string>(
    (page.data.channelPreviewsFilterKey ?? "all:all:default") as string,
  );
  const addSourceFeedbackCtrl =
    createChannelOverviewAddSourceFeedbackController({
      refreshVideo: (videoId) => getVideo(videoId, true),
      loadChannelVideos: async (channelId) => {
        const page = await listVideos(
          channelId,
          1,
          0,
          "all",
          undefined,
          false,
          undefined,
          true,
        );
        return page.videos;
      },
      openTarget: async (feedback: AddSourceFeedback) => {
        if (feedback.kind === "video") {
          await goto(
            buildWorkspaceViewHref({
              selectedChannelId: feedback.targetChannelId,
              selectedVideoId: feedback.videoId,
              contentMode: "info",
              videoTypeFilter,
              acknowledgedFilter,
            }),
          );
          return;
        }
        await goto(`/channels/${encodeURIComponent(feedback.channelId)}`);
      },
    });
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let WorkspaceSearchBarComponent = $state<Component<any> | null>(null);
  let searchStatus = $state<SearchStatus | null>(null);
  let selectedChannelId = $derived(page.params.id ?? null);
  let selectedChannel = $derived(
    channels.find((item) => item.id === selectedChannelId) ?? null,
  );
  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  let canManageLibrary = $derived(
    authState.current.authState === "authenticated",
  );
  let workspaceStorageKey = $derived(
    getScopedStorageKey(
      "dastill.workspace.state.v1",
      getAuthStorageScopeKey(authState.current),
    ),
  );
  let missingChannelMessage = $derived.by(() => {
    if (loadingOverview) {
      return null;
    }

    if (channels.length === 0) {
      return "Follow a channel to start shaping your workspace.";
    }

    return selectedChannel ? null : "Channel not found.";
  });

  function syncInputValue(
    currentChannel: Channel | null = selectedChannel,
    currentSyncDepth: SyncDepth | null = syncDepth,
  ) {
    earliestSyncDateInput = resolveSyncDateInputValue(
      currentChannel,
      currentSyncDepth,
    );
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
    const depth = snapshot?.sync_depth ?? null;
    syncDepth = depth;
    syncInputValue(currentChannel, depth);
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

      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
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
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    } finally {
      savingSyncDate = false;
    }
  }

  async function handleAddChannel(input: AddSourceSubmission) {
    addingChannel = true;
    errorMessage = null;
    const submittedInput =
      typeof input === "string" ? input.trim() : input.input.trim();

    try {
      if (
        typeof input === "string" &&
        looksLikeYouTubeVideoInput(submittedInput)
      ) {
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
        void addSourceFeedbackCtrl.trackAddedVideo(result);
        return true;
      }

      const addedChannel = await addChannel(
        typeof input === "string"
          ? submittedInput
          : { input: submittedInput, openalex_query: input.openalex_query },
      );
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
      void addSourceFeedbackCtrl.trackAddedChannel(addedChannel);
      return true;
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
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
      saveWorkspaceState(
        localStorage,
        {
          selectedChannelId: channelId,
          selectedVideoId: videoId,
          contentMode: "info",
          videoTypeFilter,
          acknowledgedFilter,
          channelOrder,
          channelSortMode,
        },
        workspaceStorageKey,
      );
    }

    await preloadData(href);
    await goto(href, { keepFocus: true, noScroll: true });
  }

  function reorderChannels(nextOrder: string[]) {
    channelOrder = nextOrder;
    channels = applySavedChannelOrder(channels, nextOrder);
  }

  async function handleDeleteChannel(channelId: string) {
    if (!canManageLibrary) {
      showDeleteAccessPrompt = true;
      return;
    }

    channelIdToDelete = channelId;
    showDeleteConfirmation = true;
  }

  async function confirmDeleteChannel() {
    if (!channelIdToDelete || !canManageLibrary) {
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
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
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
    const redirectTo = `${page.url.pathname}${page.url.search}`;
    await goto(`/login?redirectTo=${encodeURIComponent(redirectTo)}`);
  }

  async function handleSearchResultSelection(
    result: SearchResult,
    mode: "transcript" | "summary",
  ) {
    await goto(
      buildWorkspaceViewHref({
        selectedChannelId: result.channel_id,
        selectedVideoId: result.video_id,
        contentMode: mode,
        videoTypeFilter: "all",
        acknowledgedFilter: "all",
      }),
    );
  }

  onMount(() => {
    if (typeof localStorage !== "undefined") {
      const restored = restoreWorkspaceSnapshot(
        loadWorkspaceState(localStorage, workspaceStorageKey),
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

    void getSearchStatus().then((status) => {
      searchStatus = status;
    });

    void import("$lib/components/workspace/WorkspaceSearchBar.svelte").then(
      (m) => {
        WorkspaceSearchBarComponent = m.default;
      },
    );

    return () => {
      addSourceFeedbackCtrl.dispose();
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

    saveWorkspaceState(
      localStorage,
      {
        selectedChannelId,
        videoTypeFilter,
        acknowledgedFilter,
        channelOrder,
        channelSortMode,
      },
      workspaceStorageKey,
    );
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
    canDeleteChannels: canManageLibrary,
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
          if (!presentAuthRequiredNoticeIfNeeded(error)) {
            errorMessage = (error as Error).message;
          }
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
  {#snippet topBar()}
    <WorkspaceMinimalTopBar
      title={selectedChannel ? selectedChannel.name : "Channel Overview"}
    >
      {#snippet trailing()}
        {#if WorkspaceSearchBarComponent}
          <WorkspaceSearchBarComponent
            initialSearchStatus={searchStatus}
            onSearchResultSelect={handleSearchResultSelection}
          />
        {/if}
      {/snippet}
    </WorkspaceMinimalTopBar>
  {/snippet}
  {#snippet sidebar({ collapsed, toggle, width })}
    <WorkspaceSidebar
      videoListMode="per_channel_preview"
      previewSessionKey="workspace-sidebar-navigation"
      initialChannelPreviews={seededChannelPreviews}
      initialChannelPreviewsFilterKey={seededChannelPreviewsFilterKey}
      previewScope={{ kind: "default" }}
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

  <ChannelOverviewMobileDrawer
    open={mobileChannelsDrawerOpen}
    {errorMessage}
    initialChannelPreviews={seededChannelPreviews}
    initialChannelPreviewsFilterKey={seededChannelPreviewsFilterKey}
    channelState={overviewSidebarChannelState}
    channelActions={overviewSidebarChannelActions}
    videoState={overviewSidebarVideoState}
    videoActions={overviewSidebarVideoActions}
    onClose={() => {
      mobileChannelsDrawerOpen = false;
    }}
  />

  <ChannelOverviewMainContent
    {selectedChannel}
    {loadingOverview}
    {missingChannelMessage}
    bind:earliestSyncDateInput
    {savingSyncDate}
    canDeleteChannel={Boolean(selectedChannel)}
    onSaveSyncDate={() => {
      void saveSyncDate();
    }}
    onDeleteChannel={() => {
      if (selectedChannelId) {
        void handleDeleteChannel(selectedChannelId);
      }
    }}
    onBack={() => {
      void goto("/");
    }}
    onOpenChannels={() => {
      mobileChannelsDrawerOpen = true;
    }}
  />

  {#if errorMessage}
    <ErrorToast
      message={errorMessage}
      onDismiss={() => (errorMessage = null)}
    />
  {/if}

  {#if addSourceFeedbackCtrl.feedback && !addSourceFeedbackCtrl.dismissed}
    <AddSourceFeedbackToast
      feedback={addSourceFeedbackCtrl.feedback}
      onDismiss={addSourceFeedbackCtrl.dismiss}
      onAction={addSourceFeedbackCtrl.openTarget}
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

  <SignInRequiredModal
    show={showDeleteAccessPrompt}
    message="Sign in to remove channels and manage your library."
    onConfirm={confirmDeleteAccessPrompt}
    onCancel={cancelDeleteAccessPrompt}
  />
</WorkspaceShell>
