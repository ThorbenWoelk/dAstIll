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

function makeSnapshot(
  videos: Video[],
  channelId = "channel-1",
): ChannelSnapshot {
  return {
    channel_id: channelId,
    channel_video_count: videos.length,
    has_more: false,
    next_offset: videos.length,
    sync_depth: null,
    videos,
  } as ChannelSnapshot;
}

function createHarness(options?: {
  selectedChannelId?: string | null;
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
  let selectedChannelId: string | null =
    options?.selectedChannelId ?? "channel-1";
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
    resetVideoListState: (state) => {
      resetCalls += 1;
      if (state && "selectedVideoId" in state) {
        selectedVideoId = state?.selectedVideoId ?? null;
      }
      videos = state?.videos ?? [];
      offset = state?.offset ?? 0;
      hasMore = state?.hasMore ?? true;
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
    setSyncDepthState: () => {},
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
    getSelectedChannelId: () => selectedChannelId,
    getSelectedVideoId: () => selectedVideoId,
    getVideos: () => videos,
    getResetCalls: () => resetCalls,
    getRefreshing: () => refreshing,
    getVideoLoading: () => videoLoading,
    getHasMore: () => hasMore,
  };
}

function createDeferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

describe("createSidebarVideoOperations.selectChannelVideoOptimistically", () => {
  it("selects and seeds the clicked cross-channel video before snapshot hydration finishes", async () => {
    const deferredSnapshot = createDeferred<ChannelSnapshot>();
    let snapshotRequests = 0;
    const clickedVideo = makeVideo("video-99", "channel-2");
    const harness = createHarness({
      selectedChannelId: "channel-1",
      selectedVideoId: "video-1",
      videos: [makeVideo("video-1", "channel-1")],
      onLoadChannelSnapshot: async () => {
        snapshotRequests += 1;
        return deferredSnapshot.promise;
      },
    });

    const hydration = harness.operations.selectChannelVideoOptimistically(
      "channel-2",
      clickedVideo.id,
      clickedVideo,
    );

    expect(harness.getSelectedChannelId()).toBe("channel-2");
    expect(harness.getSelectedVideoId()).toBe("video-99");
    expect(harness.getVideos().map((video) => video.id)).toEqual(["video-99"]);
    expect(harness.getVideoLoading()).toBe(true);
    expect(snapshotRequests).toBe(1);

    deferredSnapshot.resolve(
      makeSnapshot([makeVideo("video-2", "channel-2")], "channel-2"),
    );
    await hydration;

    expect(harness.getVideos().map((video) => video.id)).toEqual(["video-2"]);
    expect(harness.getVideoLoading()).toBe(false);
  });

  it("merges the clicked video hint into cached channel state without waiting for hydration", async () => {
    const cachedVideo = makeVideo("video-2", "channel-2");
    const clickedVideo = makeVideo("video-99", "channel-2");
    const deferredSnapshot = createDeferred<ChannelSnapshot>();
    let selectedChannelId: string | null = "channel-1";
    let selectedVideoId: string | null = "video-1";
    let videos = [makeVideo("video-1", "channel-1")];
    let videoLoading = false;

    const operations = createSidebarVideoOperations({
      options: {
        onSelectVideo: () => {},
        onLoadChannelSnapshot: async () => deferredSnapshot.promise,
        onRefreshChannel: async () => ({ videos_added: 0 }),
      },
      limit: 20,
      channelLastRefreshedAt: new Map() as unknown as SvelteMap<string, number>,
      videoStateCache: {
        get: () => ({
          videos: [cachedVideo],
          offset: 1,
          hasMore: true,
          lastSyncedAt: null,
          syncDepth: null,
        }),
        delete: () => {},
      },
      getVideoStateKey: () => "workspace:channel-2",
      getChannelOrder: () => [],
      getSelectedChannelId: () => selectedChannelId,
      getSelectedVideoId: () => selectedVideoId,
      getVideos: () => videos,
      getOffset: () => videos.length,
      getVideoTypeFilter: () => "all",
      getAcknowledgedFilter: () => "all",
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
      clearChannelSelectionState: () => {},
      resetVideoListState: (state) => {
        videos = state?.videos ?? [];
      },
      applyChannelSnapshotState: (snapshot) => {
        videos = snapshot.videos;
      },
      applyVideoPageState: () => {},
      setChannelLoadingState: () => {},
      setVideoLoadingState: (value) => {
        videoLoading = value;
      },
      setRefreshingChannelState: () => {},
      setSyncDepthState: () => {},
      setVideos: (next) => {
        videos = next;
      },
      setVideoTypeFilter: () => {},
      setAcknowledgedFilter: () => {},
    });

    const hydration = operations.selectChannelVideoOptimistically(
      "channel-2",
      clickedVideo.id,
      clickedVideo,
    );

    expect(selectedChannelId).toBe("channel-2");
    expect(selectedVideoId).toBe("video-99");
    expect(videos.map((video) => video.id)).toEqual(["video-99", "video-2"]);
    expect(videoLoading).toBe(false);

    deferredSnapshot.resolve(makeSnapshot([cachedVideo], "channel-2"));
    await hydration;
  });
});

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
