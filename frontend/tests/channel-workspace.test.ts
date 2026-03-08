import { describe, expect, it } from "bun:test";
import {
  prioritizeChannelOrder,
  resolveInitialChannelSelection,
} from "../src/lib/channel-workspace";
import type { Channel } from "../src/lib/types";

function channel(id: string): Channel {
  return {
    id,
    name: id,
    added_at: "2026-02-28T00:00:00.000Z",
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
