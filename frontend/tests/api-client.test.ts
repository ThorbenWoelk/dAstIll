import { describe, expect, it } from "bun:test";

import {
  normalizeApiBase,
  resolveApiUrl,
  resolveImplicitApiBase,
} from "../src/lib/api-client";

describe("normalizeApiBase", () => {
  it("keeps local proxy mode when the API base is unset", () => {
    expect(normalizeApiBase()).toBe("");
    expect(normalizeApiBase("   ")).toBe("");
  });

  it("trims surrounding whitespace and a trailing slash", () => {
    expect(normalizeApiBase(" https://backend.example.com/ ")).toBe(
      "https://backend.example.com",
    );
  });
});

describe("resolveApiUrl", () => {
  it("uses a relative path when no production API origin is configured", () => {
    expect(resolveApiUrl("/api/channels", normalizeApiBase())).toBe(
      "/api/channels",
    );
  });

  it("uses the configured backend origin for production builds", () => {
    expect(
      resolveApiUrl(
        "/api/channels",
        normalizeApiBase("https://backend.example.com/"),
      ),
    ).toBe("https://backend.example.com/api/channels");
  });
});

describe("resolveImplicitApiBase", () => {
  it("keeps the configured api base when present", () => {
    expect(
      resolveImplicitApiBase("https://backend.example.com", {
        currentOrigin: "http://tauri.localhost",
        userAgent: "Android",
      }),
    ).toBe("https://backend.example.com");
  });

  it("falls back to the reversed localhost backend for tauri android dev", () => {
    expect(
      resolveImplicitApiBase("", {
        currentOrigin: "http://tauri.localhost",
        userAgent: "Mozilla/5.0 (Linux; Android 14)",
      }),
    ).toBe("http://127.0.0.1:3544");
  });

  it("keeps relative api paths for non-tauri clients when the api base is unset", () => {
    expect(
      resolveImplicitApiBase("", {
        currentOrigin: "http://localhost:3543",
        userAgent: "Mozilla/5.0",
      }),
    ).toBe("");
  });
});
