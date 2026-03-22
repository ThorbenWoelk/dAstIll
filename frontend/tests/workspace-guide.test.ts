import { describe, expect, it } from "bun:test";

import {
  resolveGuideStepFromUrl,
  writeGuideStepToUrl,
} from "../src/lib/utils/guide";

describe("resolveGuideStepFromUrl", () => {
  it("returns null when no guide param is present", () => {
    const url = new URL("http://localhost/");
    expect(resolveGuideStepFromUrl(url, 10)).toBeNull();
  });

  it("returns the step index when guide param is a valid integer", () => {
    const url = new URL("http://localhost/?guide=3");
    expect(resolveGuideStepFromUrl(url, 10)).toBe(3);
  });

  it("returns 0 when guide param is 0", () => {
    const url = new URL("http://localhost/?guide=0");
    expect(resolveGuideStepFromUrl(url, 10)).toBe(0);
  });

  it("returns the last valid step index", () => {
    const url = new URL("http://localhost/?guide=9");
    expect(resolveGuideStepFromUrl(url, 10)).toBe(9);
  });

  it("returns null when guide param equals stepCount (out of bounds)", () => {
    const url = new URL("http://localhost/?guide=10");
    expect(resolveGuideStepFromUrl(url, 10)).toBeNull();
  });

  it("returns null when guide param exceeds stepCount", () => {
    const url = new URL("http://localhost/?guide=99");
    expect(resolveGuideStepFromUrl(url, 10)).toBeNull();
  });

  it("returns null when guide param is negative", () => {
    const url = new URL("http://localhost/?guide=-1");
    expect(resolveGuideStepFromUrl(url, 10)).toBeNull();
  });

  it("returns null when guide param is not a number", () => {
    const url = new URL("http://localhost/?guide=abc");
    expect(resolveGuideStepFromUrl(url, 10)).toBeNull();
  });

  it("returns null when guide param is an empty string", () => {
    const url = new URL("http://localhost/?guide=");
    expect(resolveGuideStepFromUrl(url, 10)).toBeNull();
  });

  it("handles a single-step tour (stepCount=1, only step 0 valid)", () => {
    const url0 = new URL("http://localhost/?guide=0");
    const url1 = new URL("http://localhost/?guide=1");
    expect(resolveGuideStepFromUrl(url0, 1)).toBe(0);
    expect(resolveGuideStepFromUrl(url1, 1)).toBeNull();
  });
});

describe("writeGuideStepToUrl", () => {
  it("returns without error when window is not available (SSR/test environment)", () => {
    // The function guards against non-browser environments with:
    // if (typeof window === "undefined") return;
    // In a bun test environment, window is undefined, so this is a no-op.
    expect(() => writeGuideStepToUrl(2)).not.toThrow();
  });

  it("handles null step without error in non-browser environment", () => {
    expect(() => writeGuideStepToUrl(null)).not.toThrow();
  });
});
