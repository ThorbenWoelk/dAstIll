<script lang="ts">
  import { tick } from "svelte";
  import {
    beginChannelDrag,
    completeChannelDrop,
    finishChannelDrag,
    moveChannelToIndex,
    reorderChannels as reorderChannelList,
    updateChannelDragOver,
  } from "$lib/channel-workspace";
  import ChannelCard from "$lib/components/ChannelCard.svelte";
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import { clickOutside } from "$lib/actions/click-outside";
  import type { Channel, Video, SyncDepth, VideoTypeFilter } from "$lib/types";
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
    filterChannels,
    resolveChannelDropIndicatorEdge,
  } from "$lib/workspace/channels";
  import { formatShortDate } from "$lib/utils/date";
  import { formatSyncDate } from "$lib/workspace/content";
  import { resolveDisplayedSyncDepthIso } from "$lib/sync-depth";

  const VIDEO_TYPE_OPTIONS: Array<{
    value: VideoTypeFilter;
    label: string;
  }> = [
    { value: "all", label: "All Content" },
    { value: "long", label: "Full Videos" },
    { value: "short", label: "Shorts" },
  ];

  const ACKNOWLEDGED_FILTER_OPTIONS: Array<{
    value: AcknowledgedFilter;
    label: string;
  }> = [
    { value: "all", label: "All Statuses" },
    { value: "unack", label: "Unread" },
    { value: "ack", label: "Read" },
  ];

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
    },
    channelActions = {
      onChannelSortModeChange: (_next: ChannelSortMode) => {},
      onAddChannel: async (_input: string) => false,
      onSelectChannel: async (_channelId: string) => {},
      onDeleteChannel: async (_channelId: string) => {},
      onReorderChannels: (_nextOrder: string[]) => {},
    },
    videoState = {
      videos: [],
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
      allowLoadedVideoSyncDepthOverride: false,
    },
    videoActions = {
      onSelectVideo: async (_videoId: string) => {},
      onLoadMoreVideos: async () => {},
      onVideoTypeFilterChange: async (_value: VideoTypeFilter) => {},
      onAcknowledgedFilterChange: async (_value: AcknowledgedFilter) => {},
    },
  }: {
    shell?: WorkspaceSidebarShellProps;
    channelState?: WorkspaceSidebarChannelState;
    channelActions?: WorkspaceSidebarChannelActions;
    videoState?: WorkspaceSidebarVideoState;
    videoActions?: WorkspaceSidebarVideoActions;
  } = $props();

  let collapsed = $derived(shell.collapsed);
  let width = $derived(shell.width);
  let onToggleCollapse = $derived(shell.onToggleCollapse);
  let mobileVisible = $derived(shell.mobileVisible);

  let channels = $derived(channelState.channels);
  let selectedChannelId = $derived(channelState.selectedChannelId);
  let loadingChannels = $derived(channelState.loadingChannels);
  let addingChannel = $derived(channelState.addingChannel);
  let channelSortMode = $derived(channelState.channelSortMode);

  let onChannelSortModeChange = $derived(
    channelActions.onChannelSortModeChange,
  );
  let onAddChannel = $derived(channelActions.onAddChannel);
  let onSelectChannel = $derived(channelActions.onSelectChannel);
  let onDeleteChannel = $derived(channelActions.onDeleteChannel);
  let onReorderChannels = $derived(channelActions.onReorderChannels);

  let videos = $derived(videoState.videos);
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
  let onLoadMoreVideos = $derived(videoActions.onLoadMoreVideos);
  let onVideoTypeFilterChange = $derived(videoActions.onVideoTypeFilterChange);
  let onAcknowledgedFilterChange = $derived(
    videoActions.onAcknowledgedFilterChange,
  );

  let draggedChannelId = $state<string | null>(null);
  let dragOverChannelId = $state<string | null>(null);
  let channelSearchQuery = $state("");
  let channelSearchOpen = $state(false);
  let channelInputOpen = $state(false);
  let manageChannels = $state(false);
  let channelInput = $state("");
  let channelInputElement = $state<HTMLInputElement | null>(null);
  let reorderAnnouncement = $state("");
  let filterMenuOpen = $state(false);

  let filteredChannels = $derived(
    filterChannels(channels, channelSearchQuery, channelSortMode),
  );
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

  async function selectVideoTypeFilter(value: VideoTypeFilter) {
    filterMenuOpen = false;
    await onVideoTypeFilterChange(value);
  }

  async function selectAcknowledgedFilter(value: AcknowledgedFilter) {
    filterMenuOpen = false;
    await onAcknowledgedFilterChange(value);
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") filterMenuOpen = false;
  }

  function handleChannelClick(channelId: string) {
    if (collapsed) onToggleCollapse();
    void onSelectChannel(channelId);
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<aside
  id="workspace"
  class={`fade-in flex flex-col bg-[var(--surface)] ${mobileVisible ? "h-full" : "hidden lg:flex"} lg:h-full lg:shrink-0`}
  style="width: {width ?? (collapsed ? 52 : 280)}px;"
>
  {#if collapsed}
    <div class="flex items-center justify-center px-1.5 pt-3 pb-1">
      <button
        type="button"
        class="inline-flex h-7 w-7 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-60 transition-all hover:bg-[var(--accent-wash)] hover:opacity-100"
        onclick={onToggleCollapse}
        aria-label="Expand channel sidebar"
        data-tooltip="Channels"
        data-tooltip-placement="right"
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><path d="M15 18l-6-6 6-6" /><line
            x1="20"
            y1="4"
            x2="20"
            y2="20"
          /></svg
        >
      </button>
    </div>

    <div
      class="custom-scrollbar mt-1 flex min-h-0 flex-1 flex-col items-center gap-1.5 overflow-y-auto px-1.5 pb-4"
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
    <div class="flex items-center justify-between gap-2 px-4 pt-3 pb-1">
      <span
        class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
      >
        Channels
      </span>
      <div class="flex items-center gap-0.5">
        <button
          type="button"
          class={`inline-flex h-6 w-6 items-center justify-center rounded-full transition-colors ${manageChannels ? "bg-[var(--accent-wash)] text-[var(--danger)]" : "text-[var(--soft-foreground)] opacity-55 hover:bg-[var(--accent-wash)] hover:opacity-100"}`}
          data-tooltip={manageChannels ? "Exit manage mode" : "Manage"}
          data-tooltip-placement="bottom"
          onclick={() => {
            manageChannels = !manageChannels;
          }}
          aria-label={manageChannels ? "Exit manage mode" : "Manage channels"}
        >
          <svg
            width="11"
            height="11"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><path d="M3 6h18" /><path
              d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"
            /><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" /></svg
          >
        </button>
        <button
          type="button"
          class={`inline-flex h-6 w-6 items-center justify-center rounded-full transition-colors ${channelInputOpen ? "bg-[var(--accent-wash)] text-[var(--accent)]" : "text-[var(--soft-foreground)] opacity-55 hover:bg-[var(--accent-wash)] hover:opacity-100"}`}
          data-tooltip={channelInputOpen ? "Close follow" : "Follow channel"}
          data-tooltip-placement="bottom"
          onclick={() => void toggleChannelInput()}
          aria-label={channelInputOpen
            ? "Close follow channel"
            : "Follow channel"}
        >
          <svg
            width="11"
            height="11"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            class={`transition-transform ${channelInputOpen ? "rotate-45" : ""}`}
            ><line x1="12" y1="5" x2="12" y2="19" /><line
              x1="5"
              y1="12"
              x2="19"
              y2="12"
            /></svg
          >
        </button>
        <button
          type="button"
          class={`inline-flex h-6 w-6 items-center justify-center rounded-full transition-colors ${channelSearchOpen ? "bg-[var(--accent-wash)] text-[var(--accent)]" : "text-[var(--soft-foreground)] opacity-55 hover:bg-[var(--accent-wash)] hover:opacity-100"}`}
          data-tooltip={channelSearchOpen ? "Close search" : "Search"}
          data-tooltip-placement="bottom"
          onclick={() => {
            channelSearchOpen = !channelSearchOpen;
            if (!channelSearchOpen) channelSearchQuery = "";
          }}
          aria-label={channelSearchOpen ? "Close search" : "Search channels"}
        >
          <svg
            width="11"
            height="11"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><circle cx="11" cy="11" r="8" /><line
              x1="21"
              y1="21"
              x2="16.65"
              y2="16.65"
            /></svg
          >
        </button>
        <div
          class="relative"
          use:clickOutside={{
            enabled: filterMenuOpen,
            onClickOutside: () => (filterMenuOpen = false),
          }}
        >
          <button
            type="button"
            id="video-filter-button"
            class={`inline-flex h-6 w-6 items-center justify-center rounded-full transition-colors ${videoTypeFilter !== "all" || acknowledgedFilter !== "all" || filterMenuOpen ? "bg-[var(--accent)] text-white" : "text-[var(--soft-foreground)] opacity-55 hover:bg-[var(--accent-wash)] hover:opacity-100"}`}
            onclick={() => {
              filterMenuOpen = !filterMenuOpen;
            }}
            disabled={!selectedChannelId || loadingVideos}
            aria-label="Video filters"
            aria-haspopup="menu"
            aria-expanded={filterMenuOpen}
            data-tooltip="Filters"
            data-tooltip-placement="bottom"
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
              ><line x1="3" y1="6" x2="21" y2="6" /><line
                x1="7"
                y1="12"
                x2="17"
                y2="12"
              /><line x1="10" y1="18" x2="14" y2="18" /></svg
            >
          </button>
          {#if filterMenuOpen}
            <div
              role="menu"
              aria-label="Video filters"
              class="fade-in absolute left-0 top-full z-20 mt-2 w-52 overflow-hidden rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface-strong)] shadow-xl"
            >
              <div class="space-y-4 p-2">
                <div class="grid gap-1">
                  <p
                    class="px-2 pb-1 text-[10px] font-bold text-[var(--soft-foreground)] opacity-50"
                  >
                    TYPE
                  </p>
                  {#each VIDEO_TYPE_OPTIONS as opt}
                    <button
                      type="button"
                      role="menuitemradio"
                      aria-checked={videoTypeFilter === opt.value}
                      class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === opt.value ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"}`}
                      onclick={() => void selectVideoTypeFilter(opt.value)}
                    >
                      <span>{opt.label}</span>
                      {#if videoTypeFilter === opt.value}<CheckIcon
                          size={12}
                          strokeWidth={3}
                        />{/if}
                    </button>
                  {/each}
                </div>
                <div class="grid gap-1">
                  <p
                    class="px-2 pb-1 text-[10px] font-bold text-[var(--soft-foreground)] opacity-50"
                  >
                    STATUS
                  </p>
                  {#each ACKNOWLEDGED_FILTER_OPTIONS as opt}
                    <button
                      type="button"
                      role="menuitemradio"
                      aria-checked={acknowledgedFilter === opt.value}
                      class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === opt.value ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"}`}
                      onclick={() => void selectAcknowledgedFilter(opt.value)}
                    >
                      <span>{opt.label}</span>
                      {#if acknowledgedFilter === opt.value}<CheckIcon
                          size={12}
                          strokeWidth={3}
                        />{/if}
                    </button>
                  {/each}
                </div>
              </div>
            </div>
          {/if}
        </div>
        <button
          type="button"
          class="inline-flex h-6 w-6 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-55 transition-all hover:bg-[var(--accent-wash)] hover:opacity-100"
          onclick={onToggleCollapse}
          aria-label="Collapse sidebar"
          data-tooltip="Collapse"
          data-tooltip-placement="bottom"
        >
          <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><path d="M15 18l-6-6 6-6" /><line
              x1="20"
              y1="4"
              x2="20"
              y2="20"
            /></svg
          >
        </button>
      </div>
    </div>

    {#if channelInputOpen}
      <form
        class="mx-4 mt-2"
        onsubmit={handleChannelSubmit}
        aria-label="Follow channel"
      >
        <div
          class="flex min-w-0 items-center gap-2 border-b border-[var(--accent-border-soft)] pb-1 transition-all focus-within:border-[var(--accent)]/40"
        >
          <label for="channel-input" class="sr-only">Add Channel</label>
          <input
            id="channel-input"
            bind:this={channelInputElement}
            name="channel"
            autocomplete="off"
            spellcheck={false}
            class="min-w-0 flex-1 bg-transparent py-1.5 text-[13px] font-medium placeholder:text-[var(--soft-foreground)] placeholder:opacity-40 focus-visible:outline-none"
            placeholder="Paste handle or URL"
            bind:value={channelInput}
          />
          <button
            type="submit"
            class="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-[var(--foreground)] transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--accent-strong)] disabled:opacity-20"
            disabled={!channelInput.trim() || addingChannel}
            aria-label="Follow channel"
          >
            <svg
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="3"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><line x1="12" y1="5" x2="12" y2="19" /><line
                x1="5"
                y1="12"
                x2="19"
                y2="12"
              /></svg
            >
          </button>
        </div>
      </form>
    {/if}

    {#if channelSearchOpen}
      <div
        class="mx-4 mt-2 flex items-center gap-2 border-b border-[var(--accent-border-soft)] px-1 pb-2"
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
          ><circle cx="11" cy="11" r="8" /><line
            x1="21"
            y1="21"
            x2="16.65"
            y2="16.65"
          /></svg
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
            class="inline-flex h-5 w-5 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-40 transition-opacity hover:opacity-80"
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
              ><line x1="18" y1="6" x2="6" y2="18" /><line
                x1="6"
                y1="6"
                x2="18"
                y2="18"
              /></svg
            >
          </button>
        {/if}
      </div>
    {/if}

    {#if activeFilterLabel}
      <div class="mx-4 mt-2 flex items-center gap-1.5">
        <span
          class="text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--accent)]"
          >{activeFilterLabel}</span
        >
        <button
          type="button"
          class="inline-flex h-4 w-4 items-center justify-center rounded-full text-[var(--accent)] opacity-60 transition-opacity hover:opacity-100"
          onclick={() => {
            void onVideoTypeFilterChange("all");
            void onAcknowledgedFilterChange("all");
          }}
          aria-label="Clear filters"
        >
          <svg
            width="8"
            height="8"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="3"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><line x1="18" y1="6" x2="6" y2="18" /><line
              x1="6"
              y1="6"
              x2="18"
              y2="18"
            /></svg
          >
        </button>
      </div>
    {/if}

    <div class="sr-only" aria-live="polite">{reorderAnnouncement}</div>

    <div
      class="custom-scrollbar mt-2 flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-2 pb-4"
      aria-busy={loadingChannels}
    >
      {#if loadingChannels}
        <div class="space-y-3 px-1" role="status" aria-live="polite">
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
          class="px-3 py-2 text-[13px] font-medium italic text-[var(--soft-foreground)] opacity-50"
        >
          Start by following a channel.
        </p>
      {:else if filteredChannels.length === 0}
        <p
          class="px-3 py-2 text-[13px] font-medium italic text-[var(--soft-foreground)] opacity-50"
        >
          No channels match your search.
        </p>
      {:else}
        {#each filteredChannels as channel (channel.id)}
          {@const isExpanded = selectedChannelId === channel.id}
          {@const dropIndicatorEdge =
            dragOverChannelId === channel.id
              ? resolveChannelDropIndicatorEdge(
                  visibleChannelIds,
                  draggedChannelId,
                  channel.id,
                )
              : null}
          <div class="relative" data-channel-id={channel.id} role="listitem">
            {#if dropIndicatorEdge === "top"}
              <div
                class="pointer-events-none absolute inset-x-3 -top-1 z-10 flex items-center gap-2"
              >
                <span class="h-2 w-2 rounded-full bg-[var(--accent)]"></span>
                <span class="h-0.5 flex-1 rounded-full bg-[var(--accent)]"
                ></span>
              </div>
            {/if}
            {#if dropIndicatorEdge === "bottom"}
              <div
                class="pointer-events-none absolute inset-x-3 -bottom-1 z-10 flex items-center gap-2"
              >
                <span class="h-2 w-2 rounded-full bg-[var(--accent)]"></span>
                <span class="h-0.5 flex-1 rounded-full bg-[var(--accent)]"
                ></span>
              </div>
            {/if}

            {#if isExpanded && (refreshingChannel || (loadingVideos && videos.length === 0))}
              <div class="flex items-center gap-1.5 px-2 pb-1">
                <span
                  class="h-3 w-3 animate-spin rounded-full border-[1.5px] border-[var(--border)] border-t-[var(--accent)]"
                  role="status"
                  aria-label="Syncing"
                ></span>
                <span
                  class="text-[10px] text-[var(--soft-foreground)] opacity-50"
                  >Syncing</span
                >
              </div>
            {/if}

            <div
              class={isExpanded ? "sticky top-0 z-10 bg-[var(--surface)]" : ""}
            >
              <ChannelCard
                {channel}
                active={isExpanded}
                showDelete={manageChannels}
                draggableEnabled={!mobileVisible && manualReorderEnabled}
                loading={channel.id.startsWith("temp-")}
                dragging={draggedChannelId === channel.id}
                dragOver={dragOverChannelId === channel.id &&
                  draggedChannelId !== channel.id}
                onSelect={() => void onSelectChannel(channel.id)}
                onDragStart={(event) =>
                  handleChannelDragStart(channel.id, event)}
                onDragOver={(event) => handleChannelDragOver(channel.id, event)}
                onDrop={(event) => handleChannelDrop(channel.id, event)}
                onDragEnd={handleChannelDragEnd}
                onDelete={() => void onDeleteChannel(channel.id)}
              />
            </div>
          </div>

          {#if isExpanded}
            <div class="mt-1 pb-1" id="videos">
              {#if loadingVideos && videos.length === 0}
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
              {:else if videos.length === 0}
                <p
                  class="px-3 py-2 text-[12px] italic text-[var(--soft-foreground)] opacity-50"
                >
                  No videos yet.
                </p>
              {:else}
                {#each videos as video (video.id)}
                  <button
                    type="button"
                    class={`group flex w-full items-center gap-2 rounded-[var(--radius-sm)] px-2 py-1.5 text-left transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${selectedVideoId === video.id ? "bg-[var(--accent-wash)]" : "hover:bg-[var(--accent-wash)]"}`}
                    onclick={() => void onSelectVideo(video.id)}
                  >
                    <div class="min-w-0 flex-1">
                      <p
                        class="line-clamp-2 text-[12px] font-medium leading-tight tracking-tight text-[var(--foreground)]"
                      >
                        {video.title}
                      </p>
                      <div class="mt-0.5 flex items-center gap-1.5">
                        <span
                          class="text-[10px] text-[var(--soft-foreground)] opacity-50"
                          >{formatShortDate(video.published_at)}</span
                        >
                        {#if video.transcript_status === "loading" || video.summary_status === "loading"}
                          <span class="relative flex h-1.5 w-1.5"
                            ><span
                              class="absolute inline-flex h-full w-full animate-ping rounded-full bg-[var(--accent)] opacity-75"
                            ></span><span
                              class="relative inline-flex h-1.5 w-1.5 rounded-full bg-[var(--accent)]"
                            ></span></span
                          >
                        {:else if video.transcript_status === "failed" || video.summary_status === "failed"}
                          <svg
                            class="text-[var(--danger)]"
                            width="9"
                            height="9"
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
                        {:else if video.transcript_status === "ready" && video.summary_status === "ready"}
                          <svg
                            class="text-[var(--soft-foreground)] opacity-25"
                            width="9"
                            height="9"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="3"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            ><polyline points="20 6 9 17 4 12" /></svg
                          >
                        {/if}
                      </div>
                    </div>
                  </button>
                {/each}

                {#if hasMore || !historyExhausted}
                  <button
                    type="button"
                    class="mt-1 w-full rounded-[var(--radius-sm)] py-1.5 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] disabled:opacity-30"
                    onclick={() => void onLoadMoreVideos()}
                    disabled={loadingVideos || backfillingHistory}
                  >
                    {#if loadingVideos || backfillingHistory}
                      Loading...
                    {:else if hasMore}
                      Load More
                    {:else}
                      Load History
                    {/if}
                  </button>
                {/if}

                {#if videos.length > 0}
                  <p
                    class="mt-1 flex items-center gap-1 px-2 text-[10px] text-[var(--soft-foreground)] opacity-50"
                  >
                    <svg
                      width="9"
                      height="9"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2.2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      aria-hidden="true"
                      ><path d="M12 6v6l4 2" /><circle
                        cx="12"
                        cy="12"
                        r="9"
                      /></svg
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
              {/if}
            </div>
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
