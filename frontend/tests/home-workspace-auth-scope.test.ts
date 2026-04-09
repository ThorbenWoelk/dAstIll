import { describe, expect, it, mock } from "bun:test";

import { clearWorkspaceForScopeChange } from "../src/lib/workspace/home-workspace-auth-scope";

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
