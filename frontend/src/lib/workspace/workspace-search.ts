import type { SearchResult, SearchSourceFilter } from "$lib/types";

export type SearchAction = "disabled" | "submit" | "cancel" | "clear";
export type SearchResultMode = "keyword" | "semantic";

export interface SearchSectionState {
  results: SearchResult[];
  loading: boolean;
  error: string | null;
}

export type SearchSectionsState = Record<SearchResultMode, SearchSectionState>;

export const SEARCH_RESULT_MODES: SearchResultMode[] = ["keyword", "semantic"];

export function createEmptySearchSections(): SearchSectionsState {
  return {
    keyword: {
      results: [],
      loading: false,
      error: null,
    },
    semantic: {
      results: [],
      loading: false,
      error: null,
    },
  };
}

export function anySearchSectionLoading(sections: SearchSectionsState) {
  return SEARCH_RESULT_MODES.some((mode) => sections[mode].loading);
}

export function hasRetainedSearchState(
  retainedQuery: string,
  sections: SearchSectionsState,
) {
  return (
    retainedQuery.trim().length > 0 ||
    SEARCH_RESULT_MODES.some(
      (mode) =>
        sections[mode].results.length > 0 || sections[mode].error !== null,
    )
  );
}

export function resolveSearchAction({
  query,
  retainedQuery,
  loading,
  hasRetainedState,
}: {
  query: string;
  retainedQuery: string;
  loading: boolean;
  hasRetainedState: boolean;
}): SearchAction {
  const trimmedQuery = query.trim();
  const trimmedRetainedQuery = retainedQuery.trim();

  if (loading) {
    return "cancel";
  }

  if (trimmedQuery.length > 0 && trimmedQuery !== trimmedRetainedQuery) {
    return "submit";
  }

  if (hasRetainedState) {
    return "clear";
  }

  return "disabled";
}

export function filterSearchResults(
  results: SearchResult[],
  source: SearchSourceFilter,
): SearchResult[] {
  if (source === "all") {
    return results;
  }

  return results.flatMap((result) => {
    const matches = result.matches.filter((match) => match.source === source);
    if (matches.length === 0) {
      return [];
    }

    return [
      {
        ...result,
        matches,
      },
    ];
  });
}

export function filterSearchSections(
  sections: SearchSectionsState,
  source: SearchSourceFilter,
): SearchSectionsState {
  return {
    keyword: {
      ...sections.keyword,
      results: filterSearchResults(sections.keyword.results, source),
    },
    semantic: {
      ...sections.semantic,
      results: filterSearchResults(sections.semantic.results, source),
    },
  };
}
