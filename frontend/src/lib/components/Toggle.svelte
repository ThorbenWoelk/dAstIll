<script lang="ts">
  import HighlighterIcon from "$lib/components/icons/HighlighterIcon.svelte";

  export let options: string[] = [];
  export let value = "";
  export let onChange: (next: string) => void = () => {};
</script>

<div
  class="inline-flex items-center gap-1"
  role="tablist"
  aria-label="Content Mode"
>
  {#each options as option}
    <button
      type="button"
      role="tab"
      aria-selected={value === option}
      class={`flex h-7 flex-none items-center gap-1.5 rounded-full border px-3 text-[11px] font-bold uppercase tracking-[0.1em] transition-all duration-200 focus-visible:outline-none ${
        value === option
          ? "border-[var(--accent)]/20 bg-[var(--accent-soft)] text-[var(--accent-strong)] shadow-sm"
          : "border-transparent text-[var(--soft-foreground)] opacity-65 hover:border-[var(--accent)]/15 hover:bg-[var(--accent-soft)]/40 hover:text-[var(--foreground)] hover:opacity-100"
      }`}
      onclick={() => onChange(option)}
    >
      {#if option === "transcript"}
        <svg
          class="size-[11px] shrink-0"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><path
            d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"
          /><polyline points="14 2 14 8 20 8" /><line
            x1="16"
            y1="13"
            x2="8"
            y2="13"
          /><line x1="16" y1="17" x2="8" y2="17" /></svg
        >
      {:else if option === "summary"}
        <svg
          class="size-[11px] shrink-0"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><line x1="21" y1="4" x2="3" y2="4" /><line
            x1="14"
            y1="8"
            x2="3"
            y2="8"
          /><line x1="21" y1="12" x2="3" y2="12" /><line
            x1="14"
            y1="16"
            x2="3"
            y2="16"
          /></svg
        >
      {:else if option === "highlights"}
        <HighlighterIcon size={11} strokeWidth={2.5} class="shrink-0" />
      {:else}
        <svg
          class="size-[11px] shrink-0"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="9" />
          <path d="M12 17v-5" />
          <path d="M12 8h.01" />
        </svg>
      {/if}
      <span class="leading-none">{option}</span>
    </button>
  {/each}
</div>
