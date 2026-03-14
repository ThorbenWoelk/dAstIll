<script lang="ts">
  import { tick } from "svelte";
  import {
    beginChannelDrag,
    completeChannelDrop,
    finishChannelDrag,
    reorderChannels as reorderChannelList,
    updateChannelDragOver,
  } from "$lib/channel-workspace";
  import ChannelCard from "$lib/components/ChannelCard.svelte";
  import type { Channel } from "$lib/types";
  import {
    canManualReorderChannels,
    channelOrderFromList,
    cycleChannelSortMode,
    filterChannels,
    moveChannelByStep,
    resolveChannelDropIndicatorEdge,
  } from "$lib/workspace/channels";
  import type { ChannelReorderDirection } from "$lib/workspace/channels";
  import type { ChannelSortMode } from "$lib/workspace/types";

  type ChannelHandlePointerState = {
    channelId: string;
    pointerId: number;
    clientX: number;
    clientY: number;
  };

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
    onReorderChannels?: (nextOrder: string[]) => void;
  } = $props();

  let draggedChannelId = $state<string | null>(null);
  let dragOverChannelId = $state<string | null>(null);
  let channelSearchQuery = $state("");
  let channelSearchOpen = $state(false);
  let manageChannels = $state(false);
  let channelInput = $state("");
  let mobileReorderMode = $state(false);
  let pointerDrag = $state<ChannelHandlePointerState | null>(null);
  let reorderAnnouncement = $state("");

  let filteredChannels = $derived(
    filterChannels(channels, channelSearchQuery, channelSortMode),
  );
  let visibleChannelIds = $derived(channelOrderFromList(filteredChannels));
  let manualReorderEnabled = $derived(
    canManualReorderChannels(channelSortMode, channelSearchQuery),
  );
  let activeDraggedChannel = $derived(
    filteredChannels.find((channel) => channel.id === draggedChannelId) ?? null,
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

  function clearPointerReorderState() {
    pointerDrag = null;
    draggedChannelId = null;
    dragOverChannelId = null;
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
      if (reordered) {
        onReorderChannels(reordered.channelOrder);
      }
    }
    draggedChannelId = dragState.draggedChannelId;
    dragOverChannelId = dragState.dragOverChannelId;
  }

  function handleChannelDragEnd() {
    const dragState = finishChannelDrag();
    draggedChannelId = dragState.draggedChannelId;
    dragOverChannelId = dragState.dragOverChannelId;
  }

  async function toggleMobileReorderMode() {
    if (mobileReorderMode) {
      mobileReorderMode = false;
      clearPointerReorderState();
      return;
    }

    if (filteredChannels.length < 2) {
      return;
    }

    manageChannels = false;
    if (channelSearchQuery.trim()) {
      channelSearchQuery = "";
    }
    channelSearchOpen = false;
    if (channelSortMode !== "custom") {
      onChannelSortModeChange("custom");
      await tick();
    }
    mobileReorderMode = true;
  }

  function announceReorder(label: string) {
    reorderAnnouncement = `${label} moved.`;
  }

  function handleStepReorder(
    channelId: string,
    direction: ChannelReorderDirection,
  ) {
    const reordered = moveChannelByStep(filteredChannels, channelId, direction);
    if (!reordered) {
      return;
    }

    onReorderChannels(reordered.channelOrder);

    const channel = filteredChannels.find((item) => item.id === channelId);
    if (channel) {
      announceReorder(channel.name);
    }
  }

  function resolvePointerDropTarget(
    clientX: number,
    clientY: number,
  ): string | null {
    if (typeof document === "undefined") {
      return null;
    }

    const target = document
      .elementsFromPoint(clientX, clientY)
      .find(
        (element) =>
          element instanceof HTMLElement &&
          Boolean(element.dataset.channelDropId),
      );

    return target instanceof HTMLElement
      ? (target.dataset.channelDropId ?? null)
      : null;
  }

  function handleHandlePointerDown(channelId: string, event: PointerEvent) {
    if (!mobileReorderMode || !manualReorderEnabled) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    const handle = event.currentTarget as HTMLElement | null;
    handle?.setPointerCapture?.(event.pointerId);

    pointerDrag = {
      channelId,
      pointerId: event.pointerId,
      clientX: event.clientX,
      clientY: event.clientY,
    };
    draggedChannelId = channelId;
    dragOverChannelId = channelId;
  }

  function handleHandlePointerMove(event: PointerEvent) {
    if (!pointerDrag || event.pointerId !== pointerDrag.pointerId) {
      return;
    }

    event.preventDefault();

    pointerDrag = {
      ...pointerDrag,
      clientX: event.clientX,
      clientY: event.clientY,
    };

    const nextTargetId = resolvePointerDropTarget(event.clientX, event.clientY);
    if (nextTargetId) {
      dragOverChannelId = nextTargetId;
    }
  }

  function handleHandlePointerEnd(event: PointerEvent) {
    if (!pointerDrag || event.pointerId !== pointerDrag.pointerId) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    const draggedId = draggedChannelId;
    const targetId = dragOverChannelId;
    const movedChannel = activeDraggedChannel;

    clearPointerReorderState();

    if (!draggedId || !targetId || draggedId === targetId) {
      return;
    }

    const reordered = reorderChannelList(channels, draggedId, targetId);
    if (!reordered) {
      return;
    }

    onReorderChannels(reordered.channelOrder);
    if (movedChannel) {
      announceReorder(movedChannel.name);
    }
  }

  function handleHandlePointerCancel(event: PointerEvent) {
    if (!pointerDrag || event.pointerId !== pointerDrag.pointerId) {
      return;
    }

    event.preventDefault();
    clearPointerReorderState();
  }

  $effect(() => {
    if (!mobileReorderMode) {
      if (pointerDrag) {
        clearPointerReorderState();
      }
      return;
    }

    if (channelSortMode !== "custom" || channelSearchQuery.trim()) {
      mobileReorderMode = false;
      clearPointerReorderState();
    }
  });
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
        class={`inline-flex h-7 items-center justify-center gap-1 rounded-full px-1.5 transition-colors lg:w-7 lg:px-0 ${channelSortMode !== "custom" ? "text-[var(--accent)]" : "text-[var(--soft-foreground)] opacity-40 hover:opacity-80"}`}
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
        <span
          class="text-[9px] font-bold uppercase tracking-wider lg:hidden"
          aria-hidden="true"
        >
          {channelSortMode === "alpha"
            ? "A–Z"
            : channelSortMode === "newest"
              ? "New"
              : "Custom"}
        </span>
      </button>
      <button
        type="button"
        class={`inline-flex h-8 w-8 items-center justify-center rounded-full border transition-all lg:hidden ${mobileReorderMode ? "border-[var(--accent)]/20 bg-[var(--accent-soft)] text-[var(--accent-strong)] shadow-sm" : "border-transparent text-[var(--soft-foreground)] opacity-55 hover:border-[var(--border-soft)] hover:bg-[var(--surface)] hover:opacity-90"} disabled:cursor-not-allowed disabled:opacity-20`}
        onclick={() => void toggleMobileReorderMode()}
        disabled={filteredChannels.length < 2}
        aria-pressed={mobileReorderMode}
        aria-label={mobileReorderMode
          ? "Finish reordering channels"
          : "Reorder channels"}
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.3"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M9 6h11"></path>
          <path d="M9 12h11"></path>
          <path d="M9 18h11"></path>
          <path d="M4 6h.01"></path>
          <path d="M4 12h.01"></path>
          <path d="M4 18h.01"></path>
        </svg>
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

  {#if mobileVisible && mobileReorderMode}
    <div
      class="rounded-[var(--radius-md)] border border-[var(--accent)]/15 bg-[var(--accent-soft)]/75 px-3 py-3 shadow-sm"
    >
      <div class="flex items-start justify-between gap-3">
        <div>
          <p
            class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--accent-strong)]"
          >
            Reorder Channels
          </p>
          <p
            id="mobile-channel-reorder-help"
            class="mt-1 max-w-[18rem] text-[12px] leading-relaxed text-[var(--foreground)]/75"
          >
            Drag the grip handles to move a channel. Prefer taps? Use the arrows
            to nudge it up or down.
          </p>
        </div>
        <button
          type="button"
          class="inline-flex shrink-0 items-center rounded-full border border-[var(--accent)]/15 bg-[var(--surface-strong)] px-3 py-1.5 text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--accent-strong)] shadow-sm transition-colors hover:border-[var(--accent)]/35"
          onclick={() => {
            mobileReorderMode = false;
            clearPointerReorderState();
          }}
        >
          Done
        </button>
      </div>
    </div>
  {/if}

  <div class="sr-only" aria-live="polite">{reorderAnnouncement}</div>

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
      {#each filteredChannels as channel, index (channel.id)}
        {@const dropIndicatorEdge =
          dragOverChannelId === channel.id
            ? resolveChannelDropIndicatorEdge(
                visibleChannelIds,
                draggedChannelId,
                channel.id,
              )
            : null}
        <div
          class="relative"
          data-channel-drop-id={mobileVisible && mobileReorderMode
            ? channel.id
            : undefined}
        >
          {#if dropIndicatorEdge === "top"}
            <div
              class="pointer-events-none absolute inset-x-3 -top-1 z-10 flex items-center gap-2"
            >
              <span class="h-2 w-2 rounded-full bg-[var(--accent)]"></span>
              <span class="h-0.5 flex-1 rounded-full bg-[var(--accent)]"></span>
            </div>
          {/if}

          {#if dropIndicatorEdge === "bottom"}
            <div
              class="pointer-events-none absolute inset-x-3 -bottom-1 z-10 flex items-center gap-2"
            >
              <span class="h-2 w-2 rounded-full bg-[var(--accent)]"></span>
              <span class="h-0.5 flex-1 rounded-full bg-[var(--accent)]"></span>
            </div>
          {/if}

          <ChannelCard
            {channel}
            active={selectedChannelId === channel.id}
            showDelete={manageChannels && !mobileReorderMode}
            trailingSpace={mobileVisible && mobileReorderMode
              ? "wide"
              : manageChannels
                ? "compact"
                : "none"}
            draggableEnabled={!mobileVisible && manualReorderEnabled}
            loading={channel.id.startsWith("temp-")}
            dragging={draggedChannelId === channel.id}
            dragOver={!pointerDrag &&
              dragOverChannelId === channel.id &&
              draggedChannelId !== channel.id}
            onSelect={() => {
              if (mobileVisible && mobileReorderMode) {
                return;
              }
              void onSelectChannel(channel.id);
            }}
            onDragStart={(event) => handleChannelDragStart(channel.id, event)}
            onDragOver={(event) => handleChannelDragOver(channel.id, event)}
            onDrop={(event) => handleChannelDrop(channel.id, event)}
            onDragEnd={handleChannelDragEnd}
            onDelete={() => void onDeleteChannel(channel.id)}
          />

          {#if mobileVisible && mobileReorderMode}
            <div
              class="absolute right-2 top-1/2 z-20 flex -translate-y-1/2 items-center gap-1.5"
            >
              <div
                class="inline-flex items-center overflow-hidden rounded-full border border-[var(--border-soft)] bg-[var(--surface-strong)] shadow-sm"
              >
                <button
                  type="button"
                  class="inline-flex h-9 w-9 items-center justify-center text-[var(--soft-foreground)] transition-colors hover:bg-[var(--muted)]/70 hover:text-[var(--foreground)] disabled:opacity-20"
                  onclick={(event) => {
                    event.stopPropagation();
                    handleStepReorder(channel.id, "up");
                  }}
                  disabled={index === 0 || Boolean(pointerDrag)}
                  aria-label={`Move ${channel.name} up`}
                >
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.6"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path d="m18 15-6-6-6 6"></path>
                  </svg>
                </button>
                <span class="h-5 w-px bg-[var(--border-soft)]"></span>
                <button
                  type="button"
                  class="inline-flex h-9 w-9 items-center justify-center text-[var(--soft-foreground)] transition-colors hover:bg-[var(--muted)]/70 hover:text-[var(--foreground)] disabled:opacity-20"
                  onclick={(event) => {
                    event.stopPropagation();
                    handleStepReorder(channel.id, "down");
                  }}
                  disabled={index === filteredChannels.length - 1 ||
                    Boolean(pointerDrag)}
                  aria-label={`Move ${channel.name} down`}
                >
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2.6"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <path d="m6 9 6 6 6-6"></path>
                  </svg>
                </button>
              </div>

              <button
                type="button"
                class={`inline-flex h-11 w-11 touch-none items-center justify-center rounded-full border transition-all ${draggedChannelId === channel.id ? "border-[var(--accent)] bg-[var(--accent)] text-white shadow-lg shadow-[var(--accent)]/20" : "border-[var(--border-soft)] bg-[var(--surface-strong)] text-[var(--soft-foreground)] shadow-sm hover:border-[var(--accent)]/20 hover:text-[var(--foreground)]"}`}
                onpointerdown={(event) =>
                  handleHandlePointerDown(channel.id, event)}
                onpointermove={handleHandlePointerMove}
                onpointerup={handleHandlePointerEnd}
                onpointercancel={handleHandlePointerCancel}
                aria-label={`Drag to reorder ${channel.name}`}
                aria-describedby="mobile-channel-reorder-help"
                disabled={Boolean(
                  pointerDrag && draggedChannelId !== channel.id,
                )}
              >
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.3"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path d="M9 6h6"></path>
                  <path d="M9 12h6"></path>
                  <path d="M9 18h6"></path>
                  <path d="M5 6h.01"></path>
                  <path d="M5 12h.01"></path>
                  <path d="M5 18h.01"></path>
                  <path d="M19 6h.01"></path>
                  <path d="M19 12h.01"></path>
                  <path d="M19 18h.01"></path>
                </svg>
              </button>
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>

  {#if pointerDrag && activeDraggedChannel}
    <div
      class="pointer-events-none fixed z-[90] flex -translate-x-1/2 -translate-y-[calc(100%+0.75rem)] items-center gap-2 rounded-full border border-[var(--accent)]/20 bg-[var(--surface-strong)] px-3 py-2 shadow-xl shadow-[var(--shadow-soft)]"
      style:left={`${pointerDrag.clientX}px`}
      style:top={`${pointerDrag.clientY}px`}
    >
      <span
        class="inline-flex h-6 w-6 items-center justify-center rounded-full bg-[var(--accent-soft)] text-[var(--accent-strong)]"
      >
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.3"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M9 6h6"></path>
          <path d="M9 12h6"></path>
          <path d="M9 18h6"></path>
        </svg>
      </span>
      <div class="min-w-0">
        <p
          class="truncate text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--accent-strong)]"
        >
          Moving
        </p>
        <p class="truncate text-[13px] font-semibold text-[var(--foreground)]">
          {activeDraggedChannel.name}
        </p>
      </div>
    </div>
  {/if}
</aside>
