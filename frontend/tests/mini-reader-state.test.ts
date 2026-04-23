import { describe, expect, it } from "bun:test";

import type { MiniSummaryItem } from "../src/lib/transport-types";
import {
  chooseActiveVideoId,
  findNextMiniChannelId,
  findNextUnreadVideoId,
  MINI_DEFAULT_SHOW_UNREAD_ONLY,
  miniChannelIsCaughtUp,
  selectMiniSummaryHighlights,
} from "../src/lib/mini/mini-reader-state.svelte";
import type { Highlight } from "../src/lib/types";

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

describe("MINI_DEFAULT_SHOW_UNREAD_ONLY", () => {
  it("shows only unread summaries by default", () => {
    expect(MINI_DEFAULT_SHOW_UNREAD_ONLY).toBe(true);
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

describe("miniChannelIsCaughtUp", () => {
  it("returns true only when a channel has summaries and all are read", () => {
    expect(
      miniChannelIsCaughtUp([makeSummary("a", true), makeSummary("b", true)]),
    ).toBe(true);
    expect(
      miniChannelIsCaughtUp([makeSummary("a", true), makeSummary("b", false)]),
    ).toBe(false);
    expect(miniChannelIsCaughtUp([])).toBe(false);
  });
});

describe("findNextMiniChannelId", () => {
  const channels = [{ id: "a" }, { id: "b" }, { id: "c" }];

  it("chooses the next channel after the selected channel", () => {
    expect(findNextMiniChannelId(channels, "a")).toBe("b");
    expect(findNextMiniChannelId(channels, "b")).toBe("c");
  });

  it("wraps to the first channel after the last channel", () => {
    expect(findNextMiniChannelId(channels, "c")).toBe("a");
  });

  it("returns null when no other channel exists", () => {
    expect(findNextMiniChannelId([{ id: "a" }], "a")).toBeNull();
  });
});

describe("selectMiniSummaryHighlights", () => {
  it("returns only summary highlights for the active mini summary", () => {
    const highlightsByVideoId: Record<string, Highlight[]> = {
      "video-1": [
        {
          id: 1,
          video_id: "video-1",
          source: "transcript",
          text: "Transcript note",
          prefix_context: "",
          suffix_context: "",
          created_at: "2026-04-16T10:00:00.000Z",
        },
        {
          id: 2,
          video_id: "video-1",
          source: "summary",
          text: "Summary note",
          prefix_context: "",
          suffix_context: "",
          created_at: "2026-04-16T10:01:00.000Z",
        },
      ],
      "video-2": [
        {
          id: 3,
          video_id: "video-2",
          source: "summary",
          text: "Other summary note",
          prefix_context: "",
          suffix_context: "",
          created_at: "2026-04-16T10:02:00.000Z",
        },
      ],
    };

    expect(selectMiniSummaryHighlights("video-1", highlightsByVideoId)).toEqual(
      [highlightsByVideoId["video-1"][1]],
    );
  });
});
