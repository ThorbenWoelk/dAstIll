import type {
  Channel,
  ChannelSnapshot,
  WorkspaceBootstrap,
} from "$lib/types";

const DB_NAME = "dastill-workspace-cache";
const DB_VERSION = 1;
const BOOTSTRAP_META_KEY = "bootstrap";

type WorkspaceBootstrapMeta = Pick<
  WorkspaceBootstrap,
  "ai_available" | "ai_status" | "search_status"
>;

type BootstrapMetaRecord = WorkspaceBootstrapMeta & {
  key: string;
};

let workspaceCacheDbPromise: Promise<IDBDatabase> | null = null;

function requestToPromise<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("IDB request failed"));
  });
}

function transactionDone(transaction: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    transaction.oncomplete = () => resolve();
    transaction.onabort = () =>
      reject(transaction.error ?? new Error("IDB transaction aborted"));
    transaction.onerror = () =>
      reject(transaction.error ?? new Error("IDB transaction failed"));
  });
}

function openWorkspaceCacheInternal(): Promise<IDBDatabase> {
  if (typeof indexedDB === "undefined") {
    return Promise.reject(new Error("IndexedDB is unavailable"));
  }

  if (!workspaceCacheDbPromise) {
    workspaceCacheDbPromise = new Promise((resolve, reject) => {
      const openRequest = indexedDB.open(DB_NAME, DB_VERSION);

      openRequest.onupgradeneeded = () => {
        const db = openRequest.result;

        if (!db.objectStoreNames.contains("channels")) {
          db.createObjectStore("channels", { keyPath: "id" });
        }

        if (!db.objectStoreNames.contains("videos")) {
          const videosStore = db.createObjectStore("videos", { keyPath: "id" });
          videosStore.createIndex("channel_id", "channel_id", { unique: false });
        }

        if (!db.objectStoreNames.contains("snapshots")) {
          db.createObjectStore("snapshots", { keyPath: "channel_id" });
        }

        if (!db.objectStoreNames.contains("meta")) {
          db.createObjectStore("meta", { keyPath: "key" });
        }
      };

      openRequest.onsuccess = () => {
        const db = openRequest.result;
        db.onversionchange = () => {
          db.close();
        };
        resolve(db);
      };

      openRequest.onerror = () => {
        workspaceCacheDbPromise = null;
        reject(openRequest.error ?? new Error("Failed to open workspace cache"));
      };

      openRequest.onblocked = () => {
        workspaceCacheDbPromise = null;
        reject(new Error("Workspace cache open blocked"));
      };
    });
  }

  return workspaceCacheDbPromise;
}

async function deleteVideosByChannel(
  videoStore: IDBObjectStore,
  channelId: string,
): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    const cursorRequest = videoStore
      .index("channel_id")
      .openCursor(IDBKeyRange.only(channelId));

    cursorRequest.onsuccess = () => {
      const cursor = cursorRequest.result;
      if (!cursor) {
        resolve();
        return;
      }
      cursor.delete();
      cursor.continue();
    };

    cursorRequest.onerror = () => {
      reject(cursorRequest.error ?? new Error("Failed to delete channel videos"));
    };
  });
}

export async function openWorkspaceCache(): Promise<IDBDatabase> {
  return openWorkspaceCacheInternal();
}

export async function getCachedChannels(): Promise<Channel[] | null> {
  try {
    const db = await openWorkspaceCacheInternal();
    const transaction = db.transaction("channels", "readonly");
    const channels = await requestToPromise<Channel[]>(
      transaction.objectStore("channels").getAll(),
    );
    return channels.length > 0 ? channels : null;
  } catch {
    return null;
  }
}

export async function putCachedChannels(channels: Channel[]): Promise<void> {
  try {
    const db = await openWorkspaceCacheInternal();
    const transaction = db.transaction("channels", "readwrite");
    const channelStore = transaction.objectStore("channels");
    channelStore.clear();
    for (const channel of channels) {
      channelStore.put(channel);
    }
    await transactionDone(transaction);
  } catch {
    return;
  }
}

export async function getCachedSnapshot(
  channelId: string,
): Promise<ChannelSnapshot | null> {
  try {
    const db = await openWorkspaceCacheInternal();
    const transaction = db.transaction("snapshots", "readonly");
    const snapshot = await requestToPromise<ChannelSnapshot | undefined>(
      transaction.objectStore("snapshots").get(channelId),
    );
    return snapshot ?? null;
  } catch {
    return null;
  }
}

export async function putCachedSnapshot(
  snapshot: ChannelSnapshot,
): Promise<void> {
  try {
    const db = await openWorkspaceCacheInternal();
    const transaction = db.transaction(["snapshots", "videos"], "readwrite");
    const snapshotStore = transaction.objectStore("snapshots");
    const videoStore = transaction.objectStore("videos");

    snapshotStore.put(snapshot);
    for (const video of snapshot.videos) {
      videoStore.put(video);
    }

    await transactionDone(transaction);
  } catch {
    return;
  }
}

export async function getCachedBootstrapMeta(): Promise<WorkspaceBootstrapMeta | null> {
  try {
    const db = await openWorkspaceCacheInternal();
    const transaction = db.transaction("meta", "readonly");
    const record = await requestToPromise<BootstrapMetaRecord | undefined>(
      transaction.objectStore("meta").get(BOOTSTRAP_META_KEY),
    );

    if (!record) {
      return null;
    }

    return {
      ai_available: record.ai_available,
      ai_status: record.ai_status,
      search_status: record.search_status,
    };
  } catch {
    return null;
  }
}

export async function putCachedBootstrapMeta(
  meta: WorkspaceBootstrapMeta,
): Promise<void> {
  try {
    const db = await openWorkspaceCacheInternal();
    const transaction = db.transaction("meta", "readwrite");
    transaction.objectStore("meta").put({
      key: BOOTSTRAP_META_KEY,
      ...meta,
    });
    await transactionDone(transaction);
  } catch {
    return;
  }
}

export async function clearWorkspaceCache(): Promise<void> {
  try {
    const db = await openWorkspaceCacheInternal();
    const transaction = db.transaction(
      ["channels", "videos", "snapshots", "meta"],
      "readwrite",
    );

    transaction.objectStore("channels").clear();
    transaction.objectStore("videos").clear();
    transaction.objectStore("snapshots").clear();
    transaction.objectStore("meta").clear();

    await transactionDone(transaction);
  } catch {
    return;
  }
}

export async function removeCachedChannel(channelId: string): Promise<void> {
  try {
    const db = await openWorkspaceCacheInternal();
    const transaction = db.transaction(
      ["channels", "snapshots", "videos"],
      "readwrite",
    );

    transaction.objectStore("channels").delete(channelId);
    transaction.objectStore("snapshots").delete(channelId);
    await deleteVideosByChannel(transaction.objectStore("videos"), channelId);

    await transactionDone(transaction);
  } catch {
    return;
  }
}
