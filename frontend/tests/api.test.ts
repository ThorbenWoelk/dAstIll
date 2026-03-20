import { afterEach, describe, expect, it } from "bun:test";
import {
  cleanTranscriptFormatting,
  createHighlight,
  deleteHighlight,
  getChannelSnapshot,
  getVideoHighlights,
  getWorkspaceBootstrap,
  isAiAvailable,
  listHighlights,
  listChannelsWhenAvailable,
  refreshChannel,
  resetApiCacheForTests,
  searchContent,
} from "../src/lib/api";
import type {
  Channel,
  HighlightChannelGroup,
  HighlightSource,
  SearchResponse,
  SearchStatus,
  SyncDepth,
  Video,
} from "../src/lib/types";

const originalFetch = globalThis.fetch;

function channel(id: string): Channel {
  return {
    id,
    name: `Channel ${id}`,
    added_at: "2026-03-02T00:00:00.000Z",
  };
}

function video(id: string, channelId = "abc"): Video {
  return {
    id,
    channel_id: channelId,
    title: `Video ${id}`,
    published_at: "2026-03-02T00:00:00.000Z",
    is_short: false,
    transcript_status: "ready",
    summary_status: "ready",
    acknowledged: false,
    retry_count: 0,
  };
}

function syncDepth(): SyncDepth {
  return {
    earliest_sync_date: "2026-02-01T00:00:00.000Z",
    earliest_sync_date_user_set: false,
    derived_earliest_ready_date: "2026-02-10T00:00:00.000Z",
  };
}

function searchStatus(): SearchStatus {
  return {
    available: false,
    model: "",
    dimensions: 0,
    pending: 1,
    indexing: 0,
    ready: 3,
    failed: 0,
    total_sources: 4,
    total_chunk_count: 12,
    embedded_chunk_count: 0,
    vector_index_ready: false,
    retrieval_mode: "fts_only",
  };
}

afterEach(() => {
  globalThis.fetch = originalFetch;
  resetApiCacheForTests();
});

describe("listChannelsWhenAvailable", () => {
  it("retries when backend is unreachable and resolves once reachable", async () => {
    const expected = [channel("abc")];
    let attempts = 0;

    globalThis.fetch = (async () => {
      attempts += 1;
      if (attempts === 1) {
        throw new TypeError("fetch failed");
      }
      return new Response(JSON.stringify(expected), { status: 200 });
    }) as typeof fetch;

    const result = await listChannelsWhenAvailable({ retryDelayMs: 0 });
    expect(result).toEqual(expected);
    expect(attempts).toBe(2);
  });

  it("does not retry non-reachability failures", async () => {
    let attempts = 0;

    globalThis.fetch = (async () => {
      attempts += 1;
      return new Response("bad request", { status: 400 });
    }) as typeof fetch;

    await expect(
      listChannelsWhenAvailable({ retryDelayMs: 0 }),
    ).rejects.toThrow("bad request");
    expect(attempts).toBe(1);
  });
});

describe("getWorkspaceBootstrap", () => {
  it("requests combined startup data with the active filters", async () => {
    const payload = {
      ai_available: true,
      ai_status: "cloud",
      channels: [channel("abc")],
      selected_channel_id: "abc",
      search_status: searchStatus(),
      snapshot: {
        channel_id: "abc",
        sync_depth: syncDepth(),
        videos: [video("vid-1")],
      },
    };
    let requestedUrl = "";

    globalThis.fetch = (async (input) => {
      requestedUrl = String(input);
      return new Response(JSON.stringify(payload), { status: 200 });
    }) as typeof fetch;

    const result = await getWorkspaceBootstrap({
      selectedChannelId: "abc",
      limit: 12,
      offset: 24,
      videoType: "short",
      acknowledged: true,
    });

    expect(result).toEqual(payload);
    expect(result.search_status.total_sources).toBe(4);
    expect(requestedUrl).toContain("/api/workspace/bootstrap?");
    expect(requestedUrl).toContain("selected_channel_id=abc");
    expect(requestedUrl).toContain("limit=12");
    expect(requestedUrl).toContain("offset=24");
    expect(requestedUrl).toContain("video_type=short");
    expect(requestedUrl).toContain("acknowledged=true");
  });

  it("reuses cached bootstrap responses for identical requests", async () => {
    const payload = {
      ai_available: true,
      ai_status: "cloud",
      channels: [channel("abc")],
      selected_channel_id: "abc",
      search_status: searchStatus(),
      snapshot: {
        channel_id: "abc",
        sync_depth: syncDepth(),
        videos: [video("vid-1")],
      },
    };
    let attempts = 0;

    globalThis.fetch = (async () => {
      attempts += 1;
      return new Response(JSON.stringify(payload), { status: 200 });
    }) as typeof fetch;

    const first = await getWorkspaceBootstrap({ selectedChannelId: "abc" });
    const second = await getWorkspaceBootstrap({ selectedChannelId: "abc" });

    expect(first).toEqual(payload);
    expect(second).toEqual(payload);
    expect(attempts).toBe(1);
  });
});

