import type { Channel, Video, VideoTypeFilter } from "$lib/types";
import {
  markChannelRefreshed,
  shouldRefreshChannel,
} from "$lib/channel-workspace";
import type { AcknowledgedFilter } from "$lib/workspace/types";

type ChannelRefreshWorkflowOptions<TSnapshot> = {
  channelId: string;
  refreshedAtByChannel: Map<string, number>;
  ttlMs: number;
  bypassTtl?: boolean;
  initialSilent?: boolean;
  /**
   * When the video list was mutated (e.g. read toggle) while a snapshot was
   * in flight, skip applying that snapshot so stale data cannot overwrite UI.
   */
  getMutationEpoch?: () => number;
  loadSnapshot: () => Promise<TSnapshot>;
  applySnapshot: (snapshot: TSnapshot, silent?: boolean) => Promise<void>;
  refreshChannel: () => Promise<unknown>;
  shouldReloadAfterRefresh: () => boolean;
  onRefreshingChange: (refreshing: boolean) => void;
  onError: (message: string) => void;
};

export async function loadChannelSnapshotWithRefresh<TSnapshot>({
  channelId,
  refreshedAtByChannel,
  ttlMs,
  bypassTtl = false,
  initialSilent = false,
  getMutationEpoch,
  loadSnapshot,
  applySnapshot,
  refreshChannel,
  shouldReloadAfterRefresh,
  onRefreshingChange,
  onError,
}: ChannelRefreshWorkflowOptions<TSnapshot>) {
  const epochBeforeFirst = getMutationEpoch?.() ?? 0;
  const snapshot = await loadSnapshot();
  if (getMutationEpoch && getMutationEpoch() !== epochBeforeFirst) {
    return;
  }
  await applySnapshot(snapshot, initialSilent);

  if (
    !bypassTtl &&
    !shouldRefreshChannel(refreshedAtByChannel, channelId, ttlMs)
  ) {
    return;
  }

  onRefreshingChange(true);
  try {
    await refreshChannel();
    markChannelRefreshed(refreshedAtByChannel, channelId);
    if (!shouldReloadAfterRefresh()) {
      return;
    }

    const epochBeforeSecond = getMutationEpoch?.() ?? 0;
    const refreshedSnapshot = await loadSnapshot();
    if (getMutationEpoch && getMutationEpoch() !== epochBeforeSecond) {
      return;
    }
    await applySnapshot(refreshedSnapshot, true);
  } catch (error) {
    onError((error as Error).message);
  } finally {
    onRefreshingChange(false);
  }
}

/**
 * Stable-unique by `video.id` (first occurrence wins). Prevents duplicate keys
 * in keyed `{#each}` when API pagination or merges return overlapping rows.
 */
export function dedupeVideosById(videos: Video[]): Video[] {
  const seen = new Set<string>();
  const out: Video[] = [];
  for (const v of videos) {
    if (seen.has(v.id)) continue;
    seen.add(v.id);
    out.push(v);
  }
  return out;
}

export function shouldForceReloadMissingSelectedVideo(params: {
  selectedVideoId: string | null;
  videos: Array<Pick<Video, "id">>;
  probeKey: string;
  lastProbeKey: string | null;
}): boolean {
  if (!params.selectedVideoId) {
    return false;
  }

  if (params.videos.some((video) => video.id === params.selectedVideoId)) {
    return false;
  }

  return params.lastProbeKey !== params.probeKey;
}

export function shouldLoadAllChannelVideosForSelection(params: {
  selectedVideoId: string | null;
  videos: Array<Pick<Video, "id">>;
  loadedMode: "preview" | "paged" | null;
  hasMore: boolean;
}): boolean {
  if (!params.selectedVideoId) {
    return false;
  }

  if (params.videos.some((video) => video.id === params.selectedVideoId)) {
    return false;
  }

  if (params.loadedMode === "preview") {
    return true;
  }

  if (params.loadedMode === "paged") {
    return params.hasMore;
  }

  return false;
}

