<script lang="ts">
  import { onMount } from "svelte";
  import VideoCard from "$lib/components/VideoCard.svelte";
  import { resolveDisplayedSyncDepthIso } from "$lib/sync-depth";
  import type { Channel, SyncDepth, Video, VideoTypeFilter } from "$lib/types";
  import { formatSyncDate } from "$lib/workspace/content";
  import type { AcknowledgedFilter } from "$lib/workspace/types";

  let {
    mobileVisible = false,
    selectedChannelId = null,
    selectedVideoId = null,
    selectedChannel = null,
    videos = [],
    loadingVideos = false,
    refreshingChannel = false,
    hasMore = true,
    historyExhausted = false,
    backfillingHistory = false,
    videoTypeFilter = "all",
    acknowledgedFilter = "all",
    syncDepth = null,
    allowLoadedVideoSyncDepthOverride = false,
    onSelectVideo = async () => {},
    onLoadMoreVideos = async () => {},
    onVideoTypeFilterChange = async () => {},
    onAcknowledgedFilterChange = async () => {},
  }: {
    mobileVisible?: boolean;
    selectedChannelId?: string | null;
    selectedVideoId?: string | null;
    selectedChannel?: Channel | null;
    videos?: Video[];
    loadingVideos?: boolean;
    refreshingChannel?: boolean;
    hasMore?: boolean;
    historyExhausted?: boolean;
    backfillingHistory?: boolean;
    videoTypeFilter?: VideoTypeFilter;
    acknowledgedFilter?: AcknowledgedFilter;
    syncDepth?: SyncDepth | null;
    allowLoadedVideoSyncDepthOverride?: boolean;
    onSelectVideo?: (videoId: string) => Promise<void> | void;
    onLoadMoreVideos?: () => Promise<void> | void;
    onVideoTypeFilterChange?: (value: VideoTypeFilter) => Promise<void> | void;
    onAcknowledgedFilterChange?: (
      value: AcknowledgedFilter,
    ) => Promise<void> | void;
  } = $props();

  let filterMenuOpen = $state(false);
  let filterMenuContainer = $state<HTMLDivElement | null>(null);

  let filterMenuLabel = $derived(
    videoTypeFilter === "all"
      ? "Open video filter menu."
      : `Video type filter set to ${videoTypeFilter}. Open filter menu.`,
  );

  onMount(() => {
    const handlePointerDown = (event: PointerEvent) => {
      if (
        filterMenuOpen &&
        filterMenuContainer &&
        !filterMenuContainer.contains(event.target as Node)
      ) {
        filterMenuOpen = false;
      }
    };

    document.addEventListener("pointerdown", handlePointerDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
    };
  });

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      filterMenuOpen = false;
    }
  }

  async function selectVideoTypeFilter(value: VideoTypeFilter) {
    filterMenuOpen = false;
    await onVideoTypeFilterChange(value);
  }

  async function selectAcknowledgedFilter(value: AcknowledgedFilter) {
    filterMenuOpen = false;
    await onAcknowledgedFilterChange(value);
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<aside
  class={`fade-in stagger-2 flex min-h-0 min-w-0 flex-col border-0 lg:sticky lg:top-4 lg:h-[calc(100vh-4rem)] lg:gap-3 lg:border-r lg:border-[var(--border-soft)] lg:px-5 ${mobileVisible ? "h-full gap-4 p-3" : "hidden lg:flex"}`}
  id="videos"
>
  <div class="flex flex-wrap items-center justify-between gap-3">
    <div class="flex min-w-0 items-center gap-2">
      <h2
        class="text-base font-bold tracking-tight text-[var(--soft-foreground)]"
      >
        Videos
      </h2>
      {#if refreshingChannel}
        <span
          class="h-3 w-3 animate-spin rounded-full border-[1.5px] border-[var(--border)] border-t-[var(--accent)]"
          role="status"
          aria-label="Syncing"
        ></span>
      {/if}
    </div>
    <div class="relative" bind:this={filterMenuContainer}>
      <button
        type="button"
        class={`group flex h-7 w-7 items-center justify-center rounded-full transition-all duration-200 ${videoTypeFilter !== "all" || acknowledgedFilter !== "all" || filterMenuOpen ? "bg-[var(--accent)] text-white" : "text-[var(--soft-foreground)] opacity-40 hover:opacity-80"} focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 disabled:opacity-20`}
        onclick={() => {
          filterMenuOpen = !filterMenuOpen;
        }}
        disabled={!selectedChannelId || loadingVideos}
        aria-label={filterMenuLabel}
        aria-haspopup="menu"
        aria-expanded={filterMenuOpen}
        aria-controls="video-filter-menu"
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="3" y1="6" x2="21" y2="6"></line>
          <line x1="7" y1="12" x2="17" y2="12"></line>
          <line x1="10" y1="18" x2="14" y2="18"></line>
        </svg>
      </button>
      {#if filterMenuOpen}
        <div
          id="video-filter-menu"
          role="menu"
          aria-label="Video filters"
          class="fade-in absolute right-0 top-full z-20 mt-2 w-56 overflow-hidden rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface)] shadow-xl"
        >
          <div class="space-y-4 p-2">
            <div class="grid gap-1">
              <p
                class="px-2 pb-1 text-[10px] font-bold text-[var(--soft-foreground)] opacity-50"
              >
                TYPE
              </p>
              <button
                type="button"
                role="menuitemradio"
                aria-checked={videoTypeFilter === "all"}
                class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === "all" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
                onclick={() => void selectVideoTypeFilter("all")}
              >
                <span>All Content</span>
                {#if videoTypeFilter === "all"}
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="3"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                {/if}
              </button>
              <button
                type="button"
                role="menuitemradio"
                aria-checked={videoTypeFilter === "long"}
                class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === "long" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
                onclick={() => void selectVideoTypeFilter("long")}
              >
                <span>Full Videos</span>
                {#if videoTypeFilter === "long"}
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="3"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                {/if}
              </button>
              <button
                type="button"
                role="menuitemradio"
                aria-checked={videoTypeFilter === "short"}
                class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === "short" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
                onclick={() => void selectVideoTypeFilter("short")}
              >
                <span>Shorts</span>
                {#if videoTypeFilter === "short"}
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="3"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                {/if}
              </button>
            </div>

            <div class="grid gap-1">
              <p
                class="px-2 pb-1 text-[10px] font-bold text-[var(--soft-foreground)] opacity-50"
              >
                STATUS
              </p>
              <button
                type="button"
                role="menuitemradio"
                aria-checked={acknowledgedFilter === "all"}
                class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === "all" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
                onclick={() => void selectAcknowledgedFilter("all")}
              >
                <span>All Statuses</span>
                {#if acknowledgedFilter === "all"}
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="3"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                {/if}
              </button>
              <button
                type="button"
                role="menuitemradio"
                aria-checked={acknowledgedFilter === "unack"}
                class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === "unack" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
                onclick={() => void selectAcknowledgedFilter("unack")}
              >
                <span>Unread</span>
                {#if acknowledgedFilter === "unack"}
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="3"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                {/if}
              </button>
              <button
                type="button"
                role="menuitemradio"
                aria-checked={acknowledgedFilter === "ack"}
                class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === "ack" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
                onclick={() => void selectAcknowledgedFilter("ack")}
              >
                <span>Read</span>
                {#if acknowledgedFilter === "ack"}
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="3"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                {/if}
              </button>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div
    class="custom-scrollbar mobile-bottom-stack-padding grid min-h-0 flex-1 gap-4 overflow-y-auto pr-1 lg:pb-0"
    aria-busy={loadingVideos}
  >
    {#if loadingVideos && videos.length === 0}
      {#each Array.from({ length: 3 }) as _, index (index)}
        <article
          class="flex min-h-[14rem] flex-col gap-4 rounded-[var(--radius-md)] bg-[var(--muted)]/30 p-4 animate-pulse"
        >
          <div
            class="aspect-video rounded-[var(--radius-sm)] bg-[var(--muted)] opacity-60"
          ></div>
          <div
            class="h-4 w-11/12 rounded-full bg-[var(--muted)] opacity-60"
          ></div>
          <div
            class="h-3 w-2/5 rounded-full bg-[var(--muted)] opacity-40"
          ></div>
        </article>
      {/each}
    {:else if videos.length === 0}
      <p
        class="px-1 text-[14px] font-medium italic text-[var(--soft-foreground)] opacity-50"
      >
        Waiting for the library to fill.
      </p>
    {:else}
      {#each videos as video}
        <VideoCard
          {video}
          active={selectedVideoId === video.id}
          onSelect={() => void onSelectVideo(video.id)}
        />
      {/each}
    {/if}

    {#if selectedChannelId}
      <div class="flex flex-col gap-3 pb-4 pt-1">
        {#if hasMore || !historyExhausted}
          <button
            type="button"
            class="w-full rounded-[var(--radius-sm)] border border-[var(--border-soft)] py-2.5 text-[11px] font-bold uppercase tracking-[0.15em] text-[var(--soft-foreground)] transition-all hover:border-[var(--accent)]/40 hover:text-[var(--foreground)] disabled:opacity-30"
            onclick={() => void onLoadMoreVideos()}
            disabled={loadingVideos || backfillingHistory}
          >
            {#if loadingVideos || backfillingHistory}
              Loading...
            {:else if hasMore}
              More
            {:else}
              Explore History
            {/if}
          </button>
        {/if}

        {#if videos.length > 0}
          <p
            class="px-0.5 text-[11px] text-[var(--soft-foreground)] opacity-40"
          >
            Synced to {formatSyncDate(
              resolveDisplayedSyncDepthIso({
                videos,
                selectedChannel,
                syncDepth,
                allowLoadedVideoOverride: allowLoadedVideoSyncDepthOverride,
              }),
            )}
          </p>
        {/if}
      </div>
    {/if}
  </div>
</aside>
