/**
 * Targeted cache invalidation tests.
 *
 * Each test pre-caches a set of GET responses, then triggers a mutation and
 * verifies that:
 *   - Only related cache entries were invalidated (re-fetched after mutation)
 *   - Unrelated cached entries survive (served from cache without a new fetch)
 */

import { afterEach, describe, expect, it } from "bun:test";
import {
  createHighlight,
  deleteHighlight,
  ensureSummary,
  ensureTranscript,
  getChannelSnapshot,
  getVideoHighlights,
  isAiAvailable,
  listHighlights,
  resetApiCacheForTests,
  updateAcknowledged,
} from "../src/lib/api";
import type {
  AiHealthResponse,
  ChannelSnapshot,
  Highlight,
  HighlightChannelGroup,
  HighlightSource,
  Summary,
  SyncDepth,
  Transcript,
  Video,
} from "../src/lib/types";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  resetApiCacheForTests();
});

function syncDepth(): SyncDepth {
  return {
    earliest_sync_date: "2026-02-01T00:00:00.000Z",
    earliest_sync_date_user_set: false,
    derived_earliest_ready_date: "2026-02-10T00:00:00.000Z",
  };
}

function video(id: string, channelId = "chan-a"): Video {
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

function snapshot(channelId: string, videoId: string): ChannelSnapshot {
  return {
    channel_id: channelId,
    sync_depth: syncDepth(),
    channel_video_count: 1,
    videos: [video(videoId, channelId)],
  };
}

function aiHealth(): AiHealthResponse {
  return { available: true, status: "cloud" };
}

function highlight(id: number, videoId: string): Highlight {
  return {
    id,
    video_id: videoId,
    source: "transcript" as HighlightSource,
    text: "Important text",
    prefix_context: "Before",
    suffix_context: "After",
    created_at: "2026-03-12T20:00:00.000Z",
  };
}

function highlightGroups(): HighlightChannelGroup[] {
  return [
    {
      channel_id: "chan-a",
      channel_name: "Channel A",
      channel_thumbnail_url: null,
      videos: [
        {
          video_id: "vid-1",
          title: "Video 1",
          thumbnail_url: null,
          published_at: "2026-03-02T00:00:00.000Z",
          highlights: [highlight(1, "vid-1")],
        },
      ],
    },
  ];
}

// ---------------------------------------------------------------------------
// 1. updateAcknowledged — only invalidates the video's channel snapshot
// ---------------------------------------------------------------------------

describe("updateAcknowledged targeted invalidation", () => {
  it("only invalidates the mutated video's channel snapshot, leaving other channel snapshots and AI health cached", async () => {
    const snapshotA = snapshot("chan-a", "vid-1");
    const snapshotB = snapshot("chan-b", "vid-2");
    const ai = aiHealth();
    const requests: string[] = [];

    globalThis.fetch = (async (input, init) => {
      const url = String(input);
      const method = (init?.method ?? "GET").toUpperCase();
      requests.push(`${method} ${url}`);

      if (url.includes("/acknowledged")) {
        return new Response(
          JSON.stringify({ ...video("vid-1", "chan-a"), acknowledged: true }),
          { status: 200 },
        );
      }
      if (url.includes("/api/channels/chan-a/snapshot")) {
        return new Response(JSON.stringify(snapshotA), { status: 200 });
      }
      if (url.includes("/api/channels/chan-b/snapshot")) {
        return new Response(JSON.stringify(snapshotB), { status: 200 });
      }
      if (url.includes("/api/health/ai")) {
        return new Response(JSON.stringify(ai), { status: 200 });
      }
      throw new Error(`Unexpected request: ${url}`);
    }) as typeof fetch;

    // Pre-cache all three responses
    await getChannelSnapshot("chan-a");
    await getChannelSnapshot("chan-b");
    await isAiAvailable();

    const preMutationRequests = requests.length;
    expect(preMutationRequests).toBe(3);

    // Perform mutation — acknowledges vid-1 which belongs to chan-a
    await updateAcknowledged("vid-1", true);

    // chan-a snapshot should be re-fetched (invalidated)
    await getChannelSnapshot("chan-a");
    // chan-b snapshot should still be served from cache
    await getChannelSnapshot("chan-b");
    // AI health should still be served from cache
    await isAiAvailable();

    const chanAFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/channels/chan-a/snapshot"),
    ).length;
    const chanBFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/channels/chan-b/snapshot"),
    ).length;
    const aiFetches = requests.filter((r) =>
      r.includes("/api/health/ai"),
    ).length;

    // chan-a: 1 (pre-cache) + 1 (after mutation) = 2
    expect(chanAFetches).toBe(2);
    // chan-b: only pre-cache (still cached after mutation)
    expect(chanBFetches).toBe(1);
    // AI health: only pre-cache (unrelated to acknowledge)
    expect(aiFetches).toBe(1);
  });

  it("re-fetches when a GET completes after invalidation instead of applying stale data", async () => {
    const snapshotStale = snapshot("chan-a", "vid-1");
    const snapshotFresh: ChannelSnapshot = {
      ...snapshotStale,
      videos: [],
    };
    let releaseFirst: () => void;
    const firstBarrier = new Promise<void>((resolve) => {
      releaseFirst = resolve;
    });
    let chanASnapshotGets = 0;
    const requests: string[] = [];

    globalThis.fetch = (async (input, init) => {
      const url = String(input);
      const method = (init?.method ?? "GET").toUpperCase();
      requests.push(`${method} ${url}`);

      if (url.includes("/acknowledged")) {
        return new Response(
          JSON.stringify({ ...video("vid-1", "chan-a"), acknowledged: true }),
          { status: 200 },
        );
      }
      if (url.includes("/api/channels/chan-a/snapshot")) {
        chanASnapshotGets += 1;
        if (chanASnapshotGets === 1) {
          await firstBarrier;
          return new Response(JSON.stringify(snapshotStale), { status: 200 });
        }
        return new Response(JSON.stringify(snapshotFresh), { status: 200 });
      }
      throw new Error(`Unexpected request: ${url}`);
    }) as typeof fetch;

    const pendingSnapshot = getChannelSnapshot("chan-a");
    await updateAcknowledged("vid-1", true);
    releaseFirst!();
    const result = await pendingSnapshot;

    expect(result.videos).toEqual([]);
    const chanAGets = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/channels/chan-a/snapshot"),
    );
    expect(chanAGets.length).toBe(2);
  });
});

