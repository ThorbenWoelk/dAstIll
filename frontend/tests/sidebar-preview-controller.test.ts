import { describe, expect, it } from "bun:test";

import { shouldSkipAutoExpandForCollapsedSelection } from "../src/lib/workspace/sidebar-preview-controller.svelte";

describe("shouldSkipAutoExpandForCollapsedSelection", () => {
  it("keeps a chevron-collapsed selected channel collapsed", () => {
    expect(
      shouldSkipAutoExpandForCollapsedSelection({
        targetChannelId: "channel-1",
        selectedChannelId: "channel-1",
        selectedVideoId: "video-1",
        userCollapsedSelectionKey: "channel-1:video-1",
      }),
    ).toBe(true);
  });

  it("allows auto-expand for a different selected video", () => {
    expect(
      shouldSkipAutoExpandForCollapsedSelection({
        targetChannelId: "channel-1",
        selectedChannelId: "channel-1",
        selectedVideoId: "video-2",
        userCollapsedSelectionKey: "channel-1:video-1",
      }),
    ).toBe(false);
  });

  it("allows auto-expand when the target is not the selected channel", () => {
    expect(
      shouldSkipAutoExpandForCollapsedSelection({
        targetChannelId: "channel-2",
        selectedChannelId: "channel-1",
        selectedVideoId: "video-1",
        userCollapsedSelectionKey: "channel-1:video-1",
      }),
    ).toBe(false);
  });
});
