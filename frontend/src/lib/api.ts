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
  resolveApiUrl,
} from "./api-client";

export { API_BASE, BackendUnavailableError };

// Give the backend a small grace window to return a structured timeout response
// before the browser-level safeguard aborts the request.
const FORMAT_REQUEST_TIMEOUT_MS = 5 * 60 * 1000 + 15 * 1000;
const FORMAT_REQUEST_TIMEOUT_MESSAGE = "Formatting took too long to complete.";
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

function invalidateGetRequestCache(matcher: (path: string) => boolean) {
  for (const key of getResponseCache.keys()) {
    if (matcher(key)) {
      getResponseCache.delete(key);
    }
  }

  for (const key of inFlightGetRequests.keys()) {
    if (matcher(key)) {
      inFlightGetRequests.delete(key);
    }
  }
}

/**
 * Invalidates cache entries for a specific channel's reads (snapshot, videos,
 * sync-depth) and the workspace bootstrap. Used after per-channel mutations
 * such as refresh, backfill, and acknowledge toggle.
 */
function invalidateChannelReadCache(channelId: string) {
  invalidateGetRequestCache(
    (path) =>
      path.startsWith(`/api/channels/${channelId}/snapshot`) ||
      path.startsWith(`/api/channels/${channelId}/videos`) ||
      path.startsWith(`/api/channels/${channelId}/sync-depth`) ||
      path.startsWith("/api/workspace/bootstrap"),
  );
}

/**
 * Invalidates the channel list and workspace bootstrap. Used after mutations
 * that add channels (the new channel changes the global list).
 */
function invalidateChannelListCache() {
  invalidateGetRequestCache(
    (path) =>
      path === "/api/channels" || path.startsWith("/api/workspace/bootstrap"),
  );
}

/**
 * Invalidates all reads for a specific channel plus the channel list and
 * bootstrap. Used after mutations that update or delete a channel.
 */
function invalidateChannelAndListCache(channelId: string) {
  invalidateGetRequestCache(
    (path) =>
      path === "/api/channels" ||
      path.startsWith(`/api/channels/${channelId}/`) ||
      path.startsWith("/api/workspace/bootstrap"),
  );
}

/**
 * Invalidates the transcript content cache for a video, plus any channel
 * snapshots and video lists that reflect transcript_status.
 */
function invalidateVideoTranscriptCache(videoId: string) {
  invalidateGetRequestCache((path) => {
    if (path.startsWith(`/api/videos/${videoId}/transcript`)) return true;
    // Channel snapshots and video lists include transcript_status
    if (
      path.startsWith("/api/channels/") &&
      (path.includes("/snapshot") || path.includes("/videos?"))
    )
      return true;
    if (path.startsWith("/api/workspace/bootstrap")) return true;
    return false;
  });
}

/**
 * Invalidates the summary content cache for a video, plus any channel
 * snapshots and video lists that reflect summary_status.
 */
function invalidateVideoSummaryCache(videoId: string) {
  invalidateGetRequestCache((path) => {
    if (path.startsWith(`/api/videos/${videoId}/summary`)) return true;
    // Channel snapshots and video lists include summary_status
    if (
      path.startsWith("/api/channels/") &&
      (path.includes("/snapshot") || path.includes("/videos?"))
    )
      return true;
    if (path.startsWith("/api/workspace/bootstrap")) return true;
    return false;
  });
}

/**
 * Invalidates the video info cache. Info changes do not affect video status
 * fields in channel snapshots or video lists.
 */
function invalidateVideoInfoCache(videoId: string) {
  invalidateGetRequestCache((path) =>
    path.startsWith(`/api/videos/${videoId}/info`),
  );
}

/**
 * Invalidates highlight-related caches. Optionally scoped to a specific
 * video's per-video highlight list when the videoId is known.
 */
function invalidateHighlightCache(videoId?: string) {
  invalidateGetRequestCache((path) => {
    if (path === "/api/highlights" || path.startsWith("/api/highlights/"))
      return true;
    if (videoId && path.startsWith(`/api/videos/${videoId}/highlights`))
      return true;
    return false;
  });
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
    invalidateChannelListCache();
    return result;
  });
}

export function updateChannel(id: string, payload: Partial<Channel>) {
  return request<Channel>(`/api/channels/${id}`, {
    method: "PUT",
    body: JSON.stringify(payload),
  }).then((result) => {
    invalidateChannelAndListCache(id);
    return result;
  });
}

export function deleteChannel(id: string) {
  return request<void>(`/api/channels/${id}`, {
    method: "DELETE",
  }).then((result) => {
    invalidateChannelAndListCache(id);
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
    invalidateChannelReadCache(id);
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
    invalidateChannelReadCache(id);
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
    // The returned Video includes channel_id — invalidate only that channel's reads
    invalidateChannelReadCache(result.channel_id);
    return result;
  });
}

export function getVideo(videoId: string) {
  return cachedGetRequest<Video>(`/api/videos/${videoId}`);
}

export function getVideoInfo(videoId: string) {
  return cachedGetRequest<VideoInfo>(`/api/videos/${videoId}/info`);
}

export function ensureVideoInfo(videoId: string) {
  return request<VideoInfo>(`/api/videos/${videoId}/info/ensure`, {
    method: "POST",
  }).then((result) => {
    invalidateVideoInfoCache(videoId);
    return result;
  });
}

export function getTranscript(videoId: string) {
  return cachedGetRequest<Transcript>(`/api/videos/${videoId}/transcript`);
}

export function ensureTranscript(videoId: string) {
  return request<Transcript>(`/api/videos/${videoId}/transcript/ensure`, {
    method: "POST",
  }).then((result) => {
    invalidateVideoTranscriptCache(videoId);
    return result;
  });
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
    invalidateVideoTranscriptCache(videoId);
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

export function ensureSummary(videoId: string) {
  return request<Summary>(`/api/videos/${videoId}/summary/ensure`, {
    method: "POST",
  }).then((result) => {
    invalidateVideoSummaryCache(videoId);
    return result;
  });
}

export function updateSummary(videoId: string, content: string) {
  return request<Summary>(`/api/videos/${videoId}/summary`, {
    method: "PUT",
    body: JSON.stringify({ content }),
  }).then((result) => {
    invalidateVideoSummaryCache(videoId);
    return result;
  });
}

export function regenerateSummary(videoId: string) {
  return request<Summary>(`/api/videos/${videoId}/summary/regenerate`, {
    method: "POST",
  }).then((result) => {
    invalidateVideoSummaryCache(videoId);
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
    invalidateHighlightCache(videoId);
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
  return new EventSource(resolveApiUrl("/api/search/status/stream"));
}

export function deleteHighlight(highlightId: number) {
  return request<void>(`/api/highlights/${highlightId}`, {
    method: "DELETE",
  }).then((result) => {
    // videoId is not available from the highlight ID alone — invalidate all
    // highlight endpoints (grouped list and any per-video lists)
    invalidateHighlightCache();
    return result;
  });
}
