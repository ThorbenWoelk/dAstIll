import { describe, expect, it } from "bun:test";

import type {
  Channel,
  ChannelSnapshot,
  SyncDepth,
  Video,
} from "../src/lib/types";
import {
  buildQueueGalleryChannelPreviews,
  deriveEarliestSyncDateInput,
  deriveEffectiveEarliestSyncDate,
  deriveQueueRefreshCadence,
  deriveQueueStats,
  videoPipelineInFlight,
} from "../src/lib/queue/route-state";

function makeVideo(
  overrides: Partial<Video> & {
    id: string;
    transcript_status: Video["transcript_status"];
    summary_status: Video["summary_status"];
  },
): Video {
  return {
    id: overrides.id,
    channel_id: overrides.channel_id ?? "channel-1",
    title: overrides.title ?? "Video",
    thumbnail_url: null,
    published_at: "2024-01-01T00:00:00Z",
    is_short: false,
    transcript_status: overrides.transcript_status,
    summary_status: overrides.summary_status,
    acknowledged: false,
    ...overrides,
  };
}

function makeChannel(overrides: Partial<Channel> & { id: string }): Channel {
  return {
    id: overrides.id,
    handle: overrides.handle ?? "@channel",
    name: overrides.name ?? "Channel",
    thumbnail_url: null,
    added_at: "2024-01-01T00:00:00Z",
    earliest_sync_date: overrides.earliest_sync_date ?? null,
    earliest_sync_date_user_set: overrides.earliest_sync_date_user_set ?? false,
  };
}

function makeSyncDepth(overrides: Partial<SyncDepth> = {}): SyncDepth {
  return {
    earliest_sync_date: overrides.earliest_sync_date ?? null,
    earliest_sync_date_user_set: overrides.earliest_sync_date_user_set ?? false,
    derived_earliest_ready_date: overrides.derived_earliest_ready_date ?? null,
  };
}

describe("videoPipelineInFlight", () => {
  it("treats pending transcript work as in flight", () => {
    expect(
      videoPipelineInFlight(
        makeVideo({
          id: "video-1",
          transcript_status: "pending",
          summary_status: "pending",
        }),
      ),
    ).toBe(true);
  });

  it("treats ready transcript plus pending summary as in flight", () => {
    expect(
      videoPipelineInFlight(
        makeVideo({
          id: "video-1",
          transcript_status: "ready",
          summary_status: "pending",
        }),
      ),
    ).toBe(true);
  });

  it("treats fully ready content as settled", () => {
    expect(
      videoPipelineInFlight(
        makeVideo({
          id: "video-1",
          transcript_status: "ready",
          summary_status: "ready",
        }),
      ),
    ).toBe(false);
  });
});

describe("deriveQueueStats", () => {
  it("counts loading, pending, and failed queue work across transcript and summary stages", () => {
    const stats = deriveQueueStats([
      makeVideo({
        id: "loading-transcript",
        transcript_status: "loading",
        summary_status: "pending",
      }),
      makeVideo({
        id: "pending-summary",
        transcript_status: "ready",
        summary_status: "pending",
      }),
      makeVideo({
        id: "failed-summary",
        transcript_status: "ready",
        summary_status: "failed",
      }),
      makeVideo({
        id: "ready",
        transcript_status: "ready",
        summary_status: "ready",
      }),
    ]);

    expect(stats).toEqual({
      total: 4,
      loading: 1,
      pending: 1,
      failed: 1,
    });
  });
});

