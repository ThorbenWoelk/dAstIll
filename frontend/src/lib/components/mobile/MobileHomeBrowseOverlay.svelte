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

  let {
    open,
    channels,
    selectedChannelId,
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
    previewSessionKey = undefined as string | undefined,
    onChannelSyncDateSaved = undefined,
  }: {
    open: boolean;
    channels: Channel[];
    selectedChannelId: string | null;
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
    previewSessionKey?: string;
    onChannelSyncDateSaved?: (channelId: string) => void | Promise<void>;
  } = $props();
</script>

{#if open}
  <section
    class="relative z-[70] flex h-full min-h-0 flex-col overflow-hidden bg-[var(--background)] lg:hidden"
    aria-label="Browse"
  >
    <WorkspaceSidebar
      videoListMode="per_channel_preview"
      {previewSessionKey}
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
      {onChannelSyncDateSaved}
    />
  </section>
{/if}
