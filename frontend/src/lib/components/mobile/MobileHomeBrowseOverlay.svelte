<script lang="ts">
  import type { Channel, ChannelSnapshot, QueueTab } from "$lib/types";
  import type {
    WorkspaceSidebarChannelActions,
    WorkspaceSidebarChannelState,
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
    channelSnapshotQueueTab = undefined as QueueTab | undefined,
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
    channelSnapshotQueueTab?: QueueTab;
    onChannelSyncDateSaved?: (channelId: string) => void | Promise<void>;
  } = $props();
</script>

{#if open}
  <!-- z-[70] above .mobile-tab-bar (z-60). No full-screen backdrop button: it sat in the same stacking context as the sheet and could steal taps from "Synced to" on some engines. -->
  <section
    class="relative z-[70] flex h-full min-h-0 flex-col overflow-hidden bg-[var(--background)] lg:hidden"
    aria-label="Browse"
  >
    <MobileChannelGallery
      {channels}
      {selectedChannelId}
      onSelectChannel={(channelId) => {
        onSelectChannel(channelId);
      }}
      onAddChannel={readOnly ? undefined : channelActions.onAddChannel}
      addingChannel={channelState.addingChannel}
      {addSourceErrorMessage}
    />

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
        {channelSnapshotQueueTab}
        hideChannelUi
        suppressVideoLoadMoreButton
        {onChannelSyncDateSaved}
      />
    </div>
  </section>
{/if}
