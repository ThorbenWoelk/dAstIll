<script lang="ts">
  import WorkspaceSidebar from "$lib/components/workspace/WorkspaceSidebar.svelte";
  import type { ChannelSnapshot } from "$lib/types";
  import type {
    WorkspaceSidebarChannelActions,
    WorkspaceSidebarChannelState,
    WorkspaceSidebarVideoActions,
    WorkspaceSidebarVideoState,
  } from "$lib/workspace/component-props";

  let {
    collapsed,
    toggle,
    width,
    errorMessage = null,
    initialChannelPreviews = {},
    initialChannelPreviewsFilterKey = "all:all:unified",
    queueVideoRefreshTick = 0,
    channelState,
    channelActions,
    videoState,
    videoActions,
  }: {
    collapsed: boolean;
    toggle: () => void;
    width?: number;
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

<WorkspaceSidebar
  videoListMode="per_channel_preview"
  previewSessionKey="download-queue-sidebar-navigation"
  addSourceErrorMessage={errorMessage}
  {initialChannelPreviews}
  {initialChannelPreviewsFilterKey}
  previewScope={{ kind: "unified" }}
  {queueVideoRefreshTick}
  readOnly={true}
  shell={{
    collapsed,
    width,
    mobileVisible: false,
    onToggleCollapse: toggle,
  }}
  {channelState}
  {channelActions}
  {videoState}
  {videoActions}
/>
