import { describe, expect, it } from "bun:test";

import {
  filterVideosByAcknowledged,
  filterVideosByType,
  resolveInitialPreviewExpandedChannelId,
  resolveNextChannelSelection,
  resolveVirtualWindow,
  shouldLoadAllChannelVideosForSelection,
  shouldForceReloadMissingSelectedVideo,
} from "../src/lib/workspace/route-helpers";
import type { Video } from "../src/lib/types";
import type { Channel } from "../src/lib/types";

function makeVideo(
  overrides: Partial<Video> & {
    id: string;
    is_short: boolean;
    acknowledged: boolean;
  },
): Video {
  return {
    published_at: "2024-01-01T00:00:00Z",
    title: "Test Video",
    channel_id: "channel-1",
    thumbnail_url: null,
    transcript_status: "ready",
    summary_status: "ready",
    duration_seconds: 300,
    description: null,
    ...overrides,
  } as unknown as Video;
}

function makeChannel(id: string): Channel {
  return {
    id,
    name: `Channel ${id}`,
    handle: `@channel-${id}`,
    thumbnail_url: null,
    earliest_sync_date: null,
    earliest_sync_date_user_set: false,
    added_at: "2024-01-01T00:00:00Z",
  } as unknown as Channel;
}

describe("filterVideosByType", () => {
  const longVideo = makeVideo({
    id: "long-1",
    is_short: false,
    acknowledged: false,
  });
  const shortVideo = makeVideo({
    id: "short-1",
    is_short: true,
    acknowledged: false,
  });
  const videos = [longVideo, shortVideo];

  it("returns all videos when filter is 'all'", () => {
    expect(filterVideosByType(videos, "all")).toHaveLength(2);
  });

  it("filters to only long videos when filter is 'long'", () => {
    const result = filterVideosByType(videos, "long");
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe("long-1");
  });

  it("filters to only short videos when filter is 'short'", () => {
    const result = filterVideosByType(videos, "short");
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe("short-1");
  });

  it("returns empty list when no videos match the filter", () => {
    expect(filterVideosByType([longVideo], "short")).toHaveLength(0);
  });
});

describe("filterVideosByAcknowledged", () => {
  const unacknowledgedVideo = makeVideo({
    id: "unack-1",
    is_short: false,
    acknowledged: false,
  });
  const acknowledgedVideo = makeVideo({
    id: "ack-1",
    is_short: false,
    acknowledged: true,
  });
  const videos = [unacknowledgedVideo, acknowledgedVideo];

  it("returns all videos when filter is 'all'", () => {
    expect(filterVideosByAcknowledged(videos, "all")).toHaveLength(2);
  });

  it("filters to only acknowledged videos when filter is 'ack'", () => {
    const result = filterVideosByAcknowledged(videos, "ack");
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe("ack-1");
  });

  it("filters to only unacknowledged videos when filter is 'unack'", () => {
    const result = filterVideosByAcknowledged(videos, "unack");
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe("unack-1");
  });

  it("returns empty list when no videos match the filter", () => {
    expect(
      filterVideosByAcknowledged([acknowledgedVideo], "unack"),
    ).toHaveLength(0);
  });
});

describe("resolveNextChannelSelection", () => {
  const channelA = makeChannel("channel-a");
  const channelB = makeChannel("channel-b");
  const channelC = makeChannel("channel-c");

  it("returns the first channel that is not the deleted one", () => {
    const result = resolveNextChannelSelection(
      [channelA, channelB, channelC],
      "channel-a",
    );
    expect(result).toBe("channel-b");
  });

  it("returns the remaining channel when only two channels exist", () => {
    const result = resolveNextChannelSelection(
      [channelA, channelB],
      "channel-b",
    );
    expect(result).toBe("channel-a");
  });

  it("returns null when only the deleted channel remains", () => {
    const result = resolveNextChannelSelection([channelA], "channel-a");
    expect(result).toBeNull();
  });

  it("returns null when the channel list is empty", () => {
    const result = resolveNextChannelSelection([], "channel-a");
    expect(result).toBeNull();
  });
});

