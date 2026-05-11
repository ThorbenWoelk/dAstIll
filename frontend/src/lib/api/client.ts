import { getCurrentAuthToken } from "$lib/auth/token";
import { normalizeUserErrorMessage } from "$lib/api/user-error";

const TAURI_ANDROID_DEV_API_BASE = "http://127.0.0.1:3544";

export function normalizeApiBase(value?: string) {
  const normalized = value?.trim();
  if (!normalized) {
    return "";
  }

  return normalized.endsWith("/") ? normalized.slice(0, -1) : normalized;
}

const BUILD_ENV = (
  import.meta as {
    env?: { PUBLIC_API_BASE?: string; VITE_API_BASE?: string };
  }
).env;

export const API_BASE = normalizeApiBase(
  BUILD_ENV?.PUBLIC_API_BASE || BUILD_ENV?.VITE_API_BASE,
);

export function resolveImplicitApiBase(
  apiBase = API_BASE,
  options?: { currentOrigin?: string; userAgent?: string },
): string {
  if (apiBase) {
    return apiBase;
  }

  const currentOrigin =
    options?.currentOrigin ??
    (typeof window !== "undefined" ? window.location.origin : undefined);
  const userAgent =
    options?.userAgent ??
    (typeof navigator !== "undefined" ? navigator.userAgent : undefined);

  if (
    currentOrigin === "http://tauri.localhost" &&
    userAgent &&
    /android/i.test(userAgent)
  ) {
    return TAURI_ANDROID_DEV_API_BASE;
  }

  return "";
}

export function requiresExplicitApiBase(options?: {
  currentOrigin?: string;
}): boolean {
  const currentOrigin =
    options?.currentOrigin ??
    (typeof window !== "undefined" ? window.location.origin : undefined);

  if (!currentOrigin) {
    return false;
  }

  if (
    currentOrigin.startsWith("http://localhost") ||
    currentOrigin.startsWith("http://127.0.0.1") ||
    currentOrigin === "http://tauri.localhost"
  ) {
    return false;
  }

  return currentOrigin.startsWith("https://");
}

export function assertHostedApiBaseConfigured(
  apiBase = API_BASE,
  options?: { currentOrigin?: string; userAgent?: string },
) {
  if (!requiresExplicitApiBase(options)) {
    return;
  }

  if (resolveImplicitApiBase(apiBase, options)) {
    return;
  }

  throw new Error(
    "PUBLIC_API_BASE must be set for hosted dAstIll frontend builds.",
  );
}

export class BackendUnavailableError extends Error {
  constructor(
    message = "Sorry, we could not connect right now. Please try again.",
  ) {
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

/** Backend returned 403 with a sign-in requirement (library and other protected routes). */
export class AuthRequiredError extends Error {
  readonly status = 403;

  constructor(message = "Sign in to continue.") {
    super(message);
    this.name = "AuthRequiredError";
  }
}

export function isAuthRequiredError(
  error: unknown,
): error is AuthRequiredError {
  return error instanceof AuthRequiredError;
}

function isLikelyAuthRequiredMessage(message: string): boolean {
  if (isSignInRequiredBody(message)) return true;
  const t = message.trim();
  return t === "Sign in to continue." || t === "Sign in to use this feature.";
}

/** True for `AuthRequiredError` or a plain `Error` whose message is a sign-in requirement. */
export function isSignInRequiredFailure(error: unknown): boolean {
  if (isAuthRequiredError(error)) return true;
  if (!(error instanceof Error)) return false;
  return isLikelyAuthRequiredMessage(error.message);
}

function isSignInRequiredBody(text: string): boolean {
  const t = text.trim();
  return t === "Sign-in required" || t.includes("Sign-in required");
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

export function resolveApiUrl(path: string, apiBase = API_BASE): string {
  return `${resolveImplicitApiBase(apiBase)}${path}`;
}

export async function createApiRequestInit(
  init?: RequestInit,
  options?: { includeJsonContentType?: boolean },
): Promise<RequestInit> {
  const headers = new Headers(init?.headers);
  if (
    options?.includeJsonContentType !== false &&
    !headers.has("Content-Type")
  ) {
    headers.set("Content-Type", "application/json");
  }

  const token = await getCurrentAuthToken();
  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }

  return {
    ...init,
    headers,
  };
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
      ...(await createApiRequestInit(init)),
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
        normalizeUserErrorMessage(message, { status: 429 }),
        retryAfterMs,
      );
    }
    const message = await response.text();
    const trimmed = message.trim();
    if (response.status === 403 && isSignInRequiredBody(trimmed)) {
      console.warn(`[API] ${method} ${path} sign-in required`);
      throw trimmed === "Sign-in required"
        ? new AuthRequiredError()
        : new AuthRequiredError(trimmed);
    }
    console.error(`[API Error] ${method} ${path}`, {
      status: response.status,
    });
    throw new Error(
      normalizeUserErrorMessage(
        trimmed || `Request failed (${response.status})`,
        {
          status: response.status,
        },
      ),
    );
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}
