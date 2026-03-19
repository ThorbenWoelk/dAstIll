<script lang="ts">
  import { clickOutside } from "$lib/actions/click-outside";
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import VideoCard from "$lib/components/VideoCard.svelte";
  import { resolveDisplayedSyncDepthIso } from "$lib/sync-depth";
  import type { Channel, SyncDepth, Video, VideoTypeFilter } from "$lib/types";
  import { formatSyncDate } from "$lib/workspace/content";
  import type { AcknowledgedFilter } from "$lib/workspace/types";
  import { swipeBack } from "$lib/mobile-shell/swipe";

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
    onBack = () => {},
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
    onBack?: () => void;
    onSelectVideo?: (videoId: string) => Promise<void> | void;
    onLoadMoreVideos?: () => Promise<void> | void;
    onVideoTypeFilterChange?: (value: VideoTypeFilter) => Promise<void> | void;
    onAcknowledgedFilterChange?: (
      value: AcknowledgedFilter,
    ) => Promise<void> | void;
  } = $props();

  let filterMenuOpen = $state(false);
  let activeFilterLabel = $derived.by(() => {
    const labels: string[] = [];

    if (videoTypeFilter !== "all") {
      labels.push(videoTypeFilter === "long" ? "Full videos" : "Shorts");
    }

    if (acknowledgedFilter !== "all") {
      labels.push(acknowledgedFilter === "ack" ? "Read" : "Unread");
    }

    return labels.join(" · ");
  });

  let filterMenuLabel = $derived(
    videoTypeFilter === "all"
      ? "Open video filter menu."
      : `Video type filter set to ${videoTypeFilter}. Open filter menu.`,
  );

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
  class={`fade-in stagger-2 flex min-h-0 min-w-0 flex-col border-0 lg:sticky lg:top-4 lg:h-[calc(100vh-4rem)] lg:gap-3 lg:border-r lg:border-[var(--accent-border-soft)] lg:px-5 ${mobileVisible ? "h-full gap-4 p-3" : "hidden lg:flex"}`}
  id="videos"
  use:swipeBack={{ enabled: mobileVisible, onBack }}
>
  <div class="flex items-center justify-between gap-3 max-lg:flex-nowrap">
    <div class="flex min-w-0 items-center gap-1.5">
      <h2 class="text-base font-bold tracking-tight text-[var(--foreground)]">
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
        class={`group flex h-8 min-w-8 items-center justify-center gap-1 rounded-full px-2 transition-all duration-200 ${videoTypeFilter !== "all" || acknowledgedFilter !== "all" || filterMenuOpen ? "bg-[var(--accent)] text-white" : "text-[var(--soft-foreground)] opacity-60 hover:opacity-90"} focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 disabled:opacity-20`}
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
        {#if activeFilterLabel}
          <span
            class="hidden max-w-[8rem] truncate text-[10px] font-bold uppercase tracking-[0.08em] sm:inline"
          >
            {activeFilterLabel}
          </span>
        {/if}
      </button>
      {#if filterMenuOpen}
        <div
          id="video-filter-menu"
          role="menu"
          aria-label="Video filters"
          class="fade-in absolute right-0 top-full z-20 mt-2 w-56 overflow-hidden rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface-strong)] shadow-xl"
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
                class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === "all" ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"}`}
                onclick={() => void selectVideoTypeFilter("all")}
              >
                <span>All Content</span>
                {#if videoTypeFilter === "all"}
                  <CheckIcon size={12} strokeWidth={3} />
                {/if}
              </button>
              <button
                type="button"
                role="menuitemradio"
                aria-checked={videoTypeFilter === "long"}
                class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === "long" ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"}`}
                onclick={() => void selectVideoTypeFilter("long")}
              >
                <span>Full Videos</span>
                {#if videoTypeFilter === "long"}
                  <CheckIcon size={12} strokeWidth={3} />
                {/if}
              </button>
              <button
                type="button"
                role="menuitemradio"
                aria-checked={videoTypeFilter === "short"}
                class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === "short" ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"}`}
                onclick={() => void selectVideoTypeFilter("short")}
              >
                <span>Shorts</span>
                {#if videoTypeFilter === "short"}
                  <CheckIcon size={12} strokeWidth={3} />
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
                class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === "all" ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"}`}
                onclick={() => void selectAcknowledgedFilter("all")}
              >
                <span>All Statuses</span>
                {#if acknowledgedFilter === "all"}
                  <CheckIcon size={12} strokeWidth={3} />
                {/if}
              </button>
              <button
                type="button"
                role="menuitemradio"
                aria-checked={acknowledgedFilter === "unack"}
                class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === "unack" ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"}`}
                onclick={() => void selectAcknowledgedFilter("unack")}
              >
                <span>Unread</span>
                {#if acknowledgedFilter === "unack"}
                  <CheckIcon size={12} strokeWidth={3} />
                {/if}
              </button>
              <button
                type="button"
                role="menuitemradio"
                aria-checked={acknowledgedFilter === "ack"}
                class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === "ack" ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"}`}
                onclick={() => void selectAcknowledgedFilter("ack")}
              >
                <span>Read</span>
                {#if acknowledgedFilter === "ack"}
                  <CheckIcon size={12} strokeWidth={3} />
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
      {#each Array.from({ length: 5 }) as _, index (index)}
        <article
          class="flex min-h-[14rem] flex-col gap-4 rounded-[var(--radius-md)] bg-[var(--muted)]/30 p-4 animate-pulse"
        >
          <div
            class="aspect-video rounded-[var(--radius-sm)] bg-[var(--border)] opacity-80"
          ></div>
          <div
            class="h-4 w-11/12 rounded-full bg-[var(--border)] opacity-80"
          ></div>
          <div
            class="h-3 w-2/5 rounded-full bg-[var(--border)] opacity-60"
          ></div>
        </article>
      {/each}
    {:else if videos.length === 0}
      <div class="flex flex-1 items-center justify-center py-12">
        <div class="max-w-[17rem] text-center">
          <div
            class="mx-auto flex h-10 w-10 items-center justify-center text-[var(--soft-foreground)] opacity-65"
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
            >
              <polygon points="7,5 19,12 7,19" />
            </svg>
          </div>
          <p class="mt-4 text-[15px] font-medium text-[var(--foreground)]">
            No videos yet
          </p>
          <p
            class="mt-2 text-[13px] leading-6 text-[var(--soft-foreground)] opacity-80"
          >
            Select a channel to browse its videos and queue up your next
            distillation.
          </p>
        </div>
      </div>
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
            class="w-full rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] py-2.5 text-[11px] font-bold uppercase tracking-[0.15em] text-[var(--soft-foreground)] transition-all hover:border-[var(--accent)]/40 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] disabled:opacity-30"
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
            class="flex items-center gap-1.5 px-0.5 text-[11px] text-[var(--soft-foreground)] opacity-60"
          >
            <svg
              width="10"
              height="10"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <path d="M12 6v6l4 2" />
              <circle cx="12" cy="12" r="9" />
            </svg>
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
