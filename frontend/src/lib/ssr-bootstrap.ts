import type {
  AiStatus,
  Channel,
  ChannelSnapshot,
  SearchStatus,
  WorkspaceBootstrap,
} from "$lib/types";
import {
  getCachedBootstrapMeta,
  getCachedChannels,
  getCachedViewSnapshot,
} from "$lib/workspace-cache";

export interface OnMountBootstrapResult {
  channels: Channel[] | null;
  aiAvailable: boolean | null;
  aiStatus: AiStatus | null;
  searchStatus: SearchStatus | null;
  snapshot: ChannelSnapshot | null;
  /** Whether the result was sourced from a server-side load (true) or IndexedDB only (false). */
  fromServer: boolean;
}

/**
 * Resolves the initial workspace bootstrap data for onMount.
 *
 * Always reads IndexedDB in parallel with processing the server data.
 * This preserves the warm-start guarantee (VAL-CROSS-004): IndexedDB is
 * read before any network API call, and its data is used as a fallback
 * when the server bootstrap is unavailable.
 *
 * Priority: server bootstrap > IndexedDB
 */
export async function resolveBootstrapOnMount(options: {
  serverBootstrap: WorkspaceBootstrap | null;
  selectedChannelId: string | null;
  viewSnapshotCacheKey: string | null;
}): Promise<OnMountBootstrapResult> {
  // Always read IndexedDB — ensures IndexedDB is consulted before any network
  // API call (satisfies VAL-CROSS-004 warm-start requirement).
  const [cachedChannels, cachedSnapshot, cachedMeta] = await Promise.all([
    getCachedChannels(),
    options.viewSnapshotCacheKey
      ? getCachedViewSnapshot(options.viewSnapshotCacheKey)
      : Promise.resolve(null),
    getCachedBootstrapMeta(),
  ]);

  const { serverBootstrap } = options;

  if (serverBootstrap) {
    return {
      // Prefer server channels; fall back to IDB if server returned empty list
      channels:
        serverBootstrap.channels.length > 0
          ? serverBootstrap.channels
          : (cachedChannels ?? null),
      aiAvailable: serverBootstrap.ai_available,
      aiStatus: serverBootstrap.ai_status,
      searchStatus: serverBootstrap.search_status,
      // Use server snapshot; fall back to IDB snapshot if server has none
      snapshot: serverBootstrap.snapshot ?? cachedSnapshot,
      fromServer: true,
    };
  }

  // No server bootstrap — use IndexedDB entirely
  return {
    channels:
      cachedChannels && cachedChannels.length > 0 ? cachedChannels : null,
    aiAvailable: cachedMeta?.ai_available ?? null,
    aiStatus: cachedMeta?.ai_status ?? null,
    searchStatus: cachedMeta?.search_status ?? null,
    snapshot: cachedSnapshot,
    fromServer: false,
  };
}
