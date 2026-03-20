import type {
  AiHealthResponse,
  Channel,
  ChannelSnapshot,
  CleanTranscriptResponse,
  CreateHighlightRequest,
  Highlight,
  HighlightChannelGroup,
  QueueTab,
  SearchResponse,
  SearchSourceFilter,
  SearchStatus,
  Summary,
  SyncDepth,
  Transcript,
  TranscriptRenderMode,
  Video,
  VideoInfo,
  VideoTypeFilter,
  WorkspaceBootstrap,
} from "./types";
import {
  API_BASE,
  BackendUnavailableError,
  createAbortError,
  isAbortError,
  request,
} from "./api-client";

export { API_BASE, BackendUnavailableError };

// Give the backend a small grace window to return a structured timeout response
// before the browser-level safeguard aborts the request.
const FORMAT_REQUEST_TIMEOUT_MS = 5 * 60 * 1000 + 15 * 1000;
const FORMAT_REQUEST_TIMEOUT_MESSAGE =
  "Formatting took too long to complete.";
const BACKEND_RETRY_DELAY_MS = 1500;
const GET_CACHE_TTL_MS = 30 * 1000;
const getResponseCache = new Map<
  string,
  { expiresAt: number; value: unknown }
>();
const inFlightGetRequests = new Map<string, Promise<unknown>>();

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

function clearGetRequestCache() {
  getResponseCache.clear();
  inFlightGetRequests.clear();
}

export function resetApiCacheForTests() {
  clearGetRequestCache();
}

async function cachedGetRequest<T>(
  path: string,
  options?: {
    bypassCache?: boolean;
  },
): Promise<T> {
  if (options?.bypassCache) {
    return request<T>(path);
  }

  const now = Date.now();
  const cached = getResponseCache.get(path);
  if (cached && cached.expiresAt > now) {
    return cached.value as T;
  }

  const inFlight = inFlightGetRequests.get(path);
  if (inFlight) {
    return (await inFlight) as T;
  }

  const pendingRequest = request<T>(path)
    .then((value) => {
      getResponseCache.set(path, {
        expiresAt: Date.now() + GET_CACHE_TTL_MS,
        value,
      });
      inFlightGetRequests.delete(path);
      return value;
    })
    .catch((error) => {
      inFlightGetRequests.delete(path);
      throw error;
    });

  inFlightGetRequests.set(path, pendingRequest as Promise<unknown>);
  return pendingRequest;
}

export function listChannels() {
  return cachedGetRequest<Channel[]>("/api/channels");
}

interface VideoQueryOptions {
  limit?: number;
  offset?: number;
  videoType?: VideoTypeFilter;
  acknowledged?: boolean;
  queueOnly?: boolean;
  queueTab?: QueueTab;
}

function appendVideoQueryParams(
  params: URLSearchParams,
  options?: VideoQueryOptions,
) {
  if (!options) {
    return;
  }

  if (options.limit !== undefined) {
    params.set("limit", `${options.limit}`);
  }
  if (options.offset !== undefined) {
    params.set("offset", `${options.offset}`);
  }
  if (options.videoType) {
    params.set("video_type", options.videoType);
  }
  if (options.acknowledged !== undefined) {
    params.set("acknowledged", options.acknowledged.toString());
  }
  if (options.queueOnly) {
    params.set("queue_only", "true");
  }
  if (options.queueTab) {
    params.set("queue_tab", options.queueTab);
  }
}

export function getWorkspaceBootstrap(
  options?: VideoQueryOptions & {
    selectedChannelId?: string | null;
    bypassCache?: boolean;
  },
) {
  const params = new URLSearchParams();
  if (options?.selectedChannelId) {
    params.set("selected_channel_id", options.selectedChannelId);
  }
  appendVideoQueryParams(params, options);

  return cachedGetRequest<WorkspaceBootstrap>(
    `/api/workspace/bootstrap${params.size ? `?${params.toString()}` : ""}`,
    {
      bypassCache: options?.bypassCache,
    },
  );
}

export function getChannelSnapshot(
  channelId: string,
  options?: VideoQueryOptions & { bypassCache?: boolean },
) {
  const params = new URLSearchParams();
  appendVideoQueryParams(params, options);

  return cachedGetRequest<ChannelSnapshot>(
    `/api/channels/${channelId}/snapshot${params.size ? `?${params.toString()}` : ""}`,
    {
      bypassCache: options?.bypassCache,
    },
  );
}

export function isAiAvailable() {
  return cachedGetRequest<AiHealthResponse>("/api/health/ai");
}

export function isBackendUnavailableError(
  error: unknown,
): error is BackendUnavailableError {
  return error instanceof BackendUnavailableError;
}

async function retryWhenBackendAvailable<T>(
  loader: () => Promise<T>,
  options?: {
    retryDelayMs?: number;
  },
) {
  const retryDelayMs = options?.retryDelayMs ?? BACKEND_RETRY_DELAY_MS;

  for (;;) {
    try {
      return await loader();
    } catch (error) {
      if (!isBackendUnavailableError(error)) {
        throw error;
      }
      await sleep(retryDelayMs);
    }
  }
}

export function listChannelsWhenAvailable(options?: { retryDelayMs?: number }) {
  return retryWhenBackendAvailable(() => listChannels(), options);
}

export function getWorkspaceBootstrapWhenAvailable(
  options?: (VideoQueryOptions & {
    selectedChannelId?: string | null;
  }) & {
    retryDelayMs?: number;
  },
) {
  return retryWhenBackendAvailable(
    () => getWorkspaceBootstrap(options),
    options,
  );
}

