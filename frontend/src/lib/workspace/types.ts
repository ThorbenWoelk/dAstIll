import type { VideoTypeFilter } from "$lib/types";

export type WorkspaceContentMode =
  | "transcript"
  | "summary"
  | "highlights"
  | "info";

export type AcknowledgedFilter = "all" | "unack" | "ack";
export type ChannelSortMode = "custom" | "alpha" | "newest";

export const CONTENT_MODES: WorkspaceContentMode[] = [
  "transcript",
  "summary",
  "highlights",
  "info",
];

export const VIDEO_TYPE_FILTERS: VideoTypeFilter[] = ["all", "long", "short"];

export const ACKNOWLEDGED_FILTERS: AcknowledgedFilter[] = [
  "all",
  "unack",
  "ack",
];

export function isWorkspaceContentMode(
  value: unknown,
): value is WorkspaceContentMode {
  return (
    typeof value === "string" &&
    CONTENT_MODES.includes(value as WorkspaceContentMode)
  );
}

export function isWorkspaceVideoTypeFilter(
  value: unknown,
): value is VideoTypeFilter {
  return (
    typeof value === "string" &&
    VIDEO_TYPE_FILTERS.includes(value as VideoTypeFilter)
  );
}

export function isAcknowledgedFilter(
  value: unknown,
): value is AcknowledgedFilter {
  return (
    typeof value === "string" &&
    ACKNOWLEDGED_FILTERS.includes(value as AcknowledgedFilter)
  );
}

export function resolveAcknowledgedParam(
  filter: AcknowledgedFilter,
): boolean | undefined {
  if (filter === "ack") return true;
  if (filter === "unack") return false;
  return undefined;
}

export interface QueueStats {
  total: number;
  loading: number;
  pending: number;
  failed: number;
}

export interface DistillationStatusCopy {
  kind: "processing" | "queued" | "failed";
  label: string;
  detail: string;
}
