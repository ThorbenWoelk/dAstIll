<script lang="ts">
  import { tick } from "svelte";
  import { slide } from "svelte/transition";
  import { getTranscript, getVideo, updateChannel } from "$lib/api";
  import {
    beginChannelDrag,
    completeChannelDrop,
    finishChannelDrag,
    moveChannelToIndex,
    reorderChannels as reorderChannelList,
    updateChannelDragOver,
  } from "$lib/channel-workspace";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import WorkspaceSidebarChannelRow from "$lib/components/workspace/WorkspaceSidebarChannelRow.svelte";
  import WorkspaceSidebarChannelControls from "$lib/components/workspace/WorkspaceSidebarChannelControls.svelte";
  import WorkspaceSidebarVideoFilterControl from "$lib/components/workspace/WorkspaceSidebarVideoFilterControl.svelte";
  import WorkspaceSidebarVideoRow from "$lib/components/workspace/WorkspaceSidebarVideoRow.svelte";
  import WorkspaceSidebarSelectedVideoList from "$lib/components/workspace/WorkspaceSidebarSelectedVideoList.svelte";
  import WorkspaceSidebarSyncDateControl from "$lib/components/workspace/WorkspaceSidebarSyncDateControl.svelte";
  import type { Channel, Video, VideoTypeFilter } from "$lib/types";
  import { OTHERS_CHANNEL_ID } from "$lib/types";
  import type {
    WorkspaceSidebarChannelActions,
    WorkspaceSidebarChannelState,
    WorkspaceSidebarShellProps,
    WorkspaceSidebarVideoActions,
    WorkspaceSidebarVideoState,
  } from "$lib/workspace/component-props";
  import type {
    AcknowledgedFilter,
    ChannelSortMode,
  } from "$lib/workspace/types";
  import {
    canManualReorderChannels,
    channelOrderFromList,
    cycleChannelSortMode,
    filterChannels,
    resolveChannelDropIndicatorEdge,
  } from "$lib/workspace/channels";
  import { formatShortDate } from "$lib/utils/date";
  import { resolveDisplayedSyncDepthIso } from "$lib/sync-depth";
  import { formatSyncDate } from "$lib/workspace/content";
  import {
    resolveSyncDateInputValue,
    toIsoDateStart,
  } from "$lib/workspace/sidebar-sync-date";
  import {
    createSidebarPreviewController,
    createEmptyChannelVideoCollection,
  } from "$lib/workspace/sidebar-preview-controller.svelte";
  import type { ChannelSnapshot, QueueTab, SyncDepth } from "$lib/types";

  let {
    shell = {
      collapsed: false,
      width: undefined,
      mobileVisible: false,
      onToggleCollapse: () => {},
    },
    channelState = {
      channels: [],
      selectedChannelId: null,
      loadingChannels: false,
      addingChannel: false,
      channelSortMode: "custom",
      canDeleteChannels: false,
    },
    channelActions = {
      onChannelSortModeChange: (_next: ChannelSortMode) => {},
      onAddChannel: async (_input: string) => false,
      onSelectChannel: async (_channelId: string) => {},
      onDeleteChannel: async (_channelId: string) => {},
      onDeleteAccessRequired: () => {},
      onReorderChannels: (_nextOrder: string[]) => {},
    },
    videoState = {
      videos: [],
      pendingSelectedVideo: null,
      selectedVideoId: null,
      selectedChannel: null,
      loadingVideos: false,
      refreshingChannel: false,
      hasMore: true,
      historyExhausted: false,
      backfillingHistory: false,
      videoTypeFilter: "all",
      acknowledgedFilter: "all",
      syncDepth: null,
      offset: 0,
      allowLoadedVideoSyncDepthOverride: false,
    },
    videoActions = {
      onSelectVideo: async (_videoId: string) => {},
      onSelectChannelVideo: async (_channelId: string, _videoId: string) => {},
      onLoadMoreVideos: async () => {},
      onVideoTypeFilterChange: async (_value: VideoTypeFilter) => {},
      onAcknowledgedFilterChange: async (_value: AcknowledgedFilter) => {},
    },
    videoListMode = "selected_channel",
    hideChannelUi = false,
    /** When true with `hideChannelUi`, hides Load More / Load History (parent loads everything). */
    suppressVideoLoadMoreButton = false,
    readOnly = false,
    addSourceErrorMessage = null as string | null,
    initialChannelPreviews = {} as Record<string, ChannelSnapshot>,
    initialChannelPreviewsFilterKey = undefined as string | undefined,
    /** When set (download queue), snapshot and list APIs use queue_tab so in-progress work appears. */
    channelSnapshotQueueTab = undefined as QueueTab | undefined,
    /**
     * When true (download queue, unified pipeline view), snapshot/list use
     * `queue_only` without `queue_tab` (any incomplete transcript or summary).
     */
    channelQueueSnapshotUnified = false,
    /**
     * When bumped (download queue), forces reload of expanded per-channel queue lists.
     */
    queueVideoRefreshTick = 0,
    /**
     * When `videoListMode` is `per_channel_preview`, the visible list lives in
     * `channelVideoCollections`, not `videoState.videos`. Parent bumps `seq` after
     * acknowledge toggles so this component can merge the server (or optimistic)
     * video and re-apply type + read filters.
     */
    videoAcknowledgeSync = null,
    /** After earliest-sync date is saved (standalone or preview collection). */
    onChannelSyncDateSaved = undefined,
    /** Shared session key for preserving per-channel preview state across routes. */
    previewSessionKey = undefined as string | undefined,
  }: {
    shell?: WorkspaceSidebarShellProps;
    channelState?: WorkspaceSidebarChannelState;
    channelActions?: WorkspaceSidebarChannelActions;
    videoState?: WorkspaceSidebarVideoState;
    videoActions?: WorkspaceSidebarVideoActions;
    videoListMode?: "selected_channel" | "per_channel_preview";
    hideChannelUi?: boolean;
    suppressVideoLoadMoreButton?: boolean;
    readOnly?: boolean;
    onChannelSyncDateSaved?: (channelId: string) => void | Promise<void>;
    addSourceErrorMessage?: string | null;
    /**
     * Server-side pre-loaded channel snapshots (keyed by channel id) for the
     * per_channel_preview mode. When provided and the current filter key matches
     * `initialChannelPreviewsFilterKey`, the sidebar uses this data directly
     * instead of making client-side getChannelSnapshot API calls on initial
     * render (VAL-DATA-002).
     */
    initialChannelPreviews?: Record<string, ChannelSnapshot>;
    /**
     * The filter key (`"${videoType}:${acknowledgedFilter}:${queueSegment}"`,
     * queueSegment `default` or the active queue tab) used when the server
     * fetched the channel preview snapshots.
     */
    initialChannelPreviewsFilterKey?: string;
    channelSnapshotQueueTab?: QueueTab;
    channelQueueSnapshotUnified?: boolean;
    queueVideoRefreshTick?: number;
    videoAcknowledgeSync?: {
      seq: number;
      video: Video;
      confirmed: boolean;
    } | null;
    previewSessionKey?: string;
  } = $props();

  let collapsed = $derived(shell.collapsed);
  let width = $derived(shell.width);
  let onToggleCollapse = $derived(shell.onToggleCollapse);
  let mobileVisible = $derived(shell.mobileVisible);
  /** Always above trigger: dialog appears above the button so it's visible without scrolling. */
  let syncDatePopupStackClass = "flex flex-col-reverse gap-2";

  function scrollIntoViewWhenSelected(node: HTMLElement, selected: boolean) {
    let wasSelected = selected;
    if (selected) {
      void tick().then(() =>
        node.scrollIntoView({ behavior: "smooth", block: "nearest" }),
      );
    }
    return {
      update(nextSelected: boolean) {
        if (nextSelected && !wasSelected) {
          void tick().then(() =>
            node.scrollIntoView({ behavior: "smooth", block: "nearest" }),
          );
        }
        wasSelected = nextSelected;
      },
    };
  }

  let channels = $derived(channelState.channels);
  let selectedChannelId = $derived(channelState.selectedChannelId);
  let channelUiHidden = $derived(hideChannelUi);
  let suppressLoadMoreButton = $derived(suppressVideoLoadMoreButton);
  let loadingChannels = $derived(channelState.loadingChannels);
  let addingChannel = $derived(channelState.addingChannel);
  let channelSortMode = $derived(channelState.channelSortMode);
  let canDeleteChannels = $derived(channelState.canDeleteChannels ?? false);

  let onChannelSortModeChange = $derived(
    channelActions.onChannelSortModeChange,
  );
  let onAddChannel = $derived(channelActions.onAddChannel);
  let onSelectChannel = $derived(channelActions.onSelectChannel);
  let onOpenChannelOverview = $derived(channelActions.onOpenChannelOverview);
  let onDeleteChannel = $derived(channelActions.onDeleteChannel);
  let onDeleteAccessRequired = $derived(
    channelActions.onDeleteAccessRequired ?? (() => {}),
  );
  let onReorderChannels = $derived(channelActions.onReorderChannels);
  let onChannelUpdated = $derived(
    channelActions.onChannelUpdated ?? (() => {}),
  );

  let videos = $derived(videoState.videos);
  let pendingSelectedVideo = $derived(videoState.pendingSelectedVideo ?? null);
  let selectedVideoId = $derived(videoState.selectedVideoId);
  let selectedChannel = $derived(videoState.selectedChannel);
  let loadingVideos = $derived(videoState.loadingVideos);
  let refreshingChannel = $derived(videoState.refreshingChannel);
  let hasMore = $derived(videoState.hasMore);
  let historyExhausted = $derived(videoState.historyExhausted);
  let backfillingHistory = $derived(videoState.backfillingHistory);
  let videoTypeFilter = $derived(videoState.videoTypeFilter);
  let acknowledgedFilter = $derived(videoState.acknowledgedFilter);
  let syncDepth = $derived(videoState.syncDepth);
  let allowLoadedVideoSyncDepthOverride = $derived(
    videoState.allowLoadedVideoSyncDepthOverride,
  );

  let onSelectVideo = $derived(videoActions.onSelectVideo);
  let onSelectChannelVideo = $derived(videoActions.onSelectChannelVideo);
  let onLoadMoreVideos = $derived(videoActions.onLoadMoreVideos);
  let onVideoTypeFilterChange = $derived(videoActions.onVideoTypeFilterChange);
  let onAcknowledgedFilterChange = $derived(
    videoActions.onAcknowledgedFilterChange,
  );
  let onClearAllFilters = $derived(videoActions.onClearAllFilters);

  let draggedChannelId = $state<string | null>(null);
  let dragOverChannelId = $state<string | null>(null);
  let channelSearchQuery = $state("");
  let channelSearchOpen = $state(false);
  let channelInputOpen = $state(false);

  let channelInput = $state("");
  let channelInputElement = $state<HTMLInputElement | null>(null);
  let reorderAnnouncement = $state("");

  let syncDatePickerChannelId = $state<string | null>(null);
  /** Input when editing sync date without a per-channel collection row. */
  let earliestSyncDateInputSelected = $state("");
  let savingStandaloneSyncDate = $state(false);

  let filteredChannels = $derived(
    filterChannels(channels, channelSearchQuery, channelSortMode),
  );
  let renderChannels = $derived.by((): Channel[] => {
    if (!channelUiHidden) return filteredChannels;
    if (!selectedChannelId) return [];
    const selected = channels.find(
      (channel) => channel.id === selectedChannelId,
    );
    return selected ? [selected] : [];
  });
  let visibleChannelIds = $derived(channelOrderFromList(filteredChannels));
  let manualReorderEnabled = $derived(
    canManualReorderChannels(channelSortMode, channelSearchQuery),
  );
  let activeFilterLabel = $derived.by(() => {
    const labels: string[] = [];
    if (videoTypeFilter !== "all")
      labels.push(videoTypeFilter === "long" ? "Full videos" : "Shorts");
    if (acknowledgedFilter !== "all")
      labels.push(acknowledgedFilter === "ack" ? "Read" : "Unread");
    return labels.join(" · ");
  });
  let hasActiveVideoFilters = $derived(
    videoTypeFilter !== "all" || acknowledgedFilter !== "all",
  );
  let showPendingSelectedVideo = $derived(
    Boolean(
      pendingSelectedVideo &&
      selectedVideoId === pendingSelectedVideo.id &&
      !videos.some((video) => video.id === pendingSelectedVideo.id),
    ),
  );

  const previewController = createSidebarPreviewController({
    getEnabled: () => videoListMode === "per_channel_preview",
    getChannels: () => channels,
    getFilteredChannels: () => filteredChannels,
    getSelectedChannelId: () => selectedChannelId,
    getSelectedChannel: () => selectedChannel,
    getSelectedVideoId: () => selectedVideoId,
    getVideoTypeFilter: () => videoTypeFilter,
    getAcknowledgedFilter: () => acknowledgedFilter,
    getHasActiveVideoFilters: () => hasActiveVideoFilters,
    getReadOnly: () => readOnly,
    getInitialChannelPreviews: () => initialChannelPreviews,
    getInitialChannelPreviewsFilterKey: () => initialChannelPreviewsFilterKey,
    getChannelSnapshotQueueTab: () => channelSnapshotQueueTab,
    getChannelQueueSnapshotUnified: () => channelQueueSnapshotUnified,
    getQueueVideoRefreshTick: () => queueVideoRefreshTick,
    getVideoAcknowledgeSync: () => videoAcknowledgeSync,
    getPreviewSessionKey: () => previewSessionKey,
    onChannelUpdated: (channel) => onChannelUpdated(channel),
    onChannelSyncDateSaved: (channelId) => onChannelSyncDateSaved?.(channelId),
  });
  let channelVideoCollections = $derived(
    previewController.channelVideoCollections,
  );
  let previewSyncDatePickerChannelId = $derived(
    previewController.syncDatePickerChannelId,
  );

  function isVirtualChannel(channel: Channel) {
    return channel.id === OTHERS_CHANNEL_ID;
  }

  async function handlePerChannelPreviewSelect(channel: Channel) {
    if (isVirtualChannel(channel)) {
      return;
    }
    await previewController.toggleChannelVideoCollection(channel);
    if (onOpenChannelOverview) {
      void onOpenChannelOverview(channel.id);
    }
  }

  async function handleChannelVideoClick(
    channelId: string,
    videoId: string,
    video?: Video,
  ) {
    if (collapsed) onToggleCollapse();
    if (onSelectChannelVideo) {
      await onSelectChannelVideo(channelId, videoId, video);
      return;
    }

    await onSelectVideo(videoId);
  }

  async function saveStandaloneChannelSyncDate(channel: Channel) {
    if (!earliestSyncDateInputSelected.trim() || savingStandaloneSyncDate) {
      return;
    }

    savingStandaloneSyncDate = true;
    try {
      const updatedChannel = await updateChannel(channel.id, {
        earliest_sync_date: toIsoDateStart(earliestSyncDateInputSelected),
        earliest_sync_date_user_set: true,
      });
      onChannelUpdated(updatedChannel);
      syncDatePickerChannelId = null;
      await onChannelSyncDateSaved?.(channel.id);
    } finally {
      savingStandaloneSyncDate = false;
    }
  }

  function toggleStandaloneSyncDatePicker(
    channel: Channel,
    depth: SyncDepth | null,
  ) {
    if (syncDatePickerChannelId === channel.id) {
      syncDatePickerChannelId = null;
      return;
    }
    syncDatePickerChannelId = channel.id;
    earliestSyncDateInputSelected = resolveSyncDateInputValue(channel, depth);
  }

  $effect(() => {
    if (readOnly) {
      channelInputOpen = false;
    }
  });

  async function handleChannelSubmit(event: SubmitEvent) {
    event.preventDefault();
    const submittedInput = channelInput.trim();
    if (!submittedInput || addingChannel) return;
    channelInput = "";
    const success = await onAddChannel(submittedInput);
    if (!success) {
      channelInput = submittedInput;
      return;
    }

    channelInputOpen = false;
  }

  async function toggleChannelInput() {
    channelInputOpen = !channelInputOpen;

    if (!channelInputOpen) {
      channelInput = "";
      return;
    }

    channelSearchOpen = false;
    channelSearchQuery = "";
    await tick();
    channelInputElement?.focus();
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
      const reordered = reorderChannelList(channels, sourceId, channelId);
      if (reordered) onReorderChannels(reordered.channelOrder);
    }
    draggedChannelId = dragState.draggedChannelId;
    dragOverChannelId = dragState.dragOverChannelId;
  }

  function handleChannelListDragOver(event: DragEvent) {
    event.preventDefault();
    const lastVisibleChannelId = filteredChannels.at(-1)?.id ?? null;
    if (!draggedChannelId || !lastVisibleChannelId) {
      return;
    }

    dragOverChannelId = lastVisibleChannelId;
  }

  function handleChannelListDrop(event: DragEvent) {
    event.preventDefault();
    const lastVisibleChannelId = filteredChannels.at(-1)?.id ?? null;
    const lastChannelIndex = channels.length - 1;
    const { sourceId, dragState } = completeChannelDrop(
      lastVisibleChannelId,
      draggedChannelId,
      event.dataTransfer?.getData("text/plain") || null,
    );
    if (sourceId && lastVisibleChannelId && lastChannelIndex >= 0) {
      const reordered = moveChannelToIndex(
        channels,
        sourceId,
        lastChannelIndex,
      );
      if (reordered) onReorderChannels(reordered.channelOrder);
    }
    draggedChannelId = dragState.draggedChannelId;
    dragOverChannelId = dragState.dragOverChannelId;
  }

  function handleChannelDragEnd() {
    const dragState = finishChannelDrag();
    draggedChannelId = dragState.draggedChannelId;
    dragOverChannelId = dragState.dragOverChannelId;
  }

  async function clearAllFilters() {
    if (onClearAllFilters) {
      await onClearAllFilters();
    } else {
      await onVideoTypeFilterChange("all");
      await onAcknowledgedFilterChange("all");
    }
  }

  function handleChannelClick(channelId: string) {
    if (collapsed) onToggleCollapse();
    if (channelId === OTHERS_CHANNEL_ID) {
      return;
    }
    void onSelectChannel(channelId);
  }

  let hoverPrefetchTimer: ReturnType<typeof setTimeout> | null = null;
  function handleVideoMouseEnter(videoId: string) {
    if (hoverPrefetchTimer) clearTimeout(hoverPrefetchTimer);
    hoverPrefetchTimer = setTimeout(() => {
      void getTranscript(videoId).catch(() => {});
      void getVideo(videoId).catch(() => {});
    }, 200);
  }
  function handleVideoMouseLeave() {
    if (hoverPrefetchTimer) clearTimeout(hoverPrefetchTimer);
  }
