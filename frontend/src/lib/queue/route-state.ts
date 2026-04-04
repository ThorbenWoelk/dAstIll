import type {
  Channel,
  ChannelSnapshot,
  ContentItem,
  ContentPart,
  ContentSource,
  SubscriptionContainer,
  SyncDepth,
  Video,
} from "$lib/types";
import type { ChannelSyncDepthState } from "$lib/channel-view-cache";
import { defaultEarliestSyncFloorDateInputValue } from "$lib/workspace/sidebar-sync-date";
import type { QueueStats } from "$lib/workspace/types";

export type QueueRefreshCadence = "off" | "fast" | "slow" | "idle";

function buildQueuePreviewContainer(channelId: string): SubscriptionContainer {
  return {
    id: `youtube:series:${channelId}`,
    kind: "series",
    title: channelId,
    provider: "you_tube",
    backing_kind: "feed",
    user_editable: false,
    source_ids: [channelId],
  };
}

function buildQueuePreviewSource(channelId: string): ContentSource {
  const container = buildQueuePreviewContainer(channelId);
  return {
    id: channelId,
    provider: "you_tube",
    source_kind: "you_tube_channel",
    container_id: container.id,
    container_kind: container.kind,
    backing_kind: "feed",
    title: channelId,
    subtitle: undefined,
    handle: undefined,
    thumbnail_url: undefined,
    requires_auth: false,
    public_content_available: true,
    entitled_content_available: true,
    external_ids: [{ provider: "you_tube", external_id: channelId }],
  };
}

function buildQueuePreviewItems(videos: Video[]): ContentItem[] {
  return videos.map((video) => ({
    id: video.id,
    source_id: video.channel_id,
    provider: "you_tube",
    item_kind: "video",
    title: video.title,
    thumbnail_url: video.thumbnail_url ?? undefined,
    published_at: video.published_at,
    external_ids: [{ provider: "you_tube", external_id: video.id }],
  }));
}

function buildQueuePreviewParts(videos: Video[]): ContentPart[] {
  return videos.flatMap((video) => [
    {
      id: `transcript:${video.id}`,
      source_id: video.channel_id,
      item_id: video.id,
      provider: "you_tube",
      part_kind: "transcript",
      status: video.transcript_status,
      text_available: video.transcript_status === "ready",
    },
    {
      id: `summary:${video.id}`,
      source_id: video.channel_id,
      item_id: video.id,
      provider: "you_tube",
      part_kind: "generated_summary",
      status: video.summary_status,
      text_available: video.summary_status === "ready",
    },
  ]);
}

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
  return (
    selectedChannel?.earliest_sync_date ?? syncDepth?.earliest_sync_date ?? null
  );
}

export function deriveEarliestSyncDateInput(
  selectedChannel: Channel | null,
  syncDepth: ChannelSyncDepthState | null,
  now: Date = new Date(),
): string {
  const effective = deriveEffectiveEarliestSyncDate(selectedChannel, syncDepth);
  if (effective) {
    return new Date(effective).toISOString().split("T")[0];
  }
  return defaultEarliestSyncFloorDateInputValue(now);
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
    const container = buildQueuePreviewContainer(selectedChannelId);
    merged[selectedChannelId] = {
      channel_id: selectedChannelId,
      source_id: selectedChannelId,
      container,
      source: buildQueuePreviewSource(selectedChannelId),
      sync_depth: syncDepth,
      channel_video_count: videos.length,
      has_more: hasMore,
      next_offset: offset,
      videos,
      items: buildQueuePreviewItems(videos),
      parts: buildQueuePreviewParts(videos),
    };
  }

  return merged;
}
