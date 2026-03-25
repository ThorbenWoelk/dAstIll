<script lang="ts">
  import { tick } from "svelte";
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
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
    onSelectVideoType,
    onSelectAcknowledged,
    onClearAllFilters,
  }: {
    videoTypeFilter: VideoTypeFilter;
    acknowledgedFilter: AcknowledgedFilter;
    disabled?: boolean;
    onSelectVideoType: (value: VideoTypeFilter) => void | Promise<void>;
    onSelectAcknowledged: (value: AcknowledgedFilter) => void | Promise<void>;
    onClearAllFilters: () => void | Promise<void>;
  } = $props();

  let filterMenuOpen = $state(false);
  let videoFilterButtonEl = $state<HTMLButtonElement | null>(null);
  let videoFilterMenuStyle = $state("");
  const VIDEO_FILTER_MENU_WIDTH_PX = 208;

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
    void tick().then(() => {
      run();
      requestAnimationFrame(() => {
        run();
        requestAnimationFrame(run);
      });
    });
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
    await onSelectVideoType(value);
  }

  async function selectAcknowledged(value: AcknowledgedFilter) {
    filterMenuOpen = false;
    await onSelectAcknowledged(value);
  }

  async function clearAllFilters() {
    filterMenuOpen = false;
    await onClearAllFilters();
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

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
    bind:this={videoFilterButtonEl}
    class={`inline-flex h-6 w-6 items-center justify-center rounded-full transition-colors ${videoTypeFilter !== "all" || acknowledgedFilter !== "all" || filterMenuOpen ? "bg-[var(--accent)] text-white" : "text-[var(--soft-foreground)] opacity-55 hover:bg-[var(--accent-wash)] hover:opacity-100"}`}
    onclick={() => {
      filterMenuOpen = !filterMenuOpen;
    }}
    {disabled}
    aria-label="Video filters"
    aria-haspopup="menu"
    aria-expanded={filterMenuOpen}
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
      style={videoFilterMenuStyle || "visibility:hidden"}
      class="fixed z-[90] w-52 overflow-hidden rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--surface-strong)] shadow-xl popover-rise"
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
              class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[14px] font-medium transition-colors ${videoTypeFilter === opt.value ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"}`}
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
