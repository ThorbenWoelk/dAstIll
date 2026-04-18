<script lang="ts">
  import {
    goHintKeyForWorkspaceContentMode,
    WORKSPACE_CONTENT_MODE_ORDER,
  } from "$lib/workspace/navigation";
  import type { WorkspaceContentMode } from "$lib/workspace/types";

  let {
    contentMode,
    onSetMode,
  }: {
    contentMode: WorkspaceContentMode;
    onSetMode: (mode: WorkspaceContentMode) => void | Promise<void>;
  } = $props();
</script>

<nav
  class="flex min-w-0 items-center gap-6"
  id="workspace-tabs-desktop"
  aria-label="Content mode"
>
  {#each WORKSPACE_CONTENT_MODE_ORDER as mode}
    <button
      type="button"
      data-workspace-content-tab={mode}
      data-go-hint-key={goHintKeyForWorkspaceContentMode(mode)}
      class={`-mb-px border-b-2 pb-3 text-sm transition-colors ${
        contentMode === mode
          ? "border-[var(--foreground)] font-semibold text-[var(--foreground)]"
          : "border-transparent font-medium text-[var(--soft-foreground)] hover:text-[var(--foreground)]"
      }`}
      aria-pressed={contentMode === mode}
      onclick={() => void onSetMode(mode)}
    >
      {mode === "transcript"
        ? "Transcript"
        : mode === "summary"
          ? "Summary"
          : mode === "highlights"
            ? "Highlights"
            : "Info"}
    </button>
  {/each}
</nav>
