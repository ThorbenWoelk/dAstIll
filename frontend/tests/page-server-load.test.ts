/**
 * Tests for the root route server load function (+page.server.ts).
 *
 * Verifies that URL query parameters for type and ack filters are correctly
 * forwarded to the workspace bootstrap API call so the SSR-rendered state
 * matches the URL-specified filter state on first paint.
 *
 * Also verifies that channel preview snapshots are pre-fetched server-side
 * for all channels so the client sidebar does not need to make N separate
 * snapshot API calls on initial mount (VAL-DATA-002).
 */
import { describe, expect, it } from "bun:test";
import type { ChannelSnapshot, WorkspaceBootstrap } from "../src/lib/types";

// Dynamically import to avoid SvelteKit-generated $types resolution at test time.
// All imports inside +page.server.ts are type-only, so the runtime module has
// no special dependencies.
async function importLoad() {
  const mod = await import("../src/routes/+page.server.js");
  return mod.load as (event: { fetch: typeof fetch; url: URL }) => Promise<{
    bootstrap: WorkspaceBootstrap | null;
    channelPreviews: Record<string, ChannelSnapshot>;
    channelPreviewsFilterKey: string;
  }>;
}

function makeBootstrapPayload(): WorkspaceBootstrap {
  return {
    ai_available: true,
    ai_status: "cloud",
    channels: [],
    selected_channel_id: null,
    snapshot: null,
    search_status: {
      available: false,
      model: "test",
      dimensions: 512,
      pending: 0,
      indexing: 0,
      ready: 0,
      failed: 0,
      total_sources: 0,
      total_chunk_count: 0,
      embedded_chunk_count: 0,
      vector_index_ready: false,
      retrieval_mode: "fts_only",
    },
  };
}

type CapturedCall = { url: URL };

function makeBootstrapWithChannels(channelIds: string[]): WorkspaceBootstrap {
  return {
    ai_available: true,
    ai_status: "cloud",
    channels: channelIds.map((id) => ({
      id,
      name: `Channel ${id}`,
      added_at: "2026-01-01T00:00:00Z",
    })),
    selected_channel_id: channelIds[0] ?? null,
    snapshot: null,
    search_status: {
      available: false,
      model: "test",
      dimensions: 512,
      pending: 0,
      indexing: 0,
      ready: 0,
      failed: 0,
      total_sources: 0,
      total_chunk_count: 0,
      embedded_chunk_count: 0,
      vector_index_ready: false,
      retrieval_mode: "fts_only",
    },
  };
}

function makeChannelSnapshot(
  channelId: string,
  videoCount = 3,
): ChannelSnapshot {
  return {
    channel_id: channelId,
    sync_depth: {
      earliest_sync_date: null,
      earliest_sync_date_user_set: false,
      derived_earliest_ready_date: null,
    },
    videos: Array.from({ length: videoCount }, (_, i) => ({
      id: `vid-${channelId}-${i}`,
      channel_id: channelId,
      title: `Video ${i}`,
      published_at: "2026-01-01T00:00:00Z",
      is_short: false,
      transcript_status: "ready" as const,
      summary_status: "ready" as const,
      acknowledged: false,
      retry_count: 0,
    })),
  };
}

function createMockFetch(status = 200): {
  fetch: typeof fetch;
  calls: CapturedCall[];
} {
  const calls: CapturedCall[] = [];
  const mockFetch = async (input: string | URL | Request) => {
    const url = new URL(
      typeof input === "string"
        ? input
        : input instanceof URL
          ? input.href
          : input.url,
      "http://localhost",
    );
    calls.push({ url });
    return new Response(JSON.stringify(makeBootstrapPayload()), {
      status,
      headers: { "Content-Type": "application/json" },
    });
  };
  return { fetch: mockFetch as unknown as typeof fetch, calls };
}

/**
 * Creates a mock fetch that returns different payloads depending on the URL:
 * - Bootstrap endpoint → the provided bootstrap payload
 * - Channel snapshot endpoints → a ChannelSnapshot payload for that channel
 */
function createSmartMockFetch(bootstrap: WorkspaceBootstrap): {
  fetch: typeof fetch;
  calls: CapturedCall[];
} {
  const calls: CapturedCall[] = [];
  const mockFetch = async (input: string | URL | Request) => {
    const url = new URL(
      typeof input === "string"
        ? input
        : input instanceof URL
          ? input.href
          : input.url,
      "http://localhost",
    );
    calls.push({ url });

    // Detect channel snapshot requests
    const snapshotMatch = url.pathname.match(
      /^\/api\/channels\/([^/]+)\/snapshot$/,
    );
    if (snapshotMatch) {
      const channelId = snapshotMatch[1];
      const snapshot = makeChannelSnapshot(channelId);
      return new Response(JSON.stringify(snapshot), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }

    // Default: return bootstrap payload
    return new Response(JSON.stringify(bootstrap), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    });
  };
  return { fetch: mockFetch as unknown as typeof fetch, calls };
}

