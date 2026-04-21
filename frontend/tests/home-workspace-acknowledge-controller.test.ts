import { describe, expect, it, mock } from "bun:test";

import type { Video } from "../src/lib/types";

function makeVideo(id: string, acknowledged: boolean): Video {
  return {
    id,
    channel_id: "channel-1",
    title: `Video ${id}`,
    thumbnail_url: null,
    published_at: "2026-04-11T18:30:00.000Z",
    is_short: false,
    transcript_status: "ready",
    summary_status: "ready",
    acknowledged,
    retry_count: 0,
  };
}

describe("createHomeWorkspaceAcknowledgeController", () => {
  it("selects the next visible row when the current unread video is marked read", async () => {
    const { createHomeWorkspaceAcknowledgeController } =
      await import("../src/lib/workspace/home-workspace-acknowledge-controller.svelte");

    let videos = [
      makeVideo("v1", false),
      makeVideo("v2", false),
      makeVideo("v3", false),
    ];
    let selectedVideoId: string | null = "v2";
    let pendingSelectedVideo: Video | null = null;
    const selectedVideoCalls: string[] = [];
    const resetInteractionCalls: Array<{ clearDisplayedContent?: boolean }> =
      [];
    const updateAcknowledgedMock = mock(
      async (_videoId: string, _read: boolean) => makeVideo("v2", true),
    );

    const controller = createHomeWorkspaceAcknowledgeController({
      sidebarState: {
        get selectedVideoId() {
          return selectedVideoId;
        },
        get videos() {
          return videos;
        },
        get acknowledgedFilter() {
          return "unack" as const;
        },
        bumpVideoListMutationEpoch: () => {},
        replaceVideos: (nextVideos: Video[]) => {
          videos = nextVideos;
        },
        selectVideo: (videoId: string | null) => {
          selectedVideoId = videoId;
        },
      } as never,
      content: {
        resetInteractionState: (options?: {
          clearDisplayedContent?: boolean;
        }) => {
          resetInteractionCalls.push(options ?? {});
        },
      } as never,
      getPendingSelectedVideo: () => pendingSelectedVideo,
      setPendingSelectedVideo: (value) => {
        pendingSelectedVideo = value;
      },
      setErrorMessage: () => {},
      getSelectedChannelId: () => "channel-1",
      selectVideo: async (videoId: string) => {
        selectedVideoCalls.push(videoId);
        selectedVideoId = videoId;
      },
      setVideoAcknowledgeSync: () => {},
      updateAcknowledged: updateAcknowledgedMock,
    });

    await controller.toggleAcknowledge();

    expect(updateAcknowledgedMock).toHaveBeenCalledWith("v2", true);
    expect(videos.map((video) => video.id)).toEqual(["v1", "v3"]);
    expect(selectedVideoCalls).toEqual(["v3"]);
    expect(selectedVideoId).toBe("v3");
    expect(resetInteractionCalls).toEqual([{}]);
  });
});
