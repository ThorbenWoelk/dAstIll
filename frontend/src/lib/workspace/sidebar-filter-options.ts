import type { VideoTypeFilter } from "$lib/types";
import type { AcknowledgedFilter } from "$lib/workspace/types";

export const SIDEBAR_VIDEO_TYPE_OPTIONS: Array<{
  value: VideoTypeFilter;
  label: string;
}> = [
  { value: "all", label: "All Content" },
  { value: "long", label: "Full Videos" },
  { value: "short", label: "Shorts" },
];

export const SIDEBAR_ACKNOWLEDGED_FILTER_OPTIONS: Array<{
  value: AcknowledgedFilter;
  label: string;
}> = [
  { value: "all", label: "All Statuses" },
  { value: "unack", label: "Unread" },
  { value: "ack", label: "Read" },
];
