import type { SearchResult, SearchSourceFilter } from "$lib/types";
import {
  createEmptySearchSections,
  type SearchSectionState,
  type SearchSectionsState,
} from "$lib/workspace-search";

export interface WorkspaceSearchSessionState {
  query: string;
  retainedQuery: string;
  source: SearchSourceFilter;
  sections: SearchSectionsState;
  modeKeyword: boolean;
  modeSemantic: boolean;
}

const STORAGE_KEY = "workspace-search-session";
const SEARCH_SOURCES = new Set<SearchSourceFilter>([
  "all",
  "summary",
  "transcript",
]);

export function createWorkspaceSearchSessionState(): WorkspaceSearchSessionState {
  return {
    query: "",
    retainedQuery: "",
    source: "all",
    sections: createEmptySearchSections(),
    modeKeyword: true,
    modeSemantic: true,
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isSearchSourceFilter(value: unknown): value is SearchSourceFilter {
  return (
    typeof value === "string" && SEARCH_SOURCES.has(value as SearchSourceFilter)
  );
}

function isSearchResult(value: unknown): value is SearchResult {
  if (!isRecord(value)) {
    return false;
  }

  if (
    typeof value.video_id !== "string" ||
    typeof value.channel_id !== "string" ||
    typeof value.channel_name !== "string" ||
    typeof value.video_title !== "string" ||
    typeof value.published_at !== "string" ||
    !Array.isArray(value.matches)
  ) {
    return false;
  }

  return value.matches.every((match) => {
    if (!isRecord(match)) {
      return false;
    }

    return (
      (match.source === "summary" || match.source === "transcript") &&
      (match.section_title === undefined ||
        match.section_title === null ||
        typeof match.section_title === "string") &&
      typeof match.snippet === "string" &&
      typeof match.score === "number"
    );
  });
}

function parseSearchSectionState(value: unknown): SearchSectionState {
  if (!isRecord(value)) {
    return {
      results: [],
      loading: false,
      error: null,
    };
  }

  return {
    results: Array.isArray(value.results)
      ? value.results.filter(isSearchResult)
      : [],
    loading: typeof value.loading === "boolean" ? value.loading : false,
    error:
      value.error === null || typeof value.error === "string"
        ? value.error
        : null,
  };
}

function parseSearchSectionsState(value: unknown): SearchSectionsState {
  const fallback = createEmptySearchSections();

  if (!isRecord(value)) {
    return fallback;
  }

  return {
    keyword: parseSearchSectionState(value.keyword),
    semantic: parseSearchSectionState(value.semantic),
  };
}

export function readWorkspaceSearchSession(
  storage: Pick<Storage, "getItem">,
  key = STORAGE_KEY,
): WorkspaceSearchSessionState {
  const fallback = createWorkspaceSearchSessionState();

  try {
    const raw = storage.getItem(key);
    if (!raw) {
      return fallback;
    }

    const parsed = JSON.parse(raw) as unknown;
    if (!isRecord(parsed)) {
      return fallback;
    }

    return {
      query: typeof parsed.query === "string" ? parsed.query : "",
      retainedQuery:
        typeof parsed.retainedQuery === "string" ? parsed.retainedQuery : "",
      source: isSearchSourceFilter(parsed.source) ? parsed.source : "all",
      sections: parseSearchSectionsState(parsed.sections),
      modeKeyword:
        typeof parsed.modeKeyword === "boolean" ? parsed.modeKeyword : true,
      modeSemantic:
        typeof parsed.modeSemantic === "boolean" ? parsed.modeSemantic : true,
    };
  } catch {
    return fallback;
  }
}

export function writeWorkspaceSearchSession(
  storage: Pick<Storage, "setItem">,
  state: WorkspaceSearchSessionState,
  key = STORAGE_KEY,
) {
  storage.setItem(key, JSON.stringify(state));
}
