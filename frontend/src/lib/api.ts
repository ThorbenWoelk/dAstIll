import type {
  Channel,
  CleanTranscriptResponse,
  Summary,
  SyncDepth,
  Transcript,
  VideoInfo,
  Video,
  VideoTypeFilter,
} from "./types";

const API_BASE =
  (import.meta as { env?: { VITE_API_BASE?: string } }).env?.VITE_API_BASE ??
  "http://localhost:3001";

if (typeof window !== "undefined") {
  console.log("Using API_BASE:", API_BASE);
}
const FORMAT_REQUEST_TIMEOUT_MS = 5 * 60 * 1000;
const BACKEND_RETRY_DELAY_MS = 1500;

export class BackendUnavailableError extends Error {
  constructor(message = "Backend is unreachable.") {
    super(message);
    this.name = "BackendUnavailableError";
  }
}

function isAbortError(error: unknown): boolean {
  return error instanceof Error && error.name === "AbortError";
}

function createAbortError(): Error {
  if (typeof DOMException !== "undefined") {
    return new DOMException("The operation was aborted.", "AbortError");
  }
  const error = new Error("The operation was aborted.");
  error.name = "AbortError";
  return error;
}

function sleep(ms: number, signal?: AbortSignal) {
  return new Promise<void>((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      cleanup();
      resolve();
    }, ms);

    const onAbort = () => {
      clearTimeout(timeoutId);
      cleanup();
      reject(createAbortError());
    };

    const cleanup = () => signal?.removeEventListener("abort", onAbort);

    if (!signal) return;
    if (signal.aborted) {
      onAbort();
      return;
    }
    signal.addEventListener("abort", onAbort, { once: true });
  });
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`${API_BASE}${path}`, {
      headers: {
        "Content-Type": "application/json",
      },
      ...init,
    });
  } catch (error) {
    if (isAbortError(error)) {
      throw error;
    }
    throw new BackendUnavailableError();
  }

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `Request failed (${response.status})`);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}

export function listChannels() {
  return request<Channel[]>("/api/channels");
}

export function isAiAvailable() {
  return request<{ available: boolean }>("/api/health/ai");
}

export function isBackendUnavailableError(
  error: unknown,
): error is BackendUnavailableError {
  return error instanceof BackendUnavailableError;
}

export async function listChannelsWhenAvailable(options?: {
  retryDelayMs?: number;
}) {
  const retryDelayMs = options?.retryDelayMs ?? BACKEND_RETRY_DELAY_MS;

  for (;;) {
    try {
      return await listChannels();
    } catch (error) {
      if (!isBackendUnavailableError(error)) {
        throw error;
      }
      await sleep(retryDelayMs);
    }
  }
}

export function addChannel(input: string) {
  return request<Channel>("/api/channels", {
    method: "POST",
    body: JSON.stringify({ input }),
  });
}

export function updateChannel(id: string, payload: Partial<Channel>) {
  return request<Channel>(`/api/channels/${id}`, {
    method: "PUT",
    body: JSON.stringify(payload),
  });
}

export function deleteChannel(id: string) {
  return request<void>(`/api/channels/${id}`, {
    method: "DELETE",
  });
}

export function getChannelSyncDepth(channelId: string) {
  return request<SyncDepth>(`/api/channels/${channelId}/sync-depth`);
}

export function refreshChannel(id: string) {
  return request<{ videos_added: number }>(`/api/channels/${id}/refresh`, {
    method: "POST",
  });
}

export interface BackfillChannelVideosResponse {
  videos_added: number;
  fetched_count: number;
  exhausted: boolean;
}

export function backfillChannelVideos(id: string, limit = 15, until?: string) {
  const params = new URLSearchParams({
    limit: `${limit}`,
  });
  if (until) {
    params.append("until", until);
  }
  return request<BackfillChannelVideosResponse>(
    `/api/channels/${id}/backfill?${params.toString()}`,
    { method: "POST" },
  );
}

export function listVideos(
  channelId: string,
  limit = 12,
  offset = 0,
  videoType: VideoTypeFilter = "all",
  acknowledged?: boolean,
  queueOnly = false,
) {
  const params = new URLSearchParams({
    limit: `${limit}`,
    offset: `${offset}`,
    video_type: videoType,
  });
  if (acknowledged !== undefined) {
    params.append("acknowledged", acknowledged.toString());
  }
  if (queueOnly) {
    params.append("queue_only", "true");
  }
  return request<Video[]>(
    `/api/channels/${channelId}/videos?${params.toString()}`,
  );
}

export function updateAcknowledged(videoId: string, acknowledged: boolean) {
  return request<Video>(`/api/videos/${videoId}/acknowledged`, {
    method: "PUT",
    body: JSON.stringify({ acknowledged }),
  });
}

export function getVideoInfo(videoId: string) {
  return request<VideoInfo>(`/api/videos/${videoId}/info`);
}

export function getTranscript(videoId: string) {
  return request<Transcript>(`/api/videos/${videoId}/transcript`);
}

export function updateTranscript(videoId: string, content: string) {
  return request<Transcript>(`/api/videos/${videoId}/transcript`, {
    method: "PUT",
    body: JSON.stringify({ content }),
  });
}

export async function cleanTranscriptFormatting(
  videoId: string,
  content: string,
) {
  const controller = new AbortController();
  const timeoutId = setTimeout(
    () => controller.abort(),
    FORMAT_REQUEST_TIMEOUT_MS,
  );

  try {
    return await request<CleanTranscriptResponse>(
      `/api/videos/${videoId}/transcript/clean`,
      {
        method: "POST",
        body: JSON.stringify({ content }),
        signal: controller.signal,
      },
    );
  } catch (error) {
    if ((error as Error).name === "AbortError") {
      throw new Error("Formatting timed out after 5 minutes.");
    }
    throw error;
  } finally {
    clearTimeout(timeoutId);
  }
}

export function getSummary(videoId: string) {
  return request<Summary>(`/api/videos/${videoId}/summary`);
}

export function updateSummary(videoId: string, content: string) {
  return request<Summary>(`/api/videos/${videoId}/summary`, {
    method: "PUT",
    body: JSON.stringify({ content }),
  });
}

export function regenerateSummary(videoId: string) {
  return request<Summary>(`/api/videos/${videoId}/summary/regenerate`, {
    method: "POST",
  });
}
