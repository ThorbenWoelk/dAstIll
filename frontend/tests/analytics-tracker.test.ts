import { afterEach, beforeEach, describe, expect, it } from "bun:test";

const originalFetch = globalThis.fetch;
const originalWindow = globalThis.window;
type TrackerModule = typeof import("../src/lib/analytics/tracker");
const loadedTrackers: TrackerModule[] = [];

async function loadTracker(): Promise<TrackerModule> {
  const tracker = (await import(
    `../src/lib/analytics/tracker.ts?test=${Date.now()}-${Math.random()}`
  )) as TrackerModule;
  loadedTrackers.push(tracker);
  return tracker;
}

function restoreWindow() {
  if (originalWindow === undefined) {
    delete (globalThis as typeof globalThis & { window?: unknown }).window;
  } else {
    Object.defineProperty(globalThis, "window", {
      value: originalWindow,
      configurable: true,
    });
  }
}

async function settleAsyncFlush() {
  await Promise.resolve();
  await new Promise((resolve) => setTimeout(resolve, 20));
}

beforeEach(() => {
  Object.defineProperty(globalThis, "window", {
    value: {
      location: { origin: "http://localhost:3543" },
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
    },
    configurable: true,
  });
});

afterEach(() => {
  for (const tracker of loadedTrackers) {
    tracker.setAnalyticsEnabled(false);
  }
  loadedTrackers.length = 0;
  globalThis.fetch = originalFetch;
  restoreWindow();
});

describe("analytics tracker sink gating", () => {
  it("drops events before the backend reports that analytics is enabled", async () => {
    const { flush, track } = await loadTracker();
    let requests = 0;
    globalThis.fetch = (async () => {
      requests += 1;
      return new Response(null, { status: 202 });
    }) as typeof fetch;

    track({
      event: "video_opened",
      video_id: "video-1",
      channel_id: "channel-1",
    });
    flush();
    await settleAsyncFlush();

    expect(requests).toBe(0);
  });

  it("disables itself when a stale client reaches a backend with no sink", async () => {
    const { flush, isAnalyticsEnabled, setAnalyticsEnabled, track } =
      await loadTracker();
    let requests = 0;
    globalThis.fetch = (async () => {
      requests += 1;
      return new Response(null, { status: 204 });
    }) as typeof fetch;

    setAnalyticsEnabled(true);
    track({
      event: "video_opened",
      video_id: "video-1",
      channel_id: "channel-1",
    });
    flush();
    await settleAsyncFlush();

    expect(requests).toBe(1);
    expect(isAnalyticsEnabled()).toBe(false);

    track({
      event: "video_opened",
      video_id: "video-2",
      channel_id: "channel-1",
    });
    flush();
    await settleAsyncFlush();

    expect(requests).toBe(1);
  });
});
