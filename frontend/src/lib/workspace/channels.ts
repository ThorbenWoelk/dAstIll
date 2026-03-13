import type { Channel } from "$lib/types";
import type { ChannelSortMode } from "$lib/workspace/types";

export function filterChannels(
  channels: Channel[],
  query: string,
  sortMode: ChannelSortMode,
): Channel[] {
  let result = channels;

  if (query.trim()) {
    const normalizedQuery = query.trim().toLowerCase();
    result = result.filter(
      (channel) =>
        channel.name?.toLowerCase().includes(normalizedQuery) ||
        channel.handle?.toLowerCase().includes(normalizedQuery),
    );
  }

  if (sortMode === "alpha") {
    return [...result].sort((left, right) =>
      (left.name ?? "").localeCompare(right.name ?? ""),
    );
  }

  if (sortMode === "newest") {
    return [...result].sort((left, right) =>
      (right.added_at ?? "").localeCompare(left.added_at ?? ""),
    );
  }

  return result;
}

export function cycleChannelSortMode(
  current: ChannelSortMode,
): ChannelSortMode {
  if (current === "custom") return "alpha";
  if (current === "alpha") return "newest";
  return "custom";
}

export function channelOrderFromList(channels: Channel[]): string[] {
  return channels.map((channel) => channel.id);
}
