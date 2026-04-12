<script lang="ts">
  import MobileChannelGallery from "$lib/components/mobile/MobileChannelGallery.svelte";
  import WorkspaceSidebar from "$lib/components/workspace/WorkspaceSidebar.svelte";
  import type { Channel, ChannelSnapshot } from "$lib/types";
  import type {
    WorkspaceSidebarChannelActions,
    WorkspaceSidebarChannelState,
    WorkspaceSidebarVideoActions,
    WorkspaceSidebarVideoState,
  } from "$lib/workspace/component-props";

  let {
    channels,
    selectedChannelId,
    onSelectChannel,
    channelPreviews,
    errorMessage = null,
    initialChannelPreviews = {},
    initialChannelPreviewsFilterKey = "all:all:unified",
    queueVideoRefreshTick = 0,
    channelState,
    channelActions,
    videoState,
    videoActions,
  }: {
    channels: Channel[];
    selectedChannelId: string | null;
    onSelectChannel: (channelId: string) => void | Promise<void>;
    channelPreviews: Record<string, ChannelSnapshot>;
    errorMessage?: string | null;
    initialChannelPreviews?: Record<string, ChannelSnapshot>;
    initialChannelPreviewsFilterKey?: string;
    queueVideoRefreshTick?: number;
    channelState: WorkspaceSidebarChannelState;
    channelActions: WorkspaceSidebarChannelActions;
    videoState: WorkspaceSidebarVideoState;
    videoActions: WorkspaceSidebarVideoActions;
  } = $props();
</script>

<div class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden lg:hidden">
  <MobileChannelGallery
    {channels}
    {selectedChannelId}
    onSelectChannel={(channelId) => {
      void onSelectChannel(channelId);
    }}
    {channelPreviews}
    queueUnifiedSummary={true}
  />
  <div
    class="min-h-0 flex-1 overflow-hidden border-t border-[var(--border-soft)]/50"
  >
    <WorkspaceSidebar
      videoListMode="selected_channel"
      addSourceErrorMessage={errorMessage}
      {initialChannelPreviews}
      {initialChannelPreviewsFilterKey}
      previewScope={{ kind: "unified" }}
      {queueVideoRefreshTick}
      readOnly={true}
      shell={{
        collapsed: false,
        width: undefined,
        mobileVisible: true,
        onToggleCollapse: () => {},
      }}
      channelState={{
        ...channelState,
        channels,
        selectedChannelId,
        canDeleteChannels: false,
      }}
      {channelActions}
      {videoState}
      {videoActions}
      hideChannelUi
    />
  </div>
</div>
