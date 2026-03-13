import { describe, expect, it } from "bun:test";
import {
  channelOrderFromList,
  cycleChannelSortMode,
  filterChannels,
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
