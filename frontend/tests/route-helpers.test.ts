import { describe, expect, it } from "bun:test";

import {
  dedupeVideosById,
  loadChannelSnapshotWithRefresh,
} from "../src/lib/workspace/route-helpers";
import type { Video } from "../src/lib/types";

describe("dedupeVideosById", () => {
  it("keeps first occurrence and stable order", () => {
    const a = { id: "a" } as Video;
    const b = { id: "b" } as Video;
    const aDup = { id: "a" } as Video;
    expect(dedupeVideosById([a, b, aDup])).toEqual([a, b]);
  });
});

describe("loadChannelSnapshotWithRefresh", () => {
  it("loads the snapshot without hitting the refresh endpoint inside the TTL window", async () => {
    const events: string[] = [];
    const refreshedAtByChannel = new Map<string, number>([
      ["channel-1", Date.now()],
    ]);

    await loadChannelSnapshotWithRefresh({
      channelId: "channel-1",
      refreshedAtByChannel,
      ttlMs: 60_000,
      loadSnapshot: async () => {
        events.push("load");
        return { id: "snapshot-1" };
      },
      applySnapshot: async (snapshot) => {
        events.push(`apply:${snapshot.id}`);
      },
      refreshChannel: async () => {
        events.push("refresh");
      },
      shouldReloadAfterRefresh: () => true,
      onRefreshingChange: (refreshing) => {
        events.push(`refreshing:${refreshing}`);
      },
      onError: (message) => {
        events.push(`error:${message}`);
      },
    });

    expect(events).toEqual(["load", "apply:snapshot-1"]);
    expect(refreshedAtByChannel.size).toBe(1);
  });

  it("refreshes and reloads when refresh is enabled and the TTL has expired", async () => {
    const events: string[] = [];
    const refreshedAtByChannel = new Map<string, number>();
    let loads = 0;

    await loadChannelSnapshotWithRefresh({
      channelId: "channel-1",
      refreshedAtByChannel,
      ttlMs: 1_000,
      loadSnapshot: async () => {
        loads += 1;
        events.push(`load:${loads}`);
        return { id: `snapshot-${loads}` };
      },
      applySnapshot: async (snapshot, silent) => {
        events.push(`apply:${snapshot.id}:${silent ?? false}`);
      },
      refreshChannel: async () => {
        events.push("refresh");
      },
      shouldReloadAfterRefresh: () => true,
      onRefreshingChange: (refreshing) => {
        events.push(`refreshing:${refreshing}`);
      },
      onError: (message) => {
        events.push(`error:${message}`);
      },
    });

    expect(events).toEqual([
      "load:1",
      "apply:snapshot-1:false",
      "refreshing:true",
      "refresh",
      "load:2",
      "apply:snapshot-2:true",
      "refreshing:false",
    ]);
    expect(refreshedAtByChannel.has("channel-1")).toBe(true);
  });

  it("skips applying snapshot when mutation epoch changes during load", async () => {
    const events: string[] = [];
    let epoch = 0;

    await loadChannelSnapshotWithRefresh({
      channelId: "channel-1",
      refreshedAtByChannel: new Map(),
      ttlMs: 60_000,
      getMutationEpoch: () => epoch,
      loadSnapshot: async () => {
        events.push("load");
        epoch += 1;
        return { id: "snapshot-1" };
      },
      applySnapshot: async (snapshot) => {
        events.push(`apply:${snapshot.id}`);
      },
      refreshChannel: async () => {
        events.push("refresh");
      },
      shouldReloadAfterRefresh: () => true,
      onRefreshingChange: (refreshing) => {
        events.push(`refreshing:${refreshing}`);
      },
      onError: (message) => {
        events.push(`error:${message}`);
      },
    });

    expect(events).toEqual(["load"]);
  });
});
