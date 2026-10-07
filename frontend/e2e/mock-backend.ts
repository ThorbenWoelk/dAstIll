import type { Page, Route } from "@playwright/test";

/** The local-session key read by `src/lib/session.svelte.ts` in dev builds. */
const LOCAL_SESSION_KEY = "__dastill_e2e_auth";

export interface MockChannel {
  id: string;
  name: string;
  handle?: string | null;
}

export interface MockStory {
  id: string;
  channelId: string;
  title: string;
  publishedAt: string;
  summary?: string;
}

export function summaryFor(title: string): string {
  return `## At a glance
- The main point of ${title}.
- A second point.

## Overview
Standfirst for ${title}.

## Key Points
- **First idea**: Detail about ${title}.
- **Second idea**: More detail.

## Takeaways
- Something to remember.
`;
}

export interface MockHighlight {
  id: string;
  video_id: string;
  source: "summary" | "transcript";
  text: string;
  prefix_context: string;
  suffix_context: string;
  created_at: string;
}

export interface MockBackend {
  channels: MockChannel[];
  stories: MockStory[];
  /** Read story ids, in the order they were finished. */
  read: Set<string>;
  highlights: MockHighlight[];
  deletedHighlightIds: string[];
  readRequests: { id: string; read: boolean }[];
  miniRequests: (string | null)[];
}

function channelJson(channel: MockChannel) {
  return {
    id: channel.id,
    handle: channel.handle ?? null,
    name: channel.name,
    thumbnail_url: null,
    added_at: "2026-01-01T00:00:00Z",
    earliest_sync_date: null,
    earliest_sync_date_user_set: false,
  };
}

