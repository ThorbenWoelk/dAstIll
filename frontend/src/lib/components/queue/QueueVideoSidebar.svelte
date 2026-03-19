<script lang="ts">
  import Toggle from "$lib/components/Toggle.svelte";
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
  import type { Channel, QueueTab, Video } from "$lib/types";

  const SWIPE_BACK_THRESHOLD_PX = 72;
  const SWIPE_BACK_EDGE_PX = 32;

  interface DistillationStatusCopy {
    kind: "processing" | "queued" | "failed";
    label: string;
    detail: string;
  }

  interface QueueListItem {
    video: Video;
    distillationStatus: DistillationStatusCopy;
  }

  interface QueueStats {
    total: number;
    loading: number;
    pending: number;
    failed: number;
  }

  let {
    mobileVisible = false,
    selectedChannelId = null,
    selectedChannel = null,
    queueTab = "transcripts",
    loadingVideos = false,
    refreshingChannel = false,
    hasMore = true,
    lastSyncedAt = null,
    queueStats = { total: 0, loading: 0, pending: 0, failed: 0 },
    items = [],
    onBack = () => {},
    onQueueTabChange = () => {},
    onOpenVideo = async () => {},
    onLoadMoreVideos = async () => {},
  }: {
    mobileVisible?: boolean;
    selectedChannelId?: string | null;
    selectedChannel?: Channel | null;
    queueTab?: QueueTab;
    loadingVideos?: boolean;
    refreshingChannel?: boolean;
    hasMore?: boolean;
    lastSyncedAt?: Date | null;
    queueStats?: QueueStats;
    items?: QueueListItem[];
    onBack?: () => void;
    onQueueTabChange?: (value: QueueTab) => void;
    onOpenVideo?: (video: Video) => Promise<void> | void;
    onLoadMoreVideos?: () => Promise<void> | void;
  } = $props();

  let touchGesture = $state<{
    startX: number;
    startY: number;
    edgeStart: boolean;
    interactive: boolean;
  } | null>(null);

  const dateFormatter = new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });

  const syncTimeFormatter = new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
    second: "2-digit",
  });

  function isInteractiveSwipeTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) {
      return false;
    }

    return Boolean(
      target.closest(
        "button, a, input, textarea, select, label, [role='button'], [role='tab']",
      ),
    );
  }

  function handleSwipeStart(event: TouchEvent) {
    if (!mobileVisible || event.touches.length !== 1) {
      touchGesture = null;
      return;
    }

    const touch = event.touches[0];
    const edgeStart = touch.clientX <= SWIPE_BACK_EDGE_PX;
    touchGesture = {
      startX: touch.clientX,
      startY: touch.clientY,
      edgeStart,
      interactive: edgeStart ? false : isInteractiveSwipeTarget(event.target),
    };
  }

  function handleSwipeEnd(event: TouchEvent) {
    if (
      !touchGesture ||
      !touchGesture.edgeStart ||
      touchGesture.interactive ||
      !mobileVisible ||
      event.changedTouches.length !== 1
    ) {
      touchGesture = null;
      return;
    }

    const touch = event.changedTouches[0];
    const deltaX = touch.clientX - touchGesture.startX;
    const deltaY = touch.clientY - touchGesture.startY;

    touchGesture = null;

    if (
      deltaX < SWIPE_BACK_THRESHOLD_PX ||
      Math.abs(deltaX) <= Math.abs(deltaY) * 1.25
    ) {
      return;
    }

    onBack();
  }

  function formatDate(value: string) {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return "Date unavailable";
    return dateFormatter.format(date);
  }
</script>

<aside
  class={`fade-in stagger-2 flex min-h-0 min-w-0 flex-col border-0 lg:sticky lg:top-4 lg:h-[calc(100vh-4rem)] lg:gap-3 lg:border-r lg:border-[var(--accent-border-soft)] lg:px-5 ${mobileVisible ? "h-full gap-4 p-3" : "hidden lg:flex"}`}
  id="videos"
  ontouchstart={handleSwipeStart}
  ontouchend={handleSwipeEnd}
  ontouchcancel={() => {
    touchGesture = null;
  }}
