import type { Video } from "$lib/types";

export type ChannelViewCacheKeyPart =
  | string
  | number
  | boolean
  | null
  | undefined;

export interface ChannelSyncDepthState {
  earliest_sync_date: string | null;
  earliest_sync_date_user_set: boolean;
  derived_earliest_ready_date: string | null;
}

export function buildChannelViewCacheKey(
  channelId: string,
  ...parts: ChannelViewCacheKeyPart[]
) {
  return [channelId, ...parts.map((part) => String(part ?? ""))].join("::");
}

export function createChannelViewCache<TState>(
  cloneState: (state: TState) => TState,
) {
  const stateByKey = new Map<string, TState>();

  return {
    get(key: string) {
      const state = stateByKey.get(key);
      return state ? cloneState(state) : null;
    },
    set(key: string, state: TState) {
      stateByKey.set(key, cloneState(state));
    },
    delete(key: string) {
      stateByKey.delete(key);
    },
  };
}

export function cloneVideos(videos: Video[]) {
  return [...videos];
}

export function cloneSyncDepthState(syncDepth: ChannelSyncDepthState | null) {
  return syncDepth ? { ...syncDepth } : null;
}

export function cloneDate(value: Date | null) {
  return value ? new Date(value) : null;
}
