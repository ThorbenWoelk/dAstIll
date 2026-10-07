import type { HighlightChannelGroup } from "$lib/api";

/**
 * Keeps highlights whose text, story title, or channel name contain every
 * word of the query. Empty stories and channels drop out.
 */
export function filterHighlightGroups(
  groups: HighlightChannelGroup[],
  query: string,
): HighlightChannelGroup[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return groups;
  return dropEmpty(
    groups.map((channel) => ({
      ...channel,
      videos: channel.videos.map((video) => ({
        ...video,
        highlights: video.highlights.filter((highlight) => {
          const haystack =
            `${highlight.text} ${video.title} ${channel.channel_name}`.toLowerCase();
          return words.every((word) => haystack.includes(word));
        }),
      })),
    })),
  );
}

export function withoutHighlight(
  groups: HighlightChannelGroup[],
  highlightId: string,
): HighlightChannelGroup[] {
  return dropEmpty(
    groups.map((channel) => ({
      ...channel,
      videos: channel.videos.map((video) => ({
        ...video,
        highlights: video.highlights.filter((h) => h.id !== highlightId),
      })),
    })),
  );
}

export function countHighlights(groups: HighlightChannelGroup[]): number {
  return groups.reduce(
    (total, channel) =>
      total +
      channel.videos.reduce((sum, video) => sum + video.highlights.length, 0),
    0,
  );
}

function dropEmpty(groups: HighlightChannelGroup[]): HighlightChannelGroup[] {
  return groups
    .map((channel) => ({
      ...channel,
      videos: channel.videos.filter((video) => video.highlights.length > 0),
    }))
    .filter((channel) => channel.videos.length > 0);
}
