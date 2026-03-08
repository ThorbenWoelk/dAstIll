import { afterEach, describe, expect, it } from "bun:test";
import {
  getChannelSnapshot,
  getWorkspaceBootstrap,
  isAiAvailable,
  listChannelsWhenAvailable,
  resetApiCacheForTests,
} from "../src/lib/api";
import type { Channel, SyncDepth, Video } from "../src/lib/types";

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
});
