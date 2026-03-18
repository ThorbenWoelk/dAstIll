import { describe, expect, it } from "bun:test";
import {
  resolveSearchCoverageHint,
  resolveSearchCoveragePercent,
} from "../src/lib/search-status";
import type { SearchStatus } from "../src/lib/types";

function searchStatus(overrides: Partial<SearchStatus> = {}): SearchStatus {
  return {
    available: true,
    model: "qwen3-embedding",
    dimensions: 512,
    pending: 0,
    indexing: 0,
    ready: 3,
    failed: 0,
    total_sources: 4,
    total_chunk_count: 10,
    embedded_chunk_count: 7,
    vector_index_ready: false,
    retrieval_mode: "hybrid_exact",
    ...overrides,
  };
}

describe("resolveSearchCoveragePercent", () => {
  it("returns null when there is no searchable source count yet", () => {
    expect(resolveSearchCoveragePercent(null)).toBeNull();
    expect(
      resolveSearchCoveragePercent(searchStatus({ total_sources: 0 })),
    ).toBeNull();
  });

  it("rounds the ready coverage to the nearest whole percent", () => {
    expect(resolveSearchCoveragePercent(searchStatus())).toBe(75);
    expect(
      resolveSearchCoveragePercent(
        searchStatus({ ready: 8, total_sources: 8 }),
      ),
    ).toBe(100);
  });

  it("returns semantic coverage when embeddings are available", () => {
    expect(resolveSearchCoveragePercent(searchStatus(), "semantic")).toBe(70);
    expect(
      resolveSearchCoveragePercent(
        searchStatus({ available: false, embedded_chunk_count: 0 }),
        "semantic",
      ),
    ).toBeNull();
  });
});

describe("resolveSearchCoverageHint", () => {
  it("shows keyword and semantic coverage when hybrid search is available", () => {
    expect(resolveSearchCoverageHint(searchStatus())).toBe(
      "75% keyword | 70% semantic",
    );
  });

  it("keeps the completed coverage hint visible at 100 percent", () => {
    expect(
      resolveSearchCoverageHint(
        searchStatus({
          ready: 8,
          total_sources: 8,
          total_chunk_count: 12,
          embedded_chunk_count: 12,
        }),
      ),
    ).toBe("100% keyword | 100% semantic");
  });

  it("falls back to the keyword-only label when semantic search is unavailable", () => {
    expect(resolveSearchCoverageHint(searchStatus({ available: false }))).toBe(
      "75% indexed",
    );
  });

  it("falls back to raw counts when rounding would hide progress", () => {
    expect(
      resolveSearchCoverageHint(searchStatus({ ready: 1, total_sources: 400 })),
    ).toBe("1 / 400 indexed");
  });
});
