import { afterEach, describe, expect, it } from "bun:test";

import type { MiniReader, MiniSummaryItem } from "../src/lib/transport-types";
import {
  chooseActiveVideoId,
  findNextMiniChannelId,
  findNextUnreadVideoId,
  MINI_DEFAULT_SHOW_UNREAD_ONLY,
  miniChannelIsCaughtUp,
  miniReaderReadyStatus,
  saveMiniVocabularyPreferences,
  selectMiniSummaryHighlights,
  visibleMiniSummaries,
} from "../src/lib/mini/mini-reader-state.svelte";
import { resetApiCacheForTests } from "../src/lib/api";
import type { Highlight, UserPreferences } from "../src/lib/types";

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

function makeReader(summaries: MiniSummaryItem[]): MiniReader {
  return {
    channels: [
      {
        id: "channel-1",
        handle: null,
        name: "Channel",
        thumbnail_url: null,
        added_at: "2026-04-16T00:00:00.000Z",
        earliest_sync_date: null,
        earliest_sync_date_user_set: false,
      },
    ],
    selected_channel_id: "channel-1",
    summaries,
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

describe("mini reader visible status", () => {
  it("reports empty when unread filtering hides every summary", () => {
    const reader = makeReader([makeSummary("a", true), makeSummary("b", true)]);

    expect(visibleMiniSummaries(reader, true)).toEqual([]);
    expect(miniReaderReadyStatus(reader, true)).toBe("empty");
    expect(miniReaderReadyStatus(reader, false)).toBe("ready");
  });

  it("reports empty when there are no subscribed channels", () => {
    const reader: MiniReader = {
      channels: [],
      selected_channel_id: null,
      summaries: [],
    };

    expect(miniReaderReadyStatus(reader, true)).toBe("empty");
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
