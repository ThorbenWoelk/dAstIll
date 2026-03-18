import { describe, expect, it } from "bun:test";
import {
  anySearchSectionLoading,
  createEmptySearchSections,
  filterSearchResults,
  filterSearchSections,
  hasRetainedSearchState,
  resolveSearchAction,
} from "../src/lib/workspace-search";
import type { SearchResult } from "../src/lib/types";

const sampleResults: SearchResult[] = [
  {
    video_id: "video-1",
    channel_id: "channel-1",
    channel_name: "Channel 1",
    video_title: "Semantic Search Basics",
    published_at: "2026-03-18T00:00:00Z",
    matches: [
      {
        source: "summary",
        snippet: "Summary match",
        score: 0.9,
      },
      {
        source: "transcript",
        snippet: "Transcript match",
        score: 0.8,
      },
    ],
  },
  {
    video_id: "video-2",
    channel_id: "channel-2",
    channel_name: "Channel 2",
    video_title: "Transcript Only",
    published_at: "2026-03-17T00:00:00Z",
    matches: [
      {
        source: "transcript",
        snippet: "Transcript only match",
        score: 0.7,
      },
    ],
  },
];

function searchSections() {
  return {
    ...createEmptySearchSections(),
    keyword: {
      results: sampleResults,
      loading: false,
      error: null,
    },
  };
}

describe("hasRetainedSearchState", () => {
  it("treats retained queries as reopenable state", () => {
    expect(
      hasRetainedSearchState("semantic search", createEmptySearchSections()),
    ).toBe(true);
  });

  it("treats retained results or errors as reopenable state", () => {
    expect(hasRetainedSearchState("", searchSections())).toBe(true);
    expect(
      hasRetainedSearchState("", {
        ...createEmptySearchSections(),
        semantic: {
          results: [],
          loading: false,
          error: "timed out",
        },
      }),
    ).toBe(true);
    expect(hasRetainedSearchState("", createEmptySearchSections())).toBe(false);
  });
});

describe("anySearchSectionLoading", () => {
  it("returns true when either search section is still loading", () => {
    expect(
      anySearchSectionLoading({
        ...createEmptySearchSections(),
        semantic: {
          results: [],
          loading: true,
          error: null,
        },
      }),
    ).toBe(true);
    expect(anySearchSectionLoading(createEmptySearchSections())).toBe(false);
  });
});

describe("resolveSearchAction", () => {
  it("returns cancel while a submitted search is running", () => {
    expect(
      resolveSearchAction({
        query: "semantic search",
        retainedQuery: "semantic search",
        loading: true,
        hasRetainedState: true,
      }),
    ).toBe("cancel");
  });

  it("returns submit only for dirty non-empty input", () => {
    expect(
      resolveSearchAction({
        query: "semantic search",
        retainedQuery: "",
        loading: false,
        hasRetainedState: false,
      }),
    ).toBe("submit");

    expect(
      resolveSearchAction({
        query: "semantic search updated",
        retainedQuery: "semantic search",
        loading: false,
        hasRetainedState: true,
      }),
    ).toBe("submit");
  });

  it("returns clear for settled retained searches", () => {
    expect(
      resolveSearchAction({
        query: "semantic search",
        retainedQuery: "semantic search",
        loading: false,
        hasRetainedState: true,
      }),
    ).toBe("clear");

    expect(
      resolveSearchAction({
        query: "",
        retainedQuery: "semantic search",
        loading: false,
        hasRetainedState: true,
      }),
    ).toBe("clear");
  });

  it("returns disabled when there is nothing to submit or clear", () => {
    expect(
      resolveSearchAction({
        query: "",
        retainedQuery: "",
        loading: false,
        hasRetainedState: false,
      }),
    ).toBe("disabled");
  });
});

describe("filterSearchResults", () => {
  it("keeps all matches when the all filter is selected", () => {
    expect(filterSearchResults(sampleResults, "all")).toEqual(sampleResults);
  });

  it("filters matches in place and drops empty results", () => {
    expect(filterSearchResults(sampleResults, "summary")).toEqual([
      {
        ...sampleResults[0],
        matches: [sampleResults[0].matches[0]],
      },
    ]);

    expect(filterSearchResults(sampleResults, "transcript")).toEqual([
      {
        ...sampleResults[0],
        matches: [sampleResults[0].matches[1]],
      },
      sampleResults[1],
    ]);
  });
});

describe("filterSearchSections", () => {
  it("filters each search section independently", () => {
    expect(filterSearchSections(searchSections(), "summary")).toEqual({
      keyword: {
        results: [
          {
            ...sampleResults[0],
            matches: [sampleResults[0].matches[0]],
          },
        ],
        loading: false,
        error: null,
      },
      semantic: {
        results: [],
        loading: false,
        error: null,
      },
    });
  });
});
