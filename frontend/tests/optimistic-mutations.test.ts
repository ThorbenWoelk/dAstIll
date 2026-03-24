import { describe, expect, it } from "bun:test";
import type { Channel, Video } from "../src/lib/types";
import type { CreateHighlightRequest, Highlight } from "../src/lib/types";
import {
  applyOptimisticAcknowledge,
  buildOptimisticChannel,
  removeChannelFromCollection,
  removeChannelId,
  replaceOptimisticChannel,
  replaceOptimisticChannelId,
} from "../src/lib/workspace/channel-actions";
import {
  buildOptimisticHighlight,
  mergeHighlightIntoList,
  reconcileOptimisticHighlight,
} from "../src/lib/utils/highlights";

// ---- Test helpers ----

function makeVideo(
  overrides: Partial<Video> & { id: string; acknowledged: boolean },
): Video {
  return {
    channel_id: "ch-1",
    title: "Test Video",
    thumbnail_url: null,
    published_at: "2026-01-01T00:00:00Z",
    is_short: false,
    transcript_status: "ready",
    summary_status: "ready",
    retry_count: 0,
    ...overrides,
  } as Video;
}

function makeChannel(id: string, overrides: Partial<Channel> = {}): Channel {
  return {
    id,
    name: `Channel ${id}`,
    handle: `@channel-${id}`,
    thumbnail_url: null,
    added_at: "2026-01-01T00:00:00Z",
    earliest_sync_date: null,
    earliest_sync_date_user_set: false,
    ...overrides,
  };
}

function makeHighlightPayload(
  text: string,
  overrides: Partial<CreateHighlightRequest> = {},
): CreateHighlightRequest {
  return {
    source: "transcript",
    text,
    prefix_context: "",
    suffix_context: "",
    ...overrides,
  };
}

