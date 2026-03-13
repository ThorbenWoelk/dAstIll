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
});

describe("resolveSearchCoverageHint", () => {
  it("returns the subtle percent-indexed label once enough content is indexed", () => {
    expect(resolveSearchCoverageHint(searchStatus())).toBe("75% indexed");
  });

  it("keeps the completed coverage hint visible at 100 percent", () => {
    expect(
      resolveSearchCoverageHint(searchStatus({ ready: 8, total_sources: 8 })),
    ).toBe("100% indexed");
  });

  it("falls back to raw counts when rounding would hide progress", () => {
    expect(
      resolveSearchCoverageHint(searchStatus({ ready: 1, total_sources: 400 })),
    ).toBe("1 / 400 indexed");
  });
});
