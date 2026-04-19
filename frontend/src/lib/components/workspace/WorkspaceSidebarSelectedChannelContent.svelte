<script lang="ts">
  import WorkspaceSidebarSelectedVideoList from "$lib/components/workspace/WorkspaceSidebarSelectedVideoList.svelte";
  import WorkspaceSidebarSyncDateControl from "$lib/components/workspace/WorkspaceSidebarSyncDateControl.svelte";
  import { resolveDisplayedSyncDepthIso } from "$lib/sync-depth";
  import type { Channel, Video } from "$lib/types";
  import { formatSyncDate } from "$lib/workspace/content";
  import { shouldShowSelectedChannelSyncSettingsLink } from "$lib/workspace/sidebar-sync-boundary-link";
  import type { SyncDepth } from "$lib/types";

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
    selectedChannel,
    isVirtualChannel,
    syncDepth,
    allowLoadedVideoSyncDepthOverride,
    syncDateWrapperClass,
    syncDateLinkClass,
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
    selectedChannel: Channel | null;
    isVirtualChannel: boolean;
    syncDepth: SyncDepth | null;
    allowLoadedVideoSyncDepthOverride: boolean;
    syncDateWrapperClass: string;
    syncDateLinkClass: string;
    onSelectVideo: (videoId: string) => void | Promise<void>;
    onLoadMoreVideos: () => void | Promise<void>;
    onVideoMouseEnter: (videoId: string) => void;
    onVideoMouseLeave: () => void;
  } = $props();

  const showSyncSettingsLink = $derived(
    shouldShowSelectedChannelSyncSettingsLink({
      videosCount: videos.length,
      hasMore,
      historyExhausted,
      loadingVideos,
      backfillingHistory,
      isVirtualChannel,
    }),
  );
</script>

<WorkspaceSidebarSelectedVideoList
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
  {emptyLabel}
  {wrapperClass}
  {listId}
  {rowClassName}
  onSelectVideo={(videoId) => void onSelectVideo(videoId)}
  onLoadMoreVideos={() => void onLoadMoreVideos()}
  {onVideoMouseEnter}
  {onVideoMouseLeave}
>
  {#snippet footer()}
    {#if selectedChannel && showSyncSettingsLink}
      <WorkspaceSidebarSyncDateControl
        channelId={selectedChannel.id}
        label={formatSyncDate(
          resolveDisplayedSyncDepthIso({
            videos,
            selectedChannel,
            syncDepth,
            allowLoadedVideoOverride: allowLoadedVideoSyncDepthOverride,
          }),
        )}
        wrapperClass={syncDateWrapperClass}
        linkClass={syncDateLinkClass}
      />
    {/if}
  {/snippet}
</WorkspaceSidebarSelectedVideoList>
