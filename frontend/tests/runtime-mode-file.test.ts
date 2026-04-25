import { describe, expect, it } from "bun:test";

import {
  normalizeRuntimeMode,
  resolveRuntimeMode,
} from "../../scripts/resolve-runtime-mode.mjs";

describe("resolveRuntimeMode", () => {
  it("defaults to live mode when the file is missing", () => {
    expect(resolveRuntimeMode("/tmp/does-not-exist-runtime-mode.env")).toBe(
      "live",
    );
  });

  it("ignores inline comments", () => {
    expect(
      normalizeRuntimeMode("APP_RUNTIME_MODE=maintenance  # or normal\n"),
    ).toBe("maintenance");
  });

  it("normalizes case and whitespace", () => {
    expect(normalizeRuntimeMode("  APP_RUNTIME_MODE = Maintenance \n")).toBe(
      "maintenance",
    );
  });
});
