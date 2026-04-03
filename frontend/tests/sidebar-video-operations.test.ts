import { describe, expect, it } from "bun:test";
import type { SvelteMap } from "svelte/reactivity";

import { createSidebarVideoOperations } from "../src/lib/workspace/sidebar-video-operations";
import type { ChannelSnapshot, Video, VideoTypeFilter } from "../src/lib/types";
import type { AcknowledgedFilter } from "../src/lib/workspace/types";

function makeVideo(id: string, channelId = "channel-1"): Video {
  return {
    id,
    channel_id: channelId,
    title: `Video ${id}`,
    thumbnail_url: null,
    published_at: "2024-01-01T00:00:00Z",
    is_short: false,
    transcript_status: "ready",
    summary_status: "ready",
    acknowledged: false,
  } as Video;
}

function makeSnapshot(videos: Video[]): ChannelSnapshot {
  return {
    channel_id: "channel-1",
    channel_video_count: videos.length,
    has_more: false,
    next_offset: videos.length,
    sync_depth: null,
    videos,
  } as ChannelSnapshot;
}

function createHarness(options?: {
  selectedVideoId?: string | null;
  videos?: Video[];
  onListVideos?: (selectedChannelId: string) => Promise<{
    videos: Video[];
    has_more: boolean;
    next_offset: number | null;
  }>;
  onLoadChannelSnapshot?: (
    selectedChannelId: string,
  ) => Promise<ChannelSnapshot>;
  onRefreshChannel?: (
    selectedChannelId: string,
  ) => Promise<{ videos_added: number }>;
}) {
  let selectedChannelId: string | null = "channel-1";
  let selectedVideoId = options?.selectedVideoId ?? "video-2";
  let videos = options?.videos ?? [makeVideo("video-1"), makeVideo("video-2")];
  let offset = videos.length;
  let hasMore = false;
  let videoLoading = false;
  let refreshing = false;
  let videoTypeFilter: VideoTypeFilter = "all";
  let acknowledgedFilter: AcknowledgedFilter = "all";
  let resetCalls = 0;

  const operations = createSidebarVideoOperations({
    options: {
      onSelectVideo: () => {},
      onListVideos: async (channelId) =>
        options?.onListVideos?.(channelId) ?? {
          videos,
          has_more: false,
          next_offset: videos.length,
        },
      onLoadChannelSnapshot: async (channelId) =>
        options?.onLoadChannelSnapshot?.(channelId) ?? makeSnapshot(videos),
      onRefreshChannel: async (channelId) =>
        options?.onRefreshChannel?.(channelId) ?? { videos_added: 0 },
    },
    limit: 20,
    channelLastRefreshedAt: new Map() as unknown as SvelteMap<string, number>,
    videoStateCache: {
      get: () => null,
      delete: () => {},
    },
    getVideoStateKey: () => "queue:channel-1",
    getChannelOrder: () => [],
    getSelectedChannelId: () => selectedChannelId,
    getSelectedVideoId: () => selectedVideoId,
    getVideos: () => videos,
    getOffset: () => offset,
    getVideoTypeFilter: () => videoTypeFilter,
    getAcknowledgedFilter: () => acknowledgedFilter,
    getLoadingVideos: () => videoLoading,
    getVideoListMutationEpoch: () => 0,
    applyLoadedChannelsState: () => {},
    applySelectionState: (selection) => {
      if ("selectedChannelId" in selection) {
        selectedChannelId = selection.selectedChannelId ?? null;
      }
      if ("selectedVideoId" in selection) {
        selectedVideoId = selection.selectedVideoId ?? null;
      }
    },
    clearChannelSelectionState: () => {
      selectedChannelId = null;
      selectedVideoId = null;
      videos = [];
      offset = 0;
    },
    resetVideoListState: () => {
      resetCalls += 1;
      videos = [];
      offset = 0;
      hasMore = true;
    },
    applyChannelSnapshotState: (snapshot) => {
      videos = snapshot.videos;
      offset = snapshot.next_offset ?? snapshot.videos.length;
      hasMore = snapshot.has_more;
    },
    applyVideoPageState: (page, applyOptions) => {
      videos = applyOptions?.reset ? page.videos : [...videos, ...page.videos];
      offset = page.next_offset ?? videos.length;
      hasMore = page.has_more;
    },
    setChannelLoadingState: () => {},
    setVideoLoadingState: (value) => {
      videoLoading = value;
    },
    setRefreshingChannelState: (value) => {
      refreshing = value;
    },
    setVideos: (next) => {
      videos = next;
    },
    setVideoTypeFilter: (value) => {
      videoTypeFilter = value;
    },
    setAcknowledgedFilter: (value) => {
      acknowledgedFilter = value;
    },
  });

  return {
    operations,
    getSelectedVideoId: () => selectedVideoId,
    getVideos: () => videos,
    getResetCalls: () => resetCalls,
    getRefreshing: () => refreshing,
    getHasMore: () => hasMore,
  };
}

describe("createSidebarVideoOperations.reloadSelectedChannelVideos", () => {
  it("clears a stale selected video after a reset reload removes it from scope", async () => {
    const harness = createHarness({
      selectedVideoId: "video-2",
      videos: [makeVideo("video-1"), makeVideo("video-2")],
      onListVideos: async () => ({
        videos: [makeVideo("video-1")],
        has_more: false,
        next_offset: 1,
      }),
    });

    await harness.operations.reloadSelectedChannelVideos({
      reset: true,
      silent: true,
      clearMissingSelectedVideo: true,
    });

    expect(harness.getVideos().map((video) => video.id)).toEqual(["video-1"]);
    expect(harness.getSelectedVideoId()).toBeNull();
  });

  it("preserves selection when the reset reload still includes the selected video", async () => {
    const harness = createHarness({
      selectedVideoId: "video-2",
      videos: [makeVideo("video-1"), makeVideo("video-2")],
      onListVideos: async () => ({
        videos: [makeVideo("video-1"), makeVideo("video-2")],
        has_more: false,
        next_offset: 2,
      }),
    });

    await harness.operations.reloadSelectedChannelVideos({
      reset: true,
      silent: true,
      clearMissingSelectedVideo: true,
    });

    expect(harness.getSelectedVideoId()).toBe("video-2");
  });

  it("resets cached rows before a refresh-backed reload", async () => {
    const harness = createHarness({
      selectedVideoId: "video-2",
      videos: [makeVideo("video-1"), makeVideo("video-2")],
      onLoadChannelSnapshot: async () => makeSnapshot([makeVideo("video-3")]),
    });

    await harness.operations.reloadSelectedChannelVideos({
      reset: true,
      refresh: true,
      silent: true,
    });

    expect(harness.getResetCalls()).toBe(1);
    expect(harness.getVideos().map((video) => video.id)).toEqual(["video-3"]);
    expect(harness.getHasMore()).toBe(false);
    expect(harness.getRefreshing()).toBe(false);
  });
});
