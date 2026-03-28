import { describe, expect, it } from "bun:test";

import { shouldUseSessionHighlights } from "../src/lib/workspace/session-highlights";

describe("session-highlights", () => {
  it("shouldUseSessionHighlights is false when authenticated", () => {
    expect(
      shouldUseSessionHighlights({ authState: "authenticated", userId: "u1" }),
    ).toBe(false);
  });

  it("shouldUseSessionHighlights is true when anonymous", () => {
    expect(shouldUseSessionHighlights({ authState: "anonymous" })).toBe(true);
  });
});
