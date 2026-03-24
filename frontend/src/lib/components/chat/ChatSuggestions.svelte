<script lang="ts">
  let {
    suggestions,
    disabled = false,
    onPick = (_s: string) => {},
  }: {
    suggestions: readonly string[];
    disabled?: boolean;
    onPick?: (s: string) => void;
  } = $props();
</script>

{#if suggestions.length > 0}
  <div class="mb-3 min-w-0">
    <p
      class="mb-2 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-70"
    >
      Try asking
    </p>
    <div
      class="custom-scrollbar flex max-w-full flex-nowrap items-center gap-2 overflow-x-auto pb-0.5 [-ms-overflow-style:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
    >
      {#each suggestions as suggestion (suggestion)}
        <button
          type="button"
          class="shrink-0 rounded-full border border-[var(--accent-border-soft)] bg-[var(--surface-frost)] px-3.5 py-1.5 text-left text-[12px] leading-snug text-[var(--foreground)] transition-colors hover:bg-[var(--accent-wash)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 disabled:pointer-events-none disabled:opacity-50"
          {disabled}
          onclick={() => onPick(suggestion)}
        >
          {suggestion}
        </button>
      {/each}
    </div>
  </div>
{/if}
