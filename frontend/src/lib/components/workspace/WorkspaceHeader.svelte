<script lang="ts">
  import { onMount } from "svelte";
  import {
    getSearchStatus,
    openSearchStatusStream,
    searchContent,
  } from "$lib/api";
  import { DOCS_URL } from "$lib/app-config";
  import AiStatusIndicator from "$lib/components/AiStatusIndicator.svelte";
  import SearchResultsPopover from "$lib/components/SearchResultsPopover.svelte";
  import SectionNavigation from "$lib/components/SectionNavigation.svelte";
  import ThemePanel from "$lib/components/ThemePanel.svelte";
  import { resolveSearchCoverageHint } from "$lib/search-status";
  import {
    anySearchSectionLoading,
    createEmptySearchSections,
    filterSearchSections,
    hasRetainedSearchState,
    resolveSearchAction,
    type SearchResultMode,
    type SearchSectionsState,
  } from "$lib/workspace-search";
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

  function searchCapabilityLabel(status: SearchStatus | null) {
    if (!status) {
      return "keyword only";
    }

    return status.retrieval_mode === "fts_only"
      ? "keyword only"
      : "traditional + semantic";
  }

  const SEARCH_RESULT_LIMIT = 8;

  let {
    aiIndicator = null,
    initialSearchStatus = null,
    onOpenGuide = () => {},
    onSearchResultSelect = async () => {},
  }: {
    aiIndicator?: AiIndicatorPresentation | null;
    initialSearchStatus?: SearchStatus | null;
    onOpenGuide?: () => void;
    onSearchResultSelect?: (
      result: SearchResult,
      mode: "transcript" | "summary",
    ) => Promise<void> | void;
  } = $props();

  let searchQuery = $state("");
  let searchSource = $state<SearchSourceFilter>("all");
  let searchSections = $state<SearchSectionsState>(createEmptySearchSections());
  let searchPanelOpen = $state(false);
  let searchPanelContainer = $state<HTMLDivElement | null>(null);
  let searchInputElement = $state<HTMLInputElement | null>(null);
  let searchRequestId = 0;
  let retainedSearchQuery = $state("");
  let pendingSearchQuery = $state<string | null>(null);
  let liveSearchStatus = $state<SearchStatus | null>(null);
  let searchAbortControllers = new Map<SearchResultMode, AbortController>();

  let searchQueryTrimmed = $derived(searchQuery.trim());
  let searchStatus = $derived(liveSearchStatus ?? initialSearchStatus);
  let searchLoading = $derived(anySearchSectionLoading(searchSections));
  let semanticSearchEnabled = $derived(Boolean(searchStatus?.available));
  let hasRecentSearchState = $derived(
    hasRetainedSearchState(retainedSearchQuery, searchSections),
  );
  let searchAction = $derived(
    resolveSearchAction({
      query: searchQuery,
      retainedQuery: retainedSearchQuery,
      loading: searchLoading,
      hasRetainedState: hasRecentSearchState,
    }),
  );
  let showSubmitHint = $derived(searchAction === "submit");
  let searchResultsVisible = $derived(
    searchPanelOpen && (hasRecentSearchState || searchLoading),
  );
  let searchCoverageHint = $derived(resolveSearchCoverageHint(searchStatus));
  let displayedSearchSections = $derived(
    filterSearchSections(searchSections, searchSource),
  );

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
        liveSearchStatus = await getSearchStatus({ bypassCache });
      } catch {
        // Search status is informational only.
      }
    };

    document.addEventListener("pointerdown", handlePointerDown);

    let eventSource: EventSource | null = null;
    if (typeof EventSource !== "undefined") {
      eventSource = openSearchStatusStream();
      eventSource.onmessage = (event) => {
        liveSearchStatus = JSON.parse(event.data) as SearchStatus;
      };
      eventSource.onerror = () => {
        if (!searchStatus) {
          void pollSearchStatus(true);
        }
      };
    } else if (!searchStatus) {
      void pollSearchStatus();
    }

    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      abortSearchRequests();
      eventSource?.close();
    };
  });

  function abortSearchRequests() {
    for (const controller of searchAbortControllers.values()) {
      controller.abort();
    }
    searchAbortControllers.clear();
  }

  function updateSearchSection(
    mode: SearchResultMode,
    nextState: Partial<SearchSectionsState[SearchResultMode]>,
  ) {
    searchSections = {
      ...searchSections,
      [mode]: {
        ...searchSections[mode],
        ...nextState,
      },
    };
  }

  function clearPendingQueryWhenSettled(requestId: number) {
    if (requestId !== searchRequestId) {
      return;
    }

    if (!anySearchSectionLoading(searchSections)) {
      pendingSearchQuery = null;
    }
  }

  async function runSearchSection(
    query: string,
    mode: SearchResultMode,
    requestId: number,
  ) {
    const abortController = new AbortController();
    searchAbortControllers.set(mode, abortController);

    try {
      const response = await searchContent(query, {
        limit: SEARCH_RESULT_LIMIT,
        mode,
        signal: abortController.signal,
      });
      if (requestId !== searchRequestId) {
        return;
      }

      searchPanelOpen = true;
      updateSearchSection(mode, {
        results: response.results,
        error: null,
        loading: false,
      });
    } catch (error) {
      if ((error as Error).name === "AbortError") {
        return;
      }
      if (requestId !== searchRequestId) {
        return;
      }

      searchPanelOpen = true;
      updateSearchSection(mode, {
        results: [],
        error: (error as Error).message,
        loading: false,
      });
    } finally {
      if (searchAbortControllers.get(mode) === abortController) {
        searchAbortControllers.delete(mode);
      }
      clearPendingQueryWhenSettled(requestId);
    }
  }

  function runSearch(query: string) {
    const requestId = ++searchRequestId;
    abortSearchRequests();
    retainedSearchQuery = query;
    pendingSearchQuery = query;
    searchPanelOpen = true;

    const nextSections = createEmptySearchSections();
    nextSections.keyword.loading = true;
    nextSections.semantic.loading = semanticSearchEnabled;
    searchSections = nextSections;

    void runSearchSection(query, "keyword", requestId);
    if (semanticSearchEnabled) {
      void runSearchSection(query, "semantic", requestId);
    }
  }

  function submitSearch() {
    const query = searchQueryTrimmed;
    if (!query) {
      searchPanelOpen = hasRecentSearchState;
      return;
    }

    runSearch(query);
  }

  function clearSearch() {
    abortSearchRequests();
    searchRequestId += 1;
    searchPanelOpen = false;
    searchQuery = "";
    searchSource = "all";
    retainedSearchQuery = "";
    pendingSearchQuery = null;
    searchSections = createEmptySearchSections();
  }

  function cancelSubmittedSearch() {
    abortSearchRequests();
    searchRequestId += 1;
    pendingSearchQuery = null;
    searchSections = {
      keyword: {
        ...searchSections.keyword,
        loading: false,
      },
      semantic: {
        ...searchSections.semantic,
        loading: false,
      },
    };
    searchPanelOpen = hasRetainedSearchState(
      retainedSearchQuery,
      searchSections,
    );
    searchInputElement?.focus();
  }

  function handleSearchAction() {
    if (searchAction === "submit") {
      submitSearch();
      return;
    }

    if (searchAction === "cancel") {
      cancelSubmittedSearch();
      return;
    }

    if (searchAction === "clear") {
      clearSearch();
    }
  }

  function closeSearchPanel() {
    searchPanelOpen = false;
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      if (searchLoading) {
        cancelSubmittedSearch();
      } else {
        searchPanelOpen = false;
      }
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

{#snippet searchForm()}
  <div
    id="workspace-search-panel"
    class="relative w-full lg:max-w-[22rem]"
    bind:this={searchPanelContainer}
  >
    <form
      class={`flex items-center gap-2 rounded-[var(--radius-md)] border bg-[var(--panel-surface)] px-3 py-2 shadow-sm transition-colors ${searchResultsVisible ? "border-[var(--accent)]/35" : "border-[var(--accent-border-soft)]"}`}
      onsubmit={(event) => {
        event.preventDefault();
        submitSearch();
      }}
    >
      <button
        type="button"
        class={`inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-full transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${searchAction === "submit" ? "bg-[var(--accent)]/10 text-[var(--accent-strong)] opacity-100 hover:bg-[var(--accent)]/15" : ""} ${searchAction === "cancel" ? "animate-pulse bg-[var(--accent)] text-white hover:bg-[var(--accent-strong)]" : ""} ${searchAction === "clear" ? "text-[var(--soft-foreground)] opacity-70 hover:bg-[var(--muted)] hover:text-[var(--foreground)]" : ""} ${searchAction === "disabled" ? "cursor-not-allowed text-[var(--soft-foreground)] opacity-40" : ""}`}
        aria-label={searchAction === "submit"
          ? "Submit search"
          : searchAction === "cancel"
            ? "Cancel search"
            : searchAction === "clear"
              ? "Clear search"
              : "Search"}
        title={searchAction === "submit"
          ? "Press Enter or click to search"
          : searchAction === "cancel"
            ? "Cancel search"
            : searchAction === "clear"
              ? "Clear search"
              : "Search"}
        disabled={searchAction === "disabled"}
        onclick={handleSearchAction}
      >
        {#if searchAction === "clear" || searchAction === "cancel"}
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
        {:else}
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.4"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
        {/if}
      </button>
      <input
        type="search"
        class="search-input min-w-0 flex-1 bg-transparent text-[13px] font-medium placeholder:text-[var(--soft-foreground)] placeholder:opacity-40 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-60"
        placeholder="Search transcripts and summaries..."
        bind:value={searchQuery}
        bind:this={searchInputElement}
        disabled={searchLoading}
        oninput={() => {
          if (hasRecentSearchState) {
            searchPanelOpen = true;
          }
        }}
        onfocus={() => {
          if (hasRecentSearchState) {
            searchPanelOpen = true;
          }
        }}
        aria-label="Search transcripts and summaries"
      />
      {#if searchLoading}
        <div class="flex items-center gap-2">
          <span
            class="h-4 w-4 animate-spin rounded-full border-[1.5px] border-[var(--border)] border-t-[var(--accent)]"
            aria-hidden="true"
          ></span>
        </div>
      {/if}
      {#if showSubmitHint}
        <span
          class="shrink-0 rounded-full border border-[var(--accent-border-soft)] bg-[var(--accent-wash)] px-2 py-0.5 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] opacity-80"
          title="Press Enter or click the search icon to search with semantic ranking when available"
        >
          ask ↵
        </span>
      {:else if !searchLoading && searchStatus && searchCoverageHint}
        <div class="group relative shrink-0">
          <button
            type="button"
            class="inline-flex h-5 w-5 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-65 transition-opacity hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
            aria-label="Search index status"
          >
            <svg
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.4"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="16" x2="12" y2="12"></line>
              <line x1="12" y1="8" x2="12.01" y2="8"></line>
            </svg>
          </button>
          <div
            class="pointer-events-none absolute right-0 top-full z-50 mt-2 w-56 rounded-lg border border-[var(--accent-border-soft)] bg-[var(--panel-surface-strong)] p-3 opacity-0 shadow-lg transition-opacity group-hover:pointer-events-auto group-hover:opacity-100 group-focus-within:pointer-events-auto group-focus-within:opacity-100"
          >
            <p
              class="text-[11px] font-bold tabular-nums text-[var(--foreground)]"
            >
              {searchCoverageHint}
            </p>
            <p
              class="mt-1 text-[10px] leading-snug text-[var(--soft-foreground)]"
            >
              {searchStatus.ready} / {searchStatus.total_sources} keyword sources
              indexed{searchStatus.available &&
              searchStatus.total_chunk_count > 0
                ? `. ${searchStatus.embedded_chunk_count} / ${searchStatus.total_chunk_count} semantic chunks embedded`
                : ""}. Mode: {searchCapabilityLabel(searchStatus)}.
            </p>
          </div>
        </div>
      {/if}
    </form>

    <SearchResultsPopover
      show={searchResultsVisible}
      query={retainedSearchQuery}
      pendingQuery={pendingSearchQuery}
      source={searchSource}
      sections={displayedSearchSections}
      onClose={closeSearchPanel}
      onSourceChange={(nextValue) => {
        searchSource = nextValue;
        searchPanelOpen = true;
      }}
      onResultSelect={(result) => void handleResultSelect(result)}
    />
  </div>
{/snippet}

<div class="space-y-3 lg:space-y-0">
  <header class="mx-auto w-full max-w-[1440px] min-w-0 px-4 pb-2 sm:px-2">
    <div
      class="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-3 lg:grid-cols-[minmax(0,1fr)_auto_minmax(18rem,22rem)] lg:gap-6"
    >
      <div class="flex min-w-0 items-center gap-3 lg:justify-self-start">
        <a
          href="/"
          class="text-xl font-bold tracking-tighter text-[var(--foreground)] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] sm:text-2xl"
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
          class="inline-flex h-8 w-8 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-70 transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
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

      <div class="justify-self-end lg:hidden">
        <ThemePanel />
      </div>

      <div class="hidden lg:flex lg:justify-center">
        <SectionNavigation
          currentSection="workspace"
          docsUrl={DOCS_URL}
          showMobile={false}
        />
      </div>

      <div class="col-span-2 min-w-0 lg:col-span-1 lg:col-start-3">
        <div class="min-w-0 lg:flex lg:items-center lg:gap-2">
          {@render searchForm()}
          <div class="mt-3 hidden shrink-0 lg:mt-0 lg:block">
            <ThemePanel />
          </div>
        </div>
      </div>
    </div>
  </header>
</div>