>
  <div class="flex items-center justify-between gap-3 max-lg:flex-nowrap">
    <div class="flex min-w-0 items-center gap-1.5">
      <h2 class="text-base font-bold tracking-tight text-[var(--foreground)]">
        Queue
      </h2>
      {#if refreshingChannel}
        <span
          class="h-3 w-3 animate-spin rounded-full border-[1.5px] border-[var(--border)] border-t-[var(--accent)]"
          role="status"
          aria-label="Syncing"
        ></span>
      {/if}
    </div>
    {#if lastSyncedAt}
      <span
        class="text-[11px] font-medium text-[var(--soft-foreground)] opacity-60"
      >
        {syncTimeFormatter.format(lastSyncedAt)}
      </span>
    {/if}
  </div>

  {#if selectedChannel}
    <div
      class="flex min-w-0 items-center gap-3 border-b border-[var(--accent-border-soft)] pb-3"
    >
      <button
        type="button"
        onclick={onBack}
        class="inline-flex min-w-0 items-center gap-2 text-[13px] font-semibold text-[var(--foreground)] transition-transform active:scale-95 lg:pointer-events-none"
      >
        <div
          class="h-6 w-6 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]"
        >
          <img
            src={selectedChannel.thumbnail_url || defaultChannelIcon}
            alt=""
            class="h-full w-full object-cover"
          />
        </div>
        <span class="truncate">{selectedChannel.name}</span>
        <svg
          class="h-3 w-3 shrink-0 opacity-30 lg:hidden"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="3"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="m6 9 6 6 6-6" />
        </svg>
      </button>
    </div>

    <div aria-label="Queue tabs" class="min-w-0 overflow-x-auto pt-1">
      <Toggle
        ariaLabel="Queue tabs"
        options={["transcripts", "summaries", "evaluations"]}
        value={queueTab}
        showIcons={false}
        labels={{
          transcripts: "Transcripts",
          summaries: "Summaries",
          evaluations: "Evals",
        }}
        onChange={(value) => {
          onQueueTabChange(value as QueueTab);
        }}
      />
    </div>
  {/if}

  <div
    class="custom-scrollbar mobile-bottom-stack-padding flex-1 overflow-y-auto lg:pb-0"
  >
    {#if !selectedChannelId}
      <div
        class="flex h-full flex-col items-center justify-center py-20 text-center"
      >
        <div
          class="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-[var(--accent-soft)]/60 text-[var(--accent-strong)]"
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
            <rect x="3" y="4" width="6" height="16" rx="1.5" />
            <rect x="10" y="4" width="5" height="16" rx="1.5" />
            <rect x="16" y="4" width="5" height="16" rx="1.5" />
          </svg>
        </div>
        <p class="mt-4 text-[16px] font-semibold text-[var(--foreground)]">
          Select a channel
        </p>
        <p
          class="mt-2 max-w-[24rem] text-[13px] leading-6 text-[var(--soft-foreground)]"
        >
          Choose a followed channel to inspect queued transcripts, summaries,
          and evaluations.
        </p>
      </div>
    {:else if loadingVideos && items.length === 0}
      <div class="mt-4 space-y-4" role="status" aria-live="polite">
        {#each Array.from({ length: 6 }) as _, index (index)}
          <div
            class="animate-pulse rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] p-6"
          >
            <div
              class="h-4 w-3/4 rounded-full bg-[var(--border)] opacity-80"
            ></div>
            <div
              class="mt-4 h-3 w-1/4 rounded-full bg-[var(--border)] opacity-60"
            ></div>
          </div>
        {/each}
      </div>
    {:else if queueStats.total === 0}
      <div class="flex flex-col items-center justify-center py-20 text-center">
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="mb-3 text-emerald-500"
        >
          <polyline points="20 6 9 17 4 12" />
        </svg>
        <p
          class="text-[14px] font-medium text-[var(--soft-foreground)] opacity-70"
        >
          Queue is clear.
        </p>
      </div>
    {:else}
      <div
        class="flex items-center gap-4 border-b border-[var(--accent-border-soft)] py-2 text-[11px] font-bold tabular-nums"
      >
        <span class="text-[var(--soft-foreground)] opacity-70">
          {queueStats.total} items
        </span>
        {#if queueStats.loading > 0}
          <span class="flex items-center gap-1.5 text-amber-600">
            <span class="h-1.5 w-1.5 animate-pulse rounded-full bg-amber-500"
            ></span>
            {queueStats.loading} active
          </span>
        {/if}
        {#if queueStats.failed > 0}
          <span class="text-[var(--danger-foreground)]">
            {queueStats.failed} failed
          </span>
        {/if}
      </div>

      <ul class="mt-2 flex flex-col">
        {#each items as item}
          {@const video = item.video}
          <li
            class="border-b border-[var(--accent-border-soft)] last:border-b-0"
          >
            <button
              type="button"
              class="group w-full cursor-pointer px-1 py-4 text-left focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 max-lg:py-3"
              onclick={() => void onOpenVideo(video)}
              aria-label={`Open transcript workspace for ${video.title}`}
            >
              <p
                class="line-clamp-2 text-[14px] font-semibold leading-[1.4] tracking-tight text-[var(--foreground)] transition-colors group-hover:text-[var(--accent-strong)]"
              >
                {video.title}
              </p>
              <div class="mt-2 flex flex-wrap items-center gap-3">
                <span
                  class="text-[11px] font-medium text-[var(--soft-foreground)] opacity-50"
                >
                  {formatDate(video.published_at)}
                </span>
                {#if item.distillationStatus.kind === "processing"}
                  <span
                    class="inline-flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--accent)]"
                  >
                    <span class="relative flex h-1.5 w-1.5">
                      <span
                        class="absolute inline-flex h-full w-full animate-ping rounded-full bg-[var(--accent)] opacity-75"
                      ></span>
                      <span
                        class="relative inline-flex h-1.5 w-1.5 rounded-full bg-[var(--accent)]"
                      ></span>
                    </span>
                    Processing
                  </span>
                {:else if item.distillationStatus.kind === "failed"}
                  <span
                    class="inline-flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--danger-foreground)]"
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
                      <circle cx="12" cy="12" r="10" />
                      <line x1="12" y1="8" x2="12" y2="12" />
                      <line x1="12" y1="16" x2="12.01" y2="16" />
                    </svg>
                    Failed
                  </span>
                {:else}
                  <span
                    class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-40"
                  >
                    Queued
                  </span>
                {/if}
              </div>
            </button>
          </li>
        {/each}
      </ul>
    {/if}

    {#if hasMore && selectedChannelId}
      <div class="mt-4 flex justify-center">
        <button
          type="button"
          class="w-full rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] py-2.5 text-[11px] font-bold uppercase tracking-[0.15em] text-[var(--soft-foreground)] transition-all hover:border-[var(--accent)]/40 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] disabled:opacity-30"
          onclick={() => void onLoadMoreVideos()}
          disabled={loadingVideos}
        >
          {loadingVideos ? "Loading..." : "Load More"}
        </button>
      </div>
    {/if}
  </div>
</aside>