</script>

<aside
  id="workspace"
  class={`flex flex-col bg-[var(--surface)] ${mobileVisible ? "h-full w-full min-w-0" : "hidden lg:flex"} lg:h-full lg:shrink-0`}
  style={mobileVisible
    ? undefined
    : `width: ${width ?? (collapsed ? 52 : 280)}px;`}
>
  {#if channelUiHidden}
    <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div
        class="custom-scrollbar min-h-0 flex-1 overflow-y-auto px-3 pb-4 pt-3"
        aria-busy={loadingVideos}
      >
        {#if !selectedChannelId}
          <p
            class="px-2 py-2 text-[14px] font-medium italic text-[var(--soft-foreground)] opacity-55"
          >
            Pick a channel above.
          </p>
        {:else if loadingVideos && videos.length === 0}
          <div class="space-y-1 px-1" role="status" aria-live="polite">
            {#each Array.from({ length: 6 }) as _, i (i)}
              <div class="animate-pulse px-2 py-1.5">
                <div
                  class="h-3 w-11/12 rounded-full bg-[var(--border)] opacity-60"
                ></div>
                <div
                  class="mt-1 h-2 w-1/3 rounded-full bg-[var(--border)] opacity-40"
                ></div>
              </div>
            {/each}
          </div>
        {:else if videos.length === 0 && !refreshingChannel}
          <p
            class="px-2 py-2 text-[14px] font-medium italic text-[var(--soft-foreground)] opacity-55"
          >
            No videos yet.
          </p>
        {:else}
          <WorkspaceSidebarSelectedVideoList
            {videos}
            {selectedVideoId}
            {pendingSelectedVideo}
            {showPendingSelectedVideo}
            {loadingVideos}
            {refreshingChannel}
            {hasMore}
            {historyExhausted}
            {backfillingHistory}
            {suppressLoadMoreButton}
            emptyLabel="No videos yet."
            wrapperClass=""
            rowClassName="min-h-[56px]"
            onSelectVideo={(videoId) => void onSelectVideo(videoId)}
            onLoadMoreVideos={() => void onLoadMoreVideos()}
            onVideoMouseEnter={handleVideoMouseEnter}
            onVideoMouseLeave={handleVideoMouseLeave}
          >
            {#snippet footer()}
              {#if selectedChannel}
                <WorkspaceSidebarSyncDateControl
                  readOnly={readOnly || isVirtualChannel(selectedChannel)}
                  open={syncDatePickerChannelId === selectedChannel.id}
                  label={formatSyncDate(
                    resolveDisplayedSyncDepthIso({
                      videos,
                      selectedChannel,
                      syncDepth,
                      allowLoadedVideoOverride:
                        allowLoadedVideoSyncDepthOverride,
                    }),
                  )}
                  inputValue={earliestSyncDateInputSelected}
                  saving={savingStandaloneSyncDate}
                  popupStackClass={syncDatePopupStackClass}
                  wrapperClass="relative z-10 mt-2 px-2"
                  buttonClass="touch-manipulation relative z-10 inline-flex w-full max-w-full flex-wrap items-baseline gap-x-1 rounded-[var(--radius-sm)] px-2 py-1 text-left text-[10px] text-[var(--soft-foreground)] opacity-55 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
                  readonlyClass="mt-2 px-2 text-[10px] text-[var(--soft-foreground)] opacity-55"
                  dialogClass="flex flex-wrap items-center gap-2 rounded-[var(--radius-md)] bg-[var(--surface-strong)] p-2 shadow-[var(--shadow-soft)]"
                  inputClass="min-w-0 flex-1 rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-3 py-2 text-[12px] font-medium transition-colors focus:border-[var(--accent)]/40 focus:outline-none"
                  submitClass="rounded-[var(--radius-sm)] bg-[var(--foreground)] px-3 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--background)] transition-all hover:bg-[var(--accent-strong)] disabled:opacity-30"
                  onToggle={() =>
                    toggleStandaloneSyncDatePicker(selectedChannel, syncDepth)}
                  onInputValueChange={(value) => {
                    earliestSyncDateInputSelected = value;
                  }}
                  onSubmit={() =>
                    void saveStandaloneChannelSyncDate(selectedChannel)}
                />
              {/if}
            {/snippet}
          </WorkspaceSidebarSelectedVideoList>
        {/if}
      </div>
    </div>
  {:else if collapsed}
    <div class="flex items-center justify-center px-2 pt-3 pb-1">
      <button
        type="button"
        class="inline-flex h-7 w-7 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-60 transition-all hover:bg-[var(--accent-wash)] hover:opacity-100"
        onclick={onToggleCollapse}
        aria-label="Expand channel sidebar"
      >
        <ChevronIcon direction="right" />
      </button>
    </div>

    <div
      class="custom-scrollbar mt-1 flex min-h-0 flex-1 flex-col items-center gap-2 overflow-y-auto px-2 pb-4"
    >
      {#if loadingChannels}
        {#each Array.from({ length: 5 }) as _, i (i)}
          <div
            class="h-8 w-8 animate-pulse rounded-full bg-[var(--border)] opacity-60"
          ></div>
        {/each}
      {:else}
        {#each filteredChannels as channel (channel.id)}
          <button
            type="button"
            class={`relative flex h-10 w-10 shrink-0 items-center justify-center rounded-full p-0.5 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${selectedChannelId === channel.id ? "bg-[var(--accent-soft)]/60" : "hover:bg-[var(--accent-wash)]"}`}
            onclick={() => handleChannelClick(channel.id)}
            data-tooltip={channel.name}
            data-tooltip-placement="right"
            aria-label={channel.name}
          >
            <span
              class={`flex h-full w-full items-center justify-center overflow-hidden rounded-full bg-[var(--muted)] ${selectedChannelId === channel.id ? "ring-1 ring-[var(--accent)]/20" : ""}`}
            >
              {#if channel.thumbnail_url}
                <img
                  src={channel.thumbnail_url}
                  alt={channel.name}
                  class="h-full w-full object-cover"
                  referrerpolicy="no-referrer"
                />
              {:else}
                <span
                  class="flex h-full w-full items-center justify-center text-[10px] font-bold text-[var(--soft-foreground)]"
                  >{channel.name.charAt(0)}</span
                >
              {/if}
            </span>
          </button>
        {/each}
      {/if}
    </div>
  {:else}
    <WorkspaceSidebarChannelControls
      {readOnly}
      {channelInputOpen}
      {channelSearchOpen}
      {channelInput}
      {channelInputElement}
      onChannelInputElementChange={(element) => {
        channelInputElement = element;
      }}
      {channelSearchQuery}
      {channelSortMode}
      {selectedChannelId}
      {loadingVideos}
      {videoListMode}
      {videoTypeFilter}
      {acknowledgedFilter}
      {addingChannel}
      {addSourceErrorMessage}
      {activeFilterLabel}
      onToggleChannelInput={() => void toggleChannelInput()}
      onToggleSearch={() => {
        channelSearchOpen = !channelSearchOpen;
        if (!channelSearchOpen) channelSearchQuery = "";
      }}
      onCycleSortMode={() =>
        onChannelSortModeChange(cycleChannelSortMode(channelSortMode))}
      {onToggleCollapse}
      {onVideoTypeFilterChange}
      {onAcknowledgedFilterChange}
      onClearAllFilters={clearAllFilters}
      onChannelSubmit={handleChannelSubmit}
      onChannelInputChange={(value) => {
        channelInput = value;
      }}
      onChannelSearchQueryChange={(value) => {
        channelSearchQuery = value;
      }}
      onClearSearch={() => {
        channelSearchQuery = "";
      }}
      onClearFilters={() => {
        void onVideoTypeFilterChange("all");
        void onAcknowledgedFilterChange("all");
      }}
    />

    <div class="sr-only" aria-live="polite">{reorderAnnouncement}</div>

    <div
      class="custom-scrollbar flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto px-2 pt-2 pb-4"
      aria-busy={loadingChannels}
    >
      {#if loadingChannels && channels.length === 0}
        <div class="space-y-4 px-1" role="status" aria-live="polite">
          {#each Array.from({ length: 6 }) as _, index (index)}
            <div class="flex animate-pulse items-center gap-3 px-2 py-2">
              <div
                class="h-8 w-8 shrink-0 rounded-full bg-[var(--border)] opacity-80"
              ></div>
              <div class="min-w-0 flex-1 space-y-1.5">
                <div
                  class="h-3 w-3/4 rounded-full bg-[var(--border)] opacity-80"
                ></div>
                <div
                  class="h-2 w-1/2 rounded-full bg-[var(--border)] opacity-60"
                ></div>
              </div>
            </div>
          {/each}
        </div>
      {:else if channels.length === 0}
        <p
          class="px-3 py-2 text-[14px] font-medium italic text-[var(--soft-foreground)] opacity-50"
        >
          Start by following a channel.
        </p>
      {:else if filteredChannels.length === 0}
        <p
          class="px-3 py-2 text-[14px] font-medium italic text-[var(--soft-foreground)] opacity-50"
        >
          No channels match your search.
        </p>
      {:else}
        {#if loadingChannels}
          <div
            class="flex items-center gap-2 px-3 py-1 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] opacity-60"
            role="status"
            aria-live="polite"
          >
            <span
              class="h-3 w-3 animate-spin rounded-full border-2 border-[var(--border)] border-t-[var(--accent)]"
              aria-hidden="true"
            ></span>
            Loading channels
          </div>
        {/if}

        {#each renderChannels as channel (channel.id)}
          {@const channelVideoCollection =
            channelVideoCollections[channel.id] ??
            createEmptyChannelVideoCollection()}
          {@const isExpanded =
            videoListMode === "per_channel_preview"
              ? channelVideoCollection.expanded
              : selectedChannelId === channel.id}
          {@const dropIndicatorEdge =
            dragOverChannelId === channel.id
              ? resolveChannelDropIndicatorEdge(
                  visibleChannelIds,
                  draggedChannelId,
                  channel.id,
                )
              : null}
          <WorkspaceSidebarChannelRow
            {channel}
            {isExpanded}
            isPreviewMode={videoListMode === "per_channel_preview"}
            isVirtualChannel={isVirtualChannel(channel)}
            {canDeleteChannels}
            {mobileVisible}
            {manualReorderEnabled}
            {draggedChannelId}
            {dragOverChannelId}
            {dropIndicatorEdge}
            {channelUiHidden}
            {loadingVideos}
            {refreshingChannel}
            videoCount={videos.length}
            onSelect={() =>
              videoListMode === "per_channel_preview"
                ? void handlePerChannelPreviewSelect(channel)
                : void onSelectChannel(channel.id)}
            onDragStart={(event) => handleChannelDragStart(channel.id, event)}
            onDragOver={(event) => handleChannelDragOver(channel.id, event)}
            onDrop={(event) => handleChannelDrop(channel.id, event)}
            onDragEnd={handleChannelDragEnd}
            onDelete={() => void onDeleteChannel(channel.id)}
          />

          {#if videoListMode === "per_channel_preview" && isExpanded}
            {@const renderedCollection =
              previewController.resolveRenderedCollectionVideos(
                channelVideoCollection,
              )}
            <div
              class={channelVideoCollection.loadedMode === "paged"
                ? "mt-1 max-h-[21rem] overflow-y-auto pb-1 pr-1"
                : "mt-1 pb-1"}
              id={selectedChannelId === channel.id ? "videos" : undefined}
              onscroll={(event) =>
                previewController.handleChannelCollectionScroll(channel, event)}
              transition:slide={{ duration: 180 }}
            >
              {#if channelVideoCollection.loadingInitial && channelVideoCollection.videos.length === 0}
                <div class="space-y-1 px-1" role="status" aria-live="polite">
                  {#each Array.from({ length: 4 }) as _, i (i)}
                    <div class="animate-pulse px-2 py-1.5">
                      <div
                        class="h-3 w-11/12 rounded-full bg-[var(--border)] opacity-60"
                      ></div>
                      <div
                        class="mt-1 h-2 w-1/3 rounded-full bg-[var(--border)] opacity-40"
                      ></div>
                    </div>
                  {/each}
                </div>
              {:else if channelVideoCollection.videos.length === 0 && !channelVideoCollection.requestKey}
                <p
                  class="px-3 py-2 text-[12px] italic text-[var(--soft-foreground)] opacity-50"
                >
                  {previewController.channelListEmptyCaption(
                    channelVideoCollection.channelVideoCount,
                  )}
                </p>
              {:else}
                {#if renderedCollection.virtualized}
                  <div
                    aria-hidden="true"
                    style={`height:${renderedCollection.topSpacer}px;`}
                  ></div>
                {/if}

                {#each renderedCollection.videos as video (video.id)}
                  <div
                    use:scrollIntoViewWhenSelected={selectedVideoId ===
                      video.id}
                  >
                    <WorkspaceSidebarVideoRow
                      {video}
                      selected={selectedVideoId === video.id}
                      className="min-h-[56px]"
                      onclick={() =>
                        void handleChannelVideoClick(
                          channel.id,
                          video.id,
                          video,
                        )}
                      onmouseenter={() => handleVideoMouseEnter(video.id)}
                      onmouseleave={handleVideoMouseLeave}
                    />
                  </div>
                {/each}

                {#if renderedCollection.virtualized}
                  <div
                    aria-hidden="true"
                    style={`height:${renderedCollection.bottomSpacer}px;`}
                  ></div>
                {/if}

                {#if channelVideoCollection.loadingMore}
                  <p
                    class="px-2 pt-2 text-[10px] text-[var(--soft-foreground)] opacity-50"
                  >
                    Loading videos…
                  </p>
                {/if}

                {#if channelVideoCollection.loadedMode === "paged" && channelVideoCollection.hasMore && !channelVideoCollection.loadingMore}
                  <button
                    type="button"
                    class="mt-1 w-full rounded-[var(--radius-sm)] py-1.5 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
                    onclick={() =>
                      void previewController.loadNextChannelVideoPage(channel)}
                  >
                    Load more
                  </button>
                {/if}
              {/if}
            </div>

            {#if channelVideoCollection.loadedMode === "paged" && !isVirtualChannel(channel)}
              <div class="relative z-10 mt-2 px-2 pb-4">
                <WorkspaceSidebarSyncDateControl
                  {readOnly}
                  open={previewSyncDatePickerChannelId === channel.id}
                  label={formatSyncDate(
                    resolveDisplayedSyncDepthIso({
                      videos: channelVideoCollection.videos,
                      selectedChannel: channel,
                      syncDepth: channelVideoCollection.syncDepth,
                      allowLoadedVideoOverride: true,
                    }),
                  )}
                  inputValue={channelVideoCollection.earliestSyncDateInput}
                  saving={channelVideoCollection.savingSyncDate}
                  popupStackClass={syncDatePopupStackClass}
                  buttonClass="touch-manipulation relative z-10 inline-flex w-full max-w-full flex-wrap items-baseline gap-x-1 rounded-[var(--radius-sm)] px-2 py-1 text-left text-[10px] text-[var(--soft-foreground)] opacity-50 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
                  readonlyClass="text-[10px] text-[var(--soft-foreground)] opacity-50"
                  dialogClass="flex flex-wrap items-center gap-2 rounded-[var(--radius-md)] bg-[var(--surface-strong)] p-2 shadow-[var(--shadow-soft)]"
                  inputClass="min-w-0 flex-1 rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-3 py-2 text-[12px] font-medium transition-colors focus:border-[var(--accent)]/40 focus:outline-none"
                  submitClass="rounded-[var(--radius-sm)] bg-[var(--foreground)] px-3 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--background)] transition-all hover:bg-[var(--accent-strong)] disabled:opacity-30"
                  onToggle={() =>
                    previewController.toggleSyncDatePicker(
                      channel,
                      channelVideoCollection.syncDepth,
                      channelVideoCollection,
                    )}
                  onInputValueChange={(value) => {
                    channelVideoCollection.earliestSyncDateInput = value;
                  }}
                  onSubmit={() =>
                    void previewController.saveChannelSyncDate(channel)}
                />
              </div>
            {/if}
          {:else if isExpanded}
            <WorkspaceSidebarSelectedVideoList
              {videos}
              {selectedVideoId}
              {pendingSelectedVideo}
              {showPendingSelectedVideo}
              {loadingVideos}
              {refreshingChannel}
              {hasMore}
              {historyExhausted}
              {backfillingHistory}
              listId="videos"
              onSelectVideo={(videoId) => void onSelectVideo(videoId)}
              onLoadMoreVideos={() => void onLoadMoreVideos()}
              onVideoMouseEnter={handleVideoMouseEnter}
              onVideoMouseLeave={handleVideoMouseLeave}
            >
              {#snippet footer()}
                {#if !readOnly && !isVirtualChannel(channel)}
                  <div
                    class="relative z-10 mt-1 px-2"
                    id="channel-history-sync"
                  >
                    <WorkspaceSidebarSyncDateControl
                      open={syncDatePickerChannelId === channel.id}
                      label={formatSyncDate(
                        resolveDisplayedSyncDepthIso({
                          videos,
                          selectedChannel,
                          syncDepth,
                          allowLoadedVideoOverride:
                            allowLoadedVideoSyncDepthOverride,
                        }),
                      )}
                      inputValue={earliestSyncDateInputSelected}
                      saving={savingStandaloneSyncDate}
                      popupStackClass={syncDatePopupStackClass}
                      buttonClass="touch-manipulation relative z-10 inline-flex w-full max-w-full flex-wrap items-baseline gap-x-1 rounded-[var(--radius-sm)] px-2 py-1 text-left text-[10px] text-[var(--soft-foreground)] opacity-50 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
                      readonlyClass="mt-1 px-2 text-[10px] text-[var(--soft-foreground)] opacity-50"
                      dialogClass="flex flex-wrap items-center gap-2 rounded-[var(--radius-md)] bg-[var(--surface-strong)] p-2 shadow-[var(--shadow-soft)]"
                      inputClass="min-w-0 flex-1 rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-3 py-2 text-[12px] font-medium transition-colors focus:border-[var(--accent)]/40 focus:outline-none"
                      submitClass="rounded-[var(--radius-sm)] bg-[var(--foreground)] px-3 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--background)] transition-all hover:bg-[var(--accent-strong)] disabled:opacity-30"
                      onToggle={() =>
                        toggleStandaloneSyncDatePicker(channel, syncDepth)}
                      onInputValueChange={(value) => {
                        earliestSyncDateInputSelected = value;
                      }}
                      onSubmit={() =>
                        void saveStandaloneChannelSyncDate(channel)}
                    />
                  </div>
                {:else}
                  <p
                    id="channel-history-sync"
                    class="mt-1 px-2 text-[10px] text-[var(--soft-foreground)] opacity-50"
                  >
                    Synced to {formatSyncDate(
                      resolveDisplayedSyncDepthIso({
                        videos,
                        selectedChannel,
                        syncDepth,
                        allowLoadedVideoOverride:
                          allowLoadedVideoSyncDepthOverride,
                      }),
                    )}
                  </p>
                {/if}
              {/snippet}
            </WorkspaceSidebarSelectedVideoList>
          {/if}
          {#if filteredChannels.indexOf(channel) < filteredChannels.length - 1}
            <hr
              class="mx-3 my-1 border-t border-[var(--border-soft)] opacity-40"
              aria-hidden="true"
            />
          {/if}
        {/each}
        {#if manualReorderEnabled && filteredChannels.length > 0}
          <div
            class="h-5 shrink-0"
            aria-hidden="true"
            ondragover={handleChannelListDragOver}
            ondrop={handleChannelListDrop}
          ></div>
        {/if}
      {/if}
    </div>
  {/if}
</aside>