export function resolveInitialPreviewExpandedChannelId(
  channels: Channel[],
  selectedChannelId: string | null,
  virtualChannelId: string,
): string | null {
  if (selectedChannelId) {
    const selected = channels.find(
      (channel) => channel.id === selectedChannelId,
    );
    if (selected && selected.id !== virtualChannelId) {
      return selected.id;
    }
  }

  return (
    channels.find((channel) => channel.id !== virtualChannelId)?.id ?? null
  );
}

export function filterVideosByType(
  videos: Video[],
  filter: VideoTypeFilter,
): Video[] {
  return videos.filter((video) => {
    if (filter === "long") return !video.is_short;
    if (filter === "short") return video.is_short;
    return true;
  });
}

export function filterVideosByAcknowledged(
  videos: Video[],
  filter: AcknowledgedFilter,
): Video[] {
  return videos.filter((video) => {
    if (filter === "ack") return video.acknowledged;
    if (filter === "unack") return !video.acknowledged;
    return true;
  });
}

export async function applyVideoTypeFilterChange(params: {
  currentFilter: VideoTypeFilter;
  nextFilter: VideoTypeFilter;
  videos: Video[];
  setFilter: (filter: VideoTypeFilter) => void;
  setVideos: (videos: Video[]) => void;
  reload: () => Promise<void>;
}): Promise<boolean> {
  if (params.currentFilter === params.nextFilter) {
    return false;
  }

  params.setFilter(params.nextFilter);
  params.setVideos(filterVideosByType(params.videos, params.nextFilter));
  await params.reload();
  return true;
}

export async function applyAcknowledgedFilterChange(params: {
  currentFilter: AcknowledgedFilter;
  nextFilter: AcknowledgedFilter;
  videos: Video[];
  setFilter: (filter: AcknowledgedFilter) => void;
  setVideos: (videos: Video[]) => void;
  reload: () => Promise<void>;
}): Promise<boolean> {
  if (params.currentFilter === params.nextFilter) {
    return false;
  }

  params.setFilter(params.nextFilter);
  params.setVideos(
    filterVideosByAcknowledged(params.videos, params.nextFilter),
  );
  await params.reload();
  return true;
}

export async function clearSidebarVideoFilters(params: {
  videoTypeFilter: VideoTypeFilter;
  acknowledgedFilter: AcknowledgedFilter;
  setVideoTypeFilter: (filter: VideoTypeFilter) => void;
  setAcknowledgedFilter: (filter: AcknowledgedFilter) => void;
  reload: () => Promise<void>;
}): Promise<boolean> {
  if (params.videoTypeFilter === "all" && params.acknowledgedFilter === "all") {
    return false;
  }

  params.setVideoTypeFilter("all");
  params.setAcknowledgedFilter("all");
  await params.reload();
  return true;
}

export function resolveNextChannelSelection(
  channels: Channel[],
  deletedChannelId: string,
): string | null {
  return (
    channels.find((channel) => channel.id !== deletedChannelId)?.id ?? null
  );
}

export function resolveVirtualWindow(params: {
  itemCount: number;
  itemHeight: number;
  viewportHeight: number;
  scrollTop: number;
  overscan: number;
}) {
  if (params.itemCount <= 0 || params.itemHeight <= 0) {
    return {
      startIndex: 0,
      endIndex: 0,
      offsetTop: 0,
      totalHeight: 0,
    };
  }

  const visibleCount = Math.max(
    1,
    Math.ceil(params.viewportHeight / params.itemHeight),
  );
  const rawStart = Math.floor(params.scrollTop / params.itemHeight);
  const startIndex = Math.max(0, rawStart - params.overscan);
  const endIndex = Math.min(
    params.itemCount,
    rawStart + visibleCount + params.overscan,
  );

  return {
    startIndex,
    endIndex,
    offsetTop: startIndex * params.itemHeight,
    totalHeight: params.itemCount * params.itemHeight,
  };
}
