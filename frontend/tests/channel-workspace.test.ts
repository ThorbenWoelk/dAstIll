import { describe, expect, it } from "bun:test";
import {
  buildQueueSnapshotOptions,
  loadWorkspaceState,
  prioritizeChannelOrder,
  reorderChannels,
  resolveInitialChannelSelection,
  saveWorkspaceState,
} from "../src/lib/channel-workspace";
import type { QueueTab } from "../src/lib/types";
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

describe("buildQueueSnapshotOptions", () => {
  it.each(["transcripts", "summaries", "evaluations"] satisfies QueueTab[])(
    "preserves the active %s tab in queue snapshot requests",
    (queueTab) => {
      expect(buildQueueSnapshotOptions(queueTab, 20, 5)).toEqual({
        limit: 20,
        offset: 5,
        queueTab,
      });
    },
  );
});
