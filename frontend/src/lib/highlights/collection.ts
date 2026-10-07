import type { Highlight, HighlightChannelGroup } from "$lib/api";

/** One story with the passages highlighted in it. */
export interface HighlightedStory {
  videoId: string;
  title: string;
  channelName: string;
  publishedAt: string;
  highlights: Highlight[];
}

function publishedTime(story: HighlightedStory): number {
  const time = Date.parse(story.publishedAt);
  return Number.isNaN(time) ? 0 : time;
}

/**
 * Flattens the per-channel groups from the API into one list, newest
 * story first, so a passage marked in today's paper sits at the top.
 */
export function listHighlightedStories(
  groups: HighlightChannelGroup[],
): HighlightedStory[] {
  return groups
    .flatMap((channel) =>
      channel.videos.map((video) => ({
        videoId: video.video_id,
        title: video.title,
        channelName: channel.channel_name,
        publishedAt: video.published_at,
        highlights: video.highlights,
      })),
    )
    .filter((story) => story.highlights.length > 0)
    .sort(
      (left, right) =>
        publishedTime(right) - publishedTime(left) ||
        left.title.localeCompare(right.title) ||
        left.videoId.localeCompare(right.videoId),
    );
}

/**
 * Keeps highlights whose text, story title, or channel name contain every
 * word of the query. Stories left without highlights drop out.
 */
export function filterHighlightedStories(
  stories: HighlightedStory[],
  query: string,
): HighlightedStory[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return stories;
  return stories
    .map((story) => ({
      ...story,
      highlights: story.highlights.filter((highlight) => {
        const haystack =
          `${highlight.text} ${story.title} ${story.channelName}`.toLowerCase();
        return words.every((word) => haystack.includes(word));
      }),
    }))
    .filter((story) => story.highlights.length > 0);
}

export function withoutHighlight(
  stories: HighlightedStory[],
  highlightId: string,
): HighlightedStory[] {
  return stories
    .map((story) => ({
      ...story,
      highlights: story.highlights.filter((h) => h.id !== highlightId),
    }))
    .filter((story) => story.highlights.length > 0);
}

export function countHighlights(stories: HighlightedStory[]): number {
  return stories.reduce((total, story) => total + story.highlights.length, 0);
}
