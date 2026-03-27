function normalizeApiBase(value?: string) {
  const normalized = value?.trim();
  if (!normalized) {
    return "";
  }

  return normalized.endsWith("/") ? normalized.slice(0, -1) : normalized;
}

export const API_BASE = normalizeApiBase(
  (import.meta as { env?: { VITE_API_BASE?: string } }).env?.VITE_API_BASE,
);

export class BackendUnavailableError extends Error {
  constructor(message = "Backend is unreachable.") {
    super(message);
    this.name = "BackendUnavailableError";
  }
}

/** Thrown when the backend returns HTTP 429 (e.g. expensive-operation rate limit). */
export class RateLimitedError extends Error {
  readonly status = 429;
  readonly retryAfterMs: number;

  constructor(message: string, retryAfterMs: number) {
    super(message);
    this.name = "RateLimitedError";
    this.retryAfterMs = retryAfterMs;
  }
}

export function isAbortError(error: unknown): boolean {
  return error instanceof Error && error.name === "AbortError";
}

export function createAbortError(): Error {
  if (typeof DOMException !== "undefined") {
    return new DOMException("The operation was aborted.", "AbortError");
  }
  const error = new Error("The operation was aborted.");
  error.name = "AbortError";
  return error;
}

export function resolveApiUrl(path: string): string {
  return `${API_BASE}${path}`;
}

export async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const method = (init?.method ?? "GET").toUpperCase();
  // Backend sets short Cache-Control on channel snapshots/lists; the browser HTTP
  // cache is separate from our JS GET cache. Without this, a refetch after
  // mark-as-read can briefly serve a stale cached GET and undo client updates.
  const cache: RequestCache | undefined =
    init?.cache !== undefined
      ? init.cache
      : method === "GET" || method === "HEAD"
        ? "no-store"
        : undefined;

  let response: Response;
  try {
    response = await fetch(resolveApiUrl(path), {
      headers: {
        "Content-Type": "application/json",
      },
      ...init,
      cache,
    });
  } catch (error) {
    if (isAbortError(error)) {
      throw error;
    }
    console.error(`[API Fetch Failure] ${method} ${path}`, error);
    throw new BackendUnavailableError();
  }

  if (!response.ok) {
    if (response.status === 429) {
      const retryAfterHeader = response.headers.get("Retry-After");
      const retryAfterSec = retryAfterHeader
        ? Number.parseInt(retryAfterHeader, 10)
        : NaN;
      const retryAfterMs =
        Number.isFinite(retryAfterSec) && retryAfterSec > 0
          ? retryAfterSec * 1000
          : 60_000;
      const message = await response.text();
      console.warn(`[API Rate Limited] ${method} ${path}`, {
        status: 429,
        retryAfterMs,
        message,
      });
      throw new RateLimitedError(
        message.trim() || "Rate limit exceeded",
        retryAfterMs,
      );
    }
    const message = await response.text();
    console.error(`[API Error] ${method} ${path}`, {
      status: response.status,
    });
    throw new Error(message || `Request failed (${response.status})`);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}
