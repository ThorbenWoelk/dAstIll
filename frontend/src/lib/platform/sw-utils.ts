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

/**
 * Cache name for channel / video thumbnails loaded from YouTube and Google CDNs.
 * Cache-first keeps the mobile channel strip fast on repeat visits.
 */
export const AVATAR_CACHE = `avatars-${CACHE_VERSION}`;

/** All cache names that belong to the current SW version. */
export const KNOWN_CACHES = [STATIC_CACHE, API_CACHE, AVATAR_CACHE];

/**
 * Returns true for URL pathnames that should use a cache-first strategy:
 * - `/_app/` — Vite-built JS/CSS with content-hash filenames (immutable)
 * - `/fonts/` — self-hosted WOFF2 font files (rarely change)
 */
export function isStaticAssetPath(pathname: string): boolean {
  return pathname.startsWith("/_app/") || pathname.startsWith("/fonts/");
}

/**
 * Returns true for HTTPS URLs that serve YouTube / Google-hosted channel or
 * video thumbnails. Used by the service worker for cache-first image caching.
 */
export function isChannelAvatarThumbnailUrl(url: URL): boolean {
  if (url.protocol !== "https:") {
    return false;
  }
  const h = url.hostname;
  return (
    h === "yt3.ggpht.com" ||
    h === "yt3.googleusercontent.com" ||
    h === "lh3.googleusercontent.com" ||
    h === "i.ytimg.com" ||
    /^i\d\.ytimg\.com$/.test(h)
  );
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
