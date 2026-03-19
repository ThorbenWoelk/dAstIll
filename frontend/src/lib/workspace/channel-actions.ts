import type { Channel } from "$lib/types";

export function buildOptimisticChannel(input: string) {
  const trimmedInput = input.trim();
  const tempId = `temp-${Date.now()}`;

  const optimisticChannel: Channel = {
    id: tempId,
    name:
      trimmedInput.includes("youtube.com") || trimmedInput.includes("youtu.be")
        ? "Fetching Channel..."
        : trimmedInput,
    added_at: new Date().toISOString(),
  };

  return { optimisticChannel, tempId, trimmedInput };
}

export function replaceOptimisticChannel(
  channels: Channel[],
  tempId: string,
  nextChannel: Channel,
) {
  return channels.map((channel) =>
    channel.id === tempId ? nextChannel : channel,
  );
}

export function replaceOptimisticChannelId(
  channelOrder: string[],
  tempId: string,
  nextChannelId: string,
) {
  return channelOrder.map((channelId) =>
    channelId === tempId ? nextChannelId : channelId,
  );
}

export function removeChannelFromCollection(
  channels: Channel[],
  channelId: string,
) {
  return channels.filter((channel) => channel.id !== channelId);
}

export function removeChannelId(channelOrder: string[], channelId: string) {
  return channelOrder.filter((id) => id !== channelId);
}
