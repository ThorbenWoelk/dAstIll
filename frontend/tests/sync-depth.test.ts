import { describe, expect, it } from "bun:test";
import { resolveDisplayedSyncDepthIso } from "../src/lib/sync-depth";
import type { Channel, SyncDepth, Video } from "../src/lib/types";

function createChannel(overrides: Partial<Channel> = {}): Channel {
  return {
    id: "channel-1",
    name: "Channel 1",
    added_at: "2026-03-01T00:00:00.000Z",
    earliest_sync_date: "2026-02-10T00:00:00.000Z",
    earliest_sync_date_user_set: false,
    ...overrides,
  };
}

function createSyncDepth(overrides: Partial<SyncDepth> = {}): SyncDepth {
  return {
    earliest_sync_date: "2026-02-10T00:00:00.000Z",
    earliest_sync_date_user_set: false,
    derived_earliest_ready_date: "2026-02-08T00:00:00.000Z",
    ...overrides,
  };
}

function createVideo(
  publishedAt: string,
  overrides: Partial<Video> = {},
): Video {
  return {
    id: `video-${publishedAt}`,
    channel_id: "channel-1",
    title: "Video",
    published_at: publishedAt,
    is_short: false,
    transcript_status: "ready",
    summary_status: "ready",
    acknowledged: false,
    retry_count: 0,
    ...overrides,
  };
}

describe("resolveDisplayedSyncDepthIso", () => {
  it("keeps the derived sync boundary when older ready videos were not loaded via load more", () => {
    const result = resolveDisplayedSyncDepthIso({
      videos: [
        createVideo("2026-02-01T00:00:00.000Z"),
        createVideo("2026-03-01T00:00:00.000Z"),
      ],
      selectedChannel: createChannel(),
      syncDepth: createSyncDepth(),
      allowLoadedVideoOverride: false,
    });

    expect(result).toBe("2026-02-08T00:00:00.000Z");
  });

  it("uses the oldest loaded ready video after explicit history expansion", () => {
    const result = resolveDisplayedSyncDepthIso({
      videos: [
        createVideo("2026-02-01T00:00:00.000Z"),
        createVideo("2026-03-01T00:00:00.000Z"),
      ],
      selectedChannel: createChannel(),
      syncDepth: createSyncDepth(),
      allowLoadedVideoOverride: true,
    });

    expect(result).toBe("2026-02-01T00:00:00.000Z");
  });

  it("keeps a user-set boundary authoritative", () => {
    const result = resolveDisplayedSyncDepthIso({
      videos: [createVideo("2026-01-15T00:00:00.000Z")],
      selectedChannel: createChannel({
        earliest_sync_date: "2026-02-20T00:00:00.000Z",
        earliest_sync_date_user_set: true,
      }),
      syncDepth: createSyncDepth(),
      allowLoadedVideoOverride: true,
    });

    expect(result).toBe("2026-02-20T00:00:00.000Z");
  });
});
