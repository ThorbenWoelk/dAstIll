import type { Channel, Video } from "$lib/types";

/**
 * Returns a new video list with the given video's `acknowledged` field flipped
 * to the given value. Used for the optimistic acknowledge toggle — call this
 * before the API request and restore the previous list on error.
 */
export function applyOptimisticAcknowledge(
  videos: Video[],
  videoId: string,
  acknowledged: boolean,
): Video[] {
  return videos.map((v) => (v.id === videoId ? { ...v, acknowledged } : v));
}

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
