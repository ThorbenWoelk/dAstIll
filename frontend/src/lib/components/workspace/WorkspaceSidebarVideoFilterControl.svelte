<script lang="ts">
  import { tick } from "svelte";
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import FilterIcon from "$lib/components/icons/FilterIcon.svelte";
  import { clickOutside } from "$lib/actions/click-outside";
  import type { VideoTypeFilter } from "$lib/types";
  import type { AcknowledgedFilter } from "$lib/workspace/types";
  import {
    SIDEBAR_ACKNOWLEDGED_FILTER_OPTIONS,
    SIDEBAR_VIDEO_TYPE_OPTIONS,
  } from "$lib/workspace/sidebar-filter-options";

  let {
    videoTypeFilter,
    acknowledgedFilter,
    disabled = false,
    size = "sm",
    onSelectVideoType,
    onSelectAcknowledged,
    onClearAllFilters,
  }: {
    videoTypeFilter: VideoTypeFilter;
    acknowledgedFilter: AcknowledgedFilter;
    disabled?: boolean;
    /** "sm" = desktop sidebar (h-5 w-5), "md" = mobile top-nav (h-9 w-9) */
    size?: "sm" | "md";
    onSelectVideoType: (value: VideoTypeFilter) => void | Promise<void>;
    onSelectAcknowledged: (value: AcknowledgedFilter) => void | Promise<void>;
    onClearAllFilters: () => void | Promise<void>;
  } = $props();

  let filterMenuOpen = $state(false);
  let videoFilterButtonEl = $state<HTMLButtonElement | null>(null);
  let videoFilterMenuStyle = $state("");
  const VIDEO_FILTER_MENU_WIDTH_PX = 208;
  let activeFilterCount = $derived(
    Number(videoTypeFilter !== "all") + Number(acknowledgedFilter !== "all"),
  );

  function updateVideoFilterMenuPosition() {
    if (!filterMenuOpen || !videoFilterButtonEl) return;
    const rect = videoFilterButtonEl.getBoundingClientRect();
    if (rect.width === 0 && rect.height === 0) return;
    const vw = window.innerWidth;
    const gap = 8;
    let left = rect.left;
    left = Math.max(12, Math.min(left, vw - VIDEO_FILTER_MENU_WIDTH_PX - 12));
    const top = rect.bottom + gap;
    videoFilterMenuStyle = `top:${top}px;left:${left}px;`;
  }

  $effect(() => {
    if (!filterMenuOpen) {
      videoFilterMenuStyle = "";
      return;
    }
    void videoFilterButtonEl;
    const run = () => {
      updateVideoFilterMenuPosition();
    };
    void tick().then(run);
    const onLayout = () => run();
    window.addEventListener("resize", onLayout);
    window.addEventListener("scroll", onLayout, true);
    return () => {
      window.removeEventListener("resize", onLayout);
      window.removeEventListener("scroll", onLayout, true);
    };
  });

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") filterMenuOpen = false;
  }

  async function selectVideoType(value: VideoTypeFilter) {
    filterMenuOpen = false;
    try {
      await onSelectVideoType(value);
    } finally {
      await tick();
    }
  }

  async function selectAcknowledged(value: AcknowledgedFilter) {
    filterMenuOpen = false;
    try {
      await onSelectAcknowledged(value);
    } finally {
      await tick();
    }
  }

  async function clearAllFilters() {
    filterMenuOpen = false;
    try {
      await onClearAllFilters();
    } finally {
      await tick();
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div
  class="relative"
  style="z-index: var(--z-mobile-popover-anchor);"
  use:clickOutside={{
    enabled: filterMenuOpen,
    onClickOutside: () => (filterMenuOpen = false),
  }}
>
  <button
    type="button"
    id="video-filter-button"
    bind:this={videoFilterButtonEl}
    class={`relative inline-flex items-center justify-center rounded-full transition-colors ${size === "md" ? "h-9 w-9" : "h-5 w-5"} ${activeFilterCount > 0 || filterMenuOpen ? "bg-[var(--foreground)] text-[var(--background)]" : "text-[var(--soft-foreground)] opacity-55 hover:bg-[var(--accent-wash)] hover:opacity-100"}`}
    onclick={() => {
      filterMenuOpen = !filterMenuOpen;
    }}
    {disabled}
    aria-label="Video filters"
    aria-haspopup="menu"
    aria-expanded={filterMenuOpen}
  >
    <FilterIcon
      size={size === "md" ? 18 : 10}
      strokeWidth={size === "md" ? 2.2 : 2.5}
    />
    {#if activeFilterCount > 0}
      <span
        class={`absolute grid place-items-center rounded-full bg-[var(--accent)] font-bold leading-none text-white ${size === "md" ? "-right-0.5 -top-0.5 h-4 min-w-4 px-1 text-[9px]" : "-right-1 -top-1 h-3 min-w-3 px-0.5 text-[8px]"}`}
        aria-hidden="true"
      >
        {activeFilterCount}
      </span>
    {/if}
  </button>
  {#if filterMenuOpen}
    <div
      role="menu"
      aria-label="Video filters"
      style={`${videoFilterMenuStyle || "visibility:hidden;"}z-index:var(--z-mobile-popover);`}
      class="fixed w-52 overflow-hidden rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--surface-strong)] shadow-xl popover-rise"
    >
      <div class="space-y-4 p-2">
        <div class="grid gap-1">
          <p
            class="px-2 pb-1 text-[10px] font-bold text-[var(--soft-foreground)] opacity-50"
          >
            TYPE
          </p>
          {#each SIDEBAR_VIDEO_TYPE_OPTIONS as opt}
            <button
              type="button"
              role="menuitemradio"
              aria-checked={videoTypeFilter === opt.value}
              class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[14px] font-medium transition-colors ${videoTypeFilter === opt.value ? "text-[var(--foreground)]" : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"}`}
              onclick={() => void selectVideoType(opt.value)}
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
          {#each SIDEBAR_ACKNOWLEDGED_FILTER_OPTIONS as opt}
            <button
              type="button"
              role="menuitemradio"
              aria-checked={acknowledgedFilter === opt.value}
              class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[14px] font-medium transition-colors ${acknowledgedFilter === opt.value ? "text-[var(--foreground)]" : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"}`}
              onclick={() => void selectAcknowledged(opt.value)}
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
      {#if videoTypeFilter !== "all" || acknowledgedFilter !== "all"}
        <div class="border-t border-[var(--border-soft)] px-2 py-2">
          <button
            type="button"
            role="menuitem"
            class="w-full rounded-[var(--radius-sm)] px-3 py-2 text-left text-[11px] font-bold uppercase tracking-[0.06em] text-[var(--danger)] opacity-75 transition-colors hover:bg-[var(--accent-wash)] hover:opacity-100"
            onclick={() => void clearAllFilters()}
          >
            Clear filters
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>
