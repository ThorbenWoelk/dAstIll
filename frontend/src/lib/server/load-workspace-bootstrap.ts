import type { ChannelSnapshot, QueueTab, WorkspaceBootstrap } from "$lib/types";

const VALID_VIDEO_TYPES = new Set(["long", "short"]);

const QUEUE_TAB_VALUES = new Set<string>([
  "transcripts",
  "summaries",
  "evaluations",
]);

/**
 * Number of videos to pre-load per channel for the sidebar preview.
 * Must match WorkspaceSidebar's PREVIEW_FETCH_LIMIT constant
 * (PREVIEW_VISIBLE_VIDEO_COUNT + 1 = 5 + 1 = 6).
 */
const CHANNEL_PREVIEW_LIMIT = 6;

export type WorkspaceBootstrapPageData = {
  bootstrap: WorkspaceBootstrap | null;
  channelPreviews: Record<string, ChannelSnapshot>;
  channelPreviewsFilterKey: string;
};

export type LoadWorkspaceBootstrapOptions = {
  /**
   * For `/download-queue`: send `queue_tab` on bootstrap and snapshot requests
   * so sidebar lists match the queue API (videos still processing transcripts).
   * Used as the tab when the URL has no `queue` query (aligns with client default).
   */
  ssrQueueTabDefault?: QueueTab;
};

function parseQueueTabFromUrl(
  url: URL,
  fallback: QueueTab | undefined,
): QueueTab | undefined {
  const raw = url.searchParams.get("queue")?.trim();
  if (raw && QUEUE_TAB_VALUES.has(raw)) {
    return raw as QueueTab;
  }
  return fallback;
}

function queueSegmentForFilterKey(tab: QueueTab | undefined): string {
  return tab ?? "default";
}

/**
 * Shared server load for workspace shell routes that use WorkspaceSidebar
 * (main workspace and download queue). See +page.server.ts on the home route
 * for full documentation (VAL-DATA-001, VAL-DATA-002).
 */
export async function loadWorkspaceBootstrapPageData(
  event: { fetch: typeof fetch; url: URL },
  options?: LoadWorkspaceBootstrapOptions,
): Promise<WorkspaceBootstrapPageData> {
  const { fetch, url } = event;
  const selectedChannelId = url.searchParams.get("channel") ?? null;
  const typeParam = url.searchParams.get("type");
  const ackParam = url.searchParams.get("ack");
  const effectiveQueueTab = parseQueueTabFromUrl(
    url,
    options?.ssrQueueTabDefault,
  );
  const queueSegment = queueSegmentForFilterKey(effectiveQueueTab);
  const fallbackFilterKey = `all:all:${queueSegment}`;

  try {
    const params = new URLSearchParams();
    if (selectedChannelId) {
      params.set("selected_channel_id", selectedChannelId);
    }
    params.set("limit", "20");

    if (typeParam && VALID_VIDEO_TYPES.has(typeParam)) {
      params.set("video_type", typeParam);
    }

    if (ackParam === "ack") {
      params.set("acknowledged", "true");
    } else if (ackParam === "unack") {
      params.set("acknowledged", "false");
    }

    if (effectiveQueueTab) {
      params.set("queue_tab", effectiveQueueTab);
    }

    const response = await fetch(
      `/api/workspace/bootstrap?${params.toString()}`,
    );

    if (!response.ok) {
      return {
        bootstrap: null,
        channelPreviews: {},
        channelPreviewsFilterKey: fallbackFilterKey,
      };
    }

    const bootstrap = (await response.json()) as WorkspaceBootstrap;

    const channelPreviews: Record<string, ChannelSnapshot> = {};

    if (bootstrap.channels.length > 0) {
      const previewParams = new URLSearchParams();
      previewParams.set("limit", `${CHANNEL_PREVIEW_LIMIT}`);
      previewParams.set("offset", "0");
      if (typeParam && VALID_VIDEO_TYPES.has(typeParam)) {
        previewParams.set("video_type", typeParam);
      }
      if (ackParam === "ack") {
        previewParams.set("acknowledged", "true");
      } else if (ackParam === "unack") {
        previewParams.set("acknowledged", "false");
      }
      if (effectiveQueueTab) {
        previewParams.set("queue_tab", effectiveQueueTab);
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

    const previewVideoType =
      typeParam && VALID_VIDEO_TYPES.has(typeParam) ? typeParam : "all";
    const previewAcknowledged =
      ackParam === "ack" ? "ack" : ackParam === "unack" ? "unack" : "all";
    const channelPreviewsFilterKey = `${previewVideoType}:${previewAcknowledged}:${queueSegment}`;

    return { bootstrap, channelPreviews, channelPreviewsFilterKey };
  } catch {
    return {
      bootstrap: null,
      channelPreviews: {},
      channelPreviewsFilterKey: fallbackFilterKey,
    };
  }
}
