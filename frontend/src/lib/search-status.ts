import type { SearchStatus } from "./types";

export function resolveSearchCoveragePercent(
  status: SearchStatus | null,
  mode: "keyword" | "semantic" = "keyword",
): number | null {
  if (!status) {
    return null;
  }

  if (mode === "semantic") {
    if (!status.available || status.total_chunk_count === 0) {
      return null;
    }

    return Math.round(
      (status.embedded_chunk_count / status.total_chunk_count) * 100,
    );
  }

  if (status.total_sources === 0) {
    return null;
  }

  return Math.round((status.ready / status.total_sources) * 100);
}

export function resolveSearchCoverageHint(
  status: SearchStatus | null,
): string | null {
  if (!status || status.total_sources === 0) {
    return null;
  }

  const keywordPercent = resolveSearchCoveragePercent(status, "keyword");
  if (keywordPercent !== null && keywordPercent < 1) {
    return `${status.ready} / ${status.total_sources} indexed`;
  }

  const semanticPercent = resolveSearchCoveragePercent(status, "semantic");
  if (keywordPercent !== null && semanticPercent !== null) {
    return `${keywordPercent}% keyword | ${semanticPercent}% semantic`;
  }

  if (keywordPercent !== null) {
    return `${keywordPercent}% indexed`;
  }

  return `${status.ready} / ${status.total_sources} indexed`;
}
