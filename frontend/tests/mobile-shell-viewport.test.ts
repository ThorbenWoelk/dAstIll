import { describe, expect, it } from "bun:test";

import { resolveVisualViewportBottomInset } from "../src/lib/mobile-shell/viewport";

describe("resolveVisualViewportBottomInset", () => {
  it("returns zero when visual viewport metrics are unavailable", () => {
    expect(
      resolveVisualViewportBottomInset({
        innerHeight: 844,
        visualViewportHeight: null,
        visualViewportOffsetTop: null,
      }),
    ).toBe(0);
  });

  it("returns the portion of the layout viewport hidden below the visual viewport", () => {
    expect(
      resolveVisualViewportBottomInset({
        innerHeight: 844,
        visualViewportHeight: 760,
        visualViewportOffsetTop: 0,
      }),
    ).toBe(84);
  });

  it("accounts for a shifted visual viewport", () => {
    expect(
      resolveVisualViewportBottomInset({
        innerHeight: 844,
        visualViewportHeight: 720,
        visualViewportOffsetTop: 24,
      }),
    ).toBe(100);
  });

  it("clamps negative values to zero", () => {
    expect(
      resolveVisualViewportBottomInset({
        innerHeight: 844,
        visualViewportHeight: 850,
        visualViewportOffsetTop: 0,
      }),
    ).toBe(0);
  });
});