function createUrl(params: Record<string, string> = {}): URL {
  const url = new URL("http://localhost/");
  for (const [key, value] of Object.entries(params)) {
    url.searchParams.set(key, value);
  }
  return url;
}

describe("+page.server.ts load — URL filter forwarding", () => {
  it("forwards type=short as video_type=short", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch();
    await load({ fetch, url: createUrl({ type: "short" }) });
    expect(calls).toHaveLength(1);
    expect(calls[0].url.searchParams.get("video_type")).toBe("short");
  });

  it("forwards type=long as video_type=long", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch();
    await load({ fetch, url: createUrl({ type: "long" }) });
    expect(calls).toHaveLength(1);
    expect(calls[0].url.searchParams.get("video_type")).toBe("long");
  });

  it("omits video_type when type=all (default, no filter needed)", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch();
    await load({ fetch, url: createUrl({ type: "all" }) });
    expect(calls).toHaveLength(1);
    expect(calls[0].url.searchParams.has("video_type")).toBe(false);
  });

  it("omits video_type when type param is absent", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch();
    await load({ fetch, url: createUrl({}) });
    expect(calls).toHaveLength(1);
    expect(calls[0].url.searchParams.has("video_type")).toBe(false);
  });

  it("forwards ack=ack as acknowledged=true", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch();
    await load({ fetch, url: createUrl({ ack: "ack" }) });
    expect(calls).toHaveLength(1);
    expect(calls[0].url.searchParams.get("acknowledged")).toBe("true");
  });

  it("forwards ack=unack as acknowledged=false", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch();
    await load({ fetch, url: createUrl({ ack: "unack" }) });
    expect(calls).toHaveLength(1);
    expect(calls[0].url.searchParams.get("acknowledged")).toBe("false");
  });

  it("omits acknowledged when ack=all (default, no filter needed)", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch();
    await load({ fetch, url: createUrl({ ack: "all" }) });
    expect(calls).toHaveLength(1);
    expect(calls[0].url.searchParams.has("acknowledged")).toBe(false);
  });

  it("omits acknowledged when ack param is absent", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch();
    await load({ fetch, url: createUrl({}) });
    expect(calls).toHaveLength(1);
    expect(calls[0].url.searchParams.has("acknowledged")).toBe(false);
  });

  it("ignores invalid type values", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch();
    await load({ fetch, url: createUrl({ type: "invalid" }) });
    expect(calls).toHaveLength(1);
    expect(calls[0].url.searchParams.has("video_type")).toBe(false);
  });

  it("ignores invalid ack values", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch();
    await load({ fetch, url: createUrl({ ack: "invalid" }) });
    expect(calls).toHaveLength(1);
    expect(calls[0].url.searchParams.has("acknowledged")).toBe(false);
  });

  it("forwards channel and limit alongside type and ack filters", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch();
    await load({
      fetch,
      url: createUrl({ channel: "ch-1", type: "short", ack: "ack" }),
    });
    expect(calls).toHaveLength(1);
    const sp = calls[0].url.searchParams;
    expect(sp.get("selected_channel_id")).toBe("ch-1");
    expect(sp.get("limit")).toBe("20");
    expect(sp.get("video_type")).toBe("short");
    expect(sp.get("acknowledged")).toBe("true");
  });

  it("returns null bootstrap when fetch fails", async () => {
    const load = await importLoad();
    const { fetch } = createMockFetch(500);
    const result = await load({ fetch, url: createUrl({ type: "short" }) });
    expect(result.bootstrap).toBeNull();
  });

  it("returns empty channelPreviews when bootstrap fails", async () => {
    const load = await importLoad();
    const { fetch } = createMockFetch(500);
    const result = await load({ fetch, url: createUrl() });
    expect(result.channelPreviews).toEqual({});
  });
});

