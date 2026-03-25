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
  } = $props();
</script>

{#if open}
  <section class="relative h-full min-h-0 lg:hidden" aria-label="Browse">
    <button
      type="button"
      class="absolute inset-0 z-10 bg-transparent"
      onclick={onClose}
      aria-label="Close browse"
    ></button>

    <div
      class="relative z-20 flex h-full min-h-0 flex-col overflow-hidden bg-[var(--background)]"
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
        />
      </div>
    </div>
  </section>
{/if}