describe("isAiAvailable", () => {
  it("returns availability plus the indicator status", async () => {
    const payload = {
      available: true,
      status: "local_only" as const,
    };

    globalThis.fetch = (async () => {
      return new Response(JSON.stringify(payload), { status: 200 });
    }) as typeof fetch;

    await expect(isAiAvailable()).resolves.toEqual(payload);
  });
});

describe("cleanTranscriptFormatting", () => {
  it("rewrites aborts into a generic timeout error", async () => {
    globalThis.fetch = (async () => {
      const error = new Error("The operation was aborted.");
      error.name = "AbortError";
      throw error;
    }) as typeof fetch;

    await expect(
      cleanTranscriptFormatting("video-1", "Example transcript"),
    ).rejects.toThrow("Formatting took too long to complete.");
  });
});

describe("getChannelSnapshot", () => {
  it("requests combined channel state including queue-only filters", async () => {
    const payload = {
      channel_id: "abc",
      sync_depth: syncDepth(),
      videos: [video("queued-1")],
    };
    let requestedUrl = "";

    globalThis.fetch = (async (input) => {
      requestedUrl = String(input);
      return new Response(JSON.stringify(payload), { status: 200 });
    }) as typeof fetch;

    const result = await getChannelSnapshot("abc", {
      limit: 20,
      offset: 0,
      queueOnly: true,
    });

    expect(result).toEqual(payload);
    expect(requestedUrl).toContain("/api/channels/abc/snapshot?");
    expect(requestedUrl).toContain("limit=20");
    expect(requestedUrl).toContain("offset=0");
    expect(requestedUrl).toContain("queue_only=true");
  });

  it("reuses cached snapshots until a write invalidates them", async () => {
    const payload = {
      channel_id: "abc",
      sync_depth: syncDepth(),
      videos: [video("queued-1")],
    };
    let attempts = 0;

    globalThis.fetch = (async (input, init) => {
      attempts += 1;
      if (String(input).includes("/acknowledged")) {
        return new Response(
          JSON.stringify({
            ...video("queued-1"),
            acknowledged: true,
          }),
          { status: 200 },
        );
      }

      return new Response(JSON.stringify(payload), { status: 200 });
    }) as typeof fetch;

    await getChannelSnapshot("abc", { queueOnly: true });
    await getChannelSnapshot("abc", { queueOnly: true });
    expect(attempts).toBe(1);

    const { updateAcknowledged } = await import("../src/lib/api");
    await updateAcknowledged("queued-1", true);

    await getChannelSnapshot("abc", { queueOnly: true });
    expect(attempts).toBe(3);
  });

  it("refresh invalidates only the refreshed channel reads", async () => {
    const snapshotPayload = {
      channel_id: "abc",
      sync_depth: syncDepth(),
      videos: [video("queued-1")],
    };
    const otherSnapshotPayload = {
      channel_id: "xyz",
      sync_depth: syncDepth(),
      videos: [video("queued-2", "xyz")],
    };
    const requests: string[] = [];

    globalThis.fetch = (async (input, init) => {
      const url = String(input);
      requests.push(`${init?.method ?? "GET"} ${url}`);

      if (url.includes("/api/channels/abc/refresh")) {
        return new Response(JSON.stringify({ videos_added: 1 }), {
          status: 200,
        });
      }

      if (url.includes("/api/channels/abc/snapshot")) {
        return new Response(JSON.stringify(snapshotPayload), { status: 200 });
      }

      if (url.includes("/api/channels/xyz/snapshot")) {
        return new Response(JSON.stringify(otherSnapshotPayload), {
          status: 200,
        });
      }

      if (url.includes("/api/channels")) {
        return new Response(JSON.stringify([channel("abc"), channel("xyz")]), {
          status: 200,
        });
      }

      throw new Error(`Unexpected request: ${url}`);
    }) as typeof fetch;

    await listChannelsWhenAvailable({ retryDelayMs: 0 });
    await getChannelSnapshot("abc", { queueOnly: true });
    await getChannelSnapshot("xyz", { queueOnly: true });

    await refreshChannel("abc");

    await listChannelsWhenAvailable({ retryDelayMs: 0 });
    await getChannelSnapshot("abc", { queueOnly: true });
    await getChannelSnapshot("xyz", { queueOnly: true });

    expect(
      requests.filter((request) => request === "GET /api/channels").length,
    ).toBe(1);
    expect(
      requests.filter((request) =>
        request.includes("GET /api/channels/abc/snapshot"),
      ).length,
    ).toBe(2);
    expect(
      requests.filter((request) =>
        request.includes("GET /api/channels/xyz/snapshot"),
      ).length,
    ).toBe(1);
  });
});

