import { describe, expect, it } from "bun:test";

import type { MiniReaderPayload } from "../src/lib/api";
import { printEdition } from "../src/lib/edition/printing";
import { channel, summaryItem } from "./fixtures";

const channels = [channel("a"), channel("b"), channel("c")];

function page(channelId: string, published: string): MiniReaderPayload {
  return {
    channels,
    selected_channel_id: channelId,
    summaries: [
      summaryItem(`${channelId}-1`, {
        channel_id: channelId,
        published_at: published,
      }),
    ],
  };
}

describe("printEdition", () => {
  it("asks for every channel once and merges them by release date", async () => {
    const requested: (string | undefined)[] = [];
    const edition = await printEdition(async (channelId) => {
      requested.push(channelId);
      if (!channelId) return page("a", "2026-10-01T00:00:00Z");
      return page(
        channelId,
        channelId === "b" ? "2026-10-04T00:00:00Z" : "2026-10-02T00:00:00Z",
      );
    });

    expect(requested.sort()).toEqual([undefined, "b", "c"].sort());
    expect(edition.channels).toEqual(channels);
    expect(edition.stories.map((s) => s.id)).toEqual(["b-1", "c-1", "a-1"]);
    expect(edition.missingChannelIds).toEqual([]);
  });

  it("keeps the paper when one channel fails and reports it", async () => {
    const edition = await printEdition(async (channelId) => {
      if (channelId === "c") throw new Error("boom");
      return page(channelId ?? "a", "2026-10-01T00:00:00Z");
    });
    expect(edition.stories.map((s) => s.id).sort()).toEqual(["a-1", "b-1"]);
    expect(edition.missingChannelIds).toEqual(["c"]);
  });

  it("fails when the first request fails", async () => {
    await expect(
      printEdition(async () => {
        throw new Error("offline");
      }),
    ).rejects.toThrow("offline");
  });

  it("handles a reader with no channels", async () => {
    const edition = await printEdition(async () => ({
      channels: [],
      selected_channel_id: null,
      summaries: [],
    }));
    expect(edition).toEqual({
      channels: [],
      stories: [],
      missingChannelIds: [],
    });
  });
});

describe("printEdition progress", () => {
  it("reports what has arrived after each channel", async () => {
    const seen: string[][] = [];
    await printEdition(
      async (channelId) => page(channelId ?? "a", "2026-10-01T00:00:00Z"),
      (partial) => seen.push(partial.stories.map((s) => s.id).sort()),
    );
    expect(seen[0]).toEqual(["a-1"]);
    expect(seen.at(-1)).toEqual(["a-1", "b-1", "c-1"]);
    expect(seen).toHaveLength(3);
  });
});
