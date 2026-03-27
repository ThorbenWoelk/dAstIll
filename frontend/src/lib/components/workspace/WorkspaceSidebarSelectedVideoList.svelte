<script lang="ts">
  import type { Snippet } from "svelte";
  import { formatShortDate } from "$lib/utils/date";
  import type { Video } from "$lib/types";
  import WorkspaceSidebarVideoRow from "$lib/components/workspace/WorkspaceSidebarVideoRow.svelte";

  let {
    videos,
    selectedVideoId,
    pendingSelectedVideo = null,
    showPendingSelectedVideo = false,
    loadingVideos,
    refreshingChannel,
    hasMore,
    historyExhausted,
    backfillingHistory,
    suppressLoadMoreButton = false,
    emptyLabel = "No videos yet.",
    wrapperClass = "mt-1 pb-1",
    listId = undefined,
    rowClassName = "",
    footer,
    onSelectVideo,
    onLoadMoreVideos,
    onVideoMouseEnter,
    onVideoMouseLeave,
  }: {
    videos: Video[];
    selectedVideoId: string | null;
    pendingSelectedVideo?: Video | null;
    showPendingSelectedVideo?: boolean;
    loadingVideos: boolean;
    refreshingChannel: boolean;
    hasMore: boolean;
    historyExhausted: boolean;
    backfillingHistory: boolean;
    suppressLoadMoreButton?: boolean;
    emptyLabel?: string;
    wrapperClass?: string;
    listId?: string | undefined;
    rowClassName?: string;
    footer?: Snippet;
    onSelectVideo: (videoId: string) => void | Promise<void>;
    onLoadMoreVideos: () => void | Promise<void>;
    onVideoMouseEnter: (videoId: string) => void;
    onVideoMouseLeave: () => void;
  } = $props();
</script>

<div class={wrapperClass} id={listId}>
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
  {:else if videos.length === 0 && !refreshingChannel}
    <p
      class="px-3 py-2 text-[12px] italic text-[var(--soft-foreground)] opacity-50"
    >
      {emptyLabel}
    </p>
  {:else}
    {#if showPendingSelectedVideo && pendingSelectedVideo}
      <button
        type="button"
        class="group flex w-full items-center gap-2 rounded-[var(--radius-sm)] bg-[var(--accent-wash)] px-2 py-1.5 text-left transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
        onclick={() => void onSelectVideo(pendingSelectedVideo.id)}
      >
        <div class="min-w-0 flex-1">
          <p
            class="line-clamp-2 text-[12px] font-medium leading-tight tracking-tight text-[var(--foreground)]"
          >
            {pendingSelectedVideo.title}
          </p>
          <div class="mt-1 flex items-center gap-2">
            <span class="text-[10px] text-[var(--soft-foreground)] opacity-50">
              {formatShortDate(pendingSelectedVideo.published_at)}
            </span>
            <span class="text-[10px] font-medium text-[var(--accent-strong)]">
              Restoring selection...
            </span>
          </div>
        </div>
      </button>
    {/if}

    {#each videos as video (video.id)}
      <WorkspaceSidebarVideoRow
        {video}
        selected={selectedVideoId === video.id}
        className={rowClassName}
        onclick={() => void onSelectVideo(video.id)}
        onmouseenter={() => onVideoMouseEnter(video.id)}
        onmouseleave={onVideoMouseLeave}
      />
    {/each}

    {#if !suppressLoadMoreButton && (hasMore || !historyExhausted)}
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
      {@render footer?.()}
    {/if}
  {/if}
</div>
