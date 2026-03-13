import { describe, expect, it } from "bun:test";

import { resolveDocsUrl } from "../src/lib/docs-url";

describe("resolveDocsUrl", () => {
  it("prefers the configured docs url when provided", () => {
    expect(resolveDocsUrl("https://docs.example.com")).toBe(
      "https://docs.example.com",
    );
  });

  it("falls back to the local docs frontend when unset", () => {
    expect(resolveDocsUrl()).toBe("http://localhost:4173");
  });
});
