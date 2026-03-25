import type { WorkspaceStateSnapshot } from "./channel-workspace";
import type { QueueTab, VideoTypeFilter } from "./types";
import type { AcknowledgedFilter } from "./workspace/types";
import {
  isAcknowledgedFilter,
  isWorkspaceContentMode,
  isWorkspaceVideoTypeFilter,
} from "./workspace/types";

type WorkspaceViewState = Pick<
  WorkspaceStateSnapshot,
  | "selectedChannelId"
  | "selectedVideoId"
  | "contentMode"
  | "videoTypeFilter"
  | "acknowledgedFilter"
>;

/** Same as workspace view plus optional deep-link fields for chat citations. */
export type WorkspaceViewHrefParams = WorkspaceViewState & {
  citeQuery?: string | null;
  chunkId?: string | null;
};

export type QueueViewState = {
  selectedChannelId: string | null;
  queueTab: QueueTab;
  selectedVideoId?: string | null;
  videoTypeFilter?: VideoTypeFilter;
  acknowledgedFilter?: AcknowledgedFilter;
};

/** Params for building a queue URL (defaults match sidebar defaults). */
export type QueueViewHrefParams = QueueViewState;

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
  if (isWorkspaceContentMode(contentMode)) {
    restored.contentMode = contentMode;
  }
  if (isWorkspaceVideoTypeFilter(videoTypeFilter)) {
    restored.videoTypeFilter = videoTypeFilter;
  }
  if (isAcknowledgedFilter(acknowledgedFilter)) {
    restored.acknowledgedFilter = acknowledgedFilter;
  }

  return restored;
}

export function buildWorkspaceViewHref(state: WorkspaceViewHrefParams) {
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
  if (state.chunkId) {
    params.set("chunk", state.chunkId);
  }
  if (state.citeQuery) {
    params.set("cite", state.citeQuery);
  }
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
  const selectedVideoId = parseNonEmptyParam(url, "video");
  const videoTypeFilter = parseNonEmptyParam(url, "type");
  const acknowledgedFilter = parseNonEmptyParam(url, "ack");

  if (selectedChannelId) {
    restored.selectedChannelId = selectedChannelId;
  }
  if (queueTab && QUEUE_TABS.has(queueTab as QueueTab)) {
    restored.queueTab = queueTab as QueueTab;
  }
  if (selectedVideoId) {
    restored.selectedVideoId = selectedVideoId;
  }
  if (isWorkspaceVideoTypeFilter(videoTypeFilter)) {
    restored.videoTypeFilter = videoTypeFilter;
  }
  if (isAcknowledgedFilter(acknowledgedFilter)) {
    restored.acknowledgedFilter = acknowledgedFilter;
  }

  return restored;
}

export function buildQueueViewHref(state: QueueViewHrefParams) {
  const params = new URLSearchParams();
  if (state.selectedChannelId) {
    params.set("channel", state.selectedChannelId);
  }
  params.set("queue", state.queueTab);
  if (state.selectedVideoId) {
    params.set("video", state.selectedVideoId);
  }
  params.set("type", state.videoTypeFilter ?? "all");
  params.set("ack", state.acknowledgedFilter ?? "all");
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
