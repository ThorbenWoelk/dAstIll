/**
 * Lightweight first-party analytics tracker.
 *
 * Events are buffered client-side and flushed in batches to
 * POST /api/analytics/events. Fire-and-forget: failures are
 * silently dropped so analytics never disrupts the user experience.
 */

import type { AnalyticsEvent, AnalyticsEventName } from "./events";
import { createApiRequestInit, resolveApiUrl } from "$lib/api-client";

// Distributive Omit preserves the discriminated union structure.
type DistributiveOmit<T, K extends PropertyKey> = T extends unknown
  ? Omit<T, K>
  : never;
type EventInput = DistributiveOmit<
  AnalyticsEvent,
  "event_id" | "ts" | "session_id"
>;

const FLUSH_INTERVAL_MS = 5_000;
const MAX_BATCH_SIZE = 50;

// Tab-scoped session id, stable for the lifetime of the page.
let sessionId: string;
try {
  sessionId = crypto.randomUUID();
} catch {
  sessionId = Math.random().toString(36).slice(2) + Date.now().toString(36);
}

const queue: AnalyticsEvent[] = [];
let flushTimer: ReturnType<typeof setTimeout> | null = null;
let analyticsEnabled = false;

export function setAnalyticsEnabled(enabled: boolean): void {
  analyticsEnabled = enabled;
  if (enabled) return;

  queue.length = 0;
  if (flushTimer !== null) {
    clearTimeout(flushTimer);
    flushTimer = null;
  }
}

export function isAnalyticsEnabled(): boolean {
  return analyticsEnabled;
}

function createId(): string {
  try {
    return crypto.randomUUID();
  } catch {
    return Math.random().toString(36).slice(2) + Date.now().toString(36);
  }
}

function stamp(partial: EventInput): AnalyticsEvent {
  return {
    ...partial,
    event_id: createId(),
    ts: new Date().toISOString(),
    session_id: sessionId,
  } as AnalyticsEvent;
}

async function sendBatch(batch: AnalyticsEvent[]): Promise<void> {
  if (!analyticsEnabled || batch.length === 0) return;
  try {
    const response = await fetch(resolveApiUrl("/api/analytics/events"), {
      ...(await createApiRequestInit(
        {
          method: "POST",
          body: JSON.stringify(batch),
          // keepalive allows delivery even when the page is closing.
          keepalive: true,
        },
        {
          includeJsonContentType: true,
        },
      )),
    });
    if (response.status === 204 || response.status === 404) {
      setAnalyticsEnabled(false);
    }
  } catch {
    // intentionally silent
  }
}

function sendBeaconBatch(batch: AnalyticsEvent[]): boolean {
  if (
    typeof navigator === "undefined" ||
    typeof navigator.sendBeacon !== "function" ||
    !analyticsEnabled ||
    batch.length === 0
  ) {
    return false;
  }

  try {
    const payload = JSON.stringify(batch);
    return navigator.sendBeacon(
      resolveApiUrl("/api/analytics/events"),
      new Blob([payload], { type: "application/json" }),
    );
  } catch {
    return false;
  }
}

function scheduleFlush() {
  if (flushTimer !== null) return;
  flushTimer = setTimeout(() => {
    flushTimer = null;
    flush();
  }, FLUSH_INTERVAL_MS);
}

function flushWithTransport(preferBeacon = false): void {
  if (!analyticsEnabled) {
    queue.length = 0;
    return;
  }
  if (queue.length === 0) return;
  const batch = queue.splice(0, MAX_BATCH_SIZE);
  if (!(preferBeacon && sendBeaconBatch(batch))) {
    void sendBatch(batch);
  }
  if (queue.length > 0) {
    scheduleFlush();
  }
}

export function flush(): void {
  flushWithTransport(false);
}

/** Enqueue an analytics event. */
export function track(partial: EventInput): void {
  if (typeof window === "undefined") return; // SSR guard
  if (!analyticsEnabled) return;
  const event = stamp(partial);
  queue.push(event as AnalyticsEvent);
  if (queue.length >= MAX_BATCH_SIZE) {
    if (flushTimer !== null) {
      clearTimeout(flushTimer);
      flushTimer = null;
    }
    flush();
  } else {
    scheduleFlush();
  }
}

// Re-export for convenience
export type { AnalyticsEventName };

// Flush remaining events on page unload (fallback for desktop).
if (typeof window !== "undefined") {
  window.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "hidden") {
      flushWithTransport(true);
    }
  });
  window.addEventListener("pagehide", () => flushWithTransport(true));
}
