import { describe, expect, it } from "bun:test";

import {
  assertHostedApiBaseConfigured,
  normalizeApiBase,
  requiresExplicitApiBase,
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

describe("requiresExplicitApiBase", () => {
  it("requires an explicit api base for hosted https origins", () => {
    expect(
      requiresExplicitApiBase({
        currentOrigin: "https://dastill.web.app",
      }),
    ).toBe(true);
  });

  it("does not require an explicit api base for local development origins", () => {
    expect(
      requiresExplicitApiBase({
        currentOrigin: "http://localhost:3543",
      }),
    ).toBe(false);
  });

  it("does not require an explicit api base for the tauri localhost shell", () => {
    expect(
      requiresExplicitApiBase({
        currentOrigin: "http://tauri.localhost",
      }),
    ).toBe(false);
  });
});

describe("assertHostedApiBaseConfigured", () => {
  it("throws when a hosted https origin has no configured api base", () => {
    expect(() =>
      assertHostedApiBaseConfigured("", {
        currentOrigin: "https://dastill.web.app",
      }),
    ).toThrow("PUBLIC_API_BASE must be set");
  });

  it("accepts hosted https origins when an api base is configured", () => {
    expect(() =>
      assertHostedApiBaseConfigured("https://backend.example.com", {
        currentOrigin: "https://dastill.web.app",
      }),
    ).not.toThrow();
  });
});
