import type { Channel, SyncDepth } from "$lib/types";

export function toDateInputValue(value: string | null | undefined): string {
  if (!value) {
    return "";
  }

  return value.slice(0, 10);
}

export function toIsoDateStart(value: string): string {
  return `${value}T00:00:00.000Z`;
}

export function resolveSyncDateInputValue(
  channel: Channel,
  syncDepthValue: SyncDepth | null,
) {
  const effective = channel.earliest_sync_date_user_set
    ? channel.earliest_sync_date
    : (syncDepthValue?.derived_earliest_ready_date ??
      channel.earliest_sync_date ??
      null);

  return toDateInputValue(effective);
}
