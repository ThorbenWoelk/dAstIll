import { describe, expect, it } from "bun:test";

import {
  resolveChannelOverviewMissingMessage,
  shouldReloadChannelOverviewForAuthScope,
} from "../src/lib/workspace/channel-overview-state";

describe("resolveChannelOverviewMissingMessage", () => {
  it("does not show a missing-channel message while overview data is loading", () => {
    expect(
      resolveChannelOverviewMissingMessage({
        overviewBusy: true,
        loadingChannels: false,
        channelsLength: 2,
        hasSelectedChannel: false,
      }),
    ).toBeNull();
  });

  it("does not show a missing-channel message while channels are loading", () => {
    expect(
      resolveChannelOverviewMissingMessage({
        overviewBusy: false,
        loadingChannels: true,
        channelsLength: 0,
        hasSelectedChannel: false,
      }),
    ).toBeNull();
  });

  it("shows the empty-library message only after loading finishes", () => {
    expect(
      resolveChannelOverviewMissingMessage({
        overviewBusy: false,
        loadingChannels: false,
        channelsLength: 0,
        hasSelectedChannel: false,
      }),
    ).toBe("Follow a channel to start shaping your workspace.");
  });

  it("shows channel not found only when a loaded channel list lacks the selected channel", () => {
    expect(
      resolveChannelOverviewMissingMessage({
        overviewBusy: false,
        loadingChannels: false,
        channelsLength: 2,
        hasSelectedChannel: false,
      }),
    ).toBe("Channel not found.");
  });
});

describe("shouldReloadChannelOverviewForAuthScope", () => {
  it("waits for workspace hydration and auth readiness", () => {
    expect(
      shouldReloadChannelOverviewForAuthScope({
        workspaceStateHydrated: false,
        authReady: true,
        loadedAuthScopeKey: "anonymous:bootstrap",
        loadingAuthScopeKey: null,
        authScopeKey: "user:123",
      }),
    ).toBe(false);

    expect(
      shouldReloadChannelOverviewForAuthScope({
        workspaceStateHydrated: true,
        authReady: false,
        loadedAuthScopeKey: "anonymous:bootstrap",
        loadingAuthScopeKey: null,
        authScopeKey: "user:123",
      }),
    ).toBe(false);
  });

  it("reloads when the loaded scope differs from the active auth scope", () => {
    expect(
      shouldReloadChannelOverviewForAuthScope({
        workspaceStateHydrated: true,
        authReady: true,
        loadedAuthScopeKey: "anonymous:bootstrap",
        loadingAuthScopeKey: null,
        authScopeKey: "user:123",
      }),
    ).toBe(true);
  });

  it("does not duplicate an in-flight reload for the current auth scope", () => {
    expect(
      shouldReloadChannelOverviewForAuthScope({
        workspaceStateHydrated: true,
        authReady: true,
        loadedAuthScopeKey: "anonymous:bootstrap",
        loadingAuthScopeKey: "user:123",
        authScopeKey: "user:123",
      }),
    ).toBe(false);
  });
});
