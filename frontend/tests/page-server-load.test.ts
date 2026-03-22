/**
 * Tests for the root route server load function (+page.server.ts).
 *
 * Verifies that URL query parameters for type and ack filters are correctly
 * forwarded to the workspace bootstrap API call so the SSR-rendered state
 * matches the URL-specified filter state on first paint.
 */
import { describe, expect, it } from "bun:test";
import type { WorkspaceBootstrap } from "../src/lib/types";

// Dynamically import to avoid SvelteKit-generated $types resolution at test time.
// All imports inside +page.server.ts are type-only, so the runtime module has
// no special dependencies.
async function importLoad() {
  const mod = await import("../src/routes/+page.server.js");
  return mod.load as (event: {
    fetch: typeof fetch;
    url: URL;
  }) => Promise<{ bootstrap: WorkspaceBootstrap | null }>;
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
});
