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

<div class="flex items-center justify-between gap-3">
  <div class="flex min-w-0 items-center gap-3">
    <button
      type="button"
      class="group/ds inline-flex items-center gap-2 py-1 text-[12px] font-medium text-[var(--soft-foreground)] transition-colors hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] disabled:pointer-events-none disabled:opacity-50"
      role="switch"
      aria-checked={deepResearch}
      aria-label={deepResearch ? "Deep research on" : "Deep research off"}
      data-tooltip={deepResearch
        ? "Maximum library retrieval for this message"
        : "Search more of your library (slower, richer context)"}
      disabled={disabled || busy}
      onclick={() => {
        onDeepResearchChange(!deepResearch);
      }}
    >
      <span
        class="relative inline-flex h-4 w-7 items-center rounded-full transition-colors {deepResearch
          ? 'bg-[var(--foreground)]'
          : 'bg-[var(--border-soft)] group-hover/ds:bg-[var(--soft-foreground)]/40'}"
        aria-hidden="true"
      >
        <span
          class="absolute top-[2px] h-3 w-3 rounded-full bg-[var(--background)] shadow-sm transition-transform {deepResearch
            ? 'translate-x-[14px]'
            : 'translate-x-[2px]'}"
        ></span>
      </span>
      <span class="select-none">Deep Search</span>
    </button>
  </div>

  <div class="flex shrink-0 items-center gap-2">
    <div
      class="relative min-w-0 max-w-[min(100%,14rem)]"
      title="Ollama cloud model for this message"
    >
      <select
        value={selectedModelId}
        class="w-full min-w-[8rem] cursor-pointer appearance-none rounded-md bg-transparent py-1 pl-2 pr-5 text-[11px] font-semibold text-[var(--soft-foreground)] transition-colors hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 disabled:cursor-not-allowed disabled:opacity-50"
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
        class="pointer-events-none absolute right-1 top-1/2 -translate-y-1/2 text-[var(--soft-foreground)]"
        aria-hidden="true"
      >
        <ChevronIcon direction="down" size={12} />
      </span>
    </div>
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
        class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[var(--foreground)] text-[var(--background)] transition-colors hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] disabled:cursor-not-allowed disabled:opacity-40"
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
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path d="M12 19V5" />
            <path d="m5 12 7-7 7 7" />
          </svg>
        {/if}
      </button>
    {/if}
  </div>
</div>
