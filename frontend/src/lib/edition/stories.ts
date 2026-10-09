import type { Channel, MiniSummaryItem } from "$lib/api";

/** One unread summary, shaped for the reader. */
export interface Story {
  id: string;
  channelId: string;
  channelName: string;
  title: string;
  publishedAt: string | null;
  watchUrl: string;
  summary: string;
}

/** `"front-page"` shows every channel; anything else is a channel id. */
export type SectionId = string;
export const FRONT_PAGE: SectionId = "front-page";

export interface Section {
  id: SectionId;
  name: string;
  unread: number;
}

export function toStory(item: MiniSummaryItem): Story {
  return {
    id: item.video_id,
    channelId: item.channel_id,
    channelName: item.channel_name,
    title: item.title,
    publishedAt: item.published_at,
    watchUrl: item.watch_url,
    summary: item.summary_content,
  };
}

function publishedTime(story: Story): number {
  if (!story.publishedAt) return 0;
  const time = Date.parse(story.publishedAt);
  return Number.isNaN(time) ? 0 : time;
}

/** Newest first. Ties fall back to title so the order is stable. */
export function compareByRelease(left: Story, right: Story): number {
  return (
    publishedTime(right) - publishedTime(left) ||
    left.title.localeCompare(right.title) ||
    left.id.localeCompare(right.id)
  );
}

/** Unread stories from every channel, de-duplicated, newest first. */
export function collectUnreadStories(items: MiniSummaryItem[]): Story[] {
  const byId = new Map<string, Story>();
  for (const item of items) {
    if (item.read || !item.summary_content.trim()) continue;
    byId.set(item.video_id, toStory(item));
  }
  return [...byId.values()].sort(compareByRelease);
}

export function insertStory(stories: Story[], story: Story): Story[] {
  return [...stories.filter((s) => s.id !== story.id), story].sort(
    compareByRelease,
  );
}

export function storiesInSection(
  stories: Story[],
  section: SectionId,
): Story[] {
  if (section === FRONT_PAGE) return stories;
  return stories.filter((story) => story.channelId === section);
}

/** Front page first, then channels by name, each with its unread count. */
export function listSections(channels: Channel[], stories: Story[]): Section[] {
  const unreadByChannel = new Map<string, number>();
  for (const story of stories) {
    unreadByChannel.set(
      story.channelId,
      (unreadByChannel.get(story.channelId) ?? 0) + 1,
    );
  }
  const channelSections = [...channels]
    .sort((a, b) => a.name.localeCompare(b.name))
    .map((channel) => ({
      id: channel.id,
      name: channel.name,
      unread: unreadByChannel.get(channel.id) ?? 0,
    }));
  return [
    { id: FRONT_PAGE, name: "Front page", unread: stories.length },
    ...channelSections,
  ];
}

/**
 * The story on the page: the one the reader picked from "Also in this
 * edition" while it is still unread here, otherwise the newest one.
 */
export function chooseLeadStory(
  visible: Story[],
  pickedId: string | null,
): Story | null {
  if (pickedId) {
    const picked = visible.find((story) => story.id === pickedId);
    if (picked) return picked;
  }
  return visible[0] ?? null;
}

/** "next" is the older story after the lead, "previous" the newer one. */
export type TurnDirection = "next" | "previous";

/** Where the lead sits in its section, and the stories either side of it. */
export interface StoryPosition {
  /** Counted from 1; 0 when the story is not in the section. */
  number: number;
  total: number;
  previous: Story | null;
  next: Story | null;
}

export function locateStory(
  visible: Story[],
  storyId: string | null,
): StoryPosition {
  const index = storyId ? visible.findIndex((s) => s.id === storyId) : -1;
  if (index < 0) {
    return { number: 0, total: visible.length, previous: null, next: null };
  }
  return {
    number: index + 1,
    total: visible.length,
    previous: visible[index - 1] ?? null,
    next: visible[index + 1] ?? null,
  };
}

/**
 * The story that takes a finished story's place: the one after it, or the
 * one before it when it was the last. Null means "lead with the newest",
 * which is also the one after the first story.
 */
export function storyAfterFinishing(
  visible: Story[],
  storyId: string,
): string | null {
  const index = visible.findIndex((s) => s.id === storyId);
  if (index <= 0) return null;
  return (visible[index + 1] ?? visible[index - 1]).id;
}

const WORDS_PER_MINUTE = 220;

/** Hyphenated words count once; list dashes and markup do not count. */
export function readingMinutes(markdown: string): number {
  const words = markdown
    .replace(/[#*_`>[\]()]/g, " ")
    .split(/\s+/)
    .filter((word) => /[\p{L}\p{N}]/u.test(word)).length;
  return Math.max(1, Math.round(words / WORDS_PER_MINUTE));
}

const DAY_MS = 24 * 60 * 60 * 1000;
const SHORT_MONTHS = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sept",
  "Oct",
  "Nov",
  "Dec",
];

function startOfDay(date: Date): number {
  return new Date(
    date.getFullYear(),
    date.getMonth(),
    date.getDate(),
  ).getTime();
}

/** "Today", "Yesterday", a weekday within the last week, else a date. */
export function describeReleaseDay(
  publishedAt: string | null,
  now: Date = new Date(),
): string {
  if (!publishedAt) return "";
  const published = new Date(publishedAt);
  if (Number.isNaN(published.getTime())) return "";
  const daysAgo = Math.round(
    (startOfDay(now) - startOfDay(published)) / DAY_MS,
  );
  if (daysAgo <= 0) return "Today";
  if (daysAgo === 1) return "Yesterday";
  if (daysAgo < 7) {
    return published.toLocaleDateString("en-GB", { weekday: "long" });
  }
  const shortDate = `${published.getDate()} ${SHORT_MONTHS[published.getMonth()]}`;
  return published.getFullYear() === now.getFullYear()
    ? shortDate
    : `${shortDate} ${published.getFullYear()}`;
}

/** Masthead dateline, e.g. "Tuesday, 6 October 2026". */
export function describeEditionDate(now: Date = new Date()): string {
  return now.toLocaleDateString("en-GB", {
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  });
}

/** Short dateline for narrow screens, e.g. "Tue 6 Oct". */
export function describeShortEditionDate(now: Date = new Date()): string {
  return now.toLocaleDateString("en-GB", {
    weekday: "short",
    day: "numeric",
    month: "short",
  });
}

export function describeStoriesLeft(count: number): string {
  if (count === 0) return "No stories left";
  return count === 1 ? "1 story left" : `${count} stories left`;
}