describe("shouldForceReloadMissingSelectedVideo", () => {
  it("returns true the first time a selected video is missing from scope", () => {
    const result = shouldForceReloadMissingSelectedVideo({
      selectedVideoId: "video-2",
      videos: [
        makeVideo({ id: "video-1", is_short: false, acknowledged: false }),
      ],
      probeKey: "channel-1:video-2:long:unack:default",
      lastProbeKey: null,
    });

    expect(result).toBe(true);
  });

  it("returns false after the same missing-scope probe already ran", () => {
    const result = shouldForceReloadMissingSelectedVideo({
      selectedVideoId: "video-2",
      videos: [
        makeVideo({ id: "video-1", is_short: false, acknowledged: false }),
      ],
      probeKey: "channel-1:video-2:long:unack:default",
      lastProbeKey: "channel-1:video-2:long:unack:default",
    });

    expect(result).toBe(false);
  });

  it("returns false when the selected video is already present", () => {
    const result = shouldForceReloadMissingSelectedVideo({
      selectedVideoId: "video-1",
      videos: [
        makeVideo({ id: "video-1", is_short: false, acknowledged: false }),
      ],
      probeKey: "channel-1:video-1:long:unack:default",
      lastProbeKey: null,
    });

    expect(result).toBe(false);
  });
});

describe("shouldLoadAllChannelVideosForSelection", () => {
  it("does not escalate preview loading when the selected video is already present", () => {
    const result = shouldLoadAllChannelVideosForSelection({
      selectedVideoId: "video-1",
      loadedMode: "preview",
      hasMore: true,
      videos: [
        makeVideo({ id: "video-1", is_short: false, acknowledged: false }),
      ],
    });

    expect(result).toBe(false);
  });

  it("escalates to a full load when the selected video is outside the preview rows", () => {
    const result = shouldLoadAllChannelVideosForSelection({
      selectedVideoId: "video-2",
      loadedMode: "preview",
      hasMore: true,
      videos: [
        makeVideo({ id: "video-1", is_short: false, acknowledged: false }),
      ],
    });

    expect(result).toBe(true);
  });

  it("keeps paging in paged mode while more rows are available", () => {
    const result = shouldLoadAllChannelVideosForSelection({
      selectedVideoId: "video-2",
      loadedMode: "paged",
      hasMore: true,
      videos: [
        makeVideo({ id: "video-1", is_short: false, acknowledged: false }),
      ],
    });

    expect(result).toBe(true);
  });

  it("does not escalate after paged mode is exhausted", () => {
    const result = shouldLoadAllChannelVideosForSelection({
      selectedVideoId: "video-2",
      loadedMode: "paged",
      hasMore: false,
      videos: [
        makeVideo({ id: "video-1", is_short: false, acknowledged: false }),
      ],
    });

    expect(result).toBe(false);
  });
});

describe("resolveVirtualWindow", () => {
  it("returns a bounded window with overscan", () => {
    const result = resolveVirtualWindow({
      itemCount: 100,
      itemHeight: 56,
      viewportHeight: 280,
      scrollTop: 560,
      overscan: 8,
    });

    expect(result).toEqual({
      startIndex: 2,
      endIndex: 23,
      offsetTop: 112,
      totalHeight: 5600,
    });
  });
});

describe("resolveInitialPreviewExpandedChannelId", () => {
  it("prefers the route-selected channel over the first channel", () => {
    const result = resolveInitialPreviewExpandedChannelId(
      [makeChannel("channel-a"), makeChannel("channel-b")],
      "channel-b",
      "others",
    );

    expect(result).toBe("channel-b");
  });

  it("falls back to the first non-virtual channel", () => {
    const result = resolveInitialPreviewExpandedChannelId(
      [makeChannel("others"), makeChannel("channel-b")],
      null,
      "others",
    );

    expect(result).toBe("channel-b");
  });

  it("ignores a virtual selected channel", () => {
    const result = resolveInitialPreviewExpandedChannelId(
      [makeChannel("others"), makeChannel("channel-b")],
      "others",
      "others",
    );

    expect(result).toBe("channel-b");
  });
});
