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

type MockWorkspaceBootstrap = ReturnType<typeof buildMockWorkspaceBootstrap>;
type MockWorkspaceSnapshot = MockWorkspaceBootstrap["snapshot"];
type MockWorkspaceVideo = MockWorkspaceSnapshot["videos"][number];
type MockWorkspaceSummary = {
  video_id: string;
  content: string;
  model_used: string | null;
  quality_score: number | null;
  quality_note: string | null;
  quality_model_used: string | null;
  summary_tags: string[];
  summary_tags_evaluated: boolean;
};
type MockWorkspaceTranscript = {
  video_id: string;
  raw_text: string | null;
  formatted_markdown: string | null;
  render_mode: "plain_text" | "markdown";
};
type MockWorkspaceVideoInfo = {
  video_id: string;
  watch_url: string;
  title: string;
  description: string | null;
  thumbnail_url: null;
  channel_name: string | null;
  channel_id: string | null;
  published_at: string | null;
  duration_iso8601: string | null;
  duration_seconds: number | null;
  view_count: number | null;
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
    bootstrap: MockWorkspaceBootstrap;
    snapshots?: Record<string, MockWorkspaceSnapshot>;
    summaries?: Record<string, MockWorkspaceSummary>;
    transcripts?: Record<string, MockWorkspaceTranscript>;
    videoInfos?: Record<string, MockWorkspaceVideoInfo>;
    acknowledgedVideos?: Record<string, MockWorkspaceVideo>;
    summary?: MockWorkspaceSummary;
    videoInfo?: MockWorkspaceVideoInfo;
  },
) {
  const { bootstrap, summary, videoInfo } = options;
  const channelId = bootstrap.selected_channel_id;
  const snapshots = {
    [channelId]: bootstrap.snapshot,
    ...(options.snapshots ?? {}),
  };
  const summaries = {
    ...(summary ? { [summary.video_id]: summary } : {}),
    ...(options.summaries ?? {}),
  };
  const transcripts = options.transcripts ?? {};
  const videoInfos = {
    ...(videoInfo ? { [videoInfo.video_id]: videoInfo } : {}),
    ...(options.videoInfos ?? {}),
  };
  const acknowledgedVideos = options.acknowledgedVideos ?? {};

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

    const snapshotMatch = url.pathname.match(
      /^\/api\/channels\/([^/]+)\/snapshot$/,
    );
    if (snapshotMatch) {
      const snapshot = snapshots[snapshotMatch[1]];
      if (!snapshot) {
        await route.fulfill({ status: 404, body: "snapshot not found" });
        return;
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(snapshot),
      });
      return;
    }

    const videosMatch = url.pathname.match(
      /^\/api\/channels\/([^/]+)\/videos$/,
    );
    if (videosMatch) {
      const requestedChannelId = videosMatch[1];
      const snapshot = snapshots[requestedChannelId];
      if (!snapshot) {
        await route.fulfill({ status: 404, body: "videos not found" });
        return;
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          source_id: requestedChannelId,
          videos: snapshot.videos,
          items: snapshot.items,
          parts: snapshot.parts,
          has_more: false,
          next_offset: null,
        }),
      });
      return;
    }

    const syncDepthMatch = url.pathname.match(
      /^\/api\/channels\/([^/]+)\/sync-depth$/,
    );
    if (syncDepthMatch) {
      const snapshot = snapshots[syncDepthMatch[1]];
      if (!snapshot) {
        await route.fulfill({ status: 404, body: "sync depth not found" });
        return;
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(snapshot.sync_depth),
      });
      return;
    }

    if (/^\/api\/channels\/[^/]+\/backfill$/.test(url.pathname)) {
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

    const summaryMatch = url.pathname.match(
      /^\/api\/videos\/([^/]+)\/summary$/,
    );
    if (summaryMatch) {
      const requestedSummary = summaries[summaryMatch[1]];
      if (!requestedSummary) {
        await route.fulfill({ status: 404, body: "Summary not found" });
        return;
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(requestedSummary),
      });
      return;
    }

    if (/^\/api\/videos\/[^/]+\/summary\/audio\/debug$/.test(url.pathname)) {
      await route.fulfill({
        status: 404,
        contentType: "text/plain",
        body: "audio not generated",
      });
      return;
    }

    const acknowledgedMatch = url.pathname.match(
      /^\/api\/videos\/([^/]+)\/acknowledged$/,
    );
    if (acknowledgedMatch) {
      const video = acknowledgedVideos[acknowledgedMatch[1]];
      if (video) {
        const payload = route.request().postDataJSON() as {
          acknowledged?: boolean;
        };
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({
            ...video,
            acknowledged: payload.acknowledged ?? video.acknowledged,
          }),
        });
        return;
      }

      await route.fulfill({
        status: 403,
        contentType: "text/plain",
        body: "Sign-in required",
      });
      return;
    }

    const transcriptMatch = url.pathname.match(
      /^\/api\/videos\/([^/]+)\/transcript(?:\/ensure)?$/,
    );
    if (transcriptMatch) {
      const transcript = transcripts[transcriptMatch[1]];
      if (!transcript) {
        await route.fulfill({ status: 404, body: "Transcript not found" });
        return;
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(transcript),
      });
      return;
    }

    const videoInfoMatch = url.pathname.match(
      /^\/api\/videos\/([^/]+)\/info\/ensure$/,
    );
    if (videoInfoMatch) {
      const requestedVideoInfo = videoInfos[videoInfoMatch[1]];
      if (!requestedVideoInfo) {
        await route.fulfill({ status: 404, body: "video info not found" });
        return;
      }
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(requestedVideoInfo),
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
