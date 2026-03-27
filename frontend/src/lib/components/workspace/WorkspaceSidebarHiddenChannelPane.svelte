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
    readOnly,
    isVirtualChannel,
    syncDepth,
    allowLoadedVideoSyncDepthOverride,
    syncDateOpen,
    syncDateInputValue,
    savingSyncDate,
    syncDatePopupStackClass,
    onSelectVideo,
    onLoadMoreVideos,
    onVideoMouseEnter,
    onVideoMouseLeave,
    onToggleSyncDate,
    onSyncDateInputValueChange,
    onSaveSyncDate,
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
    readOnly: boolean;
    isVirtualChannel: boolean;
    syncDepth: SyncDepth | null;
    allowLoadedVideoSyncDepthOverride: boolean;
    syncDateOpen: boolean;
    syncDateInputValue: string;
    savingSyncDate: boolean;
    syncDatePopupStackClass: string;
    onSelectVideo: (videoId: string) => void | Promise<void>;
    onLoadMoreVideos: () => void | Promise<void>;
    onVideoMouseEnter: (videoId: string) => void;
    onVideoMouseLeave: () => void;
    onToggleSyncDate: () => void;
    onSyncDateInputValueChange: (value: string) => void;
    onSaveSyncDate: () => void | Promise<void>;
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
    {:else if loadingVideos && videos.length === 0}
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
    {:else if videos.length === 0 && !refreshingChannel}
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
        {readOnly}
        {isVirtualChannel}
        {syncDepth}
        {allowLoadedVideoSyncDepthOverride}
        {syncDateOpen}
        {syncDateInputValue}
        {savingSyncDate}
        {syncDatePopupStackClass}
        syncDateWrapperClass="relative z-10 mt-2 px-2"
        syncDateButtonClass="touch-manipulation relative z-10 inline-flex w-full max-w-full flex-wrap items-baseline gap-x-1 rounded-[var(--radius-sm)] px-2 py-1 text-left text-[10px] text-[var(--soft-foreground)] opacity-55 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
        syncDateReadonlyClass="mt-2 px-2 text-[10px] text-[var(--soft-foreground)] opacity-55"
        syncDateDialogClass="flex flex-wrap items-center gap-2 rounded-[var(--radius-md)] bg-[var(--surface-strong)] p-2 shadow-[var(--shadow-soft)]"
        syncDateInputClass="min-w-0 flex-1 rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-3 py-2 text-[12px] font-medium transition-colors focus:border-[var(--accent)]/40 focus:outline-none"
        syncDateSubmitClass="rounded-[var(--radius-sm)] bg-[var(--foreground)] px-3 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--background)] transition-all hover:bg-[var(--accent-strong)] disabled:opacity-30"
        emptyLabel="No videos yet."
        wrapperClass=""
        rowClassName="min-h-[56px]"
        onSelectVideo={(videoId) => void onSelectVideo(videoId)}
        onLoadMoreVideos={() => void onLoadMoreVideos()}
        {onVideoMouseEnter}
        {onVideoMouseLeave}
        {onToggleSyncDate}
        {onSyncDateInputValueChange}
        onSaveSyncDate={() => void onSaveSyncDate()}
      />
    {/if}
  </div>
</div>
