/**
 * Tests for shared workspace-style server loads.
 *
 * Verifies that URL query parameters for type and ack filters are correctly
 * forwarded to the workspace bootstrap API call so the SSR-rendered state
 * matches the URL-specified filter state on first paint.
 *
 * Also verifies that the selected channel snapshot is surfaced as a single
 * preloaded preview entry without triggering separate snapshot fetches.
 */
import { describe, expect, it } from "bun:test";
import type { ChannelSnapshot, WorkspaceBootstrap } from "../src/lib/types";

// Dynamically import to avoid SvelteKit-generated $types resolution at test time.
// All imports inside +page.server.ts are type-only, so the runtime module has
// no special dependencies.
async function importLoad() {
  const mod = await import("../src/routes/+page.server.js");
  return mod.load as (event: {
    fetch: typeof fetch;
    url: URL;
    isDataRequest?: boolean;
  }) => Promise<{
    bootstrap: WorkspaceBootstrap | null;
    channelPreviews: Record<string, ChannelSnapshot>;
    channelPreviewsFilterKey: string;
  }>;
}

async function importChannelRouteLoad() {
  const mod = await import("../src/routes/channels/[id]/+page.server.js");
  return mod.load as (event: {
    fetch: typeof fetch;
    url: URL;
    params: { id: string };
    isDataRequest?: boolean;
  }) => Promise<{
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

function makeBootstrapWithSelectedSnapshot(
  channelId: string,
): WorkspaceBootstrap {
  return {
    ...makeBootstrapWithChannels([channelId]),
    selected_channel_id: channelId,
    snapshot: makeChannelSnapshot(channelId),
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
    channel_video_count: videoCount,
    has_more: false,
    next_offset: null,
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

describe("+page.server.ts load - URL filter forwarding", () => {
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

describe("+page.server.ts load - data request bootstrap policy", () => {
  it("skips bootstrap fetching for unscoped data requests", async () => {
    const load = await importLoad();
    let called = false;
    const fetch = (async () => {
      called = true;
      return new Response("{}", { status: 200 });
    }) as typeof fetch;

    const result = await load({
      fetch,
      url: createUrl(),
      isDataRequest: true,
    });

    expect(called).toBe(false);
    expect(result.bootstrap).toBeNull();
    expect(result.channelPreviews).toEqual({});
  });

  it("fetches bootstrap for scoped data requests with a selected channel", async () => {
    const load = await importLoad();
    const channelId = "ch-42";
    const { fetch, calls } = createSmartMockFetch(
      makeBootstrapWithSelectedSnapshot(channelId),
    );

    const result = await load({
      fetch,
      url: createUrl({ channel: channelId, video: "vid-1" }),
      isDataRequest: true,
    });

    expect(calls).toHaveLength(1);
    expect(calls[0].url.searchParams.get("selected_channel_id")).toBe(
      channelId,
    );
    expect(result.bootstrap?.selected_channel_id).toBe(channelId);
    expect(result.channelPreviews[channelId]?.channel_id).toBe(channelId);
  });
});

describe("+page.server.ts load — channel preview pre-loading (VAL-DATA-002)", () => {
  // Channel previews are now loaded client-side after paint.
  // The server load returns an empty channelPreviews object and only
  // fetches the bootstrap — no per-channel snapshot requests.

  it("makes only the bootstrap fetch — no snapshot fetches", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1", "ch-2", "ch-3"]);
    const { fetch, calls } = createSmartMockFetch(bootstrap);
    await load({ fetch, url: createUrl() });
    expect(calls).toHaveLength(1);
    const snapshotCalls = calls.filter((c) =>
      c.url.pathname.includes("/snapshot"),
    );
    expect(snapshotCalls).toHaveLength(0);
  });

  it("returns empty channelPreviews regardless of channels in bootstrap", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithChannels(["ch-1", "ch-2"]);
    const { fetch } = createSmartMockFetch(bootstrap);
    const result = await load({ fetch, url: createUrl() });
    expect(result.channelPreviews).toEqual({});
  });

  it("returns the selected channel snapshot as a preloaded preview map entry", async () => {
    const load = await importLoad();
    const bootstrap = makeBootstrapWithSelectedSnapshot("ch-1");
    const { fetch } = createSmartMockFetch(bootstrap);
    const result = await load({ fetch, url: createUrl({ channel: "ch-1" }) });

    expect(result.channelPreviews).toEqual({
      "ch-1": bootstrap.snapshot,
    });
  });

  it("returns empty channelPreviews when bootstrap has no channels", async () => {
    const load = await importLoad();
    const { fetch, calls } = createMockFetch();
    const result = await load({ fetch, url: createUrl() });
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
});

describe("channels/[id]/+page.server.ts load", () => {
  it("uses the route param as selected_channel_id for bootstrap", async () => {
    const load = await importChannelRouteLoad();
    const bootstrap = makeBootstrapWithSelectedSnapshot("ch-route");
    const { fetch, calls } = createSmartMockFetch(bootstrap);

    const result = await load({
      fetch,
      url: createUrl(),
      params: { id: "ch-route" },
      isDataRequest: false,
    });

    expect(calls).toHaveLength(1);
    expect(calls[0]?.url.searchParams.get("selected_channel_id")).toBe(
      "ch-route",
    );
    expect(result.channelPreviews).toEqual({
      "ch-route": bootstrap.snapshot!,
    });
  });

  it("returns empty bootstrap data on data requests", async () => {
    const load = await importChannelRouteLoad();
    const { fetch, calls } = createSmartMockFetch(
      makeBootstrapWithSelectedSnapshot("ch-route"),
    );

    const result = await load({
      fetch,
      url: createUrl(),
      params: { id: "ch-route" },
      isDataRequest: true,
    });

    expect(calls).toHaveLength(0);
    expect(result.bootstrap).toBeNull();
    expect(result.channelPreviews).toEqual({});
    expect(result.channelPreviewsFilterKey).toBe("all:all:default");
  });
});
