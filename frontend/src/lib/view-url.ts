import type { WorkspaceStateSnapshot } from "./channel-workspace";
import type { QueueTab } from "./types";

type WorkspaceViewState = Pick<
  WorkspaceStateSnapshot,
  | "selectedChannelId"
  | "selectedVideoId"
  | "contentMode"
  | "videoTypeFilter"
  | "acknowledgedFilter"
>;

type QueueViewState = {
  selectedChannelId: string | null;
  queueTab: QueueTab;
};

const CONTENT_MODES = new Set<WorkspaceStateSnapshot["contentMode"]>([
  "transcript",
  "summary",
  "highlights",
  "info",
]);
const VIDEO_TYPE_FILTERS = new Set<WorkspaceStateSnapshot["videoTypeFilter"]>([
  "all",
  "long",
  "short",
]);
const ACKNOWLEDGED_FILTERS = new Set<
  WorkspaceStateSnapshot["acknowledgedFilter"]
>(["all", "unack", "ack"]);
const QUEUE_TABS = new Set<QueueTab>([
  "transcripts",
  "summaries",
  "evaluations",
]);

function parseNonEmptyParam(url: URL, key: string) {
  const value = url.searchParams.get(key)?.trim();
  return value ? value : null;
}

export function parseWorkspaceViewUrlState(
  url: URL,
): Partial<WorkspaceViewState> {
  const restored: Partial<WorkspaceViewState> = {};
  const selectedChannelId = parseNonEmptyParam(url, "channel");
  const selectedVideoId = parseNonEmptyParam(url, "video");
  const contentMode = parseNonEmptyParam(url, "content");
  const videoTypeFilter = parseNonEmptyParam(url, "type");
  const acknowledgedFilter = parseNonEmptyParam(url, "ack");

  if (selectedChannelId) {
    restored.selectedChannelId = selectedChannelId;
  }
  if (selectedVideoId) {
    restored.selectedVideoId = selectedVideoId;
  }
  if (
    contentMode &&
    CONTENT_MODES.has(contentMode as WorkspaceStateSnapshot["contentMode"])
  ) {
    restored.contentMode = contentMode as WorkspaceStateSnapshot["contentMode"];
  }
  if (
    videoTypeFilter &&
    VIDEO_TYPE_FILTERS.has(
      videoTypeFilter as WorkspaceStateSnapshot["videoTypeFilter"],
    )
  ) {
    restored.videoTypeFilter =
      videoTypeFilter as WorkspaceStateSnapshot["videoTypeFilter"];
  }
  if (
    acknowledgedFilter &&
    ACKNOWLEDGED_FILTERS.has(
      acknowledgedFilter as WorkspaceStateSnapshot["acknowledgedFilter"],
    )
  ) {
    restored.acknowledgedFilter =
      acknowledgedFilter as WorkspaceStateSnapshot["acknowledgedFilter"];
  }

  return restored;
}

export function buildWorkspaceViewHref(state: WorkspaceViewState) {
  const params = new URLSearchParams();
  if (state.selectedChannelId) {
    params.set("channel", state.selectedChannelId);
  }
  if (state.selectedVideoId) {
    params.set("video", state.selectedVideoId);
  }
  params.set("content", state.contentMode);
  params.set("type", state.videoTypeFilter);
  params.set("ack", state.acknowledgedFilter);
  const query = params.toString();
  return query ? `/?${query}` : "/";
}

export function mergeWorkspaceViewState(
  restoredState: Partial<WorkspaceStateSnapshot>,
  urlState: Partial<WorkspaceViewState>,
) {
  return {
    ...restoredState,
    ...urlState,
  };
}

export function parseQueueViewUrlState(url: URL): Partial<QueueViewState> {
  const restored: Partial<QueueViewState> = {};
  const selectedChannelId = parseNonEmptyParam(url, "channel");
  const queueTab = parseNonEmptyParam(url, "queue");

  if (selectedChannelId) {
    restored.selectedChannelId = selectedChannelId;
  }
  if (queueTab && QUEUE_TABS.has(queueTab as QueueTab)) {
    restored.queueTab = queueTab as QueueTab;
  }

  return restored;
}

export function buildQueueViewHref(state: QueueViewState) {
  const params = new URLSearchParams();
  if (state.selectedChannelId) {
    params.set("channel", state.selectedChannelId);
  }
  params.set("queue", state.queueTab);
  return `/download-queue?${params.toString()}`;
}

export function mergeQueueViewState(
  restoredState: Pick<
    Partial<WorkspaceStateSnapshot>,
    "selectedChannelId" | "channelOrder" | "channelSortMode"
  >,
  urlState: Partial<QueueViewState>,
) {
  return {
    ...restoredState,
    ...urlState,
  };
}
