<script lang="ts">
  import { onMount } from "svelte";
  import { getSearchStatus, searchContent } from "$lib/api";
  import { DOCS_URL } from "$lib/app-config";
  import AiStatusIndicator from "$lib/components/AiStatusIndicator.svelte";
  import SearchResultsPopover from "$lib/components/SearchResultsPopover.svelte";
  import SectionNavigation from "$lib/components/SectionNavigation.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import { resolveSearchCoverageHint } from "$lib/search-status";
  import type {
    SearchResult,
    SearchSourceFilter,
    SearchStatus,
  } from "$lib/types";

  interface AiIndicatorPresentation {
    detail: string;
    dotClass: string;
    title: string;
  }

  const SEARCH_DEBOUNCE_MS = 280;
  const SEARCH_RESULT_LIMIT = 8;
  const SEARCH_STATUS_POLL_MS = 15_000;

  let {
    aiIndicator = null,
    onOpenGuide = () => {},
    onSearchResultSelect = async () => {},
  }: {
    aiIndicator?: AiIndicatorPresentation | null;
    onOpenGuide?: () => void;
    onSearchResultSelect?: (
      result: SearchResult,
      mode: "transcript" | "summary",
    ) => Promise<void> | void;
  } = $props();

  let searchQuery = $state("");
  let searchSource = $state<SearchSourceFilter>("all");
  let searchResults = $state<SearchResult[]>([]);
  let searchLoading = $state(false);
  let searchError = $state<string | null>(null);
  let searchPanelOpen = $state(false);
  let searchPanelContainer = $state<HTMLDivElement | null>(null);
  let searchRequestId = 0;
  let searchStatus = $state<SearchStatus | null>(null);

  let searchQueryTrimmed = $derived(searchQuery.trim());
  let searchResultsVisible = $derived(
    searchPanelOpen &&
      (searchQueryTrimmed.length > 0 || searchLoading || searchError !== null),
  );
  let searchCoverageHint = $derived(resolveSearchCoverageHint(searchStatus));

  onMount(() => {
    const handlePointerDown = (event: PointerEvent) => {
      if (
        searchPanelOpen &&
        searchPanelContainer &&
        !searchPanelContainer.contains(event.target as Node)
      ) {
        searchPanelOpen = false;
      }
    };

    const pollSearchStatus = async (bypassCache = false) => {
      try {
        searchStatus = await getSearchStatus({ bypassCache });
      } catch {
        // Search status is informational only.
      }
    };

    document.addEventListener("pointerdown", handlePointerDown);
    void pollSearchStatus();

    const statusInterval = setInterval(
      () => void pollSearchStatus(true),
      SEARCH_STATUS_POLL_MS,
    );

    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      clearInterval(statusInterval);
    };
  });

  $effect(() => {
    const query = searchQueryTrimmed;
    const source = searchSource;

    if (!query) {
      searchPanelOpen = false;
      searchResults = [];
      searchError = null;
      searchLoading = false;
      return;
    }

    const timeoutId = setTimeout(() => {
      void runSearch(query, source);
    }, SEARCH_DEBOUNCE_MS);

    return () => clearTimeout(timeoutId);
  });

  async function runSearch(query: string, source: SearchSourceFilter) {
    const requestId = ++searchRequestId;
    searchLoading = true;
    searchError = null;

    try {
      const response = await searchContent(query, {
        source,
        limit: SEARCH_RESULT_LIMIT,
      });
      if (requestId !== searchRequestId || query !== searchQueryTrimmed) {
        return;
      }
      searchResults = response.results;
    } catch (error) {
      if (requestId !== searchRequestId) {
        return;
      }
      searchResults = [];
      searchError = (error as Error).message;
    } finally {
      if (requestId === searchRequestId) {
        searchLoading = false;
      }
    }
  }

  function clearSearch() {
    searchRequestId += 1;
    searchPanelOpen = false;
    searchQuery = "";
    searchResults = [];
    searchError = null;
    searchLoading = false;
  }

  function closeSearchPanel() {
    searchPanelOpen = false;
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      searchPanelOpen = false;
    }
  }

  function primarySearchSource(result: SearchResult): "transcript" | "summary" {
    const preferredMatch = result.matches[0];
    return preferredMatch?.source === "summary" ? "summary" : "transcript";
  }

  async function handleResultSelect(result: SearchResult) {
    searchPanelOpen = false;
    await onSearchResultSelect(result, primarySearchSource(result));
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<header
  class="mx-auto flex w-full max-w-[1440px] min-w-0 flex-wrap items-start gap-3 px-4 pb-2 sm:px-2 lg:items-center"
>
  <div class="flex min-w-0 flex-1 items-center gap-3">
    <a
      href="/"
      class="text-xl font-bold tracking-tighter text-[var(--foreground)] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)] sm:text-2xl"
      aria-label="Go to dAstIll home"
    >
      DASTILL
    </a>
    {#if aiIndicator}
      <AiStatusIndicator
        detail={aiIndicator.detail}
        dotClass={aiIndicator.dotClass}
        title={aiIndicator.title}
      />
    {/if}
    <button
      type="button"
      id="guide-trigger"
      class="inline-flex h-7 w-7 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-40 transition-all hover:bg-[var(--muted)] hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
      onclick={onOpenGuide}
      aria-label="Feature guide"
    >
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="12" cy="12" r="10"></circle>
        <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
        <line x1="12" y1="17" x2="12.01" y2="17"></line>
      </svg>
    </button>
  </div>

  <div class="ml-auto flex shrink-0 items-center gap-2">
    <ThemeToggle />
    <SectionNavigation
      currentSection="workspace"
      docsUrl={DOCS_URL}
      mobileMode="inline"
    />
  </div>

  <div
    class="relative w-full sm:ml-auto sm:w-[23rem] lg:w-[27rem]"
    bind:this={searchPanelContainer}
  >
    <div
      class={`flex items-center gap-2 rounded-full border bg-[var(--surface-frost)] px-3 py-2 shadow-sm transition-colors ${searchResultsVisible ? "border-[var(--accent)]/35" : "border-[var(--border-soft)]"}`}
    >
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.4"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="shrink-0 text-[var(--soft-foreground)] opacity-50"
      >
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>
      <input
        type="search"
        class="search-input min-w-0 flex-1 bg-transparent text-[13px] font-medium placeholder:text-[var(--soft-foreground)] placeholder:opacity-40 focus-visible:outline-none"
        placeholder="Search transcripts and summaries..."
        bind:value={searchQuery}
        oninput={() => {
          searchPanelOpen = true;
        }}
        onfocus={() => {
          if (searchQueryTrimmed) {
            searchPanelOpen = true;
          }
        }}
        aria-label="Search transcripts and summaries"
      />
      {#if searchLoading}
        <span
          class="h-4 w-4 animate-spin rounded-full border-[1.5px] border-[var(--border)] border-t-[var(--accent)]"
          aria-hidden="true"
        ></span>
      {:else if searchStatus && searchCoverageHint}
        <span
          class="shrink-0 text-[10px] font-bold tabular-nums text-[var(--soft-foreground)] opacity-50"
          title="Search index: {searchStatus.ready} / {searchStatus.total_sources} transcripts and summaries indexed"
        >
          {searchCoverageHint}
        </span>
      {/if}
      {#if searchQuery}
        <button
          type="button"
          class="inline-flex h-6 w-6 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-50 transition-opacity hover:opacity-90"
          onclick={clearSearch}
          aria-label="Clear search"
        >
          <svg
            width="10"
            height="10"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="3"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      {/if}
    </div>

    <SearchResultsPopover
      show={searchResultsVisible}
      query={searchQueryTrimmed}
      source={searchSource}
      results={searchResults}
      loading={searchLoading}
      error={searchError}
      onClose={closeSearchPanel}
      onSourceChange={(nextValue) => {
        searchSource = nextValue;
        searchPanelOpen = true;
      }}
      onResultSelect={(result) => void handleResultSelect(result)}
    />
  </div>
</header>
