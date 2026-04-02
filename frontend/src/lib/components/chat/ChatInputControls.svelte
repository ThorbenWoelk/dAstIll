<script lang="ts">
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";

  let {
    deepResearch = false,
    selectedModelId = "",
    modelOptions = [],
    modelSelectDisabled = false,
    disabled = false,
    busy = false,
    canCancel = false,
    actionDisabled = false,
    ariaLabel = "Send message",
    onCancel = () => {},
    onDeepResearchChange = (_value: boolean) => {},
    onSelectedModelIdChange = (_value: string) => {},
  }: {
    deepResearch?: boolean;
    selectedModelId?: string;
    modelOptions?: { id: string; label: string }[];
    modelSelectDisabled?: boolean;
    disabled?: boolean;
    busy?: boolean;
    canCancel?: boolean;
    actionDisabled?: boolean;
    ariaLabel?: string;
    onCancel?: () => void;
    onDeepResearchChange?: (value: boolean) => void;
    onSelectedModelIdChange?: (value: string) => void;
  } = $props();
</script>

<div
  class="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between sm:gap-2"
>
  <div class="flex min-w-0 flex-wrap items-center gap-2">
    <div
      class="relative min-w-0 max-w-full sm:max-w-[min(100%,22rem)]"
      title="Ollama cloud model for this message"
    >
      <select
        value={selectedModelId}
        class="w-full min-w-[10rem] cursor-pointer appearance-none rounded-full bg-[var(--accent-wash)]/60 py-1.5 pl-2.5 pr-8 text-[11px] font-bold uppercase tracking-[0.06em] text-[var(--foreground)] transition-colors duration-200 ease-[cubic-bezier(0.16,1,0.3,1)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] disabled:cursor-not-allowed disabled:opacity-50"
        aria-label="Ollama cloud model"
        disabled={modelSelectDisabled}
        onchange={(event) => {
          if (!(event.currentTarget instanceof HTMLSelectElement)) return;
          onSelectedModelIdChange(event.currentTarget.value);
        }}
      >
        {#if modelOptions.length === 0}
          <option value="">Loading…</option>
        {:else}
          {#each modelOptions as option (option.id)}
            <option value={option.id}>{option.label}</option>
          {/each}
        {/if}
      </select>
      <span
        class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-[var(--soft-foreground)]"
        aria-hidden="true"
      >
        <ChevronIcon direction="down" size={12} />
      </span>
    </div>
    <button
      type="button"
      class="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-[11px] font-bold uppercase tracking-[0.06em] transition-colors duration-200 ease-[cubic-bezier(0.16,1,0.3,1)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] disabled:pointer-events-none disabled:opacity-50 {deepResearch
        ? 'border-[var(--accent)]/25 bg-[var(--accent-soft)] text-[var(--accent-strong)] shadow-sm'
        : 'border-transparent bg-transparent text-[var(--soft-foreground)] hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]'}"
      aria-pressed={deepResearch}
      aria-label={deepResearch ? "Deep research on" : "Deep research off"}
      data-tooltip={deepResearch
        ? "Maximum library retrieval for this message"
        : "Search more of your library (slower, richer context)"}
      disabled={disabled || busy}
      onclick={() => {
        onDeepResearchChange(!deepResearch);
      }}
    >
      <svg
        viewBox="0 0 24 24"
        class="h-3.5 w-3.5 shrink-0"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M4 19h4" />
        <path d="M6 19v-2" />
        <path d="M8 17h8" />
        <path d="M10 17V9l4-2 2 6-4 2" />
        <path d="m14 7 3-3" />
        <circle cx="17.5" cy="4.5" r="1.5" />
      </svg>
      Deep research
    </button>
  </div>

  <div class="flex items-end justify-end gap-2">
    {#if canCancel}
      <button
        type="button"
        class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)]"
        onclick={onCancel}
        aria-label={ariaLabel}
      >
        <svg
          viewBox="0 0 24 24"
          class="h-4 w-4"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          aria-hidden="true"
        >
          <path d="M18 6 6 18M6 6l12 12" />
        </svg>
      </button>
    {:else}
      <button
        type="submit"
        class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] disabled:cursor-not-allowed disabled:opacity-50"
        disabled={actionDisabled}
        aria-label={ariaLabel}
      >
        {#if busy}
          <svg
            viewBox="0 0 24 24"
            class="h-4 w-4 animate-spin"
            aria-hidden="true"
          >
            <circle
              cx="12"
              cy="12"
              r="9"
              fill="none"
              stroke="currentColor"
              stroke-opacity="0.25"
              stroke-width="2"
            />
            <path
              d="M12 3a9 9 0 0 1 9 9"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
            />
          </svg>
        {:else}
          <svg
            viewBox="0 0 24 24"
            class="h-4 w-4"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path d="M22 2 11 13" />
            <path d="M22 2 15 22l-4-9-9-4Z" />
          </svg>
        {/if}
      </button>
    {/if}
  </div>
</div>
