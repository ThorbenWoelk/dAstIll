<script lang="ts">
  import WorkspaceSidebarSyncDateControl from "$lib/components/workspace/WorkspaceSidebarSyncDateControl.svelte";
  import WorkspaceSidebarVideoRow from "$lib/components/workspace/WorkspaceSidebarVideoRow.svelte";
  import { resolveDisplayedSyncDepthIso } from "$lib/sync-depth";
  import { OTHERS_CHANNEL_ID, type Channel, type Video } from "$lib/types";
  import { formatSyncDate } from "$lib/workspace/content";
  import { shouldShowPagedCollectionSyncSettingsLink } from "$lib/workspace/sidebar-sync-boundary-link";
  import type {
    ChannelVideoCollectionState,
    RenderedCollectionVideos,
  } from "$lib/workspace/sidebar-preview-controller.svelte";

  let {
    channel,
    channelVideoCollection,
    renderedCollection,
    selectedVideoId,
    scrollIntoViewWhenSelected,
    emptyCaption,
    onChannelVideoClick,
    onVideoMouseEnter,
    onVideoMouseLeave,
    onCollectionScroll,
    onLoadMore,
  }: {
    channel: Channel;
    channelVideoCollection: ChannelVideoCollectionState;
    renderedCollection: RenderedCollectionVideos;
    selectedVideoId: string | null;
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
  } = $props();

  const showSyncSettingsLink = $derived(
    shouldShowPagedCollectionSyncSettingsLink({
      videosCount: channelVideoCollection.videos.length,
      hasMore: channelVideoCollection.hasMore,
      loadingInitial: channelVideoCollection.loadingInitial,
      loadingMore: channelVideoCollection.loadingMore,
      isVirtualChannel: channel.id === OTHERS_CHANNEL_ID,
    }),
  );
</script>

<div
  class={channelVideoCollection.loadedMode === "paged"
    ? "mt-1 max-h-[21rem] overflow-y-auto pb-1 pr-1 [overscroll-behavior-y:contain]"
    : "mt-1 pb-1"}
  id={selectedVideoId ? "videos" : undefined}
  data-channel-video-list={channel.id}
  onscroll={onCollectionScroll}
>
  {#if channelVideoCollection.loadingInitial && channelVideoCollection.videos.length === 0}
    <div class="space-y-1 px-1" role="status" aria-live="polite">
      {#each Array.from({ length: 4 }) as _, i (i)}
        <div class="animate-pulse px-3 py-2">
          <div class="h-3 w-11/12 rounded bg-[var(--border)] opacity-50"></div>
          <div
            class="mt-1.5 h-2 w-1/3 rounded bg-[var(--border)] opacity-30"
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

    {#each renderedCollection.videos as video, index (video.id)}
      <div
        class="preview-video-enter"
        style={`animation-delay: ${Math.min(index, 8) * 80}ms;`}
        use:scrollIntoViewWhenSelected={selectedVideoId === video.id}
      >
        <WorkspaceSidebarVideoRow
          {video}
          selected={selectedVideoId === video.id}
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
      <div class="space-y-1 px-1 pt-1" role="status" aria-live="polite">
        {#each Array.from({ length: 2 }) as _, i (i)}
          <div class="animate-pulse px-3 py-2">
            <div
              class="h-3 w-11/12 rounded bg-[var(--border)] opacity-50"
            ></div>
            <div
              class="mt-1.5 h-2 w-1/3 rounded bg-[var(--border)] opacity-30"
            ></div>
          </div>
        {/each}
      </div>
    {/if}

    {#if channelVideoCollection.loadedMode === "paged" && channelVideoCollection.hasMore && !channelVideoCollection.loadingMore}
      <button
        type="button"
        class="mt-1 w-full rounded-md py-1.5 text-[11px] font-medium text-[var(--soft-foreground)] transition-colors hover:bg-[var(--surface)] hover:text-[var(--foreground)]"
        onclick={() => void onLoadMore()}
      >
        Load more
      </button>
    {/if}

    {#if channelVideoCollection.loadedMode === "paged" && showSyncSettingsLink}
      <WorkspaceSidebarSyncDateControl
        channelId={channel.id}
        label={formatSyncDate(
          resolveDisplayedSyncDepthIso({
            videos: channelVideoCollection.videos,
            selectedChannel: channel,
            syncDepth: channelVideoCollection.syncDepth,
            allowLoadedVideoOverride: true,
          }),
        )}
        wrapperClass="relative z-10 mt-2 px-2 pb-4"
        linkClass="touch-manipulation relative z-10 inline-flex w-full max-w-full flex-wrap items-baseline gap-x-1 gap-y-0.5 rounded-[var(--radius-sm)] px-2 py-1 text-left text-[10px] text-[var(--soft-foreground)] opacity-50 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
      />
    {/if}
  {/if}
</div>

<style>
  .preview-video-enter {
    opacity: 0;
    transform: translateY(6px);
    animation: preview-video-enter 320ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  @keyframes preview-video-enter {
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .preview-video-enter {
      opacity: 1;
      transform: none;
      animation: none;
    }
  }
</style>