describe("deriveQueueRefreshCadence", () => {
  it("disables refresh when not in the browser or no channel is selected", () => {
    expect(
      deriveQueueRefreshCadence({
        browser: false,
        selectedChannelId: "channel-1",
        loadingVideos: false,
        videos: [],
      }),
    ).toBe("off");

    expect(
      deriveQueueRefreshCadence({
        browser: true,
        selectedChannelId: null,
        loadingVideos: false,
        videos: [],
      }),
    ).toBe("off");
  });

  it("uses fast cadence while work is in flight", () => {
    expect(
      deriveQueueRefreshCadence({
        browser: true,
        selectedChannelId: "channel-1",
        loadingVideos: false,
        videos: [
          makeVideo({
            id: "video-1",
            transcript_status: "ready",
            summary_status: "pending",
          }),
        ],
      }),
    ).toBe("fast");
  });

  it("uses slow cadence when the queue is settled", () => {
    expect(
      deriveQueueRefreshCadence({
        browser: true,
        selectedChannelId: "channel-1",
        loadingVideos: false,
        videos: [
          makeVideo({
            id: "video-1",
            transcript_status: "ready",
            summary_status: "ready",
          }),
        ],
      }),
    ).toBe("slow");
  });

  it("uses idle cadence when the selected queue is empty", () => {
    expect(
      deriveQueueRefreshCadence({
        browser: true,
        selectedChannelId: "channel-1",
        loadingVideos: false,
        videos: [],
      }),
    ).toBe("idle");
  });
});

describe("deriveEffectiveEarliestSyncDate", () => {
  it("prefers a user-set channel boundary", () => {
    expect(
      deriveEffectiveEarliestSyncDate(
        makeChannel({
          id: "channel-1",
          earliest_sync_date: "2024-02-10T00:00:00.000Z",
          earliest_sync_date_user_set: true,
        }),
        makeSyncDepth({
          derived_earliest_ready_date: "2024-02-15T00:00:00.000Z",
        }),
      ),
    ).toBe("2024-02-10T00:00:00.000Z");
  });

  it("falls back to derived sync depth when no user-set boundary exists", () => {
    expect(
      deriveEffectiveEarliestSyncDate(
        makeChannel({
          id: "channel-1",
          earliest_sync_date: "2024-02-10T00:00:00.000Z",
          earliest_sync_date_user_set: false,
        }),
        makeSyncDepth({
          derived_earliest_ready_date: "2024-02-15T00:00:00.000Z",
        }),
      ),
    ).toBe("2024-02-15T00:00:00.000Z");
  });
});

describe("deriveEarliestSyncDateInput", () => {
  it("formats the effective boundary as an input date", () => {
    expect(
      deriveEarliestSyncDateInput(
        makeChannel({
          id: "channel-1",
          earliest_sync_date: "2024-02-10T08:30:00.000Z",
          earliest_sync_date_user_set: true,
        }),
        null,
      ),
    ).toBe("2024-02-10");
  });
});

describe("buildQueueGalleryChannelPreviews", () => {
  it("merges the live selected-channel queue snapshot into the gallery previews", () => {
    const basePreview: ChannelSnapshot = {
      channel_id: "channel-2",
      sync_depth: makeSyncDepth(),
      channel_video_count: 1,
      has_more: false,
      next_offset: 1,
      videos: [
        makeVideo({
          id: "existing-video",
          channel_id: "channel-2",
          transcript_status: "ready",
          summary_status: "ready",
        }),
      ],
    };

    const previews = buildQueueGalleryChannelPreviews({
      basePreviews: { "channel-2": basePreview },
      selectedChannelId: "channel-1",
      syncDepth: makeSyncDepth({
        derived_earliest_ready_date: "2024-02-15T00:00:00.000Z",
      }),
      videos: [
        makeVideo({
          id: "selected-video",
          channel_id: "channel-1",
          transcript_status: "ready",
          summary_status: "pending",
        }),
      ],
      hasMore: true,
      offset: 20,
    });

    expect(previews["channel-2"]).toBe(basePreview);
    expect(previews["channel-1"]).toEqual({
      channel_id: "channel-1",
      sync_depth: makeSyncDepth({
        derived_earliest_ready_date: "2024-02-15T00:00:00.000Z",
      }),
      channel_video_count: 1,
      has_more: true,
      next_offset: 20,
      videos: [
        makeVideo({
          id: "selected-video",
          channel_id: "channel-1",
          transcript_status: "ready",
          summary_status: "pending",
        }),
      ],
    });
  });
});
