import { describe, expect, it } from "bun:test";

import {
  clearSidebarPreviewSession,
  getSidebarPreviewSession,
  pruneSidebarPreviewCollections,
  setSidebarPreviewSession,
  type SidebarPreviewCollectionSnapshot,
} from "../src/lib/workspace/sidebar-preview-session";
import type { Video } from "../src/lib/types";

function makeVideo(id: string): Video {
  return {
    id,
    published_at: "2024-01-01T00:00:00Z",
    title: `Video ${id}`,
    channel_id: "channel-a",
    thumbnail_url: null,
    transcript_status: "ready",
    summary_status: "ready",
    duration_seconds: 300,
    description: null,
    acknowledged: false,
    is_short: false,
  } as unknown as Video;
}

function makeCollection(
  overrides: Partial<SidebarPreviewCollectionSnapshot> = {},
): SidebarPreviewCollectionSnapshot {
  return {
    videos: [makeVideo("video-1")],
    expanded: true,
    loadedMode: "preview",
    hasMoreThanPreview: false,
    channelVideoCount: 1,
    filterKey: "all:all:default",
    syncDepth: null,
    earliestSyncDateInput: "",
    selectedVideoReloadProbeKey: null,
    ...overrides,
  };
}

describe("sidebar preview session", () => {
  it("returns cloned session state so callers cannot mutate the stored snapshot", () => {
    clearSidebarPreviewSession("workspace");
    setSidebarPreviewSession("workspace", {
      "channel-a": makeCollection(),
    });

    const restored = getSidebarPreviewSession("workspace");
    expect(restored).not.toBeNull();
    restored!["channel-a"].expanded = false;
    restored!["channel-a"].videos[0]!.id = "mutated";

    const reread = getSidebarPreviewSession("workspace");
    expect(reread?.["channel-a"].expanded).toBe(true);
    expect(reread?.["channel-a"].videos[0]?.id).toBe("video-1");
  });

  it("prunes session state to the currently visible channel set", () => {
    const pruned = pruneSidebarPreviewCollections(
      {
        "channel-a": makeCollection(),
        "channel-b": makeCollection({
          videos: [makeVideo("video-2")],
        }),
      },
      ["channel-b"],
    );

    expect(Object.keys(pruned)).toEqual(["channel-b"]);
    expect(pruned["channel-b"]?.videos[0]?.id).toBe("video-2");
  });
});
