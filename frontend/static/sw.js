/**
 * dAstIll service worker.
 *
 * Caches only content-hashed build files (/_app/immutable/**), so repeat
 * visits open without re-downloading the app. API calls always go to the
 * network. Activating this worker also deletes the caches of the previous
 * app, which stored API responses.
 */

var STATIC_CACHE = "edition-static-v1";

self.addEventListener("install", function () {
  self.skipWaiting();
});

self.addEventListener("activate", function (event) {
  event.waitUntil(
    caches
      .keys()
      .then(function (names) {
        return Promise.all(
          names
            .filter(function (name) {
              return name !== STATIC_CACHE;
            })
            .map(function (name) {
              return caches.delete(name);
            }),
        );
      })
      .then(function () {
        return self.clients.claim();
      }),
  );
});

self.addEventListener("fetch", function (event) {
  var request = event.request;
  if (request.method !== "GET") return;
  var url = new URL(request.url);
  if (url.origin !== self.location.origin) return;
  if (!url.pathname.startsWith("/_app/immutable/")) return;

  event.respondWith(
    caches.open(STATIC_CACHE).then(function (cache) {
      return cache.match(request).then(function (cached) {
        if (cached) return cached;
        return fetch(request).then(function (response) {
          if (response.ok) cache.put(request, response.clone());
          return response;
        });
      });
    }),
  );
});
