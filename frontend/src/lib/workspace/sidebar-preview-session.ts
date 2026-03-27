import {
  cloneSyncDepthState,
  type ChannelSyncDepthState,
} from "$lib/channel-view-cache";
import type { Video } from "$lib/types";

export type SidebarPreviewCollectionLoadMode = "preview" | "all";

export type SidebarPreviewCollectionSnapshot = {
  videos: Video[];
  expanded: boolean;
  loadedMode: SidebarPreviewCollectionLoadMode | null;
  hasMoreThanPreview: boolean;
  channelVideoCount: number | null;
  filterKey: string | null;
  syncDepth: ChannelSyncDepthState | null;
  earliestSyncDateInput: string;
  selectedVideoReloadProbeKey: string | null;
};

const sidebarPreviewSessionByKey = new Map<
  string,
  Record<string, SidebarPreviewCollectionSnapshot>
>();

function clonePreviewVideos(videos: Video[]): Video[] {
  return videos.map((video) => ({ ...video }));
}

export function cloneSidebarPreviewCollections(
  collections: Record<string, SidebarPreviewCollectionSnapshot>,
): Record<string, SidebarPreviewCollectionSnapshot> {
  const cloned: Record<string, SidebarPreviewCollectionSnapshot> = {};

  for (const [channelId, collection] of Object.entries(collections)) {
    cloned[channelId] = {
      ...collection,
      videos: clonePreviewVideos(collection.videos),
      syncDepth: cloneSyncDepthState(collection.syncDepth),
    };
  }

  return cloned;
}

export function pruneSidebarPreviewCollections(
  collections: Record<string, SidebarPreviewCollectionSnapshot>,
  channelIds: Iterable<string>,
): Record<string, SidebarPreviewCollectionSnapshot> {
  const allowedIds = new Set(channelIds);
  const pruned: Record<string, SidebarPreviewCollectionSnapshot> = {};

  for (const [channelId, collection] of Object.entries(collections)) {
    if (!allowedIds.has(channelId)) {
      continue;
    }

    pruned[channelId] = {
      ...collection,
      videos: clonePreviewVideos(collection.videos),
      syncDepth: cloneSyncDepthState(collection.syncDepth),
    };
  }

  return pruned;
}

export function getSidebarPreviewSession(
  sessionKey: string,
): Record<string, SidebarPreviewCollectionSnapshot> | null {
  const snapshot = sidebarPreviewSessionByKey.get(sessionKey);
  return snapshot ? cloneSidebarPreviewCollections(snapshot) : null;
}

export function setSidebarPreviewSession(
  sessionKey: string,
  collections: Record<string, SidebarPreviewCollectionSnapshot>,
) {
  sidebarPreviewSessionByKey.set(
    sessionKey,
    cloneSidebarPreviewCollections(collections),
  );
}

export function clearSidebarPreviewSession(sessionKey: string) {
  sidebarPreviewSessionByKey.delete(sessionKey);
}
