<script lang="ts">
  import type { WorkspaceContentMode } from "$lib/workspace/types";

  type BrowseTab = "channels" | "videos" | "content";

  const browseTabs: Array<{ value: BrowseTab; label: string }> = [
    { value: "channels", label: "Channels" },
    { value: "videos", label: "Videos" },
    { value: "content", label: "Content" },
  ];

  const contentTabs: Array<{
    value: WorkspaceContentMode;
    label: string;
  }> = [
    { value: "transcript", label: "Transcript" },
    { value: "summary", label: "Summary" },
    { value: "highlights", label: "Highlights" },
    { value: "info", label: "Info" },
  ];

  let {
    activeTab = "channels",
    selectedVideoId = null,
    contentMode = "transcript",
    onTabChange = () => {},
    onContentModeChange = () => {},
  }: {
    activeTab?: BrowseTab;
    selectedVideoId?: string | null;
    contentMode?: WorkspaceContentMode;
    onTabChange?: (tab: BrowseTab) => void;
    onContentModeChange?: (mode: WorkspaceContentMode) => void;
  } = $props();

  let showingContentTabs = $derived(
    activeTab === "content" && !!selectedVideoId,
  );
</script>

<div class="px-4 sm:px-2 lg:hidden">
  <div class="mx-auto max-w-[1440px] pt-1">
    {#if showingContentTabs}
      <div
        id="content-mode-tabs"
        class="grid grid-cols-4 gap-1 rounded-[var(--radius-full)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] p-1 shadow-sm"
        role="tablist"
        aria-label="Content modes"
      >
        {#each contentTabs as tab}
          <button
            type="button"
            role="tab"
            aria-selected={contentMode === tab.value}
            class={`min-w-0 rounded-[var(--radius-full)] px-2 py-2 text-[10px] font-bold uppercase tracking-[0.08em] transition-all ${
              contentMode === tab.value
                ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)] shadow-sm"
                : "text-[var(--soft-foreground)] opacity-80 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
            }`}
            onclick={() => onContentModeChange(tab.value)}
          >
            <span class="block truncate">{tab.label}</span>
          </button>
        {/each}
      </div>
    {:else}
      <nav
        class="grid grid-cols-3 gap-1 rounded-[var(--radius-full)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] p-1 shadow-sm"
        aria-label="Workspace panels"
      >
        {#each browseTabs as tab}
          <button
            type="button"
            class={`min-w-0 rounded-[var(--radius-full)] px-2 py-2 text-[10px] font-bold uppercase tracking-[0.08em] transition-all ${
              activeTab === tab.value
                ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)] shadow-sm"
                : "text-[var(--soft-foreground)] opacity-80 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
            }`}
            onclick={() => onTabChange(tab.value)}
            aria-current={activeTab === tab.value ? "page" : undefined}
          >
            <span class="block truncate">{tab.label}</span>
          </button>
        {/each}
      </nav>
    {/if}
  </div>
</div>
