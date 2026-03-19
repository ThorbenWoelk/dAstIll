<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount, tick } from "svelte";

  import {
    getSearchStatus,
    openSearchStatusStream,
    searchContent,
  } from "$lib/api";
  import { clickOutside } from "$lib/actions/click-outside";
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import CloseIcon from "$lib/components/icons/CloseIcon.svelte";
  import SearchIcon from "$lib/components/icons/SearchIcon.svelte";
  import SearchResultsPopover from "$lib/components/SearchResultsPopover.svelte";
  import ThemePanel from "$lib/components/ThemePanel.svelte";
  import { resolveSearchCoverageHint } from "$lib/search-status";
  import {
    readWorkspaceSearchSession,
    writeWorkspaceSearchSession,
  } from "$lib/workspace-search-session";
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

  const SEARCH_RESULT_LIMIT = 8;
  const sourceOptions: SearchSourceFilter[] = ["all", "summary", "transcript"];

  function sourceTip(option: SearchSourceFilter) {
    if (option === "all") return "All sources";
    if (option === "summary") return "Summaries only";
    return "Transcripts only";
  }

  function searchModeTip(mode: "keyword" | "semantic") {
    return mode === "keyword" ? "Keyword matches" : "Semantic matches";
  }

  function filterOptionClass(active: boolean) {
    return `flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${
      active
        ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]"
        : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"
    }`;
  }

  function searchCapabilityLabel(status: SearchStatus | null) {
    if (!status) {
      return "keyword only";
    }

    return status.retrieval_mode === "fts_only"
      ? "keyword only"
      : "traditional + semantic";
  }

  function submitActionLabel(
    action: "disabled" | "submit" | "cancel" | "clear",
    mode: "search" | "ask",
  ) {
    if (action === "submit") {
      return mode === "ask" ? "Ask with current query" : "Submit search";
    }

    if (action === "cancel") {
      return "Cancel search";
    }

    if (action === "clear") {
      return "Clear search";
    }

    return mode === "ask" ? "Ask" : "Search";
  }

  let {
    initialSearchStatus = null,
    onSearchResultSelect = async () => {},
  }: {
    initialSearchStatus?: SearchStatus | null;
    onSearchResultSelect?: (
      result: SearchResult,
      mode: "transcript" | "summary",
    ) => Promise<void> | void;
  } = $props();

  let searchQuery = $state("");
  let searchSource = $state<SearchSourceFilter>("all");
  let searchSections = $state<SearchSectionsState>(createEmptySearchSections());
  let searchPanelOpen = $state(false);
  let mobileSearchOpen = $state(false);
  let searchInputElement = $state<HTMLInputElement | null>(null);
  let searchInputFocused = $state(false);
  let searchRequestId = 0;
  let retainedSearchQuery = $state("");
  let pendingSearchQuery = $state<string | null>(null);
  let liveSearchStatus = $state<SearchStatus | null>(null);
  let searchSessionHydrated = $state(false);
  let submitMode = $state<"search" | "ask">("search");
  let modeKeyword = $state(true);
  let modeSemantic = $state(true);
  let filterMenuOpen = $state(false);
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
  let effectiveSearchAction = $derived(searchAction);
  let showSubmitModeToggle = $derived(
    searchInputFocused || searchQueryTrimmed.length > 0 || hasRecentSearchState,
  );
  let hasActiveFilter = $derived(
    searchSource !== "all" || modeKeyword !== modeSemantic,
  );
  let activeFilterLabel = $derived.by(() => {
    const labels: string[] = [];
    if (searchSource !== "all") {
      labels.push(searchSource === "summary" ? "Summaries" : "Transcripts");
    }
    if (modeKeyword !== modeSemantic) {
      labels.push(modeKeyword ? "Keyword" : "Semantic");
    }
    return labels.join(" · ");
  });
  let searchResultsVisible = $derived(
    searchPanelOpen && (hasRecentSearchState || searchLoading),
  );
  let searchCoverageHint = $derived(resolveSearchCoverageHint(searchStatus));
  let displayedSearchSections = $derived(
    filterSearchSections(searchSections, searchSource),
  );

  onMount(() => {
    submitMode = window.location.pathname.startsWith("/chat")
      ? "ask"
      : "search";

    const restoredSearchState = readWorkspaceSearchSession(
      window.sessionStorage,
    );
    searchQuery = restoredSearchState.query;
    retainedSearchQuery = restoredSearchState.retainedQuery;
    searchSource = restoredSearchState.source;
    searchSections = restoredSearchState.sections;
    modeKeyword = restoredSearchState.modeKeyword;
    modeSemantic = restoredSearchState.modeSemantic;
    searchSessionHydrated = true;

    const pollSearchStatus = async (bypassCache = false) => {
      try {
        liveSearchStatus = await getSearchStatus({ bypassCache });
      } catch {
        // Search status is informational only.
      }
    };

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
      abortSearchRequests();
      eventSource?.close();
    };
  });

  $effect(() => {
    if (!searchSessionHydrated || typeof window === "undefined") {
      return;
    }

    writeWorkspaceSearchSession(window.sessionStorage, {
      query: searchQuery,
      retainedQuery: retainedSearchQuery,
      source: searchSource,
      sections: searchSections,
      modeKeyword,
      modeSemantic,
    });
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
    if (effectiveSearchAction === "submit") {
      if (submitMode === "ask") {
        void submitAsk();
      } else {
        submitSearch();
      }
      return;
    }

    if (effectiveSearchAction === "cancel") {
      cancelSubmittedSearch();
      return;
    }

    if (effectiveSearchAction === "clear") {
      clearSearch();
    }
  }

  async function openSearchOverlay() {
    mobileSearchOpen = true;
    searchPanelOpen = hasRecentSearchState;
    await tick();
    searchInputElement?.focus();
  }

  function closeSearchOverlay() {
    mobileSearchOpen = false;
    searchPanelOpen = false;
  }

  function closeSearchPanel() {
    searchPanelOpen = false;
  }

  function submitModePillClass(active: boolean) {
    return `inline-flex h-6 items-center justify-center rounded-full px-2.5 text-[9px] font-bold uppercase tracking-[0.08em] transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${
      active
        ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]"
        : "text-[var(--soft-foreground)] opacity-75 hover:bg-[var(--muted)] hover:text-[var(--foreground)] hover:opacity-100"
    }`;
  }

  function setSubmitMode(nextMode: "search" | "ask") {
    submitMode = nextMode;
    searchPanelOpen = hasRecentSearchState;
    searchInputElement?.focus();
  }

  async function submitAsk() {
    const query = searchQueryTrimmed;
    if (!query) {
      searchPanelOpen = hasRecentSearchState;
      return;
    }

    searchPanelOpen = false;
    mobileSearchOpen = false;

    const params = new URLSearchParams();
    params.set("prompt", query);
    await goto(`/chat?${params.toString()}`, {
      keepFocus: true,
      noScroll: true,
    });
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.metaKey && !event.ctrlKey && !event.altKey && event.key === "k") {
      event.preventDefault();
      if (searchInputElement) {
        searchInputElement.focus();
        searchInputElement.select();
      } else {
        void openSearchOverlay();
      }
      return;
    }

    if (event.key === "Escape") {
      if (filterMenuOpen) {
        filterMenuOpen = false;
      } else if (searchLoading) {
        cancelSubmittedSearch();
      } else if (searchPanelOpen) {
        searchPanelOpen = false;
      } else if (mobileSearchOpen) {
        closeSearchOverlay();
      }
    }
  }

  function primarySearchSource(result: SearchResult): "transcript" | "summary" {
    const preferredMatch = result.matches[0];
    return preferredMatch?.source === "summary" ? "summary" : "transcript";
  }

  async function handleResultSelect(result: SearchResult) {
    searchPanelOpen = false;
    mobileSearchOpen = false;
    await onSearchResultSelect(result, primarySearchSource(result));
  }

  function submitSearch() {
    const query = searchQueryTrimmed;
    if (!query) {
      searchPanelOpen = hasRecentSearchState;
      return;
    }

    runSearch(query);
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#snippet searchForm()}
  <div
    id="workspace-search-panel"
    class="relative w-full lg:max-w-[30rem]"
    use:clickOutside={{
      enabled: searchPanelOpen || mobileSearchOpen,
      onClickOutside: () => {
        if (mobileSearchOpen) {
          closeSearchOverlay();
          return;
        }
        searchPanelOpen = false;
      },
    }}
  >
    <form
      class={`flex items-center gap-2 rounded-[var(--radius-md)] border bg-[var(--panel-surface)] px-3 py-2 shadow-sm transition-colors ${searchResultsVisible ? "border-[var(--accent)]/35" : "border-[var(--accent-border-soft)]"}`}
      onsubmit={(event) => {
        event.preventDefault();
        if (submitMode === "ask") {
          void submitAsk();
          return;
        }
        submitSearch();
      }}
    >
      <button
        type="button"
        class={`inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-full transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${effectiveSearchAction === "submit" ? "bg-[var(--accent)]/10 text-[var(--accent-strong)] opacity-100 hover:bg-[var(--accent)]/15" : ""} ${effectiveSearchAction === "cancel" ? "animate-pulse bg-[var(--accent)] text-white hover:bg-[var(--accent-strong)]" : ""} ${effectiveSearchAction === "clear" ? "text-[var(--soft-foreground)] opacity-70 hover:bg-[var(--muted)] hover:text-[var(--foreground)]" : ""} ${effectiveSearchAction === "disabled" ? "cursor-not-allowed text-[var(--soft-foreground)] opacity-40" : ""}`}
        aria-label={submitActionLabel(effectiveSearchAction, submitMode)}
        disabled={effectiveSearchAction === "disabled"}
        onclick={handleSearchAction}
      >
        {#if effectiveSearchAction === "clear" || effectiveSearchAction === "cancel"}
          <CloseIcon size={10} strokeWidth={3} />
        {:else if submitMode === "ask"}
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path
              d="M21 11.5A8.5 8.5 0 0 1 12.5 20H7l-4 3v-6.5A8.5 8.5 0 1 1 21 11.5Z"
            />
            <path d="M8.5 10.5h7" />
            <path d="M8.5 14h4.5" />
          </svg>
        {:else}
          <SearchIcon size={14} strokeWidth={2.4} />
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
          searchInputFocused = true;
          if (hasRecentSearchState) {
            searchPanelOpen = true;
          }
        }}
        onblur={() => {
          searchInputFocused = false;
        }}
        aria-label="Search transcripts and summaries"
      />
      {#if showSubmitModeToggle}
        <div class="flex items-center rounded-full bg-[var(--muted)]/65 p-0.5">
          <button
            type="button"
            class={submitModePillClass(submitMode === "search")}
            aria-pressed={submitMode === "search"}
            onclick={() => setSubmitMode("search")}
          >
            Search
          </button>
          <button
            type="button"
            class={submitModePillClass(submitMode === "ask")}
            aria-pressed={submitMode === "ask"}
            onclick={() => setSubmitMode("ask")}
          >
            Ask
          </button>
        </div>
      {/if}
      {#if searchLoading}
        <span
          class="h-4 w-4 animate-spin rounded-full border-[1.5px] border-[var(--border)] border-t-[var(--accent)]"
          aria-hidden="true"
        ></span>
      {/if}
      {#if !searchQuery && !searchLoading}
        {#if !searchInputFocused}
          <kbd
            class="hidden shrink-0 font-sans text-[10px] text-[var(--soft-foreground)] opacity-40 lg:inline"
            aria-hidden="true">⌘K</kbd
          >
        {/if}
      {/if}
    </form>

    <SearchResultsPopover
      show={searchResultsVisible}
      query={retainedSearchQuery}
      pendingQuery={pendingSearchQuery}
      source={searchSource}
      sections={displayedSearchSections}
      {modeKeyword}
      {modeSemantic}
      onClose={closeSearchPanel}
      onResultSelect={(result) => void handleResultSelect(result)}
    />
  </div>
{/snippet}

{#snippet searchStatusInfo()}
  {#if !searchLoading && searchStatus && searchCoverageHint}
    <div class="group relative shrink-0">
      <button
        type="button"
        class="inline-flex h-5 w-5 items-center justify-center text-[var(--soft-foreground)] opacity-65 transition-opacity hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
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
        <p class="text-[11px] font-bold tabular-nums text-[var(--foreground)]">
          {searchCoverageHint}
        </p>
        <p class="mt-1 text-[10px] leading-snug text-[var(--soft-foreground)]">
          {searchStatus.ready} / {searchStatus.total_sources} keyword sources indexed{searchStatus.available &&
          searchStatus.total_chunk_count > 0
            ? `. ${searchStatus.embedded_chunk_count} / ${searchStatus.total_chunk_count} semantic chunks embedded`
            : ""}. Mode: {searchCapabilityLabel(searchStatus)}.
        </p>
      </div>
    </div>
  {/if}
{/snippet}

<div class="flex items-center justify-self-end gap-2 lg:hidden">
  <button
    type="button"
    class={`inline-flex h-9 min-w-9 items-center justify-center rounded-full border transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${mobileSearchOpen ? "border-[var(--accent)]/25 bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "border-[var(--accent-border-soft)] bg-[var(--panel-surface)] text-[var(--soft-foreground)] hover:border-[var(--accent)]/35 hover:text-[var(--foreground)]"}`}
    onclick={() => void openSearchOverlay()}
    aria-label="Open search"
    aria-expanded={mobileSearchOpen}
    aria-controls="workspace-search-panel"
  >
    <SearchIcon size={15} strokeWidth={2.4} />
  </button>
  <ThemePanel />
</div>

{#if mobileSearchOpen}
  <div class="fixed inset-0 z-[90] lg:hidden">
    <button
      type="button"
      class="absolute inset-0 bg-[var(--overlay)]"
      onclick={closeSearchOverlay}
      aria-label="Close search"
    ></button>
    <div
      class="relative mx-auto flex h-full w-full max-w-[36rem] flex-col px-4 pb-6 pt-[max(1rem,env(safe-area-inset-top))]"
    >
      <div
        class="rounded-[var(--radius-lg)] border border-[var(--accent-border-soft)] bg-[var(--surface-strong)] p-3 shadow-2xl"
      >
        <div class="mb-3 flex items-start justify-between gap-3">
          <div>
            <p
              class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-55"
            >
              Search
            </p>
            <p class="mt-1 text-[13px] text-[var(--foreground)]">
              Search transcripts and summaries
            </p>
          </div>
          <button
            type="button"
            class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-65 transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
            onclick={closeSearchOverlay}
            aria-label="Close search"
          >
            <CloseIcon size={12} strokeWidth={2.6} />
          </button>
        </div>
        {@render searchForm()}
      </div>
    </div>
  </div>
{:else}
  <div class="hidden min-w-0 lg:col-span-1 lg:col-start-3 lg:block">
    <div class="relative min-w-0">
      <div class="min-w-0 lg:flex lg:items-center lg:gap-2">
        {@render searchForm()}
        {@render searchStatusInfo()}
        <div class="mt-3 hidden shrink-0 lg:mt-0 lg:block">
          <ThemePanel />
        </div>
      </div>
    </div>
  </div>
{/if}
