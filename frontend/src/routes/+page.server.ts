import type { PageServerLoad } from "./$types";
import type { WorkspaceBootstrap } from "$lib/types";

const VALID_VIDEO_TYPES = new Set(["long", "short"]);

/**
 * Server-side load function for the workspace root route.
 *
 * Fetches the consolidated workspace bootstrap data (channels, AI status,
 * search status, and initial channel snapshot) in a single server-side
 * request so the initial HTML response contains meaningful workspace state
 * before JavaScript hydrates (VAL-DATA-001, VAL-DATA-002).
 *
 * Forwards URL query parameters for type and ack filters so the
 * server-rendered state matches the URL-specified filter state on first
 * paint. Only non-default filter values are forwarded:
 *   - type=short|long → video_type=short|long  (type=all → omitted)
 *   - ack=ack|unack   → acknowledged=true|false (ack=all → omitted)
 *
 * The server-side fetch goes through the existing /api/[...path] proxy,
 * which forwards auth headers and the operator role from locals.
 *
 * Returns { bootstrap: null } on any error so the page falls back to
 * the IndexedDB warm-start path in onMount.
 */
export const load: PageServerLoad = async ({ fetch, url }) => {
  const selectedChannelId = url.searchParams.get("channel") ?? null;
  const typeParam = url.searchParams.get("type");
  const ackParam = url.searchParams.get("ack");

  try {
    const params = new URLSearchParams();
    if (selectedChannelId) {
      params.set("selected_channel_id", selectedChannelId);
    }
    // Match the page-level limit constant (20 videos per snapshot page)
    params.set("limit", "20");

    // Forward type filter: only non-default values ("long" or "short")
    if (typeParam && VALID_VIDEO_TYPES.has(typeParam)) {
      params.set("video_type", typeParam);
    }

    // Forward acknowledged filter: "ack" → true, "unack" → false, "all" → omit
    if (ackParam === "ack") {
      params.set("acknowledged", "true");
    } else if (ackParam === "unack") {
      params.set("acknowledged", "false");
    }

    const response = await fetch(
      `/api/workspace/bootstrap?${params.toString()}`,
    );

    if (!response.ok) {
      return { bootstrap: null };
    }

    const bootstrap = (await response.json()) as WorkspaceBootstrap;
    return { bootstrap };
  } catch {
    return { bootstrap: null };
  }
};
