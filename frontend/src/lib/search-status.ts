import type { SearchStatus } from "./types";

export function resolveSearchCoveragePercent(
  status: SearchStatus | null,
): number | null {
  if (!status || status.total_sources === 0) {
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

  const percent = resolveSearchCoveragePercent(status);
  if (percent !== null && percent >= 1) {
    return `${percent}% indexed`;
  }

  return `${status.ready} / ${status.total_sources} indexed`;
}
