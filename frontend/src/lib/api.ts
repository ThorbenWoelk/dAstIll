import type { Channel } from "$lib/bindings/Channel";
import { API_BASE } from "$lib/config";

export type { Channel };

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

let readToken: TokenSource = async () => null;

/** The session registers how to read the current Firebase ID token. */
export function useTokenSource(source: TokenSource) {
  readToken = source;
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const headers = new Headers(init.headers);
  if (init.body !== undefined) {
    headers.set("Content-Type", "application/json");
  }
  const token = await readToken();
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
