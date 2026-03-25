import { describe, expect, it } from "bun:test";
import { isAnonymousChatQuotaError } from "../src/lib/chat/anonymous-quota";

describe("isAnonymousChatQuotaError", () => {
  it("detects quota message substring", () => {
    expect(
      isAnonymousChatQuotaError("Anonymous chat quota exceeded for this IP"),
    ).toBe(true);
  });

  it("returns false for unrelated errors", () => {
    expect(isAnonymousChatQuotaError("Network error")).toBe(false);
  });
});
