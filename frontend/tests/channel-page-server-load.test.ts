import { describe, expect, it } from "bun:test";
import type { ChannelSnapshot, WorkspaceBootstrap } from "../src/lib/types";

async function importLoad() {
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

function makeBootstrap(channelId: string): WorkspaceBootstrap {
  return {
    ai_available: true,
    ai_status: "cloud",
    channels: [
      {
        id: channelId,
        name: `Channel ${channelId}`,
        added_at: "2026-01-01T00:00:00Z",
      },
    ],
    selected_channel_id: channelId,
    snapshot: {
      channel_id: channelId,
      sync_depth: {
        earliest_sync_date: null,
        earliest_sync_date_user_set: false,
        derived_earliest_ready_date: null,
      },
      channel_video_count: null,
      has_more: false,
      next_offset: null,
      videos: [],
    },
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

describe("channels/[id] +page.server.ts", () => {
  it("uses the route param as selected channel for bootstrap", async () => {
    const load = await importLoad();
    const calls: URL[] = [];
    const fetch = (async (input: string | URL | Request) => {
      const url = new URL(
        typeof input === "string"
          ? input
          : input instanceof URL
            ? input.href
            : input.url,
        "http://localhost",
      );
      calls.push(url);
      return new Response(JSON.stringify(makeBootstrap("ch-42")), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }) as typeof fetch;

    const result = await load({
      fetch,
      url: new URL("http://localhost/channels/ch-42"),
      params: { id: "ch-42" },
      isDataRequest: false,
    });

    expect(calls).toHaveLength(1);
    expect(calls[0].searchParams.get("selected_channel_id")).toBe("ch-42");
    expect(result.channelPreviews["ch-42"]?.channel_id).toBe("ch-42");
  });

  it("skips bootstrap fetching on data requests", async () => {
    const load = await importLoad();
    let called = false;
    const fetch = (async () => {
      called = true;
      return new Response("{}", { status: 200 });
    }) as typeof fetch;

    const result = await load({
      fetch,
      url: new URL("http://localhost/channels/ch-42"),
      params: { id: "ch-42" },
      isDataRequest: true,
    });

    expect(called).toBe(false);
    expect(result.bootstrap).toBeNull();
    expect(result.channelPreviews).toEqual({});
    expect(result.channelPreviewsFilterKey).toBe("all:all:default");
  });
});
