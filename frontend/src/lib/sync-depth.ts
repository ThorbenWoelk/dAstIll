import type { Channel, SyncDepth, Video } from "$lib/types";

export function resolveOldestLoadedReadyVideoDate(
  videos: Video[],
): Date | null {
  let oldest: Date | null = null;

  for (const video of videos) {
    if (
      video.transcript_status !== "ready" ||
      video.summary_status !== "ready"
    ) {
      continue;
    }

    const parsed = new Date(video.published_at);
    if (Number.isNaN(parsed.getTime())) continue;

    if (!oldest || parsed < oldest) {
      oldest = parsed;
    }
  }

  return oldest;
}

export function resolveDisplayedSyncDepthIso({
  videos,
  selectedChannel,
  syncDepth,
  allowLoadedVideoOverride,
}: {
  videos: Video[];
  selectedChannel: Channel | null;
  syncDepth: SyncDepth | null;
  allowLoadedVideoOverride: boolean;
}): string | null {
  if (selectedChannel?.earliest_sync_date_user_set) {
    return selectedChannel.earliest_sync_date ?? null;
  }

  if (allowLoadedVideoOverride) {
    const oldestLoaded = resolveOldestLoadedReadyVideoDate(videos);
    if (oldestLoaded) {
      return oldestLoaded.toISOString();
    }
  }

  return (
    syncDepth?.derived_earliest_ready_date ??
    selectedChannel?.earliest_sync_date ??
    null
  );
}
