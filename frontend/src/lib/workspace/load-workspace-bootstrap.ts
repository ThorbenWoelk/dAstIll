import type { VideoTypeFilter } from "$lib/types";
import type { ChannelSnapshot, WorkspaceBootstrap } from "$lib/transport-types";
import { setAnalyticsEnabled } from "$lib/analytics/tracker";
import { createApiRequestInit, resolveApiUrl } from "$lib/api/client";
import type {
  AcknowledgedFilter,
  WorkspaceContentMode,
} from "$lib/workspace/types";

const VALID_VIDEO_TYPES = new Set(["long", "short"]);

export type WorkspaceBootstrapPageData = {
  bootstrap: WorkspaceBootstrap | null;
  channelPreviews: Record<string, ChannelSnapshot>;
  channelPreviewsFilterKey: string;
  selectedSourceId: string | null;
  selectedChannelId: string | null;
  selectedItemId: string | null;
  selectedVideoId: string | null;
  contentMode: WorkspaceContentMode | null;
  videoTypeFilter: VideoTypeFilter | null;
  acknowledgedFilter: AcknowledgedFilter | null;
};

export type LoadWorkspaceBootstrapOptions = {
  /**
   * Allows routes with a path-param selected channel (for example
   * `/channels/[id]`) to reuse the shared workspace bootstrap loader.
   */
  selectedChannelIdOverride?: string | null;
};

/**
 * Shared server load for workspace shell routes that use WorkspaceSidebar.
 * See +page.server.ts on the home route for full documentation.
 */
export async function loadWorkspaceBootstrapPageData(
  event: { fetch: typeof fetch; url: URL },
  options?: LoadWorkspaceBootstrapOptions,
): Promise<WorkspaceBootstrapPageData> {
  const { fetch, url } = event;
  const selectedSourceId =
    url.searchParams.get("source") ??
    options?.selectedChannelIdOverride ??
    url.searchParams.get("channel") ??
    null;
  const selectedItemId =
    url.searchParams.get("item") ?? url.searchParams.get("video") ?? null;
  const selectedChannelId = selectedSourceId;
  const selectedVideoId = selectedItemId;
  const typeParam = url.searchParams.get("type");
  const ackParam = url.searchParams.get("ack");
  const fallbackFilterKey = `all:all:default`;

  try {
    const params = new URLSearchParams();
    if (selectedSourceId) {
      params.set("selected_source_id", selectedSourceId);
      params.set("selected_channel_id", selectedSourceId);
    }
    if (selectedItemId) {
      params.set("selected_item_id", selectedItemId);
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

    const response = await fetch(
      resolveApiUrl(`/api/workspace/bootstrap?${params.toString()}`),
      await createApiRequestInit(undefined, {
        includeJsonContentType: false,
      }),
    );

    const previewVideoType =
      typeParam && VALID_VIDEO_TYPES.has(typeParam) ? typeParam : "all";
    const previewAcknowledged =
      ackParam === "ack" ? "ack" : ackParam === "unack" ? "unack" : "all";
    const channelPreviewsFilterKey = `${previewVideoType}:${previewAcknowledged}:default`;

    if (!response.ok) {
      return {
        bootstrap: null,
        channelPreviews: {},
        channelPreviewsFilterKey: fallbackFilterKey,
        selectedSourceId,
        selectedChannelId,
        selectedItemId,
        selectedVideoId,
        contentMode:
          (url.searchParams.get("content") as WorkspaceContentMode) ?? null,
        videoTypeFilter: previewVideoType as VideoTypeFilter,
        acknowledgedFilter: previewAcknowledged as AcknowledgedFilter,
      };
    }
    const bootstrap = (await response.json()) as WorkspaceBootstrap;
    setAnalyticsEnabled(bootstrap.analytics_enabled === true);

    const channelPreviews: Record<string, ChannelSnapshot> = {};
    const snapshot = bootstrap.snapshot;
    if (
      selectedChannelId &&
      snapshot &&
      snapshot.channel_id === selectedChannelId
    ) {
      channelPreviews[selectedChannelId] = snapshot;
    }

    const contentMode =
      (url.searchParams.get("content") as WorkspaceContentMode) ?? null;

    return {
      bootstrap,
      channelPreviews,
      channelPreviewsFilterKey,
      selectedSourceId,
      selectedChannelId,
      selectedItemId,
      selectedVideoId,
      contentMode,
      videoTypeFilter: previewVideoType as VideoTypeFilter,
      acknowledgedFilter: previewAcknowledged as AcknowledgedFilter,
    };
  } catch {
    return {
      bootstrap: null,
      channelPreviews: {},
      channelPreviewsFilterKey: fallbackFilterKey,
      selectedSourceId,
      selectedChannelId,
      selectedItemId,
      selectedVideoId,
      contentMode: null,
      videoTypeFilter: null,
      acknowledgedFilter: null,
    };
  }
}
