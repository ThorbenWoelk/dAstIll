import { describe, expect, it } from "bun:test";

import { resolveMaintenanceMode } from "../src/lib/config/maintenance-mode";

describe("resolveMaintenanceMode", () => {
  it("defaults to live mode when unset", () => {
    expect(resolveMaintenanceMode()).toBe(false);
    expect(resolveMaintenanceMode("")).toBe(false);
  });

  it("enables maintenance mode for truthy env values", () => {
    expect(resolveMaintenanceMode("1")).toBe(true);
    expect(resolveMaintenanceMode("true")).toBe(true);
    expect(resolveMaintenanceMode(" YES ")).toBe(true);
    expect(resolveMaintenanceMode("on")).toBe(true);
  });

  it("keeps live mode for other values", () => {
    expect(resolveMaintenanceMode("0")).toBe(false);
    expect(resolveMaintenanceMode("false")).toBe(false);
    expect(resolveMaintenanceMode("maintenance")).toBe(false);
  });
});
