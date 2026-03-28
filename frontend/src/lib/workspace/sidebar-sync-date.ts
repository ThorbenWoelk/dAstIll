import type { Channel, SyncDepth } from "$lib/types";

export function toDateInputValue(value: string | null | undefined): string {
  if (!value) {
    return "";
  }

  return value.slice(0, 10);
}

/**
 * First day of the current UTC month as `yyyy-mm-dd`, matching backend
 * `default_earliest_sync_date_floor` (used when the subscription has no floor yet).
 */
export function defaultEarliestSyncFloorDateInputValue(
  now: Date = new Date(),
): string {
  const y = now.getUTCFullYear();
  const m = now.getUTCMonth() + 1;
  return `${y}-${String(m).padStart(2, "0")}-01`;
}

export function toIsoDateStart(value: string): string {
  return `${value}T00:00:00.000Z`;
}

export function resolveSyncDateInputValue(
  channel: Channel | null,
  syncDepthValue: SyncDepth | null,
  now: Date = new Date(),
): string {
  const iso =
    channel?.earliest_sync_date ?? syncDepthValue?.earliest_sync_date ?? null;
  if (iso) {
    return toDateInputValue(iso);
  }
  return defaultEarliestSyncFloorDateInputValue(now);
}
