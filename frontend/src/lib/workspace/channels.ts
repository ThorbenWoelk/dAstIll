import type { Channel } from "$lib/types";
import type { ChannelSortMode } from "$lib/workspace/types";

export type ChannelReorderDirection = "up" | "down";
export type ChannelDropIndicatorEdge = "top" | "bottom";

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

export function canManualReorderChannels(
  sortMode: ChannelSortMode,
  query: string,
): boolean {
  return sortMode === "custom" && query.trim().length === 0;
}

export function moveChannelByStep(
  channels: Channel[],
  channelId: string,
  direction: ChannelReorderDirection,
): { channels: Channel[]; channelOrder: string[] } | null {
  const channelOrder = channelOrderFromList(channels);
  const currentIndex = channelOrder.indexOf(channelId);

  if (currentIndex < 0) {
    return null;
  }

  const targetIndex = direction === "up" ? currentIndex - 1 : currentIndex + 1;
  if (targetIndex < 0 || targetIndex >= channelOrder.length) {
    return null;
  }

  const nextOrder = [...channelOrder];
  nextOrder.splice(currentIndex, 1);
  nextOrder.splice(targetIndex, 0, channelId);

  const channelsById = new Map(
    channels.map((channel) => [channel.id, channel]),
  );

  return {
    channels: nextOrder
      .map((id) => channelsById.get(id))
      .filter((channel): channel is Channel => !!channel),
    channelOrder: nextOrder,
  };
}

export function resolveChannelDropIndicatorEdge(
  channelIds: string[],
  draggedChannelId: string | null,
  overChannelId: string | null,
): ChannelDropIndicatorEdge | null {
  if (
    !draggedChannelId ||
    !overChannelId ||
    draggedChannelId === overChannelId
  ) {
    return null;
  }

  const draggedIndex = channelIds.indexOf(draggedChannelId);
  const overIndex = channelIds.indexOf(overChannelId);

  if (draggedIndex < 0 || overIndex < 0) {
    return null;
  }

  return draggedIndex < overIndex ? "bottom" : "top";
}
