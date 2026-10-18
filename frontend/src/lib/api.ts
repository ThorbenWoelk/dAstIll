import type { Channel } from "$lib/bindings/Channel";
import type { CreateHighlightRequest } from "$lib/bindings/CreateHighlightRequest";
import type { Highlight } from "$lib/bindings/Highlight";
import type { HighlightChannelGroup } from "$lib/bindings/HighlightChannelGroup";
import { API_BASE } from "$lib/config";

export type {
  Channel,
  CreateHighlightRequest,
  Highlight,
  HighlightChannelGroup,
};

/** One summarized video as returned by `GET /api/mini`. */
export interface MiniSummaryItem {
  video_id: string;
  channel_id: string;
  channel_name: string;
  title: string;
  thumbnail_url: string | null;
  published_at: string | null;
  watch_url: string;
  summary_content: string;
  read: boolean;
}

/** `GET /api/mini` returns unread summaries for one channel at a time. */
export interface MiniReaderPayload {
  channels: Channel[];
  selected_channel_id: string | null;
  summaries: MiniSummaryItem[];
}

/** A story the reader finished, from `GET /api/mini/finished`. */
export interface FinishedSummaryItem extends MiniSummaryItem {
  finished_at: string;
}

export interface FinishedPage {
  stories: FinishedSummaryItem[];
  has_more: boolean;
}

export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

type TokenSource = () => Promise<string | null>;
type ReaderIdSource = () => string | null;

let readToken: TokenSource = async () => null;
let readReaderId: ReaderIdSource = () => null;

/** The session registers how to read the current Firebase ID token. */
export function useTokenSource(source: TokenSource) {
  readToken = source;
}

/** The session registers how to read the signed-in reader id. */
export function useReaderIdentity(source: ReaderIdSource) {
  readReaderId = source;
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const headers = new Headers(init.headers);
  if (init.body !== undefined) {
    headers.set("Content-Type", "application/json");
  }
  const readerId = readReaderId();
  const token = await readToken();
  // The token is whatever account is current after the await. Drop the call
  // when the reader changed so one account cannot write into another.
  if (readReaderId() !== readerId) {
    throw new ApiError("Please sign in again.", 401);
  }
  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }

  let response: Response;
  try {
    response = await fetch(`${API_BASE}${path}`, {
      ...init,
      headers,
      cache: "no-store",
    });
  } catch {
    throw new ApiError("Could not reach the server. Check your connection.", 0);
  }

  if (!response.ok) {
    const text = (await response.text()).trim();
    throw new ApiError(describeFailure(response.status, text), response.status);
  }
  if (response.status === 204) {
    return undefined as T;
  }
  return (await response.json()) as T;
}

function describeFailure(status: number, body: string): string {
  if (status === 401 || status === 403) {
    return "Please sign in again.";
  }
  if (status === 429) {
    return "Too many requests. Try again in a minute.";
  }
  if (status >= 500) {
    return "The server had a problem. Try again shortly.";
  }
  // Short plain-text messages from the backend are written for people.
  if (body && body.length <= 200 && !body.startsWith("{")) {
    return body;
  }
  return `Request failed (${status}).`;
}

export function fetchChannelSummaries(channelId?: string) {
  const query = channelId ? `?channel_id=${encodeURIComponent(channelId)}` : "";
  return request<MiniReaderPayload>(`/api/mini${query}`);
}

export function setStoryRead(videoId: string, read: boolean) {
  return request<{ video_id: string; read: boolean }>(
    `/api/mini/videos/${encodeURIComponent(videoId)}/read`,
    { method: "PUT", body: JSON.stringify({ read }) },
  );
}

/** Any story from a followed channel, read or unread. */
export function fetchStory(videoId: string) {
  return request<MiniSummaryItem>(
    `/api/mini/videos/${encodeURIComponent(videoId)}`,
  );
}

/** Finished stories, most recently finished first. */
export function fetchFinishedStories(offset: number, limit: number) {
  return request<FinishedPage>(
    `/api/mini/finished?offset=${offset}&limit=${limit}`,
  );
}

export function listStoryHighlights(videoId: string) {
  return request<Highlight[]>(
    `/api/videos/${encodeURIComponent(videoId)}/highlights`,
  );
}

export function createHighlight(
  videoId: string,
  draft: CreateHighlightRequest,
) {
  return request<Highlight>(
    `/api/videos/${encodeURIComponent(videoId)}/highlights`,
    { method: "POST", body: JSON.stringify(draft) },
  );
}

/** Every highlight, grouped by channel and then by video. */
export function listHighlights() {
  return request<HighlightChannelGroup[]>("/api/highlights");
}

export function deleteHighlight(highlightId: string) {
  return request<void>(`/api/highlights/${encodeURIComponent(highlightId)}`, {
    method: "DELETE",
  });
}

export function listChannels() {
  return request<Channel[]>("/api/channels");
}

export function subscribeToChannel(input: string) {
  return request<Channel>("/api/channels", {
    method: "POST",
    body: JSON.stringify({ input }),
  });
}

export function unsubscribeFromChannel(channelId: string) {
  return request<void>(`/api/channels/${encodeURIComponent(channelId)}`, {
    method: "DELETE",
  });
}
