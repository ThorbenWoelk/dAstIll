<script lang="ts">
  import type { AiIndicatorPresentation } from "$lib/ai-status";
  import type { Channel, ChannelSnapshot, VideoTypeFilter } from "$lib/types";
  import ErrorToast from "$lib/components/ErrorToast.svelte";
  import MobileTopBarVideoFilters from "$lib/components/mobile/MobileTopBarVideoFilters.svelte";
  import MobileYouTubeTopNav from "$lib/components/mobile/MobileYouTubeTopNav.svelte";
  import QueueContentPanel from "$lib/components/queue/QueueContentPanel.svelte";
  import QueueDesktopSidebar from "$lib/components/queue/QueueDesktopSidebar.svelte";
  import QueueMobileBrowsePane from "$lib/components/queue/QueueMobileBrowsePane.svelte";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import type {
    QueueContentPanelActions,
    QueueContentPanelState,
    WorkspaceSidebarChannelActions,
    WorkspaceSidebarChannelState,
    WorkspaceSidebarVideoActions,
    WorkspaceSidebarVideoState,
  } from "$lib/workspace/component-props";
  import type { AcknowledgedFilter } from "$lib/workspace/types";

  let {
    aiIndicator = null,
    openGuide = () => {},
    selectedChannelId,
    videoTypeFilter,
    acknowledgedFilter,
    queueFilterDisabled,
    onSelectVideoType,
    onSelectAcknowledged,
    onClearAllFilters,
    shellCollapsed,
    shellWidth,
    shellToggleSidebar,
    errorMessage = null,
    onDismissError,
    initialChannelPreviews = {},
    initialChannelPreviewsFilterKey = "all:all:unified",
    queueVideoRefreshTick = 0,
    channelState,
    channelActions,
    videoState,
    videoActions,
    channels,
    onSelectChannel,
    galleryChannelPreviews,
    queueContentPanelState,
    queueContentPanelActions,
  }: {
    aiIndicator?: AiIndicatorPresentation | null;
    openGuide?: () => void;
    selectedChannelId: string | null;
    videoTypeFilter: VideoTypeFilter;
    acknowledgedFilter: AcknowledgedFilter;
    queueFilterDisabled: boolean;
    onSelectVideoType: (value: VideoTypeFilter) => void | Promise<void>;
    onSelectAcknowledged: (value: AcknowledgedFilter) => void | Promise<void>;
    onClearAllFilters: () => void | Promise<void>;
    shellCollapsed: boolean;
    shellWidth?: number;
    shellToggleSidebar: () => void;
    errorMessage?: string | null;
    onDismissError: () => void;
    initialChannelPreviews?: Record<string, ChannelSnapshot>;
    initialChannelPreviewsFilterKey?: string;
    queueVideoRefreshTick?: number;
    channelState: WorkspaceSidebarChannelState;
    channelActions: WorkspaceSidebarChannelActions;
    videoState: WorkspaceSidebarVideoState;
    videoActions: WorkspaceSidebarVideoActions;
    channels: Channel[];
    onSelectChannel: (channelId: string) => void | Promise<void>;
    galleryChannelPreviews: Record<string, ChannelSnapshot>;
    queueContentPanelState: QueueContentPanelState;
    queueContentPanelActions: QueueContentPanelActions;
  } = $props();
</script>

<WorkspaceShell currentSection="queue" {aiIndicator} onOpenGuide={openGuide}>
  {#snippet mobileTopBar()}
    <MobileYouTubeTopNav>
      {#snippet trailing()}
        <MobileTopBarVideoFilters
          visible={Boolean(selectedChannelId)}
          {videoTypeFilter}
          {acknowledgedFilter}
          disabled={queueFilterDisabled}
          {onSelectVideoType}
          {onSelectAcknowledged}
          {onClearAllFilters}
        />
      {/snippet}
    </MobileYouTubeTopNav>
  {/snippet}
  {#snippet sidebar({ collapsed, toggle, width })}
    <QueueDesktopSidebar
      collapsed={shellCollapsed}
      toggle={shellToggleSidebar}
      width={shellWidth ?? width}
      {errorMessage}
      {initialChannelPreviews}
      {initialChannelPreviewsFilterKey}
      {queueVideoRefreshTick}
      {channelState}
      {channelActions}
      {videoState}
      {videoActions}
    />
  {/snippet}

  <div
    class="flex h-full min-h-0 flex-col lg:flex-row"
    aria-label="Download queue"
  >
    <QueueMobileBrowsePane
      {channels}
      {selectedChannelId}
      {onSelectChannel}
      channelPreviews={galleryChannelPreviews}
      {errorMessage}
      {initialChannelPreviews}
      {initialChannelPreviewsFilterKey}
      {queueVideoRefreshTick}
      {channelState}
      {channelActions}
      {videoState}
      {videoActions}
    />

    <div
      class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden lg:min-w-0"
    >
      <QueueContentPanel
        hideMobileBack
        readOnly={true}
        state={queueContentPanelState}
        actions={queueContentPanelActions}
      />
    </div>
  </div>

  {#if errorMessage}
    <ErrorToast message={errorMessage} onDismiss={onDismissError} />
  {/if}
</WorkspaceShell>
