<script lang="ts">
  import AddSourceDrawer from "$lib/components/AddSourceDrawer.svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import WorkspaceSidebarVideoFilterControl from "$lib/components/workspace/WorkspaceSidebarVideoFilterControl.svelte";
  import type { VideoTypeFilter } from "$lib/types";
  import type { AddSourceSubmission } from "$lib/workspace/component-props";
  import type {
    AcknowledgedFilter,
    ChannelSortMode,
  } from "$lib/workspace/types";

  let {
    readOnly,
    channelInputOpen,
    channelSearchOpen,
    channelSearchQuery,
    channelSortMode,
    selectedChannelId,
    loadingVideos,
    videoListMode,
    videoTypeFilter,
    acknowledgedFilter,
    addingChannel,
    addSourceErrorMessage,
    activeFilterLabel,
    onOpenChannelInput,
    onCloseChannelInput,
    onToggleChannelInput,
    onToggleSearch,
    onCycleSortMode,
    onToggleCollapse,
    onVideoTypeFilterChange,
    onAcknowledgedFilterChange,
    onClearAllFilters,
    onAddSourceSubmit,
    onChannelSearchQueryChange,
    onClearSearch,
    onClearFilters,
  }: {
    readOnly: boolean;
    channelInputOpen: boolean;
    channelSearchOpen: boolean;
    channelSearchQuery: string;
    channelSortMode: ChannelSortMode;
    selectedChannelId: string | null;
    loadingVideos: boolean;
    videoListMode: "selected_channel" | "per_channel_preview";
    videoTypeFilter: VideoTypeFilter;
    acknowledgedFilter: AcknowledgedFilter;
    addingChannel: boolean;
    addSourceErrorMessage: string | null;
    activeFilterLabel: string;
    onOpenChannelInput: () => void | Promise<void>;
    onCloseChannelInput: () => void | Promise<void>;
    onToggleChannelInput: () => void | Promise<void>;
    onToggleSearch: () => void;
    onCycleSortMode: () => void;
    onToggleCollapse: () => void;
    onVideoTypeFilterChange: (value: VideoTypeFilter) => void | Promise<void>;
    onAcknowledgedFilterChange: (
      value: AcknowledgedFilter,
    ) => void | Promise<void>;
    onClearAllFilters: () => Promise<void>;
    onAddSourceSubmit: (
      input: AddSourceSubmission,
    ) => Promise<boolean> | boolean;
    onChannelSearchQueryChange: (value: string) => void;
    onClearSearch: () => void;
    onClearFilters: () => void;
  } = $props();

  function handleChannelSearchInputEvent(event: Event) {
    onChannelSearchQueryChange((event.currentTarget as HTMLInputElement).value);
  }
</script>

<div
  id="tour-library-tools"
  class="flex h-14 items-center justify-between gap-2 px-4 pt-2"
>
  <span
    class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-55"
  >
    Sources
  </span>
  <div class="flex items-center gap-1">
    {#if !readOnly}
      <button
        type="button"
        id="tour-add-channel"
        class={`inline-flex h-6 w-6 items-center justify-center rounded-full transition-colors ${channelInputOpen ? "bg-[var(--accent-wash)] text-[var(--accent)]" : "text-[var(--soft-foreground)] opacity-55 hover:bg-[var(--accent-wash)] hover:opacity-100"}`}
        onclick={() => void onToggleChannelInput()}
        aria-label={channelInputOpen ? "Close add source" : "Add source"}
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
          class={`transition-transform ${channelInputOpen ? "rotate-45" : ""}`}
        >
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
    {/if}
    <button
      type="button"
      class={`inline-flex h-6 w-6 items-center justify-center rounded-full transition-colors ${channelSearchOpen ? "bg-[var(--accent-wash)] text-[var(--accent)]" : "text-[var(--soft-foreground)] opacity-55 hover:bg-[var(--accent-wash)] hover:opacity-100"}`}
      onclick={onToggleSearch}
      aria-label={channelSearchOpen ? "Close search" : "Search sources"}
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
      >
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
    </button>
    <button
      type="button"
      class={`inline-flex h-6 w-6 items-center justify-center rounded-full transition-colors ${channelSortMode !== "custom" ? "bg-[var(--accent-wash)] text-[var(--accent)]" : "text-[var(--soft-foreground)] opacity-55 hover:bg-[var(--accent-wash)] hover:opacity-100"}`}
      onclick={onCycleSortMode}
      aria-label={channelSortMode === "custom"
        ? "Sort alphabetically"
        : channelSortMode === "alpha"
          ? "Sort by newest"
          : "Reset to custom order"}
      data-tooltip={channelSortMode === "alpha"
        ? "A-Z"
        : channelSortMode === "newest"
          ? "Newest"
          : undefined}
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
      >
        <path d="M3 16l4 4 4-4M7 20V4M21 8l-4-4-4 4M17 4v16" />
      </svg>
    </button>
    <WorkspaceSidebarVideoFilterControl
      {videoTypeFilter}
      {acknowledgedFilter}
      disabled={videoListMode !== "per_channel_preview" &&
        (!selectedChannelId || loadingVideos)}
      onSelectVideoType={onVideoTypeFilterChange}
      onSelectAcknowledged={onAcknowledgedFilterChange}
      {onClearAllFilters}
    />
    <button
      type="button"
      class="inline-flex h-6 w-6 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-55 transition-all hover:bg-[var(--accent-wash)] hover:opacity-100"
      onclick={onToggleCollapse}
      aria-label="Collapse sidebar"
    >
      <ChevronIcon direction="left" />
    </button>
  </div>
</div>

<AddSourceDrawer
  open={channelInputOpen}
  busy={addingChannel}
  errorMessage={addSourceErrorMessage}
  onOpen={() => void onOpenChannelInput()}
  onClose={() => void onCloseChannelInput()}
  onSubmit={onAddSourceSubmit}
/>

{#if channelSearchOpen}
  <div
    class="mx-4 mt-1 flex items-center gap-2 rounded-full border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-3 py-2"
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
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
    </svg>
    <input
      type="text"
      class="min-w-0 flex-1 bg-transparent text-[14px] placeholder:text-[var(--soft-foreground)] placeholder:opacity-40 focus-visible:outline-none"
      placeholder="Filter..."
      value={channelSearchQuery}
      oninput={handleChannelSearchInputEvent}
    />
    {#if channelSearchQuery}
      <button
        type="button"
        class="inline-flex h-5 w-5 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-40 transition-opacity hover:opacity-80"
        onclick={onClearSearch}
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
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    {/if}
  </div>
{/if}

{#if activeFilterLabel}
  <div class="mx-4 mt-2 flex items-center gap-2">
    <span
      class="rounded-full bg-[var(--accent-wash)] px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--accent-strong)]"
    >
      {activeFilterLabel}
    </span>
    <button
      type="button"
      class="inline-flex h-4 w-4 items-center justify-center rounded-full text-[var(--accent)] opacity-60 transition-opacity hover:opacity-100"
      onclick={onClearFilters}
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
      >
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </button>
  </div>
{/if}
