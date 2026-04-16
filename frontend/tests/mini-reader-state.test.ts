import { describe, expect, it } from "bun:test";

import type { MiniSummaryItem } from "../src/lib/transport-types";
import {
  chooseActiveVideoId,
  findNextUnreadVideoId,
} from "../src/lib/mini/mini-reader-state.svelte";

function makeSummary(
  videoId: string,
  read: boolean,
  overrides: Partial<MiniSummaryItem> = {},
): MiniSummaryItem {
  return {
    video_id: videoId,
    channel_id: "channel-1",
    channel_name: "Channel",
    title: `Video ${videoId}`,
    thumbnail_url: null,
    published_at: "2026-04-16T00:00:00.000Z",
    watch_url: `https://example.com/${videoId}`,
    summary_content: "Summary",
    read,
    ...overrides,
  };
}

describe("chooseActiveVideoId", () => {
  it("keeps a preferred visible summary before choosing the first unread", () => {
    const summaries = [
      makeSummary("a", false),
      makeSummary("b", true),
      makeSummary("c", false),
    ];

    expect(chooseActiveVideoId(summaries, "b")).toBe("b");
    expect(chooseActiveVideoId(summaries, "missing")).toBe("a");
  });
});

describe("findNextUnreadVideoId", () => {
  it("chooses the next unread summary after the current one", () => {
    const summaries = [
      makeSummary("a", false),
      makeSummary("b", true),
      makeSummary("c", false),
      makeSummary("d", false),
    ];

    expect(findNextUnreadVideoId(summaries, "b")).toBe("c");
    expect(findNextUnreadVideoId(summaries, "c")).toBe("d");
  });

  it("wraps to the first unread summary when the current one is last", () => {
    const summaries = [
      makeSummary("a", false),
      makeSummary("b", true),
      makeSummary("c", true),
    ];

    expect(findNextUnreadVideoId(summaries, "c")).toBe("a");
  });

  it("returns null when every summary is read", () => {
    const summaries = [makeSummary("a", true), makeSummary("b", true)];

    expect(findNextUnreadVideoId(summaries, "a")).toBeNull();
  });
});
