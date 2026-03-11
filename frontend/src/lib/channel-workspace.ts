import type { Channel, QueueTab } from "./types";

export const WORKSPACE_STATE_KEY = "dastill.workspace.state.v1";

export interface WorkspaceStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

export interface WorkspaceStateSnapshot {
  selectedChannelId: string | null;
  selectedVideoId: string | null;
  contentMode: "transcript" | "summary" | "info";
  videoTypeFilter: "all" | "long" | "short";
  hideShorts?: boolean;
  acknowledgedFilter: "all" | "unack" | "ack";
  channelOrder: string[];
  channelSortMode?: "custom" | "alpha" | "newest";
}

export interface RestoreWorkspaceSnapshotOptions {
  includeSelectedVideoId?: boolean;
  includeContentMode?: boolean;
  includeVideoTypeFilter?: boolean;
  includeAcknowledgedFilter?: boolean;
  includeChannelSortMode?: boolean;
}

export interface ChannelDragState {
  draggedChannelId: string | null;
  dragOverChannelId: string | null;
}

interface ChannelDragTransfer {
  effectAllowed: string;
  setData(type: string, value: string): void;
}

const CONTENT_MODES = new Set<WorkspaceStateSnapshot["contentMode"]>([
  "transcript",
  "summary",
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
const CHANNEL_SORT_MODES = new Set<
  NonNullable<WorkspaceStateSnapshot["channelSortMode"]>
>(["custom", "alpha", "newest"]);

export function prioritizeChannelOrder(
  channelOrder: string[],
  channelId: string,
): string[] {
  return [channelId, ...channelOrder.filter((id) => id !== channelId)];
}

export function loadWorkspaceState(
  storage: WorkspaceStorage,
): Partial<WorkspaceStateSnapshot> | null {
  const raw = storage.getItem(WORKSPACE_STATE_KEY);
  if (!raw) {
    return null;
  }

  try {
    return JSON.parse(raw) as Partial<WorkspaceStateSnapshot>;
  } catch {
    storage.removeItem(WORKSPACE_STATE_KEY);
    return null;
  }
}

export function saveWorkspaceState(
  storage: WorkspaceStorage,
  snapshot: Partial<WorkspaceStateSnapshot>,
) {
  const current = loadWorkspaceState(storage) ?? {};
  storage.setItem(
    WORKSPACE_STATE_KEY,
    JSON.stringify({
      ...current,
      ...snapshot,
    }),
  );
}

export function restoreWorkspaceSnapshot(
  snapshot: Partial<WorkspaceStateSnapshot> | null,
  options: RestoreWorkspaceSnapshotOptions = {},
): Partial<WorkspaceStateSnapshot> {
  if (!snapshot) {
    return {};
  }

  const restored: Partial<WorkspaceStateSnapshot> = {};
  if (
    typeof snapshot.selectedChannelId === "string" ||
    snapshot.selectedChannelId === null
  ) {
    restored.selectedChannelId = snapshot.selectedChannelId;
  }

  if (Array.isArray(snapshot.channelOrder)) {
    restored.channelOrder = snapshot.channelOrder.filter(
      (id): id is string => typeof id === "string",
    );
  }

  if (
    options.includeSelectedVideoId &&
    (typeof snapshot.selectedVideoId === "string" ||
      snapshot.selectedVideoId === null)
  ) {
    restored.selectedVideoId = snapshot.selectedVideoId;
  }

  if (
    options.includeContentMode &&
    snapshot.contentMode &&
    CONTENT_MODES.has(snapshot.contentMode)
  ) {
    restored.contentMode = snapshot.contentMode;
  }

  if (options.includeVideoTypeFilter) {
    if (
      snapshot.videoTypeFilter &&
      VIDEO_TYPE_FILTERS.has(snapshot.videoTypeFilter)
    ) {
      restored.videoTypeFilter = snapshot.videoTypeFilter;
    } else if (typeof snapshot.hideShorts === "boolean") {
      restored.videoTypeFilter = snapshot.hideShorts ? "long" : "all";
    }
  }

  if (
    options.includeAcknowledgedFilter &&
    snapshot.acknowledgedFilter &&
    ACKNOWLEDGED_FILTERS.has(snapshot.acknowledgedFilter)
  ) {
    restored.acknowledgedFilter = snapshot.acknowledgedFilter;
  }

  if (
    options.includeChannelSortMode &&
    snapshot.channelSortMode &&
    CHANNEL_SORT_MODES.has(snapshot.channelSortMode)
  ) {
    restored.channelSortMode = snapshot.channelSortMode;
  }

  return restored;
}

export function applySavedChannelOrder(
  nextChannels: Channel[],
  channelOrder: string[],
): Channel[] {
  if (channelOrder.length === 0) return nextChannels;
  const byId = new Map(nextChannels.map((channel) => [channel.id, channel]));
  const ordered: Channel[] = [];
  const seen = new Set<string>();

  for (const id of channelOrder) {
    const channel = byId.get(id);
    if (!channel) continue;
    ordered.push(channel);
    seen.add(id);
  }

  for (const channel of nextChannels) {
    if (!seen.has(channel.id)) {
      ordered.push(channel);
    }
  }

  return ordered;
}

export function resolveInitialChannelSelection(
  channels: Channel[],
  selectedChannelId: string | null,
  preferredChannelId: string | null,
): string | null {
  if (channels.length === 0) return null;

  if (
    preferredChannelId &&
    channels.some((channel) => channel.id === preferredChannelId)
  ) {
    return preferredChannelId;
  }

  if (
    selectedChannelId &&
    channels.some((channel) => channel.id === selectedChannelId)
  ) {
    return selectedChannelId;
  }

  return channels[0].id;
}

export function reorderChannels(
  channels: Channel[],
  dragId: string,
  targetId: string,
): { channels: Channel[]; channelOrder: string[] } | null {
  if (dragId === targetId) {
    return null;
  }

  const channelOrder = channels.map((channel) => channel.id);
  const fromIndex = channelOrder.indexOf(dragId);
  const toIndex = channelOrder.indexOf(targetId);
  if (fromIndex < 0 || toIndex < 0) {
    return null;
  }

  channelOrder.splice(fromIndex, 1);
  channelOrder.splice(toIndex, 0, dragId);

  const channelsById = new Map(
    channels.map((channel) => [channel.id, channel]),
  );
  return {
    channels: channelOrder
      .map((id) => channelsById.get(id))
      .filter((channel): channel is Channel => !!channel),
    channelOrder,
  };
}

export function beginChannelDrag(
  channelId: string,
  dataTransfer?: ChannelDragTransfer | null,
): ChannelDragState {
  if (dataTransfer) {
    dataTransfer.effectAllowed = "move";
    dataTransfer.setData("text/plain", channelId);
  }

  return {
    draggedChannelId: channelId,
    dragOverChannelId: channelId,
  };
}

export function updateChannelDragOver(
  currentDragOverChannelId: string | null,
  channelId: string,
) {
  return currentDragOverChannelId === channelId
    ? currentDragOverChannelId
    : channelId;
}

export function finishChannelDrag(): ChannelDragState {
  return {
    draggedChannelId: null,
    dragOverChannelId: null,
  };
}

export function completeChannelDrop(
  _channelId: string,
  draggedChannelId: string | null,
  fallbackChannelId: string | null,
) {
  return {
    sourceId: draggedChannelId || fallbackChannelId,
    dragState: finishChannelDrag(),
  };
}

export function shouldRefreshChannel(
  refreshedAtByChannel: Map<string, number>,
  channelId: string,
  ttlMs: number,
  now = Date.now(),
) {
  const lastRefresh = refreshedAtByChannel.get(channelId);
  if (lastRefresh === undefined) {
    return true;
  }

  return now - lastRefresh >= ttlMs;
}

export function markChannelRefreshed(
  refreshedAtByChannel: Map<string, number>,
  channelId: string,
  now = Date.now(),
) {
  refreshedAtByChannel.set(channelId, now);
}

export function buildQueueSnapshotOptions(
  queueTab: QueueTab,
  limit: number,
  offset = 0,
) {
  return {
    limit,
    offset,
    queueTab,
  };
}
