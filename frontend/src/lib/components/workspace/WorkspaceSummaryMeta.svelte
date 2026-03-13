<script lang="ts">
  let {
    score = null,
    note = null,
    modelUsed = null,
    qualityModelUsed = null,
  }: {
    score?: number | null;
    note?: string | null;
    modelUsed?: string | null;
    qualityModelUsed?: string | null;
  } = $props();

  let noteExpanded = $state(false);
</script>

<div
  class="mb-2 flex flex-col gap-1 text-[11px] text-[var(--soft-foreground)] opacity-40"
>
  <span>Distilled by {modelUsed ?? "unknown model"}</span>
  <div
    class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-2 gap-y-1"
    role="status"
    aria-live="polite"
  >
    <svg
      class={`mt-0.5 ${note && noteExpanded ? "row-span-2" : ""}`}
      width="11"
      height="11"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="3"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <polygon
        points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"
      ></polygon>
    </svg>
    <div class="flex min-w-0 flex-wrap items-center gap-2">
      <span class="font-bold uppercase tracking-[0.08em]">
        {#if score !== null}
          Quality Analysis: {score}/10
        {:else}
          Evaluating quality...
        {/if}
      </span>
      {#if note}
        <button
          type="button"
          class="inline-flex items-center gap-1 rounded-[var(--radius-sm)] text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] opacity-70 transition-opacity hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/30"
          aria-expanded={noteExpanded}
          aria-controls="summary-quality-note"
          onclick={() => {
            noteExpanded = !noteExpanded;
          }}
        >
          {noteExpanded ? "Hide eval" : "Show eval"}
          <svg
            class={`h-3 w-3 transition-transform ${noteExpanded ? "rotate-180" : ""}`}
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </button>
      {/if}
      {#if qualityModelUsed}
        <span
          class="text-[10px] font-medium normal-case tracking-normal text-[var(--soft-foreground)] opacity-70"
        >
          Eval by {qualityModelUsed}
        </span>
      {/if}
    </div>
    {#if note && noteExpanded}
      <p id="summary-quality-note" class="min-w-0 italic leading-relaxed">
        "{note}"
      </p>
    {/if}
  </div>
</div>