/** Signs in a fake reader and answers the API from in-memory data. */
export async function openPaper(
  page: Page,
  data: {
    channels: MockChannel[];
    stories: MockStory[];
    read?: string[];
    highlights?: MockHighlight[];
  },
  options: { signedIn?: boolean; path?: string } = {},
): Promise<MockBackend> {
  const backend: MockBackend = {
    channels: [...data.channels],
    stories: [...data.stories],
    read: new Set(data.read ?? []),
    highlights: [...(data.highlights ?? [])],
    deletedHighlightIds: [],
    readRequests: [],
    miniRequests: [],
  };
  let nextHighlightId = 90_071_992_547_409_930n;

  const storyJson = (story: MockStory) => ({
    video_id: story.id,
    channel_id: story.channelId,
    channel_name:
      backend.channels.find((c) => c.id === story.channelId)?.name ?? "",
    title: story.title,
    thumbnail_url: null,
    published_at: story.publishedAt,
    watch_url: `https://www.youtube.com/watch?v=${story.id}`,
    summary_content: story.summary ?? summaryFor(story.title),
    read: backend.read.has(story.id),
  });

  if (options.signedIn !== false) {
    await page.addInitScript((key) => {
      window.localStorage.setItem(
        key,
        JSON.stringify({
          userId: "reader-1",
          email: "reader@example.com",
          token: "test-token",
        }),
      );
    }, LOCAL_SESSION_KEY);
  }

  await page.route("**/api/mini**", async (route: Route) => {
    const url = new URL(route.request().url());
    const readMatch = /\/api\/mini\/videos\/([^/]+)\/read$/.exec(url.pathname);
    if (readMatch && route.request().method() === "PUT") {
      const id = decodeURIComponent(readMatch[1]);
      const { read } = route.request().postDataJSON() as { read: boolean };
      backend.readRequests.push({ id, read });
      if (read) backend.read.add(id);
      else backend.read.delete(id);
      return route.fulfill({
        json: { video_id: id, read, updated_at: new Date().toISOString() },
      });
    }

    if (url.pathname === "/api/mini/finished") {
      const finished = [...backend.read]
        .reverse()
        .map((id) => backend.stories.find((story) => story.id === id))
        .filter((story): story is MockStory => Boolean(story));
      const offset = Number(url.searchParams.get("offset") ?? 0);
      const limit = Number(url.searchParams.get("limit") ?? 30);
      return route.fulfill({
        json: {
          stories: finished.slice(offset, offset + limit).map((story) => ({
            ...storyJson(story),
            finished_at: new Date().toISOString(),
          })),
          has_more: finished.length > offset + limit,
        },
      });
    }

    const storyMatch = /\/api\/mini\/videos\/([^/]+)$/.exec(url.pathname);
    if (storyMatch) {
      const story = backend.stories.find(
        (s) => s.id === decodeURIComponent(storyMatch[1]),
      );
      return story
        ? route.fulfill({ json: storyJson(story) })
        : route.fulfill({ status: 404, body: "Story not found" });
    }

    const requested = url.searchParams.get("channel_id");
    backend.miniRequests.push(requested);
    const selected =
      backend.channels.find((c) => c.id === requested) ?? backend.channels[0];
    const summaries = selected
      ? backend.stories
          .filter((s) => s.channelId === selected.id && !backend.read.has(s.id))
          .map((s) => ({
            video_id: s.id,
            channel_id: s.channelId,
            channel_name: selected.name,
            title: s.title,
            thumbnail_url: null,
            published_at: s.publishedAt,
            watch_url: `https://www.youtube.com/watch?v=${s.id}`,
            summary_content: s.summary ?? summaryFor(s.title),
            read: false,
          }))
      : [];
    return route.fulfill({
      json: {
        channels: backend.channels.map(channelJson),
        selected_channel_id: selected?.id ?? null,
        summaries,
      },
    });
  });

  await page.route("**/api/videos/*/highlights", async (route: Route) => {
    const request = route.request();
    const videoId = decodeURIComponent(
      new URL(request.url()).pathname.split("/")[3],
    );
    if (request.method() === "POST") {
      const draft = request.postDataJSON() as Omit<
        MockHighlight,
        "id" | "video_id" | "created_at"
      >;
      const saved: MockHighlight = {
        ...draft,
        id: String(nextHighlightId++),
        video_id: videoId,
        created_at: new Date().toISOString(),
      };
      backend.highlights.push(saved);
      return route.fulfill({ status: 201, json: saved });
    }
    return route.fulfill({
      json: backend.highlights.filter((h) => h.video_id === videoId),
    });
  });

  await page.route("**/api/highlights**", async (route: Route) => {
    const request = route.request();
    const url = new URL(request.url());
    if (request.method() === "DELETE") {
      const id = decodeURIComponent(url.pathname.split("/").pop() ?? "");
      backend.deletedHighlightIds.push(id);
      backend.highlights = backend.highlights.filter((h) => h.id !== id);
      return route.fulfill({ status: 204 });
    }
    const groups = backend.channels
      .map((channel) => ({
        source_id: channel.id,
        channel_id: channel.id,
        provider: "you_tube",
        source_kind: "you_tube_channel",
        channel_name: channel.name,
        channel_thumbnail_url: null,
        videos: backend.stories
          .filter((story) => story.channelId === channel.id)
          .map((story) => ({
            source_id: channel.id,
            video_id: story.id,
            item_id: story.id,
            provider: "you_tube",
            item_kind: "video",
            title: story.title,
            thumbnail_url: null,
            published_at: story.publishedAt,
            highlights: backend.highlights.filter(
              (h) => h.video_id === story.id,
            ),
          }))
          .filter((video) => video.highlights.length > 0),
      }))
      .filter((group) => group.videos.length > 0);
    return route.fulfill({ json: groups });
  });

  await page.route("**/api/channels**", async (route: Route) => {
    const request = route.request();
    const url = new URL(request.url());
    if (request.method() === "GET" && url.pathname === "/api/channels") {
      return route.fulfill({ json: backend.channels.map(channelJson) });
    }
    if (request.method() === "POST") {
      const { input } = request.postDataJSON() as { input: string };
      const channel = {
        id: `id-${input}`,
        name: input.replace(/^@/, ""),
        handle: input,
      };
      backend.channels.push(channel);
      return route.fulfill({ json: channelJson(channel) });
    }
    if (request.method() === "DELETE") {
      const id = decodeURIComponent(url.pathname.split("/").pop() ?? "");
      backend.channels = backend.channels.filter((c) => c.id !== id);
      return route.fulfill({ status: 204 });
    }
    return route.fallback();
  });

  await page.goto(options.path ?? "/");
  return backend;
}

export const CHANNELS: MockChannel[] = [
  { id: "science", name: "Slow Science", handle: "@slowscience" },
  { id: "build", name: "The Long Build", handle: "@longbuild" },
];

export const STORIES: MockStory[] = [
  {
    id: "v-new",
    channelId: "build",
    title: "Why small teams ship calmer software",
    publishedAt: "2026-10-06T07:00:00Z",
  },
  {
    id: "v-mid",
    channelId: "science",
    title: "What sleep pressure actually is",
    publishedAt: "2026-10-05T07:00:00Z",
  },
  {
    id: "v-old",
    channelId: "build",
    title: "Postgres was enough",
    publishedAt: "2026-10-02T07:00:00Z",
  },
];