// ---------------------------------------------------------------------------
// 2. ensureTranscript — only invalidates transcript + channel snapshots/video lists
// ---------------------------------------------------------------------------

describe("ensureTranscript targeted invalidation", () => {
  it("invalidates the video transcript cache and channel snapshots, but not AI health or highlight caches", async () => {
    const snapshotA = snapshot("chan-a", "vid-1");
    const ai = aiHealth();
    const groups = highlightGroups();
    const transcript: Transcript = {
      video_id: "vid-1",
      raw_text: "Some raw transcript",
      formatted_markdown: null,
      render_mode: "plain_text",
    };
    const requests: string[] = [];

    globalThis.fetch = (async (input, init) => {
      const url = String(input);
      const method = (init?.method ?? "GET").toUpperCase();
      requests.push(`${method} ${url}`);

      if (url.includes("/transcript/ensure")) {
        return new Response(JSON.stringify(transcript), { status: 200 });
      }
      if (url.includes("/api/videos/vid-1/transcript")) {
        return new Response(JSON.stringify(transcript), { status: 200 });
      }
      if (url.includes("/api/channels/chan-a/snapshot")) {
        return new Response(JSON.stringify(snapshotA), { status: 200 });
      }
      if (url.includes("/api/health/ai")) {
        return new Response(JSON.stringify(ai), { status: 200 });
      }
      if (url.includes("/api/highlights")) {
        return new Response(JSON.stringify(groups), { status: 200 });
      }
      throw new Error(`Unexpected request: ${url}`);
    }) as typeof fetch;

    // Pre-cache: transcript, channel snapshot, AI health, highlights
    const { getTranscript } = await import("../src/lib/api");
    await getTranscript("vid-1");
    await getChannelSnapshot("chan-a");
    await isAiAvailable();
    await listHighlights();

    const preMutationCount = requests.length;
    expect(preMutationCount).toBe(4);

    // Perform mutation
    await ensureTranscript("vid-1");

    // Re-fetch all — only transcript and channel snapshot should cause new requests
    await getTranscript("vid-1");
    await getChannelSnapshot("chan-a");
    await isAiAvailable();
    await listHighlights();

    const transcriptFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/videos/vid-1/transcript"),
    ).length;
    const snapshotFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/channels/chan-a/snapshot"),
    ).length;
    const aiFetches = requests.filter((r) =>
      r.includes("/api/health/ai"),
    ).length;
    const highlightFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/highlights"),
    ).length;

    // Transcript: pre-cache + POST ensure + re-fetch after invalidation = 1 GET + 1 POST + 1 GET
    expect(transcriptFetches).toBe(2); // 1 before + 1 after
    // Snapshot: pre-cache + re-fetch after invalidation
    expect(snapshotFetches).toBe(2);
    // AI health: only pre-cache (unrelated to transcript)
    expect(aiFetches).toBe(1);
    // Highlights: only pre-cache (unrelated to transcript)
    expect(highlightFetches).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// 3. ensureSummary — only invalidates summary + channel snapshots/video lists
// ---------------------------------------------------------------------------

describe("ensureSummary targeted invalidation", () => {
  it("invalidates the video summary cache and channel snapshots, but not AI health or transcript cache", async () => {
    const snapshotA = snapshot("chan-a", "vid-1");
    const ai = aiHealth();
    const summaryPayload: Summary = {
      video_id: "vid-1",
      content: "A summary of the video",
    };
    const transcript: Transcript = {
      video_id: "vid-1",
      raw_text: "Some transcript text",
      formatted_markdown: null,
      render_mode: "plain_text",
    };
    const requests: string[] = [];

    globalThis.fetch = (async (input, init) => {
      const url = String(input);
      const method = (init?.method ?? "GET").toUpperCase();
      requests.push(`${method} ${url}`);

      if (url.includes("/summary/ensure")) {
        return new Response(JSON.stringify(summaryPayload), { status: 200 });
      }
      if (url.includes("/api/videos/vid-1/summary")) {
        return new Response(JSON.stringify(summaryPayload), { status: 200 });
      }
      if (url.includes("/api/videos/vid-1/transcript")) {
        return new Response(JSON.stringify(transcript), { status: 200 });
      }
      if (url.includes("/api/channels/chan-a/snapshot")) {
        return new Response(JSON.stringify(snapshotA), { status: 200 });
      }
      if (url.includes("/api/health/ai")) {
        return new Response(JSON.stringify(ai), { status: 200 });
      }
      throw new Error(`Unexpected request: ${url}`);
    }) as typeof fetch;

    // Pre-cache: summary, transcript, channel snapshot, AI health
    const { getSummary, getTranscript } = await import("../src/lib/api");
    await getSummary("vid-1");
    await getTranscript("vid-1");
    await getChannelSnapshot("chan-a");
    await isAiAvailable();

    expect(requests.length).toBe(4);

    // Perform mutation
    await ensureSummary("vid-1");

    // Re-fetch all
    await getSummary("vid-1");
    await getTranscript("vid-1");
    await getChannelSnapshot("chan-a");
    await isAiAvailable();

    const summaryFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/videos/vid-1/summary"),
    ).length;
    const transcriptFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/videos/vid-1/transcript"),
    ).length;
    const snapshotFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/channels/chan-a/snapshot"),
    ).length;
    const aiFetches = requests.filter((r) =>
      r.includes("/api/health/ai"),
    ).length;

    // Summary: pre-cache + after invalidation = 2 GETs
    expect(summaryFetches).toBe(2);
    // Snapshot: pre-cache + after invalidation = 2 GETs
    expect(snapshotFetches).toBe(2);
    // Transcript: only pre-cache (unrelated to summary ensure)
    expect(transcriptFetches).toBe(1);
    // AI health: only pre-cache (unrelated)
    expect(aiFetches).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// 4. createHighlight — only invalidates highlight caches
// ---------------------------------------------------------------------------

describe("createHighlight targeted invalidation", () => {
  it("only invalidates highlight caches, not channel snapshots or AI health", async () => {
    const snapshotA = snapshot("chan-a", "vid-1");
    const ai = aiHealth();
    const groups = highlightGroups();
    const perVideoHighlights = [highlight(1, "vid-1")];
    const newHighlight = highlight(2, "vid-1");
    const requests: string[] = [];

    globalThis.fetch = (async (input, init) => {
      const url = String(input);
      const method = (init?.method ?? "GET").toUpperCase();
      requests.push(`${method} ${url}`);

      if (method === "POST" && url.includes("/api/videos/vid-1/highlights")) {
        return new Response(JSON.stringify(newHighlight), { status: 200 });
      }
      if (url.includes("/api/videos/vid-1/highlights")) {
        return new Response(JSON.stringify(perVideoHighlights), {
          status: 200,
        });
      }
      if (url === `${url.split("/api/highlights")[0]}/api/highlights`) {
        return new Response(JSON.stringify(groups), { status: 200 });
      }
      if (url.includes("/api/highlights")) {
        return new Response(JSON.stringify(groups), { status: 200 });
      }
      if (url.includes("/api/channels/chan-a/snapshot")) {
        return new Response(JSON.stringify(snapshotA), { status: 200 });
      }
      if (url.includes("/api/health/ai")) {
        return new Response(JSON.stringify(ai), { status: 200 });
      }
      throw new Error(`Unexpected request: ${url}`);
    }) as typeof fetch;

    // Pre-cache: grouped highlights, per-video highlights, snapshot, AI health
    await listHighlights();
    await getVideoHighlights("vid-1");
    await getChannelSnapshot("chan-a");
    await isAiAvailable();

    expect(requests.length).toBe(4);

    // Perform mutation
    await createHighlight("vid-1", {
      source: "transcript",
      text: "New highlight",
      prefix_context: "",
      suffix_context: "",
    });

    // Re-fetch all
    await listHighlights();
    await getVideoHighlights("vid-1");
    await getChannelSnapshot("chan-a");
    await isAiAvailable();

    const groupedFetches = requests.filter(
      (r) => r.includes("GET") && r.endsWith("/api/highlights"),
    ).length;
    const perVideoFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/videos/vid-1/highlights"),
    ).length;
    const snapshotFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/channels/chan-a/snapshot"),
    ).length;
    const aiFetches = requests.filter((r) =>
      r.includes("/api/health/ai"),
    ).length;

    // Both highlight endpoints should be re-fetched (invalidated)
    expect(groupedFetches).toBe(2); // pre-cache + after mutation
    expect(perVideoFetches).toBe(2); // pre-cache + after mutation
    // Channel snapshot should remain cached (unrelated to highlights)
    expect(snapshotFetches).toBe(1);
    // AI health should remain cached (unrelated)
    expect(aiFetches).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// 5. deleteHighlight — only invalidates highlight caches
// ---------------------------------------------------------------------------

describe("deleteHighlight targeted invalidation", () => {
  it("also evicts per-video highlight caches when videoId is not in context (regression)", async () => {
    const groups = highlightGroups();
    const perVideoHighlights = [highlight(1, "vid-1")];
    const requests: string[] = [];

    globalThis.fetch = (async (input, init) => {
      const url = String(input);
      const method = (init?.method ?? "GET").toUpperCase();
      requests.push(`${method} ${url}`);

      if (method === "DELETE" && url.includes("/api/highlights/1")) {
        return new Response(null, { status: 204 });
      }
      if (url.includes("/api/videos/vid-1/highlights")) {
        return new Response(JSON.stringify(perVideoHighlights), {
          status: 200,
        });
      }
      if (url.includes("/api/highlights")) {
        return new Response(JSON.stringify(groups), { status: 200 });
      }
      throw new Error(`Unexpected request: ${url}`);
    }) as typeof fetch;

    // Pre-cache: grouped highlights and per-video highlights for vid-1
    await listHighlights();
    await getVideoHighlights("vid-1");

    expect(requests.length).toBe(2);

    // deleteHighlight does NOT pass videoId to invalidateHighlightCache —
    // the fix must evict all per-video highlight caches in that case.
    await deleteHighlight(1);

    // Re-fetch — both caches must have been evicted
    await listHighlights();
    await getVideoHighlights("vid-1");

    const groupedFetches = requests.filter(
      (r) => r.includes("GET") && r.endsWith("/api/highlights"),
    ).length;
    const perVideoFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/videos/vid-1/highlights"),
    ).length;

    // Grouped highlights: pre-cache + after invalidation
    expect(groupedFetches).toBe(2);
    // Per-video highlights: pre-cache + after invalidation (regression check)
    expect(perVideoFetches).toBe(2);
  });

  it("only invalidates highlight caches, not channel snapshots or AI health", async () => {
    const snapshotA = snapshot("chan-a", "vid-1");
    const ai = aiHealth();
    const groups = highlightGroups();
    const requests: string[] = [];

    globalThis.fetch = (async (input, init) => {
      const url = String(input);
      const method = (init?.method ?? "GET").toUpperCase();
      requests.push(`${method} ${url}`);

      if (method === "DELETE" && url.includes("/api/highlights/1")) {
        return new Response(null, { status: 204 });
      }
      if (url.includes("/api/highlights")) {
        return new Response(JSON.stringify(groups), { status: 200 });
      }
      if (url.includes("/api/channels/chan-a/snapshot")) {
        return new Response(JSON.stringify(snapshotA), { status: 200 });
      }
      if (url.includes("/api/health/ai")) {
        return new Response(JSON.stringify(ai), { status: 200 });
      }
      throw new Error(`Unexpected request: ${url}`);
    }) as typeof fetch;

    // Pre-cache: grouped highlights, channel snapshot, AI health
    await listHighlights();
    await getChannelSnapshot("chan-a");
    await isAiAvailable();

    expect(requests.length).toBe(3);

    // Perform mutation
    await deleteHighlight(1);

    // Re-fetch all
    await listHighlights();
    await getChannelSnapshot("chan-a");
    await isAiAvailable();

    const groupedFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/highlights"),
    ).length;
    const snapshotFetches = requests.filter(
      (r) => r.includes("GET") && r.includes("/api/channels/chan-a/snapshot"),
    ).length;
    const aiFetches = requests.filter((r) =>
      r.includes("/api/health/ai"),
    ).length;

    // Grouped highlights should be re-fetched (invalidated)
    expect(groupedFetches).toBe(2); // pre-cache + after mutation
    // Channel snapshot should remain cached (unrelated to highlights)
    expect(snapshotFetches).toBe(1);
    // AI health should remain cached (unrelated)
    expect(aiFetches).toBe(1);
  });
});
