import { describe, expect, it } from "bun:test";
import {
  channelOverviewSyncSettingsHref,
  shouldShowPagedCollectionSyncSettingsLink,
  shouldShowSelectedChannelSyncSettingsLink,
} from "../src/lib/workspace/sidebar-sync-boundary-link";

describe("channelOverviewSyncSettingsHref", () => {
  it("links to the channel overview sync boundary section", () => {
    expect(channelOverviewSyncSettingsHref("channel id")).toBe(
      "/channels/channel%20id#sync-boundary",
    );
  });
});

describe("shouldShowSelectedChannelSyncSettingsLink", () => {
  it("shows only after the selected channel list reaches oldest history", () => {
    expect(
      shouldShowSelectedChannelSyncSettingsLink({
        videosCount: 3,
        hasMore: false,
        historyExhausted: true,
        loadingVideos: false,
        backfillingHistory: false,
        isVirtualChannel: false,
      }),
    ).toBe(true);
  });

  it("hides while more pages or history can still load", () => {
    expect(
      shouldShowSelectedChannelSyncSettingsLink({
        videosCount: 3,
        hasMore: true,
        historyExhausted: true,
        loadingVideos: false,
        backfillingHistory: false,
        isVirtualChannel: false,
      }),
    ).toBe(false);
    expect(
      shouldShowSelectedChannelSyncSettingsLink({
        videosCount: 3,
        hasMore: false,
        historyExhausted: false,
        loadingVideos: false,
        backfillingHistory: false,
        isVirtualChannel: false,
      }),
    ).toBe(false);
  });

  it("hides during loading and for virtual channels", () => {
    expect(
      shouldShowSelectedChannelSyncSettingsLink({
        videosCount: 3,
        hasMore: false,
        historyExhausted: true,
        loadingVideos: true,
        backfillingHistory: false,
        isVirtualChannel: false,
      }),
    ).toBe(false);
    expect(
      shouldShowSelectedChannelSyncSettingsLink({
        videosCount: 3,
        hasMore: false,
        historyExhausted: true,
        loadingVideos: false,
        backfillingHistory: false,
        isVirtualChannel: true,
      }),
    ).toBe(false);
  });
});

describe("shouldShowPagedCollectionSyncSettingsLink", () => {
  it("shows after a paged collection exhausts its video pages", () => {
    expect(
      shouldShowPagedCollectionSyncSettingsLink({
        videosCount: 3,
        hasMore: false,
        loadingInitial: false,
        loadingMore: false,
        isVirtualChannel: false,
      }),
    ).toBe(true);
  });

  it("hides before the collection reaches the oldest loaded video", () => {
    expect(
      shouldShowPagedCollectionSyncSettingsLink({
        videosCount: 3,
        hasMore: true,
        loadingInitial: false,
        loadingMore: false,
        isVirtualChannel: false,
      }),
    ).toBe(false);
  });
});
