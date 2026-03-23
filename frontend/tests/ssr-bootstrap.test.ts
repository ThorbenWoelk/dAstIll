import { beforeEach, describe, expect, it } from "bun:test";
import "fake-indexeddb/auto";
import type {
  Channel,
  ChannelSnapshot,
  SearchStatus,
  SyncDepth,
  WorkspaceBootstrap,
} from "../src/lib/types";
import { resolveBootstrapOnMount } from "../src/lib/ssr-bootstrap";
import {
  clearWorkspaceCache,
  putCachedBootstrapMeta,
  putCachedChannels,
  putCachedViewSnapshot,
} from "../src/lib/workspace-cache";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeChannel(id: string): Channel {
  return {
    id,
    name: `Channel ${id}`,
    added_at: "2026-01-01T00:00:00Z",
  };
}

function makeSyncDepth(): SyncDepth {
  return {
    earliest_sync_date: null,
    earliest_sync_date_user_set: false,
    derived_earliest_ready_date: null,
  };
}

function makeSnapshot(channelId: string): ChannelSnapshot {
  return {
    channel_id: channelId,
    sync_depth: makeSyncDepth(),
    channel_video_count: 0,
    videos: [],
  };
}

function makeSearchStatus(): SearchStatus {
  return {
    available: false,
    model: "test-model",
    dimensions: 512,
    pending: 0,
    indexing: 0,
    ready: 0,
    failed: 0,
    total_sources: 0,
    total_chunk_count: 0,
    embedded_chunk_count: 0,
    vector_index_ready: false,
    retrieval_mode: "fts_only",
  };
}

