<script lang="ts">
  import WorkspaceSidebarSyncDateControl from "$lib/components/workspace/WorkspaceSidebarSyncDateControl.svelte";
  import WorkspaceSidebarVideoRow from "$lib/components/workspace/WorkspaceSidebarVideoRow.svelte";
  import { resolveDisplayedSyncDepthIso } from "$lib/sync-depth";
  import type { Channel, Video } from "$lib/types";
  import { formatSyncDate } from "$lib/workspace/content";
  import type {
    ChannelVideoCollectionState,
    RenderedCollectionVideos,
  } from "$lib/workspace/sidebar-preview-controller.svelte";

  let {
    channel,
    channelVideoCollection,
    renderedCollection,
    selectedVideoId,
    previewSyncDatePickerChannelId,
    readOnly,
    syncDatePopupStackClass,
    scrollIntoViewWhenSelected,
    emptyCaption,
    onChannelVideoClick,
    onVideoMouseEnter,
    onVideoMouseLeave,
    onCollectionScroll,
    onLoadMore,
    onToggleSyncDatePicker,
    onEarliestSyncDateInputChange,
    onSaveSyncDate,
  }: {
    channel: Channel;
    channelVideoCollection: ChannelVideoCollectionState;
    renderedCollection: RenderedCollectionVideos;
    selectedVideoId: string | null;
    previewSyncDatePickerChannelId: string | null;
    readOnly: boolean;
    syncDatePopupStackClass: string;
    scrollIntoViewWhenSelected: (
      node: HTMLElement,
      selected: boolean,
    ) => { update: (selected: boolean) => void };
    emptyCaption: string;
    onChannelVideoClick: (
      channelId: string,
      videoId: string,
      video?: Video,
    ) => void | Promise<void>;
    onVideoMouseEnter: (videoId: string) => void;
    onVideoMouseLeave: () => void;
    onCollectionScroll: (event: Event) => void;
    onLoadMore: () => void | Promise<void>;
    onToggleSyncDatePicker: () => void;
    onEarliestSyncDateInputChange: (value: string) => void;
    onSaveSyncDate: () => void | Promise<void>;
  } = $props();
</script>

<div
  class={channelVideoCollection.loadedMode === "paged"
    ? "mt-1 max-h-[21rem] overflow-y-auto pb-1 pr-1"
    : "mt-1 pb-1"}
  id={selectedVideoId ? "videos" : undefined}
  onscroll={onCollectionScroll}
>
  {#if channelVideoCollection.loadingInitial && channelVideoCollection.videos.length === 0}
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
  {:else if channelVideoCollection.videos.length === 0 && !channelVideoCollection.requestKey}
    <p
      class="px-3 py-2 text-[12px] italic text-[var(--soft-foreground)] opacity-50"
    >
      {emptyCaption}
    </p>
  {:else}
    {#if renderedCollection.virtualized}
      <div
        aria-hidden="true"
        style={`height:${renderedCollection.topSpacer}px;`}
      ></div>
    {/if}

    {#each renderedCollection.videos as video (video.id)}
      <div use:scrollIntoViewWhenSelected={selectedVideoId === video.id}>
        <WorkspaceSidebarVideoRow
          {video}
          selected={selectedVideoId === video.id}
          className="min-h-[56px]"
          onclick={() => void onChannelVideoClick(channel.id, video.id, video)}
          onmouseenter={() => onVideoMouseEnter(video.id)}
          onmouseleave={onVideoMouseLeave}
        />
      </div>
    {/each}

    {#if renderedCollection.virtualized}
      <div
        aria-hidden="true"
        style={`height:${renderedCollection.bottomSpacer}px;`}
      ></div>
    {/if}

    {#if channelVideoCollection.loadingMore}
      <p class="px-2 pt-2 text-[10px] text-[var(--soft-foreground)] opacity-50">
        Loading videos...
      </p>
    {/if}

    {#if channelVideoCollection.loadedMode === "paged" && channelVideoCollection.hasMore && !channelVideoCollection.loadingMore}
      <button
        type="button"
        class="mt-1 w-full rounded-[var(--radius-sm)] py-1.5 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
        onclick={() => void onLoadMore()}
      >
        Load more
      </button>
    {/if}
  {/if}
</div>

{#if channelVideoCollection.loadedMode === "paged"}
  <div class="relative z-10 mt-2 px-2 pb-4">
    <WorkspaceSidebarSyncDateControl
      {readOnly}
      open={previewSyncDatePickerChannelId === channel.id}
      label={formatSyncDate(
        resolveDisplayedSyncDepthIso({
          videos: channelVideoCollection.videos,
          selectedChannel: channel,
          syncDepth: channelVideoCollection.syncDepth,
          allowLoadedVideoOverride: true,
        }),
      )}
      inputValue={channelVideoCollection.earliestSyncDateInput}
      saving={channelVideoCollection.savingSyncDate}
      popupStackClass={syncDatePopupStackClass}
      buttonClass="touch-manipulation relative z-10 inline-flex w-full max-w-full flex-wrap items-baseline gap-x-1 rounded-[var(--radius-sm)] px-2 py-1 text-left text-[10px] text-[var(--soft-foreground)] opacity-50 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
      readonlyClass="text-[10px] text-[var(--soft-foreground)] opacity-50"
      dialogClass="flex flex-wrap items-center gap-2 rounded-[var(--radius-md)] bg-[var(--surface-strong)] p-2 shadow-[var(--shadow-soft)]"
      inputClass="min-w-0 flex-1 rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-3 py-2 text-[12px] font-medium transition-colors focus:border-[var(--accent)]/40 focus:outline-none"
      submitClass="rounded-[var(--radius-sm)] bg-[var(--foreground)] px-3 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--background)] transition-all hover:bg-[var(--accent-strong)] disabled:opacity-30"
      onToggle={onToggleSyncDatePicker}
      onInputValueChange={onEarliestSyncDateInputChange}
      onSubmit={() => void onSaveSyncDate()}
    />
  </div>
{/if}
