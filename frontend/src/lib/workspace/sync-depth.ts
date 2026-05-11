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

/**
 * Persisted sync floor for labels and inputs: subscription `earliest_sync_date`.
 * Prefer `syncDepth.earliest_sync_date` when the channel row is missing it (same field from the
 * server; list/bootstrap cache can lag behind a fresh sync-depth fetch).
 */
export function resolveDisplayedSyncDepthIso({
  selectedChannel,
  syncDepth,
}: {
  videos: Video[];
  selectedChannel: Channel | null;
  syncDepth: SyncDepth | null;
  allowLoadedVideoOverride: boolean;
}): string | null {
  return (
    selectedChannel?.earliest_sync_date ?? syncDepth?.earliest_sync_date ?? null
  );
}
