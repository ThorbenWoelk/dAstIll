<script lang="ts">
  import type { Channel, ChannelSnapshot } from "$lib/types";
  import type {
    WorkspaceSidebarChannelActions,
    WorkspaceSidebarChannelState,
    WorkspaceSidebarPreviewProps,
    WorkspaceSidebarVideoActions,
    WorkspaceSidebarVideoState,
  } from "$lib/workspace/component-props";
  import WorkspaceSidebar from "$lib/components/workspace/WorkspaceSidebar.svelte";
  import MobileChannelGallery from "$lib/components/mobile/MobileChannelGallery.svelte";

  let {
    open,
    channels,
    selectedChannelId,
    onSelectChannel,
    onClose,
    channelState,
    channelActions,
    videoState,
    videoActions,
    canDeleteChannels = false,
    readOnly = false,
    addSourceErrorMessage = null as string | null,
    initialChannelPreviews = {} as Record<string, ChannelSnapshot>,
    initialChannelPreviewsFilterKey = undefined as string | undefined,
    previewScope = { kind: "default" } as NonNullable<
      WorkspaceSidebarPreviewProps["previewScope"]
    >,
    onChannelSyncDateSaved = undefined,
  }: {
    open: boolean;
    channels: Channel[];
    selectedChannelId: string | null;
    onSelectChannel: (channelId: string) => void;
    onClose: () => void;
    channelState: WorkspaceSidebarChannelState;
    channelActions: WorkspaceSidebarChannelActions;
    videoState: WorkspaceSidebarVideoState;
    videoActions: WorkspaceSidebarVideoActions;
    canDeleteChannels?: boolean;
    readOnly?: boolean;
    addSourceErrorMessage?: string | null;
    initialChannelPreviews?: Record<string, ChannelSnapshot>;
    initialChannelPreviewsFilterKey?: string | undefined;
    previewScope?: WorkspaceSidebarPreviewProps["previewScope"];
    onChannelSyncDateSaved?: (channelId: string) => void | Promise<void>;
  } = $props();

  // ---------------------------------------------------------------------------
  // Channel swipe navigation
  // Horizontal swipe on the video list changes the active channel.
  // - Swipe left  → next channel
  // - Swipe right → previous channel  (ignored when starting from the left
  //   device edge ≤40px to avoid clashing with any edge-back system gesture)
  // Vertical scrolling of the video list is unaffected (different axis).
  // ---------------------------------------------------------------------------

  const SWIPE_THRESHOLD_PX = 60;
  const SWIPE_EDGE_PX = 40;

  function selectPrevChannel() {
    const idx = channels.findIndex((c) => c.id === selectedChannelId);
    if (idx > 0) onSelectChannel(channels[idx - 1].id);
  }

  function selectNextChannel() {
    const idx = channels.findIndex((c) => c.id === selectedChannelId);
    if (idx >= 0 && idx < channels.length - 1)
      onSelectChannel(channels[idx + 1].id);
  }

  interface SwipeState {
    startX: number;
    startY: number;
  }
  let swipeState: SwipeState | null = null;

  function isInteractiveTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    return Boolean(
      target.closest("button, a, input, textarea, select, [role='button']"),
    );
  }

  function swipeChannelAction(node: HTMLElement) {
    function handleStart(e: TouchEvent) {
      if (e.touches.length !== 1 || isInteractiveTarget(e.target)) {
        swipeState = null;
        return;
      }
      swipeState = {
        startX: e.touches[0].clientX,
        startY: e.touches[0].clientY,
      };
    }

    function handleEnd(e: TouchEvent) {
      if (!swipeState || e.changedTouches.length !== 1) {
        swipeState = null;
        return;
      }
      const { startX, startY } = swipeState;
      swipeState = null;
      const dx = e.changedTouches[0].clientX - startX;
      const dy = e.changedTouches[0].clientY - startY;
      // Ignore gestures that are too short or too diagonal.
      if (
        Math.abs(dx) < SWIPE_THRESHOLD_PX ||
        Math.abs(dy) > Math.abs(dx) * 0.8
      )
        return;
      if (dx < 0) {
        selectNextChannel();
      } else if (startX > SWIPE_EDGE_PX) {
        // Right swipe not from edge → previous channel.
        selectPrevChannel();
      }
    }

    function handleCancel() {
      swipeState = null;
    }

    node.addEventListener("touchstart", handleStart, { passive: true });
    node.addEventListener("touchend", handleEnd, { passive: true });
    node.addEventListener("touchcancel", handleCancel, { passive: true });

    return {
      destroy() {
        node.removeEventListener("touchstart", handleStart);
        node.removeEventListener("touchend", handleEnd);
        node.removeEventListener("touchcancel", handleCancel);
      },
    };
  }
</script>

{#if open}
  <!-- No full-screen backdrop button: it sat in the same stacking context as the sheet and could steal taps from "Synced to" on some engines. -->
  <section
    class="relative flex h-full min-h-0 flex-col overflow-hidden bg-[var(--background)] lg:hidden"
    style="z-index: var(--z-mobile-browse-overlay);"
    aria-label="Browse"
    use:swipeChannelAction
  >
    <MobileChannelGallery
      {channels}
      {selectedChannelId}
      onSelectChannel={(channelId) => {
        onSelectChannel(channelId);
      }}
      onAddChannel={readOnly ? undefined : channelActions.onAddChannel}
      addingChannel={channelState.addingChannel}
      loadingChannels={channelState.loadingChannels}
      {addSourceErrorMessage}
    />

    <!-- Video list: scroll up/down for content. Channel swipe is on the outer section. -->
    <div class="min-h-0 flex-1 overflow-hidden">
      <WorkspaceSidebar
        videoListMode="selected_channel"
        shell={{
          collapsed: false,
          width: undefined,
          mobileVisible: true,
          onToggleCollapse: onClose,
        }}
        channelState={{
          ...channelState,
          channels,
          selectedChannelId,
          canDeleteChannels,
        }}
        {channelActions}
        {videoState}
        {videoActions}
        {readOnly}
        {addSourceErrorMessage}
        {initialChannelPreviews}
        {initialChannelPreviewsFilterKey}
        {previewScope}
        hideChannelUi
        suppressVideoLoadMoreButton
        {onChannelSyncDateSaved}
      />
    </div>
  </section>
{/if}
