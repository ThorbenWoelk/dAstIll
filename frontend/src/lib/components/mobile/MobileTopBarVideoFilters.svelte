<script lang="ts">
  import WorkspaceSidebarVideoFilterControl from "$lib/components/workspace/WorkspaceSidebarVideoFilterControl.svelte";
  import type { VideoTypeFilter } from "$lib/types";
  import type { AcknowledgedFilter } from "$lib/workspace/types";

  let {
    visible = false,
    videoTypeFilter,
    acknowledgedFilter,
    disabled = false,
    onSelectVideoType,
    onSelectAcknowledged,
    onClearAllFilters,
  }: {
    visible?: boolean;
    videoTypeFilter: VideoTypeFilter;
    acknowledgedFilter: AcknowledgedFilter;
    disabled?: boolean;
    onSelectVideoType: (value: VideoTypeFilter) => void | Promise<void>;
    onSelectAcknowledged: (value: AcknowledgedFilter) => void | Promise<void>;
    onClearAllFilters: () => void | Promise<void>;
  } = $props();
</script>

{#if visible}
  <div
    class="flex min-w-0 shrink-0 items-center justify-end"
    aria-label="Video filters"
  >
    <WorkspaceSidebarVideoFilterControl
      size="md"
      {videoTypeFilter}
      {acknowledgedFilter}
      {disabled}
      {onSelectVideoType}
      {onSelectAcknowledged}
      {onClearAllFilters}
    />
  </div>
{:else}
  <div class="w-10 shrink-0" aria-hidden="true"></div>
{/if}
