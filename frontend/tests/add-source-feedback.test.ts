import { describe, expect, it } from "bun:test";

import type { AddVideoResult, Channel, Video } from "../src/lib/types";
import {
  buildChannelAddFeedback,
  buildVideoAddFeedback,
  resolveAddedChannelStatus,
  resolveAddedVideoStatus,
} from "../src/lib/workspace/add-source-feedback";

function makeVideo(
  overrides: Partial<Video> & {
    id: string;
    channel_id?: string;
    transcript_status?: Video["transcript_status"];
    summary_status?: Video["summary_status"];
  },
): Video {
  return {
    id: overrides.id,
    channel_id: overrides.channel_id ?? "channel-1",
    title: overrides.title ?? "Fresh Upload",
    thumbnail_url: null,
    published_at: "2026-03-01T00:00:00.000Z",
    is_short: false,
    transcript_status: overrides.transcript_status ?? "pending",
    summary_status: overrides.summary_status ?? "pending",
    acknowledged: false,
  };
}

function makeChannel(overrides: Partial<Channel> & { id: string }): Channel {
  return {
    id: overrides.id,
    name: overrides.name ?? "Slow Channel",
    handle: overrides.handle ?? "@slow-channel",
    thumbnail_url: null,
    added_at: "2026-03-01T00:00:00.000Z",
    earliest_sync_date: null,
    earliest_sync_date_user_set: false,
  };
}

describe("resolveAddedVideoStatus", () => {
  it("keeps polling while transcript and summary are still pending", () => {
    expect(
      resolveAddedVideoStatus(
        makeVideo({
          id: "video-1",
          transcript_status: "loading",
          summary_status: "pending",
        }),
      ),
    ).toBe("loading");
  });

  it("marks the video ready once the summary is ready", () => {
    expect(
      resolveAddedVideoStatus(
        makeVideo({
          id: "video-1",
          transcript_status: "ready",
          summary_status: "ready",
        }),
      ),
    ).toBe("ready");
  });

  it("surfaces failure when either pipeline stage failed", () => {
    expect(
      resolveAddedVideoStatus(
        makeVideo({
          id: "video-1",
          transcript_status: "failed",
          summary_status: "pending",
        }),
      ),
    ).toBe("failed");
  });
});

describe("resolveAddedChannelStatus", () => {
  it("keeps polling while the initial channel sync has no videos yet", () => {
    expect(resolveAddedChannelStatus([])).toBe("loading");
  });

  it("marks the channel ready once at least one video is available", () => {
    expect(
      resolveAddedChannelStatus([
        makeVideo({
          id: "video-1",
          summary_status: "pending",
        }),
      ]),
    ).toBe("ready");
  });
});

describe("buildVideoAddFeedback", () => {
  const result: AddVideoResult = {
    target_channel_id: "channel-1",
    already_exists: false,
    video: makeVideo({ id: "video-1", title: "Fresh Upload" }),
  };

  it("describes accepted video URLs as asynchronous loading work", () => {
    expect(buildVideoAddFeedback(result, "loading")).toEqual(
      expect.objectContaining({
        kind: "video",
        status: "loading",
        title: "Video accepted",
        actionLabel: null,
      }),
    );
  });

  it("offers an open action once the video is ready", () => {
    expect(buildVideoAddFeedback(result, "ready")).toEqual(
      expect.objectContaining({
        kind: "video",
        status: "ready",
        title: "Video ready",
        actionLabel: "Open video",
        videoId: "video-1",
        targetChannelId: "channel-1",
      }),
    );
  });
});

describe("buildChannelAddFeedback", () => {
  const channel = makeChannel({ id: "channel-1", name: "Slow Channel" });

  it("describes accepted channels as loading until videos arrive", () => {
    expect(buildChannelAddFeedback(channel, "loading")).toEqual(
      expect.objectContaining({
        kind: "channel",
        status: "loading",
        title: "Channel added",
        actionLabel: null,
      }),
    );
  });

  it("offers an open action when the channel is ready to browse", () => {
    expect(buildChannelAddFeedback(channel, "ready")).toEqual(
      expect.objectContaining({
        kind: "channel",
        status: "ready",
        title: "Channel ready",
        actionLabel: "Open channel",
        channelId: "channel-1",
      }),
    );
  });
});
