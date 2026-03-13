<script lang="ts">
  import {
    beginChannelDrag,
    completeChannelDrop,
    finishChannelDrag,
    updateChannelDragOver,
  } from "$lib/channel-workspace";
  import ChannelCard from "$lib/components/ChannelCard.svelte";
  import type { Channel } from "$lib/types";
  import {
    cycleChannelSortMode,
    filterChannels,
  } from "$lib/workspace/channels";
  import type { ChannelSortMode } from "$lib/workspace/types";

  let {
    mobileVisible = false,
    channels = [],
    selectedChannelId = null,
    loadingChannels = false,
    addingChannel = false,
    channelSortMode = "custom",
    onChannelSortModeChange = () => {},
    onAddChannel = async () => false,
    onSelectChannel = async () => {},
    onDeleteChannel = async () => {},
    onReorderChannels = () => {},
  }: {
    mobileVisible?: boolean;
    channels?: Channel[];
    selectedChannelId?: string | null;
    loadingChannels?: boolean;
    addingChannel?: boolean;
    channelSortMode?: ChannelSortMode;
    onChannelSortModeChange?: (next: ChannelSortMode) => void;
    onAddChannel?: (input: string) => Promise<boolean> | boolean;
    onSelectChannel?: (channelId: string) => Promise<void> | void;
    onDeleteChannel?: (channelId: string) => Promise<void> | void;
    onReorderChannels?: (dragId: string, targetId: string) => void;
  } = $props();

  let draggedChannelId = $state<string | null>(null);
  let dragOverChannelId = $state<string | null>(null);
  let channelSearchQuery = $state("");
  let channelSearchOpen = $state(false);
  let manageChannels = $state(false);
  let channelInput = $state("");

  let filteredChannels = $derived(
    filterChannels(channels, channelSearchQuery, channelSortMode),
  );

  async function handleChannelSubmit(event: SubmitEvent) {
    event.preventDefault();
    const submittedInput = channelInput.trim();
    if (!submittedInput || addingChannel) {
      return;
    }

    channelInput = "";
    const success = await onAddChannel(submittedInput);
    if (!success) {
      channelInput = submittedInput;
    }
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
      onReorderChannels(sourceId, channelId);
    }
    draggedChannelId = dragState.draggedChannelId;
    dragOverChannelId = dragState.dragOverChannelId;
  }

  function handleChannelDragEnd() {
    const dragState = finishChannelDrag();
    draggedChannelId = dragState.draggedChannelId;
    dragOverChannelId = dragState.dragOverChannelId;
  }
</script>

<aside
  class={`fade-in stagger-1 flex min-h-0 min-w-0 flex-col border-0 lg:sticky lg:top-4 lg:h-[calc(100vh-4rem)] lg:gap-3 lg:border-r lg:border-[var(--border-soft)] lg:pl-2 lg:pr-5 ${mobileVisible ? "h-full gap-4 p-3" : "hidden lg:flex"}`}
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
        class={`inline-flex h-7 w-7 items-center justify-center rounded-full transition-colors ${manageChannels ? "text-[var(--danger)]" : "text-[var(--soft-foreground)] opacity-40 hover:opacity-80"}`}
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
        >
          <path d="M3 6h18"></path>
          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
        </svg>
      </button>
      <button
        type="button"
        class={`inline-flex h-7 w-7 items-center justify-center rounded-full transition-colors ${channelSearchOpen ? "text-[var(--accent)]" : "text-[var(--soft-foreground)] opacity-40 hover:opacity-80"}`}
        data-tooltip={channelSearchOpen ? "Close search" : "Search channels"}
        onclick={() => {
          channelSearchOpen = !channelSearchOpen;
          if (!channelSearchOpen) {
            channelSearchQuery = "";
          }
        }}
        aria-label={channelSearchOpen ? "Close search" : "Search channels"}
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
        >
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
      </button>
      <button
        type="button"
        class={`inline-flex h-7 w-7 items-center justify-center rounded-full transition-colors ${channelSortMode !== "custom" ? "text-[var(--accent)]" : "text-[var(--soft-foreground)] opacity-40 hover:opacity-80"}`}
        data-tooltip={channelSortMode === "custom"
          ? "Sort: Custom"
          : channelSortMode === "alpha"
            ? "Sort: A-Z"
            : "Sort: Newest"}
        onclick={() =>
          onChannelSortModeChange(cycleChannelSortMode(channelSortMode))}
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
          >
            <path d="M3 6h8"></path>
            <path d="M3 12h5"></path>
            <path d="M3 18h3"></path>
            <path d="M18 6v12"></path>
            <path d="m14 18 4 4 4-4"></path>
          </svg>
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
          >
            <path d="M3 6h3"></path>
            <path d="M3 12h5"></path>
            <path d="M3 18h8"></path>
            <path d="M18 18V6"></path>
            <path d="m14 6 4-4 4 4"></path>
          </svg>
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
          >
            <path d="M3 6h8"></path>
            <path d="M3 12h5"></path>
            <path d="M3 18h3"></path>
            <path d="M18 6v12"></path>
          </svg>
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
      >
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>
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
          >
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      {/if}
    </div>
  {/if}

  <form class="grid" onsubmit={handleChannelSubmit} aria-label="Follow channel">
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
        >
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
      </button>
    </div>
  </form>

  <div
    class="custom-scrollbar mobile-bottom-stack-padding flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto pr-1 lg:pb-0"
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
        class="px-1 text-[14px] font-medium italic text-[var(--soft-foreground)] opacity-50"
      >
        Start by following a channel.
      </p>
    {:else if filteredChannels.length === 0}
      <p
        class="px-1 text-[14px] font-medium italic text-[var(--soft-foreground)] opacity-50"
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
          onSelect={() => void onSelectChannel(channel.id)}
          onDragStart={(event) => handleChannelDragStart(channel.id, event)}
          onDragOver={(event) => handleChannelDragOver(channel.id, event)}
          onDrop={(event) => handleChannelDrop(channel.id, event)}
          onDragEnd={handleChannelDragEnd}
          onDelete={() => void onDeleteChannel(channel.id)}
        />
      {/each}
    {/if}
  </div>
</aside>
