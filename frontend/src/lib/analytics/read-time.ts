/**
 * Summary read-time session tracker.
 *
 * Tracks two metrics per summary view:
 *   read_time_ms   - wall-clock elapsed from open to close
 *   active_time_ms - time the user was actively reading (page visible +
 *                    user active within the last IDLE_TIMEOUT_MS)
 *
 * Active time pauses if:
 *   - The page is hidden (visibilitychange to hidden)
 *   - No qualifying activity has occurred for IDLE_TIMEOUT_MS (30 s)
 *
 * Scroll depth (0-100 %) is tracked as max_scroll_depth_pct.
 *
 * Works on desktop and mobile web:
 *   - Uses document.visibilityState for tab/app backgrounding
 *   - Listens to touch events in addition to pointer/keyboard events
 *   - summary_heartbeat events are emitted every HEARTBEAT_INTERVAL_MS
 *     to ensure partial data is captured even if close is unreliable
 */

import { track } from "./tracker";

const IDLE_TIMEOUT_MS = 30_000;
const HEARTBEAT_INTERVAL_MS = 12_000;

/** Uniquely identifies a summary view for deduplication in Databricks Silver. */
export interface SummarySession {
  video_id: string;
  channel_id: string;
  /** Stable string key for this summary; use video_id unless the backend provides a summary id. */
  summary_id: string;
}

interface ActiveSession extends SummarySession {
  openedAt: number;
  /** Wall-clock time already accumulated in active_time_ms before the current active span. */
  accruedActiveMs: number;
  /** When the current active span started, or null if currently paused. */
  activeSpanStart: number | null;
  lastActivityAt: number;
  maxScrollDepthPct: number;
  heartbeatTimer: ReturnType<typeof setInterval> | null;
  idleTimer: ReturnType<typeof setTimeout> | null;
}

let current: ActiveSession | null = null;

// ─── Activity listeners ────────────────────────────────────────────────────

const ACTIVITY_EVENTS: (keyof DocumentEventMap)[] = [
  "scroll",
  "click",
  "keydown",
  "touchstart",
  "touchmove",
  "touchend",
  "pointerdown",
  "pointermove",
  "selectionchange",
];

function onActivity() {
  if (!current) return;
  const now = Date.now();
  current.lastActivityAt = now;

  if (
    current.activeSpanStart === null &&
    document.visibilityState === "visible"
  ) {
    // Resume active span after idle
    current.activeSpanStart = now;
  }

  resetIdleTimer();
}

function onVisibilityChange() {
  if (!current) return;
  if (document.visibilityState === "hidden") {
    pauseActive();
    // Heartbeat-flush so data reaches the server even if close never fires.
    emitHeartbeat();
  } else {
    // Page became visible again; restart activity span if user is active.
    current.lastActivityAt = Date.now();
    current.activeSpanStart = Date.now();
    resetIdleTimer();
  }
}

function attachListeners() {
  for (const name of ACTIVITY_EVENTS) {
    document.addEventListener(name, onActivity, { passive: true });
  }
  document.addEventListener("visibilitychange", onVisibilityChange);
}

function detachListeners() {
  for (const name of ACTIVITY_EVENTS) {
    document.removeEventListener(name, onActivity);
  }
  document.removeEventListener("visibilitychange", onVisibilityChange);
}

// ─── Scroll depth ──────────────────────────────────────────────────────────

export function updateScrollDepth(scrollEl: Element | null) {
  if (!current || !scrollEl) return;
  const { scrollTop, scrollHeight, clientHeight } = scrollEl as HTMLElement;
  const scrollable = scrollHeight - clientHeight;
  if (scrollable <= 0) return;
  const pct = Math.min(100, Math.round((scrollTop / scrollable) * 100));
  if (pct > current.maxScrollDepthPct) {
    current.maxScrollDepthPct = pct;
  }
}

// ─── Idle management ──────────────────────────────────────────────────────

function resetIdleTimer() {
  if (!current) return;
  if (current.idleTimer !== null) clearTimeout(current.idleTimer);
  current.idleTimer = setTimeout(onIdle, IDLE_TIMEOUT_MS);
}

function onIdle() {
  if (!current) return;
  pauseActive();
}

// ─── Active span helpers ──────────────────────────────────────────────────

function pauseActive() {
  if (!current || current.activeSpanStart === null) return;
  current.accruedActiveMs += Date.now() - current.activeSpanStart;
  current.activeSpanStart = null;
}

function computeActiveMs(): number {
  if (!current) return 0;
  const running =
    current.activeSpanStart !== null ? Date.now() - current.activeSpanStart : 0;
  return current.accruedActiveMs + running;
}

// ─── Heartbeat ────────────────────────────────────────────────────────────

function emitHeartbeat() {
  if (!current) return;
  track({
    event: "summary_heartbeat",
    video_id: current.video_id,
    channel_id: current.channel_id,
    summary_id: current.summary_id,
    active_time_ms: computeActiveMs(),
    max_scroll_depth_pct: current.maxScrollDepthPct,
  });
}

// ─── Public API ──────────────────────────────────────────────────────────

/** Start a new summary session. Closes any existing session first. */
export function openSummarySession(session: SummarySession) {
  if (typeof window === "undefined") return;

  // Close previous session if open.
  if (current) {
    closeSummarySession();
  }

  const now = Date.now();
  current = {
    ...session,
    openedAt: now,
    accruedActiveMs: 0,
    activeSpanStart: now,
    lastActivityAt: now,
    maxScrollDepthPct: 0,
    heartbeatTimer: setInterval(emitHeartbeat, HEARTBEAT_INTERVAL_MS),
    idleTimer: setTimeout(onIdle, IDLE_TIMEOUT_MS),
  };

  attachListeners();

  track({
    event: "summary_opened",
    video_id: session.video_id,
    channel_id: session.channel_id,
    summary_id: session.summary_id,
  });
}

/** Close the active summary session and emit summary_closed. */
export function closeSummarySession() {
  if (!current) return;

  const snap = current;
  current = null;

  detachListeners();

  if (snap.heartbeatTimer !== null) clearInterval(snap.heartbeatTimer);
  if (snap.idleTimer !== null) clearTimeout(snap.idleTimer);

  // Finalise active span.
  const activeMs =
    snap.accruedActiveMs +
    (snap.activeSpanStart !== null ? Date.now() - snap.activeSpanStart : 0);

  track({
    event: "summary_closed",
    video_id: snap.video_id,
    channel_id: snap.channel_id,
    summary_id: snap.summary_id,
    read_time_ms: Date.now() - snap.openedAt,
    active_time_ms: activeMs,
    max_scroll_depth_pct: snap.maxScrollDepthPct,
  });
}

/** Returns true if there is an open session for the given summary. */
export function isSummarySessionOpen(
  videoId: string,
  summaryId: string,
): boolean {
  return (
    current !== null &&
    current.video_id === videoId &&
    current.summary_id === summaryId
  );
}
