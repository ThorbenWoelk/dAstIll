import type { ChannelSnapshot, QueueTab, WorkspaceBootstrap } from "$lib/types";

const VALID_VIDEO_TYPES = new Set(["long", "short"]);

const QUEUE_TAB_VALUES = new Set<string>([
  "transcripts",
  "summaries",
  "evaluations",
]);

export type WorkspaceBootstrapPageData = {
  bootstrap: WorkspaceBootstrap | null;
  channelPreviews: Record<string, ChannelSnapshot>;
  channelPreviewsFilterKey: string;
};

export type LoadWorkspaceBootstrapOptions = {
  /**
   * Allows routes with a path-param selected channel (for example
   * `/channels/[id]`) to reuse the shared workspace bootstrap loader.
   */
  selectedChannelIdOverride?: string | null;
  /**
   * For `/download-queue`: send `queue_tab` on bootstrap and snapshot requests
   * so sidebar lists match the queue API (videos still processing transcripts).
   * Used as the tab when the URL has no `queue` query (aligns with client default).
   */
  ssrQueueTabDefault?: QueueTab;
  /**
   * Unified download-queue view: `queue_only` without `queue_tab` (any incomplete
   * transcript or summary). Mutually exclusive with `ssrQueueTabDefault` for SSR.
   */
  ssrQueueUnified?: boolean;
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
  const selectedChannelId =
    options?.selectedChannelIdOverride ??
    url.searchParams.get("channel") ??
    null;
  const typeParam = url.searchParams.get("type");
  const ackParam = url.searchParams.get("ack");
  const unified = options?.ssrQueueUnified === true;
  const effectiveQueueTab = unified
    ? undefined
    : parseQueueTabFromUrl(url, options?.ssrQueueTabDefault);
  const queueSegment = unified
    ? "unified"
    : queueSegmentForFilterKey(effectiveQueueTab);
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

    if (unified) {
      params.set("queue_only", "true");
    } else if (effectiveQueueTab) {
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
    if (
      selectedChannelId &&
      bootstrap.snapshot &&
      bootstrap.snapshot.channel_id === selectedChannelId
    ) {
      channelPreviews[selectedChannelId] = bootstrap.snapshot;
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
