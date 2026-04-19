<script lang="ts">
  import WorkspaceSidebarSelectedChannelContent from "$lib/components/workspace/WorkspaceSidebarSelectedChannelContent.svelte";
  import type { Channel, Video } from "$lib/types";
  import type { SyncDepth } from "$lib/types";

  let {
    selectedChannelId,
    selectedChannel,
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
    isVirtualChannel,
    syncDepth,
    allowLoadedVideoSyncDepthOverride,
    onSelectVideo,
    onLoadMoreVideos,
    onVideoMouseEnter,
    onVideoMouseLeave,
  }: {
    selectedChannelId: string | null;
    selectedChannel: Channel | null;
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
    isVirtualChannel: boolean;
    syncDepth: SyncDepth | null;
    allowLoadedVideoSyncDepthOverride: boolean;
    onSelectVideo: (videoId: string) => void | Promise<void>;
    onLoadMoreVideos: () => void | Promise<void>;
    onVideoMouseEnter: (videoId: string) => void;
    onVideoMouseLeave: () => void;
  } = $props();
</script>

<div class="flex min-h-0 flex-1 flex-col overflow-hidden">
  <div
    class="custom-scrollbar min-h-0 flex-1 overflow-y-auto px-3 pb-4 pt-3"
    aria-busy={loadingVideos}
  >
    {#if !selectedChannelId}
      <p
        class="px-2 py-2 text-[14px] font-medium italic text-[var(--soft-foreground)] opacity-55"
      >
        Pick a channel above.
      </p>
    {:else if (loadingVideos || refreshingChannel) && videos.length === 0}
      <div class="space-y-1 px-1" role="status" aria-live="polite">
        {#each Array.from({ length: 6 }) as _, i (i)}
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
        class="px-2 py-2 text-[14px] font-medium italic text-[var(--soft-foreground)] opacity-55"
      >
        No videos yet.
      </p>
    {:else}
      <WorkspaceSidebarSelectedChannelContent
        {videos}
        {selectedVideoId}
        {pendingSelectedVideo}
        {showPendingSelectedVideo}
        {loadingVideos}
        {refreshingChannel}
        {hasMore}
        {historyExhausted}
        {backfillingHistory}
        {suppressLoadMoreButton}
        {selectedChannel}
        {isVirtualChannel}
        {syncDepth}
        {allowLoadedVideoSyncDepthOverride}
        syncDateWrapperClass="relative z-10 mt-2 px-2"
        syncDateLinkClass="touch-manipulation relative z-10 inline-flex w-full max-w-full flex-wrap items-baseline gap-x-1 gap-y-0.5 rounded-[var(--radius-sm)] px-2 py-1 text-left text-[10px] text-[var(--soft-foreground)] opacity-55 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
        emptyLabel="No videos yet."
        wrapperClass=""
        rowClassName="min-h-[56px]"
        onSelectVideo={(videoId) => void onSelectVideo(videoId)}
        onLoadMoreVideos={() => void onLoadMoreVideos()}
        {onVideoMouseEnter}
        {onVideoMouseLeave}
      />
    {/if}
  </div>
</div>
