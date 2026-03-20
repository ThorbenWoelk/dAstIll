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
  loadSnapshot,
  applySnapshot,
  refreshChannel,
  shouldReloadAfterRefresh,
  onRefreshingChange,
  onError,
}: ChannelRefreshWorkflowOptions<TSnapshot>) {
  const snapshot = await loadSnapshot();
  await applySnapshot(snapshot, false);

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

    const refreshedSnapshot = await loadSnapshot();
    await applySnapshot(refreshedSnapshot, true);
  } catch (error) {
    onError((error as Error).message);
  } finally {
    onRefreshingChange(false);
  }
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

export function resolveNextChannelSelection(
  channels: Channel[],
  deletedChannelId: string,
): string | null {
  return (
    channels.find((channel) => channel.id !== deletedChannelId)?.id ?? null
  );
}
