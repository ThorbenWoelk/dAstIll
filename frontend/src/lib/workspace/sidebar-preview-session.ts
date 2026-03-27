import {
  cloneSyncDepthState,
  type ChannelSyncDepthState,
} from "$lib/channel-view-cache";
import { OTHERS_CHANNEL_ID, type Video } from "$lib/types";

export type SidebarPreviewCollectionLoadMode = "preview" | "paged";

export type SidebarPreviewCollectionSnapshot = {
  videos: Video[];
  expanded: boolean;
  loadedMode: SidebarPreviewCollectionLoadMode | null;
  hasMore: boolean;
  nextOffset: number;
  channelVideoCount: number | null;
  filterKey: string | null;
  syncDepth: ChannelSyncDepthState | null;
  earliestSyncDateInput: string;
  selectedVideoReloadProbeKey: string | null;
};

type ExpandableSidebarPreviewCollection = {
  expanded: boolean;
};

const sidebarPreviewSessionByKey = new Map<
  string,
  Record<string, SidebarPreviewCollectionSnapshot>
>();

function clonePreviewVideos(videos: Video[]): Video[] {
  return videos.map((video) => ({ ...video }));
}

function previewVideosBelongToChannel(channelId: string, videos: Video[]) {
  if (channelId === OTHERS_CHANNEL_ID) {
    return true;
  }
  return videos.every((video) => video.channel_id === channelId);
}

function sanitizePreviewCollection(
  channelId: string,
  collection: SidebarPreviewCollectionSnapshot,
): SidebarPreviewCollectionSnapshot | null {
  if (!previewVideosBelongToChannel(channelId, collection.videos)) {
    return {
      ...collection,
      videos: [],
      loadedMode: null,
      hasMore: false,
      nextOffset: 0,
      channelVideoCount: null,
      filterKey: null,
      syncDepth: null,
      earliestSyncDateInput: "",
      selectedVideoReloadProbeKey: null,
    };
  }

  return {
    ...collection,
    videos: clonePreviewVideos(collection.videos),
    syncDepth: cloneSyncDepthState(collection.syncDepth),
  };
}

export function setSingleExpandedSidebarPreviewCollection<
  T extends ExpandableSidebarPreviewCollection,
>(collections: Record<string, T>, expandedChannelId: string | null) {
  for (const [channelId, collection] of Object.entries(collections)) {
    collection.expanded =
      expandedChannelId !== null && channelId === expandedChannelId;
  }
}

export function resolvePreferredExpandedSidebarPreviewCollectionId<
  T extends ExpandableSidebarPreviewCollection,
>(
  collections: Record<string, T>,
  preferredChannelId: string | null,
): string | null {
  if (preferredChannelId && collections[preferredChannelId]) {
    return preferredChannelId;
  }

  for (const [channelId, collection] of Object.entries(collections)) {
    if (collection.expanded) {
      return channelId;
    }
  }

  return null;
}

export function cloneSidebarPreviewCollections(
  collections: Record<string, SidebarPreviewCollectionSnapshot>,
): Record<string, SidebarPreviewCollectionSnapshot> {
  const cloned: Record<string, SidebarPreviewCollectionSnapshot> = {};

  for (const [channelId, collection] of Object.entries(collections)) {
    const sanitized = sanitizePreviewCollection(channelId, collection);
    if (!sanitized) {
      continue;
    }
    cloned[channelId] = sanitized;
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

    const sanitized = sanitizePreviewCollection(channelId, collection);
    if (!sanitized) {
      continue;
    }
    pruned[channelId] = sanitized;
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
