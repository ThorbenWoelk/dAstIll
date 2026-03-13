import type { VideoTypeFilter } from "$lib/types";

export type WorkspaceContentMode =
  | "transcript"
  | "summary"
  | "highlights"
  | "info";

export type AcknowledgedFilter = "all" | "unack" | "ack";
export type ChannelSortMode = "custom" | "alpha" | "newest";

export function isWorkspaceContentMode(
  value: unknown,
): value is WorkspaceContentMode {
  return (
    value === "transcript" ||
    value === "summary" ||
    value === "highlights" ||
    value === "info"
  );
}

export function isWorkspaceVideoTypeFilter(
  value: unknown,
): value is VideoTypeFilter {
  return value === "all" || value === "long" || value === "short";
}
