/**
 * Service worker caching utilities.
 *
 * Pure helper functions shared between sw.js and its unit tests. The service
 * worker itself (static/sw.js) cannot import TypeScript modules, so it mirrors
 * this logic inline. Keep the two in sync when updating caching rules.
 */

export const CACHE_VERSION = "v2";

/** Cache name for Vite-built JS/CSS chunks and self-hosted fonts. */
export const STATIC_CACHE = `static-${CACHE_VERSION}`;

/** Cache name for successful GET /api/* responses. */
export const API_CACHE = `api-${CACHE_VERSION}`;

/** All cache names that belong to the current SW version. */
export const KNOWN_CACHES = [STATIC_CACHE, API_CACHE];

/**
 * Returns true for URL pathnames that should use a cache-first strategy:
 * - `/_app/` — Vite-built JS/CSS with content-hash filenames (immutable)
 * - `/fonts/` — self-hosted WOFF2 font files (rarely change)
 */
export function isStaticAssetPath(pathname: string): boolean {
  return pathname.startsWith("/_app/") || pathname.startsWith("/fonts/");
}

/**
 * Returns true when the request targets a Server-Sent Events stream.
 * SSE responses must never be cached: they are long-lived streams.
 */
export function isSseRequest(request: {
  headers: { get: (header: string) => string | null };
}): boolean {
  return request.headers.get("accept") === "text/event-stream";
}

/**
 * Filters a list of cache names down to those that are no longer in use
 * (i.e. not in KNOWN_CACHES). Used during the activate event to prune stale
 * caches left behind by older SW versions.
 */
export function getObsoleteCacheNames(allCacheNames: string[]): string[] {
  return allCacheNames.filter((name) => !KNOWN_CACHES.includes(name));
}
