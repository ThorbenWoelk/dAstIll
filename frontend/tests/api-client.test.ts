import { describe, expect, it } from "bun:test";

import { normalizeApiBase, resolveApiUrl } from "../src/lib/api-client";

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
