<script lang="ts">
  import WorkspaceSidebarSelectedVideoList from "$lib/components/workspace/WorkspaceSidebarSelectedVideoList.svelte";
  import WorkspaceSidebarSyncDateControl from "$lib/components/workspace/WorkspaceSidebarSyncDateControl.svelte";
  import { resolveDisplayedSyncDepthIso } from "$lib/sync-depth";
  import type { Channel, Video } from "$lib/types";
  import { formatSyncDate } from "$lib/workspace/content";
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
    readOnly,
    isVirtualChannel,
    syncDepth,
    allowLoadedVideoSyncDepthOverride,
    syncDateOpen,
    syncDateInputValue,
    savingSyncDate,
    syncDatePopupStackClass,
    syncDateWrapperClass,
    syncDateButtonClass,
    syncDateReadonlyClass,
    syncDateDialogClass,
    syncDateInputClass,
    syncDateSubmitClass,
    onSelectVideo,
    onLoadMoreVideos,
    onVideoMouseEnter,
    onVideoMouseLeave,
    onToggleSyncDate,
    onSyncDateInputValueChange,
    onSaveSyncDate,
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
    readOnly: boolean;
    isVirtualChannel: boolean;
    syncDepth: SyncDepth | null;
    allowLoadedVideoSyncDepthOverride: boolean;
    syncDateOpen: boolean;
    syncDateInputValue: string;
    savingSyncDate: boolean;
    syncDatePopupStackClass: string;
    syncDateWrapperClass: string;
    syncDateButtonClass: string;
    syncDateReadonlyClass: string;
    syncDateDialogClass: string;
    syncDateInputClass: string;
    syncDateSubmitClass: string;
    onSelectVideo: (videoId: string) => void | Promise<void>;
    onLoadMoreVideos: () => void | Promise<void>;
    onVideoMouseEnter: (videoId: string) => void;
    onVideoMouseLeave: () => void;
    onToggleSyncDate: () => void;
    onSyncDateInputValueChange: (value: string) => void;
    onSaveSyncDate: () => void | Promise<void>;
  } = $props();
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
    {#if selectedChannel}
      <WorkspaceSidebarSyncDateControl
        readOnly={readOnly || isVirtualChannel}
        open={syncDateOpen}
        label={formatSyncDate(
          resolveDisplayedSyncDepthIso({
            videos,
            selectedChannel,
            syncDepth,
            allowLoadedVideoOverride: allowLoadedVideoSyncDepthOverride,
          }),
        )}
        inputValue={syncDateInputValue}
        saving={savingSyncDate}
        popupStackClass={syncDatePopupStackClass}
        wrapperClass={syncDateWrapperClass}
        buttonClass={syncDateButtonClass}
        readonlyClass={syncDateReadonlyClass}
        dialogClass={syncDateDialogClass}
        inputClass={syncDateInputClass}
        submitClass={syncDateSubmitClass}
        onToggle={onToggleSyncDate}
        onInputValueChange={onSyncDateInputValueChange}
        onSubmit={() => void onSaveSyncDate()}
      />
    {/if}
  {/snippet}
</WorkspaceSidebarSelectedVideoList>
