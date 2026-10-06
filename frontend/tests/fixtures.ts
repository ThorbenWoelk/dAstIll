import type { Channel, MiniSummaryItem } from "../src/lib/api";

export function channel(id: string, name = `Channel ${id}`): Channel {
  return {
    id,
    handle: null,
    name,
    thumbnail_url: null,
    added_at: "2026-01-01T00:00:00Z",
    earliest_sync_date: null,
    earliest_sync_date_user_set: false,
  };
}

export function summaryItem(
  videoId: string,
  overrides: Partial<MiniSummaryItem> = {},
): MiniSummaryItem {
  return {
    video_id: videoId,
    channel_id: "c1",
    channel_name: "Channel c1",
    title: `Video ${videoId}`,
    thumbnail_url: null,
    published_at: "2026-10-01T08:00:00Z",
    watch_url: `https://www.youtube.com/watch?v=${videoId}`,
    summary_content: "## Overview\nA short overview.\n\n## Key Points\n- One",
    read: false,
    ...overrides,
  };
}
