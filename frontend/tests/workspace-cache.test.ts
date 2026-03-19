import { beforeEach, describe, expect, it } from "bun:test";
import "fake-indexeddb/auto";
import type {
  Channel,
  ChannelSnapshot,
  SearchStatus,
  Video,
} from "../src/lib/types";
import {
  clearWorkspaceCache,
  getCachedBootstrapMeta,
  getCachedChannels,
  getCachedSnapshot,
  openWorkspaceCache,
  putCachedBootstrapMeta,
  putCachedChannels,
  putCachedSnapshot,
  removeCachedChannel,
} from "../src/lib/workspace-cache";

function requestToPromise<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () =>
      reject(request.error ?? new Error("IDB request failed"));
  });
}

function createChannel(id: string): Channel {
  return {
    id,
    name: `Channel ${id}`,
    added_at: "2026-03-19T00:00:00.000Z",
  };
}

function createVideo(id: string, channelId: string): Video {
  return {
    id,
    channel_id: channelId,
    title: `Video ${id}`,
    published_at: "2026-03-19T00:00:00.000Z",
    is_short: false,
    transcript_status: "ready",
    summary_status: "ready",
    acknowledged: false,
  };
}

function createSnapshot(
  channelId: string,
  videoIds: string[],
): ChannelSnapshot {
  return {
    channel_id: channelId,
    sync_depth: {
      earliest_sync_date: null,
      earliest_sync_date_user_set: false,
      derived_earliest_ready_date: null,
    },
    videos: videoIds.map((videoId) => createVideo(videoId, channelId)),
  };
}

const SEARCH_STATUS: SearchStatus = {
  available: true,
  model: "model",
  dimensions: 384,
  pending: 0,
  indexing: 0,
  ready: 1,
  failed: 0,
  total_sources: 1,
  total_chunk_count: 2,
  embedded_chunk_count: 2,
  vector_index_ready: true,
  retrieval_mode: "hybrid_exact",
};

beforeEach(async () => {
  await clearWorkspaceCache();
});

describe("workspace cache", () => {
  it("returns null for cache misses", async () => {
    expect(await getCachedChannels()).toBeNull();
    expect(await getCachedSnapshot("missing")).toBeNull();
    expect(await getCachedBootstrapMeta()).toBeNull();
  });

  it("returns stored channels, snapshots, and bootstrap meta", async () => {
    const channels = [createChannel("alpha"), createChannel("beta")];
    const snapshot = createSnapshot("alpha", ["video-a", "video-b"]);
    const meta = {
      ai_available: true,
      ai_status: "cloud" as const,
      search_status: SEARCH_STATUS,
    };

    await putCachedChannels(channels);
    await putCachedSnapshot(snapshot);
    await putCachedBootstrapMeta(meta);

    expect(await getCachedChannels()).toEqual(channels);
    expect(await getCachedSnapshot("alpha")).toEqual(snapshot);
    expect(await getCachedBootstrapMeta()).toEqual(meta);
  });

  it("removeCachedChannel removes related channel, snapshot, and videos", async () => {
    const channelA = createChannel("alpha");
    const channelB = createChannel("beta");
    await putCachedChannels([channelA, channelB]);
    await putCachedSnapshot(
      createSnapshot("alpha", ["video-a-1", "video-a-2"]),
    );
    await putCachedSnapshot(createSnapshot("beta", ["video-b-1"]));

    await removeCachedChannel("alpha");

    expect(await getCachedSnapshot("alpha")).toBeNull();
    expect(await getCachedSnapshot("beta")).toEqual(
      createSnapshot("beta", ["video-b-1"]),
    );
    expect(await getCachedChannels()).toEqual([channelB]);

    const db = await openWorkspaceCache();
    const transaction = db.transaction("videos", "readonly");
    const videoIndex = transaction.objectStore("videos").index("channel_id");
    const videosForAlpha = await requestToPromise<Video[]>(
      videoIndex.getAll("alpha"),
    );
    const videosForBeta = await requestToPromise<Video[]>(
      videoIndex.getAll("beta"),
    );

    expect(videosForAlpha).toEqual([]);
    expect(videosForBeta).toEqual([createVideo("video-b-1", "beta")]);
  });

  it("swallows indexeddb errors", async () => {
    const db = await openWorkspaceCache();
    const mutableDb = db as IDBDatabase & {
      transaction: IDBDatabase["transaction"];
    };
    const originalTransaction = mutableDb.transaction.bind(db);

    mutableDb.transaction = (() => {
      throw new Error("forced failure");
    }) as IDBDatabase["transaction"];

    expect(await getCachedChannels()).toBeNull();
    expect(await getCachedSnapshot("alpha")).toBeNull();
    expect(await getCachedBootstrapMeta()).toBeNull();
    await putCachedChannels([createChannel("alpha")]);
    await putCachedSnapshot(createSnapshot("alpha", ["video-a-1"]));
    await putCachedBootstrapMeta({
      ai_available: false,
      ai_status: "offline",
      search_status: SEARCH_STATUS,
    });
    await removeCachedChannel("alpha");
    await clearWorkspaceCache();

    mutableDb.transaction = originalTransaction;
  });
});
