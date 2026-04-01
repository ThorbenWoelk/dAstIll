<script lang="ts">
  import WorkspaceSidebar from "$lib/components/workspace/WorkspaceSidebar.svelte";
  import type {
    WorkspaceSidebarChannelActions,
    WorkspaceSidebarChannelState,
    WorkspaceSidebarVideoActions,
    WorkspaceSidebarVideoState,
  } from "$lib/workspace/component-props";
  import type { ChannelSnapshot } from "$lib/types";

  let {
    open = false,
    errorMessage = null,
    initialChannelPreviews = {},
    initialChannelPreviewsFilterKey = "all:all:default",
    channelState,
    channelActions,
    videoState,
    videoActions,
    onClose = () => {},
  }: {
    open?: boolean;
    errorMessage?: string | null;
    initialChannelPreviews?: Record<string, ChannelSnapshot>;
    initialChannelPreviewsFilterKey?: string;
    channelState: WorkspaceSidebarChannelState;
    channelActions: WorkspaceSidebarChannelActions;
    videoState: WorkspaceSidebarVideoState;
    videoActions: WorkspaceSidebarVideoActions;
    onClose?: () => void;
  } = $props();
</script>

{#if open}
  <div
    class="fixed inset-0 z-[80] lg:hidden"
    role="dialog"
    aria-modal="true"
    aria-label="Browse channels"
  >
    <button
      type="button"
      class="absolute inset-0 bg-[var(--overlay)]"
      onclick={onClose}
      aria-label="Close sidebar"
    ></button>
    <div
      class="relative z-10 h-full w-[min(85vw,20rem)] overflow-hidden border-r border-[var(--accent-border-soft)] bg-[var(--surface-strong)] shadow-2xl"
    >
      <WorkspaceSidebar
        videoListMode="per_channel_preview"
        previewSessionKey="workspace-sidebar-navigation"
        {initialChannelPreviews}
        {initialChannelPreviewsFilterKey}
        previewScope={{ kind: "default" }}
        addSourceErrorMessage={errorMessage}
        shell={{
          collapsed: false,
          width: undefined,
          mobileVisible: true,
          onToggleCollapse: () => {},
        }}
        {channelState}
        {channelActions}
        {videoState}
        {videoActions}
      />
    </div>
  </div>
{/if}
