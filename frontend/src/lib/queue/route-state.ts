import type { Channel, ChannelSnapshot, SyncDepth, Video } from "$lib/types";
import type { ChannelSyncDepthState } from "$lib/channel-view-cache";
import type { QueueStats } from "$lib/workspace/types";

export type QueueRefreshCadence = "off" | "fast" | "slow" | "idle";

/** Transcript or summary still running for unified queue visibility. */
export function videoPipelineInFlight(video: Video): boolean {
  return (
    video.transcript_status === "pending" ||
    video.transcript_status === "loading" ||
    (video.transcript_status === "ready" &&
      (video.summary_status === "pending" ||
        video.summary_status === "loading"))
  );
}

export function deriveQueueStats(videos: Video[]): QueueStats {
  return {
    total: videos.length,
    loading: videos.filter(
      (video) =>
        video.transcript_status === "loading" ||
        video.summary_status === "loading",
    ).length,
    pending: videos.filter(
      (video) =>
        video.transcript_status === "pending" ||
        (video.transcript_status === "ready" &&
          video.summary_status === "pending"),
    ).length,
    failed: videos.filter(
      (video) =>
        video.transcript_status === "failed" ||
        video.summary_status === "failed",
    ).length,
  };
}

export function deriveQueueRefreshCadence({
  browser,
  selectedChannelId,
  loadingVideos,
  videos,
}: {
  browser: boolean;
  selectedChannelId: string | null;
  loadingVideos: boolean;
  videos: Video[];
}): QueueRefreshCadence {
  if (!browser) return "off";
  if (!selectedChannelId) return "off";
  if (loadingVideos) return "off";
  if (videos.some(videoPipelineInFlight)) return "fast";
  if (videos.length > 0) return "slow";
  return "idle";
}

export function deriveEffectiveEarliestSyncDate(
  selectedChannel: Channel | null,
  syncDepth: ChannelSyncDepthState | null,
): string | null {
  if (!selectedChannel) {
    return null;
  }

  if (selectedChannel.earliest_sync_date_user_set) {
    return selectedChannel.earliest_sync_date ?? null;
  }

  return (
    syncDepth?.derived_earliest_ready_date ??
    selectedChannel.earliest_sync_date ??
    null
  );
}

export function deriveEarliestSyncDateInput(
  selectedChannel: Channel | null,
  syncDepth: ChannelSyncDepthState | null,
): string {
  const effective = deriveEffectiveEarliestSyncDate(selectedChannel, syncDepth);
  return effective ? new Date(effective).toISOString().split("T")[0] : "";
}

export function buildQueueGalleryChannelPreviews({
  basePreviews,
  selectedChannelId,
  syncDepth,
  videos,
  hasMore,
  offset,
}: {
  basePreviews: Record<string, ChannelSnapshot>;
  selectedChannelId: string | null;
  syncDepth: SyncDepth | null;
  videos: Video[];
  hasMore: boolean;
  offset: number;
}): Record<string, ChannelSnapshot> {
  const merged = { ...basePreviews };

  if (selectedChannelId && syncDepth) {
    merged[selectedChannelId] = {
      channel_id: selectedChannelId,
      sync_depth: syncDepth,
      channel_video_count: videos.length,
      has_more: hasMore,
      next_offset: offset,
      videos,
    };
  }

  return merged;
}
