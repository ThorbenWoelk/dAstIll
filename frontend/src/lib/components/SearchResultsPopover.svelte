<script lang="ts">
  import type { SearchResult, SearchSourceFilter } from "$lib/types";

  const sourceOptions: SearchSourceFilter[] = ["all", "summary", "transcript"];

  function formatPublishedAt(value: string | null | undefined) {
    if (!value) return "Unknown";
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;
    return new Intl.DateTimeFormat(undefined, {
      dateStyle: "medium",
    }).format(date);
  }

  function labelForSource(option: SearchSourceFilter) {
    if (option === "all") return "All";
    if (option === "summary") return "Summaries";
    return "Transcripts";
  }

  let {
    show = false,
    query,
    source,
    results = [],
    loading = false,
    error = null,
    onClose,
    onSourceChange,
    onResultSelect,
  }: {
    show?: boolean;
    query: string;
    source: SearchSourceFilter;
    results?: SearchResult[];
    loading?: boolean;
    error?: string | null;
    onClose: () => void;
    onSourceChange: (source: SearchSourceFilter) => void;
    onResultSelect: (result: SearchResult) => void;
  } = $props();
</script>

{#if show}
  <div
    class="absolute left-0 right-0 top-[calc(100%+0.75rem)] z-40 overflow-hidden rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-white shadow-2xl sm:left-auto sm:w-[32rem] lg:w-[40rem]"
    role="dialog"
    aria-label="Search results"
  >
    <div
      class="flex items-start justify-between gap-3 border-b border-[var(--border-soft)] bg-white/95 px-4 py-3"
    >
      <div class="min-w-0">
        <p
          class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-60"
        >
          Search
        </p>
        <p class="mt-1 text-[13px] text-[var(--foreground)]">
          Results for <span class="font-semibold">{query}</span>
        </p>
      </div>
      <button
        type="button"
        class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-55 transition-colors hover:bg-[var(--muted)] hover:text-[var(--foreground)]"
        onclick={onClose}
        aria-label="Close search results"
      >
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>

    <div
      class="flex flex-wrap items-center gap-2 border-b border-[var(--border-soft)] bg-[var(--surface)]/80 px-4 py-3"
      role="toolbar"
      aria-label="Search source filter"
    >
      {#each sourceOptions as option}
        <button
          type="button"
          class={`rounded-full px-3 py-1.5 text-[11px] font-bold uppercase tracking-[0.14em] transition-colors ${source === option ? "bg-[var(--accent)] text-white" : "bg-[var(--muted)]/70 text-[var(--soft-foreground)] hover:text-[var(--foreground)]"}`}
          aria-pressed={source === option}
          onclick={() => onSourceChange(option)}
        >
          {labelForSource(option)}
        </button>
      {/each}
    </div>

    <div class="max-h-[70vh] overflow-y-auto p-3 custom-scrollbar">
      {#if loading && results.length === 0}
        <div class="grid gap-3">
          {#each Array.from({ length: 3 }) as _, index (index)}
            <article
              class="flex min-h-[10rem] flex-col gap-3 rounded-[var(--radius-md)] bg-[var(--muted)]/30 p-4 animate-pulse"
            >
              <div
                class="h-3 w-1/4 rounded-full bg-[var(--muted)] opacity-50"
              ></div>
              <div
                class="h-5 w-11/12 rounded-full bg-[var(--muted)] opacity-60"
              ></div>
              <div
                class="h-3 w-full rounded-full bg-[var(--muted)] opacity-40"
              ></div>
              <div
                class="h-3 w-5/6 rounded-full bg-[var(--muted)] opacity-40"
              ></div>
            </article>
          {/each}
        </div>
      {:else if error}
        <p class="px-2 py-3 text-[14px] font-medium italic text-red-600">
          {error}
        </p>
      {:else if results.length === 0}
        <p
          class="px-2 py-3 text-[14px] font-medium italic text-[var(--soft-foreground)] opacity-55"
        >
          No matching summaries or transcripts found.
        </p>
      {:else}
        <div class="grid gap-3">
          {#each results as result}
            <button
              type="button"
              class="group flex w-full flex-col gap-3 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-white/80 p-4 text-left transition-colors hover:border-[var(--accent)]/45 hover:bg-white"
              onclick={() => onResultSelect(result)}
            >
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0 space-y-1">
                  <p
                    class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-70"
                  >
                    {result.channel_name}
                  </p>
                  <h3
                    class="text-[15px] font-semibold leading-snug text-[var(--foreground)]"
                  >
                    {result.video_title}
                  </h3>
                </div>
                <span
                  class="shrink-0 text-[11px] text-[var(--soft-foreground)] opacity-60"
                >
                  {formatPublishedAt(result.published_at)}
                </span>
              </div>

              <div class="grid gap-2">
                {#each result.matches as match}
                  <div
                    class="rounded-[var(--radius-sm)] bg-[var(--muted)]/55 px-3 py-2"
                  >
                    <div
                      class="mb-1 flex items-center gap-2 text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-75"
                    >
                      <span
                        >{match.source === "summary"
                          ? "Summary"
                          : "Transcript"}</span
                      >
                      {#if match.section_title}
                        <span class="opacity-50">{match.section_title}</span>
                      {/if}
                    </div>
                    <p class="text-[13px] leading-6 text-[var(--foreground)]">
                      {match.snippet}
                    </p>
                  </div>
                {/each}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}
