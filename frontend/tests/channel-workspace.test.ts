import { describe, expect, it } from "bun:test";
import {
  beginChannelDrag,
  buildQueueSnapshotOptions,
  completeChannelDrop,
  finalizeAddedChannelOrder,
  finishChannelDrag,
  loadWorkspaceState,
  markChannelRefreshed,
  moveChannelToIndex,
  prioritizeChannelOrder,
  reorderChannels,
  restoreWorkspaceSnapshot,
  resolveInitialChannelSelection,
  saveWorkspaceState,
  shouldRefreshChannel,
  updateChannelDragOver,
} from "../src/lib/channel-workspace";
import type { Channel } from "../src/lib/types";

function channel(id: string): Channel {
  return {
    id,
    name: id,
    added_at: "2026-02-28T00:00:00.000Z",
  };
}

function createStorage(seed?: Record<string, string>) {
  const values = new Map(Object.entries(seed ?? {}));
  return {
    getItem(key: string) {
      return values.get(key) ?? null;
    },
    setItem(key: string, value: string) {
      values.set(key, value);
    },
    removeItem(key: string) {
      values.delete(key);
    },
  };
}

describe("prioritizeChannelOrder", () => {
  it("moves the given channel id to the front without duplicates", () => {
    expect(prioritizeChannelOrder(["a", "b", "c"], "b")).toEqual([
      "b",
      "a",
      "c",
    ]);
    expect(prioritizeChannelOrder([], "x")).toEqual(["x"]);
  });
});

describe("finalizeAddedChannelOrder", () => {
  it("keeps a newly confirmed channel at the front when no optimistic id was stored", () => {
    expect(finalizeAddedChannelOrder(["a", "b"], "new")).toEqual([
      "new",
      "a",
      "b",
    ]);
  });

  it("replaces an optimistic id and keeps the confirmed channel at the front", () => {
    expect(
      finalizeAddedChannelOrder(["temp-1", "a", "b"], "real-1", "temp-1"),
    ).toEqual(["real-1", "a", "b"]);
  });
});

describe("resolveInitialChannelSelection", () => {
  it("prefers explicitly requested channel when it exists", () => {
    const channels = [channel("a"), channel("b")];
    expect(resolveInitialChannelSelection(channels, "a", "b")).toBe("b");
  });

  it("falls back to saved selection when preferred one is not present", () => {
    const channels = [channel("a"), channel("b")];
    expect(resolveInitialChannelSelection(channels, "b", "z")).toBe("b");
  });

  it("falls back to first channel when no saved selection exists", () => {
    const channels = [channel("a"), channel("b")];
    expect(resolveInitialChannelSelection(channels, null, null)).toBe("a");
  });

  it("returns null when there are no channels", () => {
    expect(resolveInitialChannelSelection([], "a", "b")).toBeNull();
  });
});

describe("reorderChannels", () => {
  it("returns reordered channels and channel ids together", () => {
    const result = reorderChannels(
      [channel("a"), channel("b"), channel("c")],
      "c",
      "a",
    );

    expect(result).toEqual({
      channels: [channel("c"), channel("a"), channel("b")],
      channelOrder: ["c", "a", "b"],
    });
  });
});

describe("moveChannelToIndex", () => {
  it("moves a dragged channel into the last visible position", () => {
    const result = moveChannelToIndex(
      [channel("a"), channel("b"), channel("c")],
      "a",
      2,
    );

    expect(result).toEqual({
      channels: [channel("b"), channel("c"), channel("a")],
      channelOrder: ["b", "c", "a"],
    });
  });
});

