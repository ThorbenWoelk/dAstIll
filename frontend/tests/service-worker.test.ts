import { describe, expect, it } from "bun:test";

import {
  KNOWN_CACHES,
  STATIC_CACHE,
  API_CACHE,
  AVATAR_CACHE,
  isStaticAssetPath,
  isChannelAvatarThumbnailUrl,
  isSseRequest,
  getObsoleteCacheNames,
} from "../src/lib/sw-utils";

// ── URL classification ────────────────────────────────────────────────────────

describe("isStaticAssetPath", () => {
  it("matches Vite-built JS chunks under /_app/", () => {
    expect(isStaticAssetPath("/_app/immutable/entry/start.abc123.js")).toBe(
      true,
    );
    expect(isStaticAssetPath("/_app/immutable/chunks/vendor.xyz.js")).toBe(
      true,
    );
    expect(isStaticAssetPath("/_app/immutable/assets/entry.abc.css")).toBe(
      true,
    );
  });

  it("matches self-hosted font files under /fonts/", () => {
    expect(isStaticAssetPath("/fonts/manrope-latin.woff2")).toBe(true);
    expect(isStaticAssetPath("/fonts/fraunces-normal-latin.woff2")).toBe(true);
  });

  it("does not match API paths", () => {
    expect(isStaticAssetPath("/api/channels")).toBe(false);
    expect(isStaticAssetPath("/api/workspace/bootstrap")).toBe(false);
  });

  it("does not match page routes", () => {
    expect(isStaticAssetPath("/")).toBe(false);
    expect(isStaticAssetPath("/highlights")).toBe(false);
    expect(isStaticAssetPath("/chat")).toBe(false);
    expect(isStaticAssetPath("/download-queue")).toBe(false);
  });

  it("does not match arbitrary static files at root", () => {
    expect(isStaticAssetPath("/favicon.png")).toBe(false);
    expect(isStaticAssetPath("/manifest.webmanifest")).toBe(false);
    expect(isStaticAssetPath("/robots.txt")).toBe(false);
  });
});

describe("isChannelAvatarThumbnailUrl", () => {
  it("matches common YouTube and Google CDN thumbnail hosts", () => {
    expect(
      isChannelAvatarThumbnailUrl(
        new URL("https://yt3.ggpht.com/ytc/foo=s176-c-k-c0x00ffffff-no-rj"),
      ),
    ).toBe(true);
    expect(
      isChannelAvatarThumbnailUrl(
        new URL("https://i.ytimg.com/vi/abc/hqdefault.jpg"),
      ),
    ).toBe(true);
    expect(
      isChannelAvatarThumbnailUrl(
        new URL("https://i3.ytimg.com/vi/x/mqdefault.jpg"),
      ),
    ).toBe(true);
    expect(
      isChannelAvatarThumbnailUrl(
        new URL("https://lh3.googleusercontent.com/a/xyz"),
      ),
    ).toBe(true);
  });

  it("does not match arbitrary origins", () => {
    expect(
      isChannelAvatarThumbnailUrl(new URL("https://example.com/thumb.jpg")),
    ).toBe(false);
    expect(
      isChannelAvatarThumbnailUrl(new URL("http://yt3.ggpht.com/insecure")),
    ).toBe(false);
  });
});

// ── SSE detection ─────────────────────────────────────────────────────────────

describe("isSseRequest", () => {
  it("detects SSE requests by accept header", () => {
    const req = {
      headers: {
        get: (h: string) => (h === "accept" ? "text/event-stream" : null),
      },
    };
    expect(isSseRequest(req)).toBe(true);
  });

  it("does not flag JSON API requests as SSE", () => {
    const req = {
      headers: {
        get: (h: string) => (h === "accept" ? "application/json" : null),
      },
    };
    expect(isSseRequest(req)).toBe(false);
  });

  it("does not flag requests with no accept header as SSE", () => {
    const req = { headers: { get: (_h: string) => null } };
    expect(isSseRequest(req)).toBe(false);
  });

  it("does not flag requests that include text/event-stream but are not exclusively it", () => {
    const req = {
      headers: {
        get: (h: string) =>
          h === "accept" ? "text/html, text/event-stream" : null,
      },
    };
    expect(isSseRequest(req)).toBe(false);
  });
});

// ── Cache versioning ──────────────────────────────────────────────────────────

describe("getObsoleteCacheNames", () => {
  it("returns empty array when all caches are current", () => {
    expect(
      getObsoleteCacheNames([STATIC_CACHE, API_CACHE, AVATAR_CACHE]),
    ).toEqual([]);
  });

  it("returns obsolete caches that are not in KNOWN_CACHES", () => {
    const obsolete = getObsoleteCacheNames([
      "static-v1",
      "api-v1",
      STATIC_CACHE,
      API_CACHE,
      AVATAR_CACHE,
    ]);
    expect(obsolete).toContain("static-v1");
    expect(obsolete).toContain("api-v1");
    expect(obsolete).not.toContain(STATIC_CACHE);
    expect(obsolete).not.toContain(API_CACHE);
    expect(obsolete).not.toContain(AVATAR_CACHE);
  });

  it("handles empty list gracefully", () => {
    expect(getObsoleteCacheNames([])).toEqual([]);
  });

  it("identifies legacy no-version caches as obsolete", () => {
    const obsolete = getObsoleteCacheNames(["static", "api", "app-cache"]);
    expect(obsolete).toEqual(["static", "api", "app-cache"]);
  });
});

// ── Cache name constants ──────────────────────────────────────────────────────

describe("KNOWN_CACHES", () => {
  it("includes the static, api, and avatar cache names", () => {
    expect(KNOWN_CACHES).toContain(STATIC_CACHE);
    expect(KNOWN_CACHES).toContain(API_CACHE);
    expect(KNOWN_CACHES).toContain(AVATAR_CACHE);
  });

  it("has versioned cache names", () => {
    // Cache names must include a version segment so they can be rotated
    expect(STATIC_CACHE).toMatch(/^static-v\d+$/);
    expect(API_CACHE).toMatch(/^api-v\d+$/);
    expect(AVATAR_CACHE).toMatch(/^avatars-v\d+$/);
  });
});
