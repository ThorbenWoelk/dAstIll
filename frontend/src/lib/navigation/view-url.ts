import type { WorkspaceStateSnapshot } from "$lib/workspace/channel-workspace";
import {
  isAcknowledgedFilter,
  isWorkspaceContentMode,
  isWorkspaceVideoTypeFilter,
} from "$lib/workspace/types";

export type WorkspaceViewState = Pick<
  WorkspaceStateSnapshot,
  | "selectedChannelId"
  | "selectedVideoId"
  | "contentMode"
  | "videoTypeFilter"
  | "acknowledgedFilter"
> & {
  selectedSourceId?: string | null;
  selectedItemId?: string | null;
};

/** Same as workspace view plus optional deep-link fields for chat citations. */
export type WorkspaceViewHrefParams = WorkspaceViewState & {
  citeQuery?: string | null;
  chunkId?: string | null;
};

function parseNonEmptyParam(url: URL, key: string) {
  const value = url.searchParams.get(key)?.trim();
  return value ? value : null;
}

export function parseWorkspaceViewUrlState(
  url: URL,
): Partial<WorkspaceViewState> {
  const restored: Partial<WorkspaceViewState> = {};
  const selectedSourceId =
    parseNonEmptyParam(url, "source") ?? parseNonEmptyParam(url, "channel");
  const selectedItemId =
    parseNonEmptyParam(url, "item") ?? parseNonEmptyParam(url, "video");
  const contentMode = parseNonEmptyParam(url, "content");
  const videoTypeFilter = parseNonEmptyParam(url, "type");
  const acknowledgedFilter = parseNonEmptyParam(url, "ack");

  if (selectedSourceId) {
    restored.selectedSourceId = selectedSourceId;
    restored.selectedChannelId = selectedSourceId;
  }
  if (selectedItemId) {
    restored.selectedItemId = selectedItemId;
    restored.selectedVideoId = selectedItemId;
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
  const selectedSourceId = state.selectedSourceId ?? state.selectedChannelId;
  const selectedItemId = state.selectedItemId ?? state.selectedVideoId;
  if (selectedSourceId) {
    params.set("source", selectedSourceId);
  }
  if (selectedItemId) {
    params.set("item", selectedItemId);
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