describe("workspace state storage helpers", () => {
  it("loads persisted state and ignores malformed data", () => {
    const storage = createStorage({
      "dastill.workspace.state.v1": JSON.stringify({
        selectedChannelId: "abc",
        selectedVideoId: "vid-1",
      }),
    });
    const malformed = createStorage({
      "dastill.workspace.state.v1": "{nope",
    });

    expect(loadWorkspaceState(storage)).toEqual({
      selectedChannelId: "abc",
      selectedVideoId: "vid-1",
    });
    expect(loadWorkspaceState(malformed)).toBeNull();
    expect(malformed.getItem("dastill.workspace.state.v1")).toBeNull();
  });

  it("persists merged workspace state without dropping unrelated fields", () => {
    const storage = createStorage({
      "dastill.workspace.state.v1": JSON.stringify({
        selectedVideoId: "vid-1",
        contentMode: "summary",
      }),
    });

    saveWorkspaceState(storage, {
      selectedChannelId: "abc",
      channelOrder: ["abc"],
    });

    expect(loadWorkspaceState(storage)).toEqual({
      selectedVideoId: "vid-1",
      contentMode: "summary",
      selectedChannelId: "abc",
      channelOrder: ["abc"],
    });
  });
});

describe("restoreWorkspaceSnapshot", () => {
  it("sanitizes the persisted snapshot for the main workspace", () => {
    expect(
      restoreWorkspaceSnapshot(
        {
          selectedChannelId: "abc",
          selectedVideoId: "vid-1",
          contentMode: "summary",
          hideShorts: true,
          acknowledgedFilter: "ack",
          channelOrder: ["abc", 12, "def"] as unknown as string[],
          channelSortMode: "alpha",
        },
        {
          includeSelectedVideoId: true,
          includeContentMode: true,
          includeVideoTypeFilter: true,
          includeAcknowledgedFilter: true,
          includeChannelSortMode: true,
        },
      ),
    ).toEqual({
      selectedChannelId: "abc",
      selectedVideoId: "vid-1",
      contentMode: "summary",
      videoTypeFilter: "long",
      acknowledgedFilter: "ack",
      channelOrder: ["abc", "def"],
      channelSortMode: "alpha",
    });
  });

  it("can restore only the shared channel selection fields", () => {
    expect(
      restoreWorkspaceSnapshot({
        selectedChannelId: "queue-channel",
        selectedVideoId: "ignored",
        contentMode: "transcript",
        channelOrder: ["queue-channel"],
      }),
    ).toEqual({
      selectedChannelId: "queue-channel",
      channelOrder: ["queue-channel"],
    });
  });
});

describe("channel drag helpers", () => {
  it("tracks drag state and transfers the dragged channel id", () => {
    const writes: Array<[string, string]> = [];
    const transfer = {
      effectAllowed: "copy",
      setData(type: string, value: string) {
        writes.push([type, value]);
      },
    };

    expect(beginChannelDrag("abc", transfer)).toEqual({
      draggedChannelId: "abc",
      dragOverChannelId: "abc",
    });
    expect(updateChannelDragOver("abc", "def")).toBe("def");
    expect(completeChannelDrop("def", "abc", "fallback").sourceId).toBe("abc");
    expect(finishChannelDrag()).toEqual({
      draggedChannelId: null,
      dragOverChannelId: null,
    });
    expect(writes).toEqual([["text/plain", "abc"]]);
    expect(transfer.effectAllowed).toBe("move");
  });
});

describe("channel refresh helpers", () => {
  it("skips refreshes inside the TTL window and marks refresh times", () => {
    const refreshedAt = new Map<string, number>();
    markChannelRefreshed(refreshedAt, "abc", 1_000);

    expect(shouldRefreshChannel(refreshedAt, "abc", 500, 1_400)).toBe(false);
    expect(shouldRefreshChannel(refreshedAt, "abc", 500, 1_501)).toBe(true);
    expect(shouldRefreshChannel(refreshedAt, "missing", 500, 1_400)).toBe(true);
  });
});

describe("buildQueueSnapshotOptions", () => {
  it("returns limit and offset for queue snapshot requests", () => {
    expect(buildQueueSnapshotOptions(20, 5)).toEqual({
      limit: 20,
      offset: 5,
    });
  });
});
