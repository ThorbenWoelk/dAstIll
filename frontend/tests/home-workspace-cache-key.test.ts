import { describe, expect, it } from "bun:test";

import { buildHomeWorkspaceChannelViewCacheKey } from "../src/lib/workspace/home-workspace-cache-key";

describe("buildHomeWorkspaceChannelViewCacheKey", () => {
  it("keeps the same cache identity when only transient channel state changes", () => {
    const stable = buildHomeWorkspaceChannelViewCacheKey({
      channelId: "channel-2",
      workspaceCacheScopeKey: "user-1",
      videoTypeFilter: "all",
      acknowledgedFilter: "all",
    });

    const switchedFromDifferentChannel = buildHomeWorkspaceChannelViewCacheKey({
      channelId: "channel-2",
      workspaceCacheScopeKey: "user-1",
      videoTypeFilter: "all",
      acknowledgedFilter: "all",
    });

    expect(switchedFromDifferentChannel).toBe(stable);
  });
});