describe("+page.server.ts load — channel preview pre-loading (VAL-DATA-002)", () => {
  it("fetches channel preview snapshots for all channels in bootstrap", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1", "ch-2", "ch-3"]);
    const { fetch, calls } = createSmartMockFetch(bootstrap);
    await load({ fetch, url: createUrl() });
    // 1 bootstrap + 3 channel snapshot fetches
    expect(calls).toHaveLength(4);
    const snapshotCalls = calls.filter((c) =>
      c.url.pathname.includes("/snapshot"),
    );
    expect(snapshotCalls).toHaveLength(3);
  });

  it("returns channelPreviews keyed by channel id", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1", "ch-2"]);
    const { fetch } = createSmartMockFetch(bootstrap);
    const result = await load({ fetch, url: createUrl() });
    expect(result.channelPreviews).toBeDefined();
    expect(Object.keys(result.channelPreviews)).toHaveLength(2);
    expect(result.channelPreviews["ch-1"]).toBeDefined();
    expect(result.channelPreviews["ch-2"]).toBeDefined();
  });

  it("uses limit=6 for channel preview fetches (matches sidebar PREVIEW_FETCH_LIMIT)", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1"]);
    const { fetch, calls } = createSmartMockFetch(bootstrap);
    await load({ fetch, url: createUrl() });
    const previewCall = calls.find((c) => c.url.pathname.includes("/snapshot"));
    expect(previewCall).toBeDefined();
    expect(previewCall!.url.searchParams.get("limit")).toBe("6");
  });

  it("uses offset=0 for channel preview fetches", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1"]);
    const { fetch, calls } = createSmartMockFetch(bootstrap);
    await load({ fetch, url: createUrl() });
    const previewCall = calls.find((c) => c.url.pathname.includes("/snapshot"));
    expect(previewCall!.url.searchParams.get("offset")).toBe("0");
  });

  it("forwards type filter to channel preview snapshot fetches", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1"]);
    const { fetch, calls } = createSmartMockFetch(bootstrap);
    await load({ fetch, url: createUrl({ type: "short" }) });
    const previewCall = calls.find((c) => c.url.pathname.includes("/snapshot"));
    expect(previewCall!.url.searchParams.get("video_type")).toBe("short");
  });

  it("forwards ack filter to channel preview snapshot fetches", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1"]);
    const { fetch, calls } = createSmartMockFetch(bootstrap);
    await load({ fetch, url: createUrl({ ack: "ack" }) });
    const previewCall = calls.find((c) => c.url.pathname.includes("/snapshot"));
    expect(previewCall!.url.searchParams.get("acknowledged")).toBe("true");
  });

  it("omits filters from channel preview fetches when not specified", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1"]);
    const { fetch, calls } = createSmartMockFetch(bootstrap);
    await load({ fetch, url: createUrl() });
    const previewCall = calls.find((c) => c.url.pathname.includes("/snapshot"));
    expect(previewCall!.url.searchParams.has("video_type")).toBe(false);
    expect(previewCall!.url.searchParams.has("acknowledged")).toBe(false);
  });

  it("returns empty channelPreviews when bootstrap has no channels", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch(); // returns channels: []
    const result = await load({ fetch, url: createUrl() });
    // Only bootstrap fetch, no snapshot fetches
    expect(calls).toHaveLength(1);
    expect(result.channelPreviews).toEqual({});
  });

  it("returns channelPreviewsFilterKey reflecting URL filter params", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1"]);
    const { fetch } = createSmartMockFetch(bootstrap);
    const result = await load({
      fetch,
      url: createUrl({ type: "short", ack: "unack" }),
    });
    expect(result.channelPreviewsFilterKey).toBe("short:unack:default");
  });

  it("returns channelPreviewsFilterKey as all:all when no URL filter params", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1"]);
    const { fetch } = createSmartMockFetch(bootstrap);
    const result = await load({ fetch, url: createUrl() });
    expect(result.channelPreviewsFilterKey).toBe("all:all:default");
  });

  it("returns channelPreviewsFilterKey as all:all for invalid filter params", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1"]);
    const { fetch } = createSmartMockFetch(bootstrap);
    const result = await load({
      fetch,
      url: createUrl({ type: "invalid", ack: "invalid" }),
    });
    expect(result.channelPreviewsFilterKey).toBe("all:all:default");
  });

  it("handles individual channel snapshot fetch failures gracefully", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1", "ch-2"]);
    const calls: CapturedCall[] = [];
    // ch-1 snapshot fails, ch-2 succeeds
    const mockFetch = async (input: string | URL | Request) => {
      const url = new URL(
        typeof input === "string"
          ? input
          : input instanceof URL
            ? input.href
            : input.url,
        "http://localhost",
      );
      calls.push({ url });

      const snapshotMatch = url.pathname.match(
        /^\/api\/channels\/([^/]+)\/snapshot$/,
      );
      if (snapshotMatch) {
        const channelId = snapshotMatch[1];
        if (channelId === "ch-1") {
          return new Response("error", { status: 500 });
        }
        return new Response(JSON.stringify(makeChannelSnapshot(channelId)), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        });
      }
      return new Response(JSON.stringify(bootstrap), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    };
    const result = await load({
      fetch: mockFetch as unknown as typeof fetch,
      url: createUrl(),
    });
    // ch-2 snapshot is available, ch-1 is omitted (failed gracefully)
    expect(result.channelPreviews["ch-2"]).toBeDefined();
    expect(result.channelPreviews["ch-1"]).toBeUndefined();
    expect(result.bootstrap).not.toBeNull();
  });

  it("channel preview fetches are made in parallel (all channels fetched)", async () => {
    const load = await importLoad();
    const channelIds = ["ch-1", "ch-2", "ch-3", "ch-4", "ch-5"];
    const bootstrap = makeBootstrapWithChannels(channelIds);
    const { fetch, calls } = createSmartMockFetch(bootstrap);
    await load({ fetch, url: createUrl() });
    // Verify all 5 channels got their snapshot fetched
    const snapshotCalls = calls.filter((c) =>
      c.url.pathname.includes("/snapshot"),
    );
    const fetchedChannelIds = snapshotCalls.map((c) => {
      const match = c.url.pathname.match(/\/channels\/([^/]+)\/snapshot/);
      return match?.[1];
    });
    for (const id of channelIds) {
      expect(fetchedChannelIds).toContain(id);
    }
  });
});
