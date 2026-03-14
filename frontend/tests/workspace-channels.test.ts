import { describe, expect, it } from "bun:test";
import {
  canManualReorderChannels,
  channelOrderFromList,
  cycleChannelSortMode,
  filterChannels,
  moveChannelByStep,
  resolveChannelDropIndicatorEdge,
} from "../src/lib/workspace/channels";
import type { Channel } from "../src/lib/types";

function createChannel(id: string, overrides: Partial<Channel> = {}): Channel {
  return {
    id,
    name: `Channel ${id}`,
    handle: `@${id}`,
    added_at: "2026-03-01T00:00:00.000Z",
    earliest_sync_date: null,
    earliest_sync_date_user_set: false,
    ...overrides,
  };
}

describe("filterChannels", () => {
  const channels = [
    createChannel("gamma", {
      name: "Gamma",
      handle: "@gamma",
      added_at: "2026-03-01T00:00:00.000Z",
    }),
    createChannel("alpha", {
      name: "Alpha",
      handle: "@letters",
      added_at: "2026-03-03T00:00:00.000Z",
    }),
    createChannel("beta", {
      name: "Beta",
      handle: "@beta",
      added_at: "2026-03-02T00:00:00.000Z",
    }),
  ];

  it("keeps custom order while filtering by name or handle", () => {
    expect(
      filterChannels(channels, "ga", "custom").map((channel) => channel.id),
    ).toEqual(["gamma"]);
    expect(
      filterChannels(channels, "letters", "custom").map(
        (channel) => channel.id,
      ),
    ).toEqual(["alpha"]);
  });

  it("sorts alphabetically when requested", () => {
    expect(
      filterChannels(channels, "", "alpha").map((channel) => channel.id),
    ).toEqual(["alpha", "beta", "gamma"]);
  });

  it("sorts by newest added timestamp when requested", () => {
    expect(
      filterChannels(channels, "", "newest").map((channel) => channel.id),
    ).toEqual(["alpha", "beta", "gamma"]);
  });
});

describe("cycleChannelSortMode", () => {
  it("cycles through the supported sort modes", () => {
    expect(cycleChannelSortMode("custom")).toBe("alpha");
    expect(cycleChannelSortMode("alpha")).toBe("newest");
    expect(cycleChannelSortMode("newest")).toBe("custom");
  });
});

describe("channelOrderFromList", () => {
  it("returns channel ids in their current order", () => {
    expect(
      channelOrderFromList([createChannel("a"), createChannel("b")]),
    ).toEqual(["a", "b"]);
  });
});

describe("canManualReorderChannels", () => {
  it("only allows manual reorder in custom mode without an active filter", () => {
    expect(canManualReorderChannels("custom", "")).toBe(true);
    expect(canManualReorderChannels("custom", "  ")).toBe(true);
    expect(canManualReorderChannels("alpha", "")).toBe(false);
    expect(canManualReorderChannels("newest", "")).toBe(false);
    expect(canManualReorderChannels("custom", "alpha")).toBe(false);
  });
});

describe("moveChannelByStep", () => {
  it("moves a channel one slot up or down for non-drag reorder controls", () => {
    const channels = [
      createChannel("a"),
      createChannel("b"),
      createChannel("c"),
    ];

    expect(moveChannelByStep(channels, "b", "up")).toEqual({
      channels: [createChannel("b"), createChannel("a"), createChannel("c")],
      channelOrder: ["b", "a", "c"],
    });
    expect(moveChannelByStep(channels, "b", "down")).toEqual({
      channels: [createChannel("a"), createChannel("c"), createChannel("b")],
      channelOrder: ["a", "c", "b"],
    });
  });

  it("returns null when the channel cannot move further", () => {
    const channels = [
      createChannel("a"),
      createChannel("b"),
      createChannel("c"),
    ];

    expect(moveChannelByStep(channels, "a", "up")).toBeNull();
    expect(moveChannelByStep(channels, "c", "down")).toBeNull();
    expect(moveChannelByStep(channels, "missing", "down")).toBeNull();
  });
});

describe("resolveChannelDropIndicatorEdge", () => {
  it("places the insertion marker after targets below the dragged channel", () => {
    expect(resolveChannelDropIndicatorEdge(["a", "b", "c"], "a", "c")).toBe(
      "bottom",
    );
  });

  it("places the insertion marker before targets above the dragged channel", () => {
    expect(resolveChannelDropIndicatorEdge(["a", "b", "c"], "c", "a")).toBe(
      "top",
    );
  });

  it("returns null when there is no meaningful drop target", () => {
    expect(resolveChannelDropIndicatorEdge(["a", "b"], "a", "a")).toBeNull();
    expect(resolveChannelDropIndicatorEdge(["a", "b"], null, "b")).toBeNull();
    expect(resolveChannelDropIndicatorEdge(["a", "b"], "a", null)).toBeNull();
  });
});
