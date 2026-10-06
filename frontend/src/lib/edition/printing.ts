import type { Channel, MiniReaderPayload, MiniSummaryItem } from "$lib/api";
import { collectUnreadStories, type Story } from "$lib/edition/stories";

export interface Edition {
  channels: Channel[];
  stories: Story[];
  /** Channels whose summaries could not be loaded this time. */
  missingChannelIds: string[];
}

type FetchChannelSummaries = (channelId?: string) => Promise<MiniReaderPayload>;

/** Keeps the backend to a few requests at a time; it runs on one instance. */
const PARALLEL_REQUESTS = 4;

async function mapWithLimit<T, R>(
  items: T[],
  limit: number,
  work: (item: T) => Promise<R>,
): Promise<R[]> {
  const results: R[] = new Array(items.length);
  let next = 0;
  async function worker() {
    while (next < items.length) {
      const index = next++;
      results[index] = await work(items[index]);
    }
  }
  await Promise.all(
    Array.from({ length: Math.min(limit, items.length) }, worker),
  );
  return results;
}

/**
 * Builds the front page from every subscribed channel.
 *
 * `GET /api/mini` answers for one channel at a time. The first call returns
 * the channel list plus the first channel's summaries; the remaining channels
 * are fetched in small parallel batches and merged by release date.
 */
export async function printEdition(
  fetchChannelSummaries: FetchChannelSummaries,
): Promise<Edition> {
  const first = await fetchChannelSummaries();
  const items: MiniSummaryItem[] = [...first.summaries];
  const remaining = first.channels.filter(
    (channel) => channel.id !== first.selected_channel_id,
  );

  const missingChannelIds: string[] = [];
  const pages = await mapWithLimit(remaining, PARALLEL_REQUESTS, (channel) =>
    fetchChannelSummaries(channel.id).catch(() => {
      missingChannelIds.push(channel.id);
      return null;
    }),
  );
  for (const page of pages) {
    if (page) items.push(...page.summaries);
  }

  return {
    channels: first.channels,
    stories: collectUnreadStories(items),
    missingChannelIds,
  };
}
