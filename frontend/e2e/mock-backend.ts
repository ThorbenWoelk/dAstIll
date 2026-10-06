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

export interface MockBackend {
  channels: MockChannel[];
  stories: MockStory[];
  read: Set<string>;
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
  data: { channels: MockChannel[]; stories: MockStory[] },
  options: { signedIn?: boolean } = {},
): Promise<MockBackend> {
  const backend: MockBackend = {
    channels: [...data.channels],
    stories: [...data.stories],
    read: new Set(),
    readRequests: [],
    miniRequests: [],
  };

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

  await page.goto("/");
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
