<script lang="ts">
  import type { SearchResult, SearchSourceFilter } from "$lib/types";
  import {
    SEARCH_RESULT_MODES,
    type SearchResultMode,
    type SearchSectionsState,
  } from "$lib/workspace-search";
  import { formatMediumDate } from "$lib/utils/date";
  import { escapeHtml } from "$lib/utils/html";

  function formatPublishedAt(value: string | null | undefined) {
    return formatMediumDate(value);
  }

  let {
    show = false,
    query,
    pendingQuery = null,
    source,
    sections,
    modeKeyword = true,
    modeSemantic = true,
    onClose,
    onResultSelect,
  }: {
    show?: boolean;
    query: string;
    pendingQuery?: string | null;
    source: SearchSourceFilter;
    sections: SearchSectionsState;
    modeKeyword?: boolean;
    modeSemantic?: boolean;
    onClose: () => void;
    onResultSelect: (result: SearchResult) => void;
  } = $props();

  function renderSearchText(text: string, q: string, highlight: boolean) {
    const escapedText = escapeHtml(text);
    if (!highlight || !q.trim()) {
      return escapedText;
    }

    const terms = q
      .trim()
      .split(/\s+/)
      .filter((word) => word.length > 2)
      .map((word) => word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));

    if (terms.length === 0) {
      return escapedText;
    }

    const regex = new RegExp(`(${terms.join("|")})`, "gi");
    return escapedText.replace(regex, (match) => {
      return `<mark class="rounded-[2px] bg-[var(--accent-soft)] px-0.5 font-semibold text-[var(--accent-strong)]">${match}</mark>`;
    });
  }

  function activeQueryLabel() {
    return pendingQuery ?? query;
  }

  function resultQueryLabel() {
    return query || pendingQuery || "";
  }

  function sectionLabel(mode: SearchResultMode) {
    return mode === "keyword" ? "Keyword matches" : "Semantic matches";
  }

  function sectionDescription(mode: SearchResultMode) {
    return mode === "keyword"
      ? null
      : "These items matched by overall semantic closeness to your query.";
  }

  function sectionEmptyLabel(
    mode: SearchResultMode,
    filter: SearchSourceFilter,
  ) {
    if (mode === "semantic") {
      if (filter === "summary") return "No semantic summary matches found.";
      if (filter === "transcript")
        return "No semantic transcript matches found.";
      return "No semantic matches found.";
    }

    if (filter === "summary") return "No keyword summary matches found.";
    if (filter === "transcript") return "No keyword transcript matches found.";
    return "No keyword matches found.";
  }

  function shouldRenderSection(mode: SearchResultMode) {
    if (mode === "keyword" && !modeKeyword) return false;
    if (mode === "semantic" && !modeSemantic) return false;
    return (
      mode === "keyword" ||
      sections[mode].loading ||
      sections[mode].results.length > 0 ||
      sections[mode].error !== null
    );
  }
</script>

{#if show}
  <div
    class="absolute left-0 right-0 top-[calc(100%+0.75rem)] z-40 overflow-hidden rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--surface)] shadow-2xl sm:left-auto sm:w-[32rem] lg:w-[40rem]"
    role="dialog"
    aria-label="Search results"
  >
    <div
      class="flex items-start justify-between gap-3 border-b border-[var(--border-soft)] bg-[var(--surface-frost-strong)] px-4 py-3"
    >
      <div class="min-w-0">
        <p
          class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-60"
        >
          Search
        </p>
        <p class="mt-1 text-[13px] text-[var(--foreground)]">
          {#if pendingQuery}
            Searching for <span class="font-semibold">{activeQueryLabel()}</span
            >
          {:else}
            Results for <span class="font-semibold">{resultQueryLabel()}</span>
          {/if}
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

    <div class="max-h-[70vh] overflow-y-auto p-3 custom-scrollbar">
      <div class="grid gap-4">
        {#each SEARCH_RESULT_MODES as mode}
          {#if shouldRenderSection(mode)}
            <section class="grid gap-3">
              <div class="px-1">
                <div class="flex items-center justify-between gap-3">
                  <p
                    class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-70"
                  >
                    {sectionLabel(mode)}
                  </p>
                  {#if sections[mode].loading}
                    <span
                      class="text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] opacity-60"
                    >
                      Loading…
                    </span>
                  {/if}
                </div>
                {#if sectionDescription(mode)}
                  <p
                    class="mt-1 text-[12px] text-[var(--soft-foreground)] opacity-75"
                  >
                    {sectionDescription(mode)}
                  </p>
                {/if}
              </div>

              {#if sections[mode].loading && sections[mode].results.length === 0}
                <div class="grid gap-3">
                  {#each Array.from({ length: 2 }) as _, index (index)}
                    <article
                      class="flex min-h-[8rem] flex-col gap-3 rounded-[var(--radius-md)] bg-[var(--muted)] p-4 animate-pulse"
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
              {:else if sections[mode].error}
                <p
                  class="rounded-[var(--radius-sm)] border border-[var(--danger-border)] bg-[var(--danger-soft)] px-3 py-2 text-[13px] font-medium text-[var(--danger-foreground)]"
                >
                  {sections[mode].error}
                </p>
              {:else if sections[mode].results.length === 0}
                <p
                  class="px-2 py-1 text-[14px] font-medium italic text-[var(--soft-foreground)] opacity-55"
                >
                  {sectionEmptyLabel(mode, source)}
                </p>
              {:else}
                <div class="grid gap-3">
                  {#each sections[mode].results as result}
                    <button
                      type="button"
                      class="group flex w-full flex-col gap-3 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-frost)] p-4 text-left transition-colors hover:border-[var(--accent-border-soft)] hover:bg-[var(--surface-strong)]"
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
                            {@html renderSearchText(
                              result.video_title,
                              query,
                              mode === "keyword",
                            )}
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
                            class="rounded-[var(--radius-sm)] bg-[var(--muted)] px-3 py-2"
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
                                <span class="opacity-50"
                                  >{match.section_title}</span
                                >
                              {/if}
                            </div>
                            <p
                              class="text-[13px] leading-6 text-[var(--foreground)]"
                            >
                              {@html renderSearchText(
                                match.snippet,
                                query,
                                mode === "keyword",
                              )}
                            </p>
                          </div>
                        {/each}
                      </div>
                    </button>
                  {/each}
                </div>
              {/if}
            </section>
          {/if}
        {/each}
      </div>
    </div>
  </div>
{/if}
