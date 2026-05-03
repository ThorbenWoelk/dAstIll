<script lang="ts">
  import ChannelCard from "$lib/components/ChannelCard.svelte";
  import type { Channel } from "$lib/types";

  let {
    channel,
    isExpanded,
    isPreviewMode,
    isVirtualChannel,
    mobileVisible,
    manualReorderEnabled,
    draggedChannelId,
    dragOverChannelId,
    dropIndicatorEdge = null,
    channelUiHidden,
    loadingVideos,
    refreshingChannel,
    videoCount,
    isStickyHeader = false,
    isPagedExpanded = false,
    onSelect,
    onToggleExpanded = () => {},
    onDragStart,
    onDragOver,
    onDrop,
    onDragEnd,
  }: {
    channel: Channel;
    isExpanded: boolean;
    isPreviewMode: boolean;
    isVirtualChannel: boolean;
    mobileVisible: boolean;
    manualReorderEnabled: boolean;
    draggedChannelId: string | null;
    dragOverChannelId: string | null;
    dropIndicatorEdge?: "top" | "bottom" | null;
    channelUiHidden: boolean;
    loadingVideos: boolean;
    refreshingChannel: boolean;
    videoCount: number;
    isStickyHeader?: boolean;
    isPagedExpanded?: boolean;
    onSelect: () => void | Promise<void>;
    onToggleExpanded?: () => void | Promise<void>;
    onDragStart: (event: DragEvent) => void;
    onDragOver: (event: DragEvent) => void;
    onDrop: (event: DragEvent) => void;
    onDragEnd: () => void;
  } = $props();
</script>

<div class="relative" data-channel-id={channel.id} role="listitem">
  {#if dropIndicatorEdge === "top"}
    <div
      class="pointer-events-none absolute inset-x-3 -top-1 z-10 flex items-center gap-2"
    >
      <span class="h-2 w-2 rounded-full bg-[var(--accent)]"></span>
      <span class="h-0.5 flex-1 rounded-full bg-[var(--accent)]"></span>
    </div>
  {/if}
  {#if dropIndicatorEdge === "bottom"}
    <div
      class="pointer-events-none absolute inset-x-3 -bottom-1 z-10 flex items-center gap-2"
    >
      <span class="h-2 w-2 rounded-full bg-[var(--accent)]"></span>
      <span class="h-0.5 flex-1 rounded-full bg-[var(--accent)]"></span>
    </div>
  {/if}

  {#if !isPreviewMode && isExpanded && (refreshingChannel || (loadingVideos && videoCount === 0))}
    <div class="flex items-center gap-2 px-2 pb-1">
      <span
        class="h-3 w-3 animate-spin rounded-full border-2 border-[var(--border)] border-t-[var(--accent)]"
        role="status"
        aria-label="Syncing"
      ></span>
      <span class="text-[10px] text-[var(--soft-foreground)] opacity-50">
        Syncing
      </span>
    </div>
  {/if}

  {#if !channelUiHidden}
    <div class={isStickyHeader ? "sticky top-0 z-10 bg-[var(--surface)]" : ""}>
      <ChannelCard
        {channel}
        active={isExpanded}
        expanded={isPreviewMode
          ? isVirtualChannel
            ? undefined
            : isExpanded
          : undefined}
        draggableEnabled={!mobileVisible &&
          manualReorderEnabled &&
          !isVirtualChannel}
        loading={channel.id.startsWith("temp-")}
        dragging={draggedChannelId === channel.id}
        dragOver={dragOverChannelId === channel.id &&
          draggedChannelId !== channel.id}
        onSelect={() => void onSelect()}
        onToggleExpanded={() => void onToggleExpanded()}
        {onDragStart}
        {onDragOver}
        {onDrop}
        {onDragEnd}
      />
    </div>
  {/if}
</div>
