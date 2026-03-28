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
    derived_earliest_ready_date: "2012-07-14T00:00:00.000Z",
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
  it("shows the persisted sync floor, not derived oldest-ready or loaded videos", () => {
    const result = resolveDisplayedSyncDepthIso({
      videos: [
        createVideo("2026-02-01T00:00:00.000Z"),
        createVideo("2026-03-01T00:00:00.000Z"),
      ],
      selectedChannel: createChannel({
        earliest_sync_date: "2026-02-10T00:00:00.000Z",
      }),
      syncDepth: createSyncDepth(),
      allowLoadedVideoOverride: true,
    });

    expect(result).toBe("2026-02-10T00:00:00.000Z");
  });

  it("returns null when no sync floor is stored", () => {
    const result = resolveDisplayedSyncDepthIso({
      videos: [createVideo("2026-02-01T00:00:00.000Z")],
      selectedChannel: createChannel({ earliest_sync_date: null }),
      syncDepth: createSyncDepth({ earliest_sync_date: null }),
      allowLoadedVideoOverride: true,
    });

    expect(result).toBeNull();
  });

  it("uses sync depth when the channel list row is missing earliest_sync_date", () => {
    const result = resolveDisplayedSyncDepthIso({
      videos: [],
      selectedChannel: createChannel({ earliest_sync_date: null }),
      syncDepth: createSyncDepth({
        earliest_sync_date: "2026-01-01T00:00:00.000Z",
        derived_earliest_ready_date: "2012-07-14T00:00:00.000Z",
      }),
      allowLoadedVideoOverride: false,
    });

    expect(result).toBe("2026-01-01T00:00:00.000Z");
  });
});