function makeHighlight(
  id: number,
  videoId: string,
  overrides: Partial<Highlight> = {},
): Highlight {
  return {
    id,
    video_id: videoId,
    source: "transcript",
    text: `Highlight ${id}`,
    prefix_context: "",
    suffix_context: "",
    created_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

// ---- Optimistic acknowledge toggle ----

describe("optimistic acknowledge toggle", () => {
  it("flips acknowledged state to true before API promise resolves", async () => {
    const video = makeVideo({ id: "v-1", acknowledged: false });
    const initialVideos = [video];

    let resolveApi!: (v: Video) => void;
    let _rejectApi!: (err: Error) => void;
    const deferred = new Promise<Video>((res, rej) => {
      resolveApi = res;
      _rejectApi = rej;
    });

    // Simulate optimistic update before calling API
    const previousVideos = [...initialVideos];
    let currentVideos = applyOptimisticAcknowledge(initialVideos, "v-1", true);

    // State is updated immediately — before the API promise resolves
    expect(currentVideos[0].acknowledged).toBe(true);

    // API succeeds → no rollback needed
    const updatedVideo = { ...video, acknowledged: true };
    resolveApi(updatedVideo);
    const result = await deferred;
    currentVideos = currentVideos.map((v) => (v.id === result.id ? result : v));
    expect(currentVideos[0].acknowledged).toBe(true);

    // Verify previous snapshot was unaffected (rollback source)
    expect(previousVideos[0].acknowledged).toBe(false);
  });

  it("reverts acknowledged state when API rejects", async () => {
    const video = makeVideo({ id: "v-2", acknowledged: false });
    const initialVideos = [video];

    let rejectApi!: (err: Error) => void;
    const deferred = new Promise<Video>((_, rej) => {
      rejectApi = rej;
    });

    const previousVideos = [...initialVideos];
    let currentVideos = applyOptimisticAcknowledge(initialVideos, "v-2", true);

    // State changed optimistically
    expect(currentVideos[0].acknowledged).toBe(true);

    // API fails → rollback
    const errors: string[] = [];
    rejectApi(new Error("Network error"));
    try {
      await deferred;
    } catch (error) {
      currentVideos = previousVideos;
      errors.push((error as Error).message);
    }

    // State reverted
    expect(currentVideos[0].acknowledged).toBe(false);
    expect(errors).toContain("Network error");
  });

  it("flips acknowledged from true to false optimistically", () => {
    const acknowledgedVideo = makeVideo({ id: "v-3", acknowledged: true });
    const result = applyOptimisticAcknowledge(
      [acknowledgedVideo],
      "v-3",
      false,
    );
    expect(result[0].acknowledged).toBe(false);
  });

  it("leaves other videos unchanged when one is toggled", () => {
    const videos = [
      makeVideo({ id: "v-a", acknowledged: false }),
      makeVideo({ id: "v-b", acknowledged: true }),
      makeVideo({ id: "v-c", acknowledged: false }),
    ];
    const result = applyOptimisticAcknowledge(videos, "v-b", false);
    expect(result[0].acknowledged).toBe(false);
    expect(result[1].acknowledged).toBe(false);
    expect(result[2].acknowledged).toBe(false);
    expect(result[0].id).toBe("v-a");
    expect(result[1].id).toBe("v-b");
    expect(result[2].id).toBe("v-c");
  });

  it("returns list unchanged when video id is not found", () => {
    const videos = [makeVideo({ id: "v-x", acknowledged: false })];
    const result = applyOptimisticAcknowledge(videos, "does-not-exist", true);
    expect(result[0].acknowledged).toBe(false);
  });
});

// ---- Optimistic delete channel ----

describe("optimistic delete channel", () => {
  it("removes channel from list before API promise resolves", async () => {
    const channels = [
      makeChannel("ch-a"),
      makeChannel("ch-b"),
      makeChannel("ch-c"),
    ];
    const channelOrder = ["ch-a", "ch-b", "ch-c"];

    let resolveApi!: () => void;
    let _rejectApi!: (err: Error) => void;
    const deferred = new Promise<void>((res, rej) => {
      resolveApi = res;
      _rejectApi = rej;
    });

    // Snapshot for rollback
    const previousChannels = [...channels];
    const previousChannelOrder = [...channelOrder];

    // Optimistic removal
    const currentChannels = removeChannelFromCollection(channels, "ch-b");
    const currentOrder = removeChannelId(channelOrder, "ch-b");

    // Channel is removed immediately
    expect(currentChannels).toHaveLength(2);
    expect(currentChannels.map((c) => c.id)).toEqual(["ch-a", "ch-c"]);
    expect(currentOrder).toEqual(["ch-a", "ch-c"]);

    // API succeeds
    resolveApi();
    await deferred;

    // Verify previous snapshot preserved for rollback
    expect(previousChannels).toHaveLength(3);
    expect(previousChannelOrder).toHaveLength(3);
  });

  it("reverts channel removal when API rejects", async () => {
    const channels = [makeChannel("ch-1"), makeChannel("ch-2")];
    const channelOrder = ["ch-1", "ch-2"];

    let rejectApi!: (err: Error) => void;
    const deferred = new Promise<void>((_, rej) => {
      rejectApi = rej;
    });

    const previousChannels = [...channels];
    const previousChannelOrder = [...channelOrder];

    let currentChannels = removeChannelFromCollection(channels, "ch-1");
    let currentOrder = removeChannelId(channelOrder, "ch-1");

    // Optimistically removed
    expect(currentChannels).toHaveLength(1);
    expect(currentChannels[0].id).toBe("ch-2");

    // API fails → rollback
    const errors: string[] = [];
    rejectApi(new Error("Delete failed"));
    try {
      await deferred;
    } catch (error) {
      currentChannels = previousChannels;
      currentOrder = previousChannelOrder;
      errors.push((error as Error).message);
    }

    // State reverted
    expect(currentChannels).toHaveLength(2);
    expect(currentOrder).toHaveLength(2);
    expect(errors).toContain("Delete failed");
  });

  it("removes the correct channel id from order", () => {
    const order = ["ch-a", "ch-b", "ch-c"];
    expect(removeChannelId(order, "ch-b")).toEqual(["ch-a", "ch-c"]);
  });

  it("removes the correct channel from collection", () => {
    const channels = [
      makeChannel("ch-a"),
      makeChannel("ch-b"),
      makeChannel("ch-c"),
    ];
    const result = removeChannelFromCollection(channels, "ch-b");
    expect(result.map((c) => c.id)).toEqual(["ch-a", "ch-c"]);
  });
});

// ---- Optimistic add channel (verify existing pattern) ----

describe("optimistic add channel", () => {
  it("inserts optimistic entry before API promise resolves", async () => {
    const channels = [makeChannel("existing-1")];

    let resolveApi!: (ch: Channel) => void;
    let _rejectApi!: (err: Error) => void;
    const deferred = new Promise<Channel>((res, rej) => {
      resolveApi = res;
      _rejectApi = rej;
    });

    const { optimisticChannel, tempId } = buildOptimisticChannel(
      "https://youtube.com/@newchannel",
    );
    let currentChannels = [optimisticChannel, ...channels];
    let currentOrder = [tempId, ...channels.map((c) => c.id)];

    // Optimistic entry is visible immediately
    expect(currentChannels).toHaveLength(2);
    expect(currentChannels[0].id).toBe(tempId);
    expect(currentChannels[0].name).toBe("Fetching Channel...");

    // API succeeds → replace optimistic with real channel
    const realChannel = makeChannel("real-channel-id", {
      name: "New Channel",
      handle: "@newchannel",
    });
    resolveApi(realChannel);
    const confirmed = await deferred;

    currentChannels = replaceOptimisticChannel(
      currentChannels,
      tempId,
      confirmed,
    );
    currentOrder = replaceOptimisticChannelId(
      currentOrder,
      tempId,
      confirmed.id,
    );

    expect(currentChannels).toHaveLength(2);
    expect(currentChannels[0].id).toBe("real-channel-id");
    expect(currentChannels[0].name).toBe("New Channel");
    expect(currentOrder[0]).toBe("real-channel-id");
  });

  it("reverts optimistic channel insertion when API rejects", async () => {
    const channels = [makeChannel("existing-1")];

    let rejectApi!: (err: Error) => void;
    const deferred = new Promise<Channel>((_, rej) => {
      rejectApi = rej;
    });

    const { optimisticChannel, tempId } = buildOptimisticChannel("@badchannel");

    const previousChannels = [...channels];
    let currentChannels = [optimisticChannel, ...channels];
    let currentOrder = [tempId, ...channels.map((c) => c.id)];

    // Optimistic entry exists before API resolves
    expect(currentChannels).toHaveLength(2);

    // API fails → rollback
    const errors: string[] = [];
    rejectApi(new Error("Channel not found"));
    try {
      await deferred;
    } catch (error) {
      currentChannels = previousChannels;
      currentOrder = channels.map((c) => c.id);
      errors.push((error as Error).message);
    }

    expect(currentChannels).toHaveLength(1);
    expect(currentChannels[0].id).toBe("existing-1");
    expect(currentOrder).toEqual(channels.map((c) => c.id));
    expect(errors).toContain("Channel not found");
  });
});

// ---- Optimistic highlight create (verify existing pattern) ----

describe("optimistic highlight create", () => {
  it("adds highlight to list before API promise resolves", async () => {
    const existingHighlight = makeHighlight(1, "vid-1", {
      created_at: "2026-01-01T10:00:00Z",
    });
    let currentHighlights: Highlight[] = [existingHighlight];

    let resolveApi!: (h: Highlight) => void;
    let _rejectApi!: (err: Error) => void;
    const deferred = new Promise<Highlight>((res, rej) => {
      resolveApi = res;
      _rejectApi = rej;
    });

    const payload = makeHighlightPayload("This is an important point");
    const optimisticId = -1;
    const optimisticHighlight = buildOptimisticHighlight(
      "vid-1",
      payload,
      optimisticId,
      "2026-01-02T10:00:00Z",
    );

    // Merge optimistic highlight into list immediately
    currentHighlights = mergeHighlightIntoList(
      currentHighlights,
      optimisticHighlight,
    );

    // Optimistic highlight is in the list before API resolves
    expect(currentHighlights).toHaveLength(2);
    expect(currentHighlights[0].id).toBe(optimisticId);
    expect(currentHighlights[0].text).toBe("This is an important point");

    // API succeeds → reconcile optimistic with confirmed
    const confirmedHighlight = makeHighlight(42, "vid-1", {
      text: "This is an important point",
      created_at: "2026-01-02T10:00:00Z",
    });
    resolveApi(confirmedHighlight);
    const confirmed = await deferred;

    currentHighlights = reconcileOptimisticHighlight(
      currentHighlights,
      optimisticId,
      confirmed,
    );

    expect(currentHighlights).toHaveLength(2);
    expect(currentHighlights[0].id).toBe(42);
    expect(currentHighlights.some((h) => h.id === optimisticId)).toBe(false);
  });

  it("reverts optimistic highlight on API rejection", async () => {
    const existingHighlight = makeHighlight(1, "vid-1");
    let currentHighlights: Highlight[] = [existingHighlight];

    let rejectApi!: (err: Error) => void;
    const deferred = new Promise<Highlight>((_, rej) => {
      rejectApi = rej;
    });

    const payload = makeHighlightPayload("Highlight that will fail");
    const optimisticId = -1;
    const optimisticHighlight = buildOptimisticHighlight(
      "vid-1",
      payload,
      optimisticId,
    );

    const previousHighlights = [...currentHighlights];
    currentHighlights = mergeHighlightIntoList(
      currentHighlights,
      optimisticHighlight,
    );

    // Optimistic highlight exists before API responds
    expect(currentHighlights).toHaveLength(2);
    expect(currentHighlights[0].id).toBe(optimisticId);

    // API fails → rollback (remove optimistic highlight)
    const errors: string[] = [];
    rejectApi(new Error("Failed to save highlight"));
    try {
      await deferred;
    } catch (error) {
      currentHighlights = previousHighlights;
      errors.push((error as Error).message);
    }

    expect(currentHighlights).toHaveLength(1);
    expect(currentHighlights[0].id).toBe(1);
    expect(errors).toContain("Failed to save highlight");
  });
});
