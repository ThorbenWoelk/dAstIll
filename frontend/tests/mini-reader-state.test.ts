import { afterEach, describe, expect, it } from "bun:test";

import type { MiniSummaryItem } from "../src/lib/transport-types";
import {
  chooseActiveVideoId,
  findNextMiniChannelId,
  findNextUnreadVideoId,
  MiniReaderState,
  MINI_DEFAULT_SHOW_UNREAD_ONLY,
  miniChannelIsCaughtUp,
  saveMiniVocabularyPreferences,
  selectMiniSummaryHighlights,
} from "../src/lib/mini/mini-reader-state.svelte";
import { resetApiCacheForTests } from "../src/lib/api";
import type { Highlight, UserPreferences } from "../src/lib/types";
import type { Channel } from "../src/lib/transport-types";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  resetApiCacheForTests();
});

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

function makeChannel(id = "channel-1"): Channel {
  return {
    id,
    handle: null,
    name: "Channel",
    thumbnail_url: null,
    added_at: "2026-04-16T00:00:00.000Z",
    earliest_sync_date: null,
    earliest_sync_date_user_set: false,
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

describe("MiniReaderState", () => {
  it("switches to the empty state after marking the last unread summary read", async () => {
    const mini = new MiniReaderState();
    mini.reader = {
      channels: [makeChannel()],
      selected_channel_id: "channel-1",
      summaries: [makeSummary("a", false)],
    };
    mini.selectedChannelId = "channel-1";
    mini.activeVideoId = "a";
    mini.status = "ready";

    globalThis.fetch = (async (input, init) => {
      const url = String(input);
      const method = (init?.method ?? "GET").toUpperCase();

      if (url.includes("/api/mini/videos/a/read") && method === "PUT") {
        return new Response(
          JSON.stringify({
            video_id: "a",
            read: true,
            updated_at: "2026-04-16T00:00:00.000Z",
          }),
          { status: 200 },
        );
      }
      throw new Error(`Unexpected request: ${method} ${url}`);
    }) as typeof fetch;

    await mini.markActiveSummaryRead();

    expect(mini.status).toBe("empty");
    expect(mini.visibleSummaries).toEqual([]);
    expect(mini.activeSummary).toBeNull();
  });

  it("returns to ready when clearing an unread-only empty filter reveals read summaries", () => {
    const mini = new MiniReaderState();
    mini.reader = {
      channels: [makeChannel()],
      selected_channel_id: "channel-1",
      summaries: [makeSummary("a", true)],
    };
    mini.selectedChannelId = "channel-1";
    mini.activeVideoId = null;
    mini.status = "empty";

    mini.clearUnreadFilter();

    expect(mini.status).toBe("ready");
    expect(mini.activeSummary?.video_id).toBe("a");
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

describe("saveMiniVocabularyPreferences", () => {
  it("merges vocabulary changes into fresh preferences before saving", async () => {
    const serverPreferences: UserPreferences = {
      channel_order: ["fresh-a", "fresh-b"],
      channel_sort_mode: "newest",
      vocabulary_replacements: [
        {
          from: "old",
          to: "older",
          added_at: "2026-04-16T10:00:00.000Z",
        },
      ],
    };
    const nextReplacements: UserPreferences["vocabulary_replacements"] = [
      {
        from: "LLM",
        to: "language model",
        added_at: "2026-04-16T10:01:00.000Z",
      },
    ];
    let savedPayload: UserPreferences | null = null;

    globalThis.fetch = (async (input, init) => {
      const url = String(input);
      const method = (init?.method ?? "GET").toUpperCase();

      if (url.includes("/api/preferences") && method === "GET") {
        return new Response(JSON.stringify(serverPreferences), { status: 200 });
      }
      if (url.includes("/api/preferences") && method === "PUT") {
        savedPayload = JSON.parse(String(init?.body)) as UserPreferences;
        return new Response(null, { status: 204 });
      }
      throw new Error(`Unexpected request: ${method} ${url}`);
    }) as typeof fetch;

    const result = await saveMiniVocabularyPreferences(nextReplacements);

    expect(result).toEqual({
      ...serverPreferences,
      vocabulary_replacements: nextReplacements,
    });
    expect(savedPayload).toEqual(result);
  });
});
