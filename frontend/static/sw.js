/**
 * dAstIll Service Worker
 *
 * Caching strategies:
 *   - Cache-first  : /_app/** (Vite bundles with content-hash filenames)
 *                    /fonts/** (self-hosted WOFF2 files)
 *   - Network-first: /api/** GET responses (with cache fallback)
 *   - Pass-through : everything else, POST/PUT/DELETE, and SSE streams
 *
 * Cache versioning: bump CACHE_VERSION to rotate all caches on the next SW
 * update. The activate handler deletes any cache name not in KNOWN_CACHES.
 *
 * NOTE: This file mirrors the logic in src/lib/sw-utils.ts. Keep them in sync.
 */

var CACHE_VERSION = "v2";
var STATIC_CACHE = "static-" + CACHE_VERSION;
var API_CACHE = "api-" + CACHE_VERSION;
var AVATAR_CACHE = "avatars-" + CACHE_VERSION;
var KNOWN_CACHES = [STATIC_CACHE, API_CACHE, AVATAR_CACHE];

// ── Helpers ───────────────────────────────────────────────────────────────────

function isStaticAssetPath(pathname) {
  return pathname.startsWith("/_app/") || pathname.startsWith("/fonts/");
}

function isChannelAvatarThumbnailUrl(url) {
  if (url.protocol !== "https:") {
    return false;
  }
  var h = url.hostname;
  return (
    h === "yt3.ggpht.com" ||
    h === "yt3.googleusercontent.com" ||
    h === "lh3.googleusercontent.com" ||
    h === "i.ytimg.com" ||
    /^i\d\.ytimg\.com$/.test(h)
  );
}

function isSseRequest(request) {
  return request.headers.get("accept") === "text/event-stream";
}

// ── Strategies ────────────────────────────────────────────────────────────────

/**
 * Cache-first: serve cached response immediately; on a miss, fetch from the
 * network and populate the cache before returning.
 */
async function cacheFirst(request, cacheName) {
  var cache = await caches.open(cacheName);
  var cached = await cache.match(request);
  if (cached) {
    return cached;
  }
  var response = await fetch(request);
  if (response.ok) {
    cache.put(request, response.clone());
  }
  return response;
}

/**
 * Network-first: always attempt a fresh fetch; cache the successful result so
 * it can be served as a fallback when the network is unavailable.
 */
async function networkFirst(request, cacheName) {
  var cache = await caches.open(cacheName);
  try {
    var response = await fetch(request);
    if (response.ok) {
      cache.put(request, response.clone());
    }
    return response;
  } catch (err) {
    var cached = await cache.match(request);
    if (cached) {
      return cached;
    }
    throw err;
  }
}

// ── Lifecycle events ──────────────────────────────────────────────────────────

self.addEventListener("install", function () {
  // Take control immediately without waiting for existing tabs to close.
  self.skipWaiting();
});

self.addEventListener("activate", function (event) {
  // Delete all caches that belong to older SW versions, then claim clients so
  // the new SW takes effect without requiring a page reload.
  event.waitUntil(
    caches
      .keys()
      .then(function (cacheNames) {
        return Promise.all(
          cacheNames
            .filter(function (name) {
              return !KNOWN_CACHES.includes(name);
            })
            .map(function (name) {
              return caches.delete(name);
            }),
        );
      })
      .then(function () {
        return clients.claim();
      }),
  );
});

// ── Fetch handler ─────────────────────────────────────────────────────────────

self.addEventListener("fetch", function (event) {
  var request = event.request;

  // Skip non-GET requests — mutations must always reach the server.
  if (request.method !== "GET") {
    return;
  }

  // Skip SSE streams — they are long-lived and must not be cached.
  if (isSseRequest(request)) {
    return;
  }

  var url = new URL(request.url);

  // Cache-first for immutable static assets (Vite bundles + self-hosted fonts).
  if (isStaticAssetPath(url.pathname)) {
    event.respondWith(cacheFirst(request, STATIC_CACHE));
    return;
  }

  // Network-first for GET API responses, with cache fallback for offline use.
  if (url.pathname.startsWith("/api/")) {
    event.respondWith(networkFirst(request, API_CACHE));
    return;
  }

  // Cache-first for YouTube / Google CDN channel and video thumbnails (mobile
  // channel strip, sidebar avatars). Speeds repeat visits; first visit still
  // hits the network.
  if (isChannelAvatarThumbnailUrl(url)) {
    event.respondWith(cacheFirst(request, AVATAR_CACHE));
    return;
  }

  // All other requests (HTML pages, icons, etc.) are handled by the browser.
});
