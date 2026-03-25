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
      class="flex gap-1 overflow-x-auto rounded-[var(--radius-full)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] p-1 shadow-sm"
      aria-label="Workspace panels"
      style="scroll-snap-type: x mandatory"
    >
      <div class="flex min-w-max flex-nowrap gap-1">
        {#each tabs as tab}
          <button
            type="button"
            class={`min-w-max scroll-snap-align-start rounded-[var(--radius-full)] px-3 py-2 text-[10px] font-bold uppercase tracking-[0.08em] transition-colors ${
              activeTab === tab.value
                ? "border-b-2 border-[var(--accent)] text-[var(--accent-strong)]"
                : "border-b-2 border-transparent text-[var(--soft-foreground)] opacity-80 hover:text-[var(--foreground)]"
            }`}
            onclick={() => onTabChange(tab.value)}
            aria-current={activeTab === tab.value ? "page" : undefined}
          >
            <span class="block truncate">{tab.label}</span>
          </button>
        {/each}
      </div>
    </nav>
  </div>
</div>
