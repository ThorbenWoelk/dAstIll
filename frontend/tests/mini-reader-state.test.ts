import { afterEach, describe, expect, it } from "bun:test";

import type { MiniSummaryItem } from "../src/lib/transport-types";
import {
  chooseActiveVideoId,
  findNextMiniChannelId,
  findNextUnreadVideoId,
  MINI_DEFAULT_SHOW_UNREAD_ONLY,
  miniChannelIsCaughtUp,
  saveMiniVocabularyPreferences,
  selectMiniSummaryHighlights,
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

describe("MiniReaderState auth-scope reset", () => {
  it("does not apply a stale reader response after an auth-scope reset", async () => {
    const { createMiniReaderState } =
      await import("../src/lib/mini/mini-reader-state.svelte");

    function makeChannel(id: string) {
      return {
        id,
        handle: null,
        name: `Channel ${id}`,
        thumbnail_url: null,
        added_at: "2026-04-16T00:00:00.000Z",
        earliest_sync_date: null,
        earliest_sync_date_user_set: false,
      };
    }

    function makeReader(channelId: string, videoId: string) {
      return {
        channels: [makeChannel(channelId)],
        selected_channel_id: channelId,
        summaries: [makeSummary(videoId, false, { channel_id: channelId })],
      };
    }

    let resolveFirst: ((value: Response) => void) | null = null;
    const firstRequest = new Promise<Response>((resolve) => {
      resolveFirst = resolve;
    });
    let miniRequestCount = 0;

    globalThis.fetch = (async (input) => {
      const url = String(input);
      if (url.includes("/api/mini")) {
        miniRequestCount += 1;
        if (miniRequestCount === 1) {
          return firstRequest;
        }
        return new Response(
          JSON.stringify(makeReader("channel-b", "video-b")),
          {
            status: 200,
          },
        );
      }
      if (url.includes("/api/preferences")) {
        return new Response(
          JSON.stringify({
            channel_order: [],
            channel_sort_mode: "custom",
            vocabulary_replacements: [],
          }),
          { status: 200 },
        );
      }
      throw new Error(`Unexpected request: ${url}`);
    }) as typeof fetch;

    const mini = createMiniReaderState();
    const firstLoad = mini.loadReader("channel-a");
    mini.resetForAuthScopeChange();
    const secondLoad = mini.loadReader("channel-b", null, {
      bypassCache: true,
    });

    resolveFirst?.(
      new Response(JSON.stringify(makeReader("channel-a", "video-a")), {
        status: 200,
      }),
    );

    await Promise.all([firstLoad, secondLoad]);

    expect(mini.selectedChannelId).toBe("channel-b");
    expect(mini.activeVideoId).toBe("video-b");
    expect(mini.reader?.selected_channel_id).toBe("channel-b");
  });
});
