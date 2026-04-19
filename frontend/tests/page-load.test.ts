import { describe, expect, it } from "bun:test";

import type { ChannelSnapshot, WorkspaceBootstrap } from "../src/lib/types";

async function importHomeLoad() {
  const mod = await import(
    `../src/routes/+page.ts?test=${Date.now()}-${Math.random()}`
  );
  return mod.load as (event: { fetch: typeof fetch; url: URL }) => Promise<{
    bootstrap: WorkspaceBootstrap | null;
    channelPreviews: Record<string, ChannelSnapshot>;
    channelPreviewsFilterKey: string;
    selectedChannelId: string | null;
    selectedVideoId: string | null;
  }>;
}

async function importChannelLoad() {
  const mod = await import(
    `../src/routes/channels/[id]/+page.ts?test=${Date.now()}-${Math.random()}`
  );
  return mod.load as (event: {
    fetch: typeof fetch;
    url: URL;
    params: { id: string };
  }) => Promise<{
    bootstrap: WorkspaceBootstrap | null;
    channelPreviews: Record<string, ChannelSnapshot>;
    channelPreviewsFilterKey: string;
    selectedChannelId: string | null;
    selectedVideoId: string | null;
  }>;
}

type CapturedCall = { url: URL };

function makeBootstrap(channelId: string | null = null): WorkspaceBootstrap {
  return {
    ai_available: true,
    ai_status: "cloud",
    channels: channelId
      ? [
          {
            id: channelId,
            name: `Channel ${channelId}`,
            added_at: "2026-01-01T00:00:00Z",
          },
        ]
      : [],
    selected_channel_id: channelId,
    snapshot: channelId
      ? {
          channel_id: channelId,
          sync_depth: {
            earliest_sync_date: null,
            earliest_sync_date_user_set: false,
            derived_earliest_ready_date: null,
          },
          channel_video_count: 1,
          has_more: false,
          next_offset: null,
          videos: [
            {
              id: `video-${channelId}`,
              channel_id: channelId,
              title: "Video",
              published_at: "2026-01-01T00:00:00Z",
              is_short: false,
              transcript_status: "ready",
              summary_status: "ready",
              acknowledged: false,
              retry_count: 0,
            },
          ],
        }
      : null,
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

function createMockFetch(bootstrap = makeBootstrap()): {
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
    return new Response(JSON.stringify(bootstrap), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    });
  };

  return { fetch: mockFetch as unknown as typeof fetch, calls };
}

describe("static route loads", () => {
  it("skips bootstrap fetch on the home route when no deep-link selection is present", async () => {
    const load = await importHomeLoad();
    const { fetch, calls } = createMockFetch();

    const result = await load({
      fetch,
      url: new URL("http://localhost/"),
    });

    expect(calls).toHaveLength(0);
    expect(result.bootstrap).toBeNull();
    expect(result.selectedChannelId).toBeNull();
    expect(result.selectedVideoId).toBeNull();
  });

  it("forwards deep-link filters to the workspace bootstrap API", async () => {
    const load = await importHomeLoad();
    const { fetch, calls } = createMockFetch(makeBootstrap("channel-123"));

    await load({
      fetch,
      url: new URL(
        "http://localhost/?channel=channel-123&video=video-1&type=short&ack=unack",
      ),
    });

    expect(calls).toHaveLength(1);
    expect(calls[0].url.pathname).toBe("/api/workspace/bootstrap");
    expect(calls[0].url.searchParams.get("selected_channel_id")).toBe(
      "channel-123",
    );
    expect(calls[0].url.searchParams.get("video_type")).toBe("short");
    expect(calls[0].url.searchParams.get("acknowledged")).toBe("false");
  });

  it("loads the channel route using the path param as the selected channel", async () => {
    const load = await importChannelLoad();
    const { fetch, calls } = createMockFetch(makeBootstrap("channel-abc"));

    const result = await load({
      fetch,
      url: new URL("http://localhost/channels/channel-abc"),
      params: { id: "channel-abc" },
    });

    expect(calls).toHaveLength(1);
    expect(calls[0].url.searchParams.get("selected_channel_id")).toBe(
      "channel-abc",
    );
    expect(result.channelPreviews["channel-abc"]?.channel_id).toBe(
      "channel-abc",
    );
  });
});