describe("highlight api helpers", () => {
  it("posts a new highlight and clears cached reads", async () => {
    const payload = {
      id: 7,
      video_id: "vid-7",
      source: "transcript" as HighlightSource,
      text: "Important passage",
      prefix_context: "Before",
      suffix_context: "After",
      created_at: "2026-03-12T20:00:00.000Z",
    };
    const requests: Array<{ url: string; init?: RequestInit }> = [];

    globalThis.fetch = (async (input, init) => {
      requests.push({ url: String(input), init });
      if (String(input).includes("/api/highlights")) {
        return new Response(JSON.stringify([]), { status: 200 });
      }
      return new Response(JSON.stringify(payload), { status: 200 });
    }) as typeof fetch;

    await createHighlight("vid-7", {
      source: "transcript",
      text: "Important passage",
      prefix_context: "Before",
      suffix_context: "After",
    });
    await listHighlights();

    expect(requests[0].url).toContain("/api/videos/vid-7/highlights");
    expect(requests[0].init?.method).toBe("POST");
    expect(requests[0].init?.body).toBe(
      JSON.stringify({
        source: "transcript",
        text: "Important passage",
        prefix_context: "Before",
        suffix_context: "After",
      }),
    );
    expect(requests[1].url).toContain("/api/highlights");
  });

  it("loads grouped highlights and per-video highlights from their endpoints", async () => {
    const groupedPayload: HighlightChannelGroup[] = [
      {
        channel_id: "abc",
        channel_name: "Channel ABC",
        channel_thumbnail_url: null,
        videos: [
          {
            video_id: "vid-1",
            title: "Video 1",
            thumbnail_url: null,
            published_at: "2026-03-02T00:00:00.000Z",
            highlights: [
              {
                id: 1,
                video_id: "vid-1",
                source: "summary",
                text: "Important idea",
                prefix_context: "",
                suffix_context: "",
                created_at: "2026-03-12T20:00:00.000Z",
              },
            ],
          },
        ],
      },
    ];
    const perVideoPayload = groupedPayload[0].videos[0].highlights;
    const requestedUrls: string[] = [];

    globalThis.fetch = (async (input) => {
      requestedUrls.push(String(input));
      if (String(input).includes("/api/videos/vid-1/highlights")) {
        return new Response(JSON.stringify(perVideoPayload), { status: 200 });
      }
      return new Response(JSON.stringify(groupedPayload), { status: 200 });
    }) as typeof fetch;

    await expect(listHighlights()).resolves.toEqual(groupedPayload);
    await expect(getVideoHighlights("vid-1")).resolves.toEqual(perVideoPayload);
    expect(requestedUrls[0]).toContain("/api/highlights");
    expect(requestedUrls[1]).toContain("/api/videos/vid-1/highlights");
  });

  it("deletes a highlight and clears cached reads", async () => {
    const requests: Array<{ url: string; init?: RequestInit }> = [];

    globalThis.fetch = (async (input, init) => {
      requests.push({ url: String(input), init });
      if (String(input).includes("/api/highlights/7")) {
        return new Response(null, { status: 204 });
      }
      return new Response(JSON.stringify([]), { status: 200 });
    }) as typeof fetch;

    await deleteHighlight(7);
    await listHighlights();

    expect(requests[0].url).toContain("/api/highlights/7");
    expect(requests[0].init?.method).toBe("DELETE");
    expect(requests[1].url).toContain("/api/highlights");
  });
});

describe("search api helpers", () => {
  it("queries search with source and channel filters", async () => {
    const payload: SearchResponse = {
      query: "semantic search",
      source: "summary",
      results: [],
    };
    let requestedUrl = "";

    globalThis.fetch = (async (input) => {
      requestedUrl = String(input);
      return new Response(JSON.stringify(payload), { status: 200 });
    }) as typeof fetch;

    const result = await searchContent("semantic search", {
      source: "summary",
      channelId: "abc",
      limit: 7,
      mode: "hybrid",
    });

    expect(result).toEqual(payload);
    expect(requestedUrl).toContain("/api/search?");
    expect(requestedUrl).toContain("q=semantic+search");
    expect(requestedUrl).toContain("source=summary");
    expect(requestedUrl).toContain("channel_id=abc");
    expect(requestedUrl).toContain("limit=7");
    expect(requestedUrl).toContain("mode=hybrid");
  });

  it("passes through an abort signal for cancellable searches", async () => {
    const payload: SearchResponse = {
      query: "db",
      source: "all",
      results: [],
    };
    const controller = new AbortController();
    let requestedSignal: AbortSignal | null | undefined;

    globalThis.fetch = (async (_input, init) => {
      requestedSignal = init?.signal;
      return new Response(JSON.stringify(payload), { status: 200 });
    }) as typeof fetch;

    await searchContent("db", {
      signal: controller.signal,
    });

    expect(requestedSignal).toBe(controller.signal);
  });

  it("supports decoupled semantic-only searches", async () => {
    const payload: SearchResponse = {
      query: "best db",
      source: "all",
      results: [],
    };
    let requestedUrl = "";

    globalThis.fetch = (async (input) => {
      requestedUrl = String(input);
      return new Response(JSON.stringify(payload), { status: 200 });
    }) as typeof fetch;

    await searchContent("best db", {
      mode: "semantic",
    });

    expect(requestedUrl).toContain("mode=semantic");
  });
});
