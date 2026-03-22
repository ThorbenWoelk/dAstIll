import type { PageServerLoad } from "./$types";
import type { ChannelSnapshot, WorkspaceBootstrap } from "$lib/types";

const VALID_VIDEO_TYPES = new Set(["long", "short"]);

/**
 * Number of videos to pre-load per channel for the sidebar preview.
 * Must match WorkspaceSidebar's PREVIEW_FETCH_LIMIT constant
 * (PREVIEW_VISIBLE_VIDEO_COUNT + 1 = 5 + 1 = 6).
 */
const CHANNEL_PREVIEW_LIMIT = 6;

/**
 * Server-side load function for the workspace root route.
 *
 * Fetches the consolidated workspace bootstrap data (channels, AI status,
 * search status, and initial channel snapshot) in a single server-side
 * request so the initial HTML response contains meaningful workspace state
 * before JavaScript hydrates (VAL-DATA-001, VAL-DATA-002).
 *
 * Also pre-fetches channel preview snapshots for all channels in parallel
 * (server-side) so the WorkspaceSidebar can display video previews without
 * making additional client-side API calls. This eliminates the N per-channel
 * snapshot fetches that previously happened on initial mount (VAL-DATA-002).
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
 * Returns { bootstrap: null, channelPreviews: {} } on any error so the
 * page falls back to the IndexedDB warm-start path in onMount.
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
      return { bootstrap: null, channelPreviews: {} };
    }

    const bootstrap = (await response.json()) as WorkspaceBootstrap;

    // Pre-fetch channel preview snapshots for all channels in parallel
    // (server-side, benefits from backend in-memory cache). This avoids N
    // separate client-side snapshot fetches when the WorkspaceSidebar mounts
    // in per_channel_preview mode, reducing initial browser API requests to
    // at most the background bootstrap refresh + AI health check (VAL-DATA-002).
    const channelPreviews: Record<string, ChannelSnapshot> = {};

    if (bootstrap.channels.length > 0) {
      const previewParams = new URLSearchParams();
      previewParams.set("limit", `${CHANNEL_PREVIEW_LIMIT}`);
      previewParams.set("offset", "0");
      // Forward same filter params so preview data matches initial filter state
      if (typeParam && VALID_VIDEO_TYPES.has(typeParam)) {
        previewParams.set("video_type", typeParam);
      }
      if (ackParam === "ack") {
        previewParams.set("acknowledged", "true");
      } else if (ackParam === "unack") {
        previewParams.set("acknowledged", "false");
      }

      const snapshotResults = await Promise.allSettled(
        bootstrap.channels.map((channel) =>
          fetch(
            `/api/channels/${channel.id}/snapshot?${previewParams.toString()}`,
          ).then((r) =>
            r.ok
              ? (r.json() as Promise<ChannelSnapshot>)
              : Promise.reject(new Error(`${r.status}`)),
          ),
        ),
      );

      bootstrap.channels.forEach((channel, i) => {
        const result = snapshotResults[i];
        if (result.status === "fulfilled") {
          channelPreviews[channel.id] = result.value;
        }
      });
    }

    // Build the filter key that identifies which filter the channel preview
    // snapshots were fetched with. The sidebar uses this to match pre-loaded
    // data to the current filter state (avoids using stale data when the
    // client's filter differs from the server's filter).
    const previewVideoType =
      typeParam && VALID_VIDEO_TYPES.has(typeParam) ? typeParam : "all";
    const previewAcknowledged =
      ackParam === "ack" ? "ack" : ackParam === "unack" ? "unack" : "all";
    const channelPreviewsFilterKey = `${previewVideoType}:${previewAcknowledged}`;

    return { bootstrap, channelPreviews, channelPreviewsFilterKey };
  } catch {
    return {
      bootstrap: null,
      channelPreviews: {},
      channelPreviewsFilterKey: "all:all",
    };
  }
};