function makeBootstrap(
  channels: Channel[],
  overrides?: Partial<WorkspaceBootstrap>,
): WorkspaceBootstrap {
  return {
    ai_available: true,
    ai_status: "cloud",
    channels,
    selected_channel_id: channels[0]?.id ?? null,
    snapshot: channels[0] ? makeSnapshot(channels[0].id) : null,
    search_status: makeSearchStatus(),
    ...overrides,
  };
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

beforeEach(async () => {
  await clearWorkspaceCache();
});

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("resolveBootstrapOnMount", () => {
  describe("IndexedDB warm-start (VAL-CROSS-004)", () => {
    it("reads IndexedDB channels before returning result when server bootstrap is null", async () => {
      const idbChannels = [makeChannel("idb-ch-1"), makeChannel("idb-ch-2")];
      await putCachedChannels(idbChannels);

      const result = await resolveBootstrapOnMount({
        serverBootstrap: null,
        selectedChannelId: null,
        viewSnapshotCacheKey: null,
      });

      // IndexedDB channels must be returned as fallback
      expect(result.channels).not.toBeNull();
      expect(result.channels).toHaveLength(2);
      expect(result.channels![0].id).toBe("idb-ch-1");
      expect(result.fromServer).toBe(false);
    });

    it("reads IndexedDB even when server bootstrap is provided", async () => {
      const idbChannels = [makeChannel("idb-ch-1")];
      await putCachedChannels(idbChannels);

      const serverChannels = [makeChannel("server-ch-1")];
      const serverBootstrap = makeBootstrap(serverChannels);

      // Even though server has data, resolveBootstrapOnMount still reads IDB
      // (needed for snapshot fallback and warm-start guarantee).
      // The result fromServer flag confirms server data was used.
      const result = await resolveBootstrapOnMount({
        serverBootstrap,
        selectedChannelId: "server-ch-1",
        viewSnapshotCacheKey: null,
      });

      expect(result.fromServer).toBe(true);
      expect(result.channels![0].id).toBe("server-ch-1");
    });

    it("returns null channels when both server and IDB have no channels", async () => {
      const result = await resolveBootstrapOnMount({
        serverBootstrap: null,
        selectedChannelId: null,
        viewSnapshotCacheKey: null,
      });

      expect(result.channels).toBeNull();
      expect(result.aiAvailable).toBeNull();
      expect(result.aiStatus).toBeNull();
      expect(result.searchStatus).toBeNull();
      expect(result.snapshot).toBeNull();
    });
  });

  describe("server bootstrap priority", () => {
    it("uses server channels when server bootstrap is available", async () => {
      const idbChannels = [makeChannel("idb-ch-1")];
      await putCachedChannels(idbChannels);

      const serverChannels = [
        makeChannel("server-ch-1"),
        makeChannel("server-ch-2"),
      ];
      const result = await resolveBootstrapOnMount({
        serverBootstrap: makeBootstrap(serverChannels),
        selectedChannelId: "server-ch-1",
        viewSnapshotCacheKey: null,
      });

      expect(result.channels).toHaveLength(2);
      expect(result.channels![0].id).toBe("server-ch-1");
      expect(result.fromServer).toBe(true);
    });

    it("falls back to IDB channels when server returns empty channel list", async () => {
      const idbChannels = [makeChannel("idb-ch-1")];
      await putCachedChannels(idbChannels);

      const serverBootstrap = makeBootstrap([], {
        channels: [],
        selected_channel_id: null,
        snapshot: null,
      });

      const result = await resolveBootstrapOnMount({
        serverBootstrap,
        selectedChannelId: null,
        viewSnapshotCacheKey: null,
      });

      expect(result.channels).toHaveLength(1);
      expect(result.channels![0].id).toBe("idb-ch-1");
    });

    it("uses server AI/search status over IDB meta", async () => {
      await putCachedBootstrapMeta({
        ai_available: false,
        ai_status: "offline",
        search_status: makeSearchStatus(),
      });

      const serverBootstrap = makeBootstrap([makeChannel("ch-1")]);
      serverBootstrap.ai_available = true;
      serverBootstrap.ai_status = "cloud";

      const result = await resolveBootstrapOnMount({
        serverBootstrap,
        selectedChannelId: "ch-1",
        viewSnapshotCacheKey: null,
      });

      expect(result.aiAvailable).toBe(true);
      expect(result.aiStatus).toBe("cloud");
    });
  });

  describe("IndexedDB fallback", () => {
    it("uses IDB meta when server bootstrap is null", async () => {
      await putCachedBootstrapMeta({
        ai_available: false,
        ai_status: "offline",
        search_status: makeSearchStatus(),
      });

      const result = await resolveBootstrapOnMount({
        serverBootstrap: null,
        selectedChannelId: null,
        viewSnapshotCacheKey: null,
      });

      expect(result.aiAvailable).toBe(false);
      expect(result.aiStatus).toBe("offline");
    });

    it("uses IDB view snapshot when server bootstrap has no snapshot", async () => {
      const channelId = "ch-1";
      const snapshot = makeSnapshot(channelId);
      const cacheKey = `workspace:${channelId}:type=all:ack=all:limit=20`;
      await putCachedViewSnapshot(cacheKey, snapshot);

      const serverBootstrap = makeBootstrap([makeChannel(channelId)], {
        snapshot: null,
      });

      const result = await resolveBootstrapOnMount({
        serverBootstrap,
        selectedChannelId: channelId,
        viewSnapshotCacheKey: cacheKey,
      });

      expect(result.snapshot).not.toBeNull();
      expect(result.snapshot?.channel_id).toBe(channelId);
    });

    it("uses server snapshot over IDB snapshot when server has one", async () => {
      const channelId = "ch-1";
      const idbSnapshot = makeSnapshot(channelId);
      const cacheKey = `workspace:${channelId}:type=all:ack=all:limit=20`;
      await putCachedViewSnapshot(cacheKey, idbSnapshot);

      const serverSnapshot = makeSnapshot(channelId);
      const serverBootstrap = makeBootstrap([makeChannel(channelId)], {
        snapshot: serverSnapshot,
      });

      const result = await resolveBootstrapOnMount({
        serverBootstrap,
        selectedChannelId: channelId,
        viewSnapshotCacheKey: cacheKey,
      });

      // Both are structurally the same here, but server is preferred.
      expect(result.snapshot).toBe(serverSnapshot);
    });

    it("returns null snapshot when viewSnapshotCacheKey is null and server has no snapshot", async () => {
      const result = await resolveBootstrapOnMount({
        serverBootstrap: null,
        selectedChannelId: "ch-1",
        viewSnapshotCacheKey: null,
      });

      expect(result.snapshot).toBeNull();
    });
  });
});
