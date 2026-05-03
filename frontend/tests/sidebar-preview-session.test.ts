import { describe, expect, it } from "bun:test";

import {
  clearSidebarPreviewSession,
  demoteOtherPagedSidebarPreviewCollections,
  getSidebarPreviewSession,
  pruneSidebarPreviewCollections,
  resolveSidebarPreviewSessionKey,
  resolvePreferredExpandedSidebarPreviewCollectionId,
  setSidebarPreviewCollectionExpanded,
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

function makeVideoForChannel(id: string, channelId: string): Video {
  return {
    ...makeVideo(id),
    channel_id: channelId,
  } as Video;
}

function makeCollection(
  overrides: Partial<SidebarPreviewCollectionSnapshot> = {},
): SidebarPreviewCollectionSnapshot {
  return {
    videos: [makeVideo("video-1")],
    expanded: true,
    loadedMode: "preview",
    hasMore: false,
    nextOffset: 1,
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
          videos: [makeVideoForChannel("video-2", "channel-b")],
        }),
      },
      ["channel-b"],
    );

    expect(Object.keys(pruned)).toEqual(["channel-b"]);
    expect(pruned["channel-b"]?.videos[0]?.id).toBe("video-2");
  });

  it("sanitizes restored concrete-channel collections when video rows belong elsewhere", () => {
    clearSidebarPreviewSession("workspace");
    setSidebarPreviewSession("workspace", {
      "channel-a": makeCollection({
        videos: [
          makeVideoForChannel("video-1", "channel-a"),
          makeVideoForChannel("video-2", "channel-b"),
        ],
        loadedMode: "paged",
        hasMore: true,
        nextOffset: 2,
      }),
    });

    const restored = getSidebarPreviewSession("workspace");
    expect(restored?.["channel-a"]).toEqual({
      ...makeCollection(),
      videos: [],
      loadedMode: null,
      hasMore: false,
      nextOffset: 0,
      channelVideoCount: null,
      filterKey: null,
      syncDepth: null,
      earliestSyncDateInput: "",
      selectedVideoReloadProbeKey: null,
    });
  });

  it("isolates preview sessions across auth scopes", () => {
    const anonymousKey = resolveSidebarPreviewSessionKey("workspace", {
      authState: "anonymous",
      userId: "anon-123",
    });
    const authenticatedKey = resolveSidebarPreviewSessionKey("workspace", {
      authState: "authenticated",
      userId: "user-123",
    });

    expect(anonymousKey).toBeDefined();
    expect(authenticatedKey).toBeDefined();
    expect(anonymousKey).not.toBe(authenticatedKey);

    clearSidebarPreviewSession(anonymousKey!);
    clearSidebarPreviewSession(authenticatedKey!);

    setSidebarPreviewSession(anonymousKey!, {
      "channel-a": makeCollection({ videos: [] }),
    });
    setSidebarPreviewSession(authenticatedKey!, {
      "channel-a": makeCollection({
        videos: [makeVideoForChannel("video-2", "channel-a")],
      }),
    });

    expect(
      getSidebarPreviewSession(anonymousKey!)?.["channel-a"]?.videos,
    ).toEqual([]);
    expect(
      getSidebarPreviewSession(authenticatedKey!)?.["channel-a"]?.videos[0]?.id,
    ).toBe("video-2");
  });

  it("prefers the current channel when normalizing restored expanded state", () => {
    const collections = {
      "channel-a": makeCollection({ expanded: true }),
      "channel-b": makeCollection({
        expanded: true,
        videos: [makeVideoForChannel("video-2", "channel-b")],
      }),
    };

    expect(
      resolvePreferredExpandedSidebarPreviewCollectionId(
        collections,
        "channel-b",
      ),
    ).toBe("channel-b");
  });

  it("expands and collapses one collection without changing other expanded collections", () => {
    const collections = {
      "channel-a": makeCollection({ expanded: true }),
      "channel-b": makeCollection({
        expanded: false,
        videos: [makeVideoForChannel("video-2", "channel-b")],
      }),
    };

    setSidebarPreviewCollectionExpanded(collections, "channel-b", true);

    expect(collections["channel-a"]?.expanded).toBe(true);
    expect(collections["channel-b"]?.expanded).toBe(true);

    setSidebarPreviewCollectionExpanded(collections, "channel-b", false);

    expect(collections["channel-a"]?.expanded).toBe(true);
    expect(collections["channel-b"]?.expanded).toBe(false);
  });

  it("demotes other paged collections to preview while preserving the active channel", () => {
    const collections: Record<
      string,
      SidebarPreviewCollectionSnapshot & { scrollTop?: number }
    > = {
      "channel-a": {
        ...makeCollection({ loadedMode: "paged", expanded: true }),
        scrollTop: 320,
      },
      "channel-b": {
        ...makeCollection({ loadedMode: "paged", expanded: true }),
        scrollTop: 80,
      },
      "channel-c": makeCollection({ loadedMode: "preview", expanded: true }),
    };

    demoteOtherPagedSidebarPreviewCollections(collections, "channel-b");

    expect(collections["channel-a"]?.loadedMode).toBe("preview");
    expect(collections["channel-a"]?.scrollTop).toBe(0);
    expect(collections["channel-a"]?.expanded).toBe(true);

    expect(collections["channel-b"]?.loadedMode).toBe("paged");
    expect(collections["channel-b"]?.scrollTop).toBe(80);

    expect(collections["channel-c"]?.loadedMode).toBe("preview");
    expect(collections["channel-c"]?.expanded).toBe(true);
  });
});