export function addChannel(input: string) {
  return request<Channel>("/api/channels", {
    method: "POST",
    body: JSON.stringify({ input }),
  }).then((result) => {
    clearGetRequestCache();
    return result;
  });
}

export function updateChannel(id: string, payload: Partial<Channel>) {
  return request<Channel>(`/api/channels/${id}`, {
    method: "PUT",
    body: JSON.stringify(payload),
  }).then((result) => {
    clearGetRequestCache();
    return result;
  });
}

export function deleteChannel(id: string) {
  return request<void>(`/api/channels/${id}`, {
    method: "DELETE",
  }).then((result) => {
    clearGetRequestCache();
    return result;
  });
}

export function getChannelSyncDepth(channelId: string) {
  return cachedGetRequest<SyncDepth>(`/api/channels/${channelId}/sync-depth`);
}

export function refreshChannel(id: string) {
  return request<{ videos_added: number }>(`/api/channels/${id}/refresh`, {
    method: "POST",
  }).then((result) => {
    clearGetRequestCache();
    return result;
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
  ).then((result) => {
    clearGetRequestCache();
    return result;
  });
}

export function listVideos(
  channelId: string,
  limit = 12,
  offset = 0,
  videoType: VideoTypeFilter = "all",
  acknowledged?: boolean,
  queueOnly = false,
  queueTab?: QueueTab,
) {
  const params = new URLSearchParams({
    limit: `${limit}`,
    offset: `${offset}`,
  });
  appendVideoQueryParams(params, {
    videoType,
    acknowledged,
    queueOnly,
    queueTab,
  });
  return cachedGetRequest<Video[]>(
    `/api/channels/${channelId}/videos?${params.toString()}`,
  );
}

export function updateAcknowledged(videoId: string, acknowledged: boolean) {
  return request<Video>(`/api/videos/${videoId}/acknowledged`, {
    method: "PUT",
    body: JSON.stringify({ acknowledged }),
  }).then((result) => {
    clearGetRequestCache();
    return result;
  });
}

export function getVideoInfo(videoId: string) {
  return cachedGetRequest<VideoInfo>(`/api/videos/${videoId}/info`);
}

export function getTranscript(videoId: string) {
  return cachedGetRequest<Transcript>(`/api/videos/${videoId}/transcript`);
}

export function updateTranscript(
  videoId: string,
  content: string,
  renderMode: TranscriptRenderMode,
) {
  return request<Transcript>(`/api/videos/${videoId}/transcript`, {
    method: "PUT",
    body: JSON.stringify({ content, render_mode: renderMode }),
  }).then((result) => {
    clearGetRequestCache();
    return result;
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
      throw new Error(FORMAT_REQUEST_TIMEOUT_MESSAGE);
    }
    throw error;
  } finally {
    clearTimeout(timeoutId);
  }
}

export function getSummary(videoId: string) {
  return cachedGetRequest<Summary>(`/api/videos/${videoId}/summary`);
}

export function updateSummary(videoId: string, content: string) {
  return request<Summary>(`/api/videos/${videoId}/summary`, {
    method: "PUT",
    body: JSON.stringify({ content }),
  }).then((result) => {
    clearGetRequestCache();
    return result;
  });
}

export function regenerateSummary(videoId: string) {
  return request<Summary>(`/api/videos/${videoId}/summary/regenerate`, {
    method: "POST",
  }).then((result) => {
    clearGetRequestCache();
    return result;
  });
}

export function listHighlights() {
  return cachedGetRequest<HighlightChannelGroup[]>("/api/highlights");
}

export function getVideoHighlights(videoId: string) {
  return cachedGetRequest<Highlight[]>(`/api/videos/${videoId}/highlights`);
}

export function createHighlight(
  videoId: string,
  payload: CreateHighlightRequest,
) {
  return request<Highlight>(`/api/videos/${videoId}/highlights`, {
    method: "POST",
    body: JSON.stringify(payload),
  }).then((result) => {
    clearGetRequestCache();
    return result;
  });
}

export function searchContent(
  query: string,
  options?: {
    source?: SearchSourceFilter;
    channelId?: string | null;
    limit?: number;
    mode?: "keyword" | "hybrid" | "semantic";
    signal?: AbortSignal;
  },
) {
  const params = new URLSearchParams({
    q: query,
  });
  if (options?.source && options.source !== "all") {
    params.set("source", options.source);
  }
  if (options?.channelId) {
    params.set("channel_id", options.channelId);
  }
  if (options?.limit !== undefined) {
    params.set("limit", `${options.limit}`);
  }
  if (options?.mode) {
    params.set("mode", options.mode);
  }
  return request<SearchResponse>(`/api/search?${params.toString()}`, {
    signal: options?.signal,
  });
}

export function getSearchStatus(options?: { bypassCache?: boolean }) {
  return cachedGetRequest<SearchStatus>("/api/search/status", options);
}

export function openSearchStatusStream() {
  const apiBase = API_BASE.endsWith("/") ? API_BASE.slice(0, -1) : API_BASE;
  return new EventSource(`${apiBase}/api/search/status/stream`);
}

export function deleteHighlight(highlightId: number) {
  return request<void>(`/api/highlights/${highlightId}`, {
    method: "DELETE",
  }).then((result) => {
    clearGetRequestCache();
    return result;
  });
}
