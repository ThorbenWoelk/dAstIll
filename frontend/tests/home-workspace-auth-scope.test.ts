import { describe, expect, it, mock } from "bun:test";

import {
  applyWorkspaceStateForScopeChange,
  clearWorkspaceForScopeChange,
} from "../src/lib/workspace/home-workspace-auth-scope";

describe("clearWorkspaceForScopeChange", () => {
  it("drops previously loaded channels before refetching under a new auth scope", () => {
    const calls: string[] = [];
    const sidebarState = {
      setChannels: mock((channels: unknown[]) => {
        calls.push(`setChannels:${channels.length}`);
      }),
      clearChannelSelectionState: mock(() => {
        calls.push("clearChannelSelectionState");
      }),
      setLoadingVideos: mock((value: boolean) => {
        calls.push(`setLoadingVideos:${value}`);
      }),
    };

    clearWorkspaceForScopeChange(sidebarState);

    expect(sidebarState.setChannels).toHaveBeenCalledWith([]);
    expect(sidebarState.clearChannelSelectionState).toHaveBeenCalledTimes(1);
    expect(sidebarState.setLoadingVideos).toHaveBeenCalledWith(false);
    expect(calls).toEqual([
      "setChannels:0",
      "clearChannelSelectionState",
      "setLoadingVideos:false",
    ]);
  });
});

describe("applyWorkspaceStateForScopeChange", () => {
  it("restores selected channel, video, mode, filters, and ordering for the incoming auth scope", () => {
    const calls: string[] = [];
    const sidebarState = {
      setSelectedChannel: mock((channelId: string | null) => {
        calls.push(`setSelectedChannel:${channelId}`);
      }),
      setSelectedVideoId: mock((videoId: string | null) => {
        calls.push(`setSelectedVideoId:${videoId}`);
      }),
      setChannelOrder: mock((channelOrder: string[]) => {
        calls.push(`setChannelOrder:${channelOrder.join(",")}`);
      }),
      setChannelSortMode: mock((channelSortMode: string) => {
        calls.push(`setChannelSortMode:${channelSortMode}`);
      }),
      setAcknowledgedFilter: mock((acknowledgedFilter: string) => {
        calls.push(`setAcknowledgedFilter:${acknowledgedFilter}`);
      }),
      setVideoTypeFilter: mock((videoTypeFilter: string) => {
        calls.push(`setVideoTypeFilter:${videoTypeFilter}`);
      }),
    };
    const content = {
      setMode: mock((mode: string) => {
        calls.push(`setMode:${mode}`);
      }),
    };

    applyWorkspaceStateForScopeChange(sidebarState, content, {
      selectedChannelId: "channel-user",
      selectedVideoId: "video-user",
      contentMode: "summary",
      channelOrder: ["channel-user", "channel-other"],
      channelSortMode: "custom",
      acknowledgedFilter: "unack",
      videoTypeFilter: "long",
    });

    expect(calls).toEqual([
      "setSelectedChannel:channel-user",
      "setSelectedVideoId:video-user",
      "setMode:summary",
      "setChannelOrder:channel-user,channel-other",
      "setChannelSortMode:custom",
      "setAcknowledgedFilter:unack",
      "setVideoTypeFilter:long",
    ]);
  });

  it("preserves explicit null channel and video selections", () => {
    const sidebarState = {
      setSelectedChannel: mock(() => {}),
      setSelectedVideoId: mock(() => {}),
      setChannelOrder: mock(() => {}),
      setChannelSortMode: mock(() => {}),
      setAcknowledgedFilter: mock(() => {}),
      setVideoTypeFilter: mock(() => {}),
    };
    const content = { setMode: mock(() => {}) };

    applyWorkspaceStateForScopeChange(sidebarState, content, {
      selectedChannelId: null,
      selectedVideoId: null,
    });

    expect(sidebarState.setSelectedChannel).toHaveBeenCalledWith(null);
    expect(sidebarState.setSelectedVideoId).toHaveBeenCalledWith(null);
    expect(content.setMode).not.toHaveBeenCalled();
  });
});
