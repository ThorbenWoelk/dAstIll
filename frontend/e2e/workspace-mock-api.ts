import type { Page } from "@playwright/test";

export type MockWorkspaceBootstrapOptions = {
  channelId: string;
  channelName: string;
  channelHandle: string;
  containerId: string;
  videoId: string;
  videoTitle: string;
  qualityScore: number;
  selectedItemId?: string;
  totalChunkCount?: number;
};

export function buildMockWorkspaceBootstrap(
  options: MockWorkspaceBootstrapOptions,
) {
  const {
    channelId,
    channelName,
    channelHandle,
    containerId,
    videoId,
    videoTitle,
    qualityScore,
    selectedItemId,
    totalChunkCount = 4,
  } = options;

  const source = {
    id: channelId,
    provider: "you_tube" as const,
    source_kind: "you_tube_channel" as const,
    container_id: containerId,
    container_kind: "series" as const,
    backing_kind: "feed" as const,
    title: channelName,
    subtitle: channelHandle,
    handle: channelHandle,
    thumbnail_url: null,
    requires_auth: false,
    public_content_available: true,
    entitled_content_available: true,
    external_ids: [{ provider: "you_tube" as const, external_id: channelId }],
  };

  const syncDepth = {
    earliest_sync_date: "2026-04-01T00:00:00.000Z",
    earliest_sync_date_user_set: false,
    derived_earliest_ready_date: "2026-04-10T00:00:00.000Z",
  };

  return {
    ai_available: true,
    ai_status: "cloud" as const,
    containers: [
      {
        id: containerId,
        kind: "series" as const,
        title: `${channelName} container`,
        provider: "you_tube" as const,
        backing_kind: "feed" as const,
        user_editable: true,
        source_ids: [channelId],
      },
    ],
    sources: [source],
    channels: [
      {
        id: channelId,
        handle: channelHandle,
        name: channelName,
        thumbnail_url: null,
        added_at: "2026-04-12T09:00:00.000Z",
        earliest_sync_date: syncDepth.earliest_sync_date,
        earliest_sync_date_user_set: false,
      },
    ],
    selected_source_id: channelId,
    selected_channel_id: channelId,
    ...(selectedItemId ? { selected_item_id: selectedItemId } : {}),
    snapshot: {
      channel_id: channelId,
      source_id: channelId,
      container: {
        id: containerId,
        kind: "series" as const,
        title: `${channelName} container`,
        provider: "you_tube" as const,
        backing_kind: "feed" as const,
        user_editable: true,
        source_ids: [channelId],
      },
      source,
      sync_depth: syncDepth,
      channel_video_count: 1,
      has_more: false,
      next_offset: null,
      videos: [
        {
          id: videoId,
          channel_id: channelId,
          title: videoTitle,
          thumbnail_url: null,
          published_at: "2026-04-11T18:30:00.000Z",
          is_short: false,
          transcript_status: "ready" as const,
          summary_status: "ready" as const,
          acknowledged: false,
          retry_count: 0,
          quality_score: qualityScore,
        },
      ],
      items: [],
      parts: [],
    },
    search_status: {
      available: true,
      model: "embeddinggemma",
      dimensions: 768,
      pending: 0,
      indexing: 0,
      ready: 1,
      failed: 0,
      total_sources: 1,
      total_chunk_count: totalChunkCount,
      embedded_chunk_count: totalChunkCount,
      vector_index_ready: true,
      retrieval_mode: "hybrid_ann",
    },
  };
}

export async function installMockWorkspaceApi(
  page: Page,
  options: {
    bootstrap: ReturnType<typeof buildMockWorkspaceBootstrap>;
    summary?: {
      video_id: string;
      content: string;
      model_used: string;
      quality_score: number;
      quality_note: string;
      quality_model_used: string;
      summary_tags: string[];
      summary_tags_evaluated: boolean;
    };
    videoInfo?: {
      video_id: string;
      watch_url: string;
      title: string;
      description: string;
      thumbnail_url: null;
      channel_name: string;
      channel_id: string;
      published_at: string;
      duration_iso8601: string;
      duration_seconds: number;
      view_count: number;
    };
  },
) {
  const { bootstrap, summary, videoInfo } = options;
  const channelId = bootstrap.selected_channel_id;

  await page.route("**/api/**", async (route) => {
    const url = new URL(route.request().url());

    if (url.pathname === "/api/workspace/bootstrap") {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(bootstrap),
      });
      return;
    }

    if (url.pathname === `/api/channels/${channelId}/snapshot`) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(bootstrap.snapshot),
      });
      return;
    }

    if (url.pathname === `/api/channels/${channelId}/videos`) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          source_id: channelId,
          videos: [],
          items: [],
          parts: [],
          has_more: false,
          next_offset: null,
        }),
      });
      return;
    }

    if (url.pathname === `/api/channels/${channelId}/sync-depth`) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(bootstrap.snapshot.sync_depth),
      });
      return;
    }

    if (url.pathname === `/api/channels/${channelId}/backfill`) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          videos_added: 0,
          fetched_count: 0,
          exhausted: true,
        }),
      });
      return;
    }

    if (summary && url.pathname === `/api/videos/${summary.video_id}/summary`) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(summary),
      });
      return;
    }

    if (
      summary &&
      url.pathname === `/api/videos/${summary.video_id}/summary/audio/debug`
    ) {
      await route.fulfill({
        status: 404,
        contentType: "text/plain",
        body: "audio not generated",
      });
      return;
    }

    if (
      videoInfo &&
      url.pathname === `/api/videos/${videoInfo.video_id}/info/ensure`
    ) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(videoInfo),
      });
      return;
    }

    await route.continue();
  });
}

export async function navigateViaInjectedLink(page: Page, href: string) {
  await page.evaluate(
    ({ nextHref }) => {
      const existing = document.getElementById("__test-route");
      existing?.remove();
      const link = document.createElement("a");
      link.id = "__test-route";
      link.href = nextHref;
      link.textContent = "route";
      document.body.appendChild(link);
    },
    { nextHref: href },
  );

  await page.evaluate(() => {
    const link = document.getElementById("__test-route");
    if (!(link instanceof HTMLAnchorElement)) {
      throw new Error("Client navigation link was not mounted");
    }
    link.click();
  });
}
