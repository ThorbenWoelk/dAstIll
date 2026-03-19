<script lang="ts">
  type BrowseTab = string;

  const defaultBrowseTabs: Array<{ value: BrowseTab; label: string }> = [
    { value: "channels", label: "Channels" },
    { value: "videos", label: "Videos" },
    { value: "content", label: "Content" },
  ];

  let {
    activeTab = "channels",
    tabs = defaultBrowseTabs,
    onTabChange = () => {},
  }: {
    activeTab?: BrowseTab;
    tabs?: Array<{ value: BrowseTab; label: string }>;
    onTabChange?: (tab: BrowseTab) => void;
  } = $props();
</script>

<div class="px-4 sm:px-2 lg:hidden">
  <div class="mx-auto max-w-[1440px] pt-1">
    <nav
      class="grid gap-1 rounded-[var(--radius-full)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] p-1 shadow-sm"
      aria-label="Workspace panels"
      style={`grid-template-columns: repeat(${tabs.length}, minmax(0, 1fr));`}
    >
      {#each tabs as tab}
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
  </div>
</div>
