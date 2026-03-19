import { describe, expect, it } from "bun:test";

import {
  getAdjacentContentMode,
  resolveDefaultContentMode,
  resolveSwipedContentMode,
  WORKSPACE_CONTENT_MODE_ORDER,
} from "../src/lib/workspace/navigation";

describe("WORKSPACE_CONTENT_MODE_ORDER", () => {
  it("keeps the mobile tab order stable", () => {
    expect(WORKSPACE_CONTENT_MODE_ORDER).toEqual([
      "info",
      "transcript",
      "summary",
      "highlights",
    ]);
  });
});

describe("getAdjacentContentMode", () => {
  it("moves backward and forward through the content tabs", () => {
    expect(getAdjacentContentMode("transcript", "previous")).toBe("info");
    expect(getAdjacentContentMode("summary", "next")).toBe("highlights");
  });

  it("returns null when swiping beyond the first or last tab", () => {
    expect(getAdjacentContentMode("info", "previous")).toBeNull();
    expect(getAdjacentContentMode("highlights", "next")).toBeNull();
  });
});

describe("resolveDefaultContentMode", () => {
  it("prefers highlights, then summary, then transcript, then info", () => {
    expect(
      resolveDefaultContentMode({
        hasHighlights: true,
        hasSummary: true,
        hasTranscript: true,
      }),
    ).toBe("highlights");

    expect(
      resolveDefaultContentMode({
        hasHighlights: false,
        hasSummary: true,
        hasTranscript: true,
      }),
    ).toBe("summary");

    expect(
      resolveDefaultContentMode({
        hasHighlights: false,
        hasSummary: false,
        hasTranscript: true,
      }),
    ).toBe("transcript");

    expect(
      resolveDefaultContentMode({
        hasHighlights: false,
        hasSummary: false,
        hasTranscript: false,
      }),
    ).toBe("info");
  });
});

describe("resolveSwipedContentMode", () => {
  it("switches to the next tab on a strong left swipe", () => {
    expect(resolveSwipedContentMode("summary", -80, 8)).toBe("highlights");
  });

  it("switches to the previous tab on a strong right swipe", () => {
    expect(resolveSwipedContentMode("transcript", 80, 8)).toBe("info");
  });

  it("ignores short or mostly vertical swipes", () => {
    expect(resolveSwipedContentMode("summary", 40, 4)).toBeNull();
    expect(resolveSwipedContentMode("summary", 80, 90)).toBeNull();
  });

  it("stays on the edge tabs when no adjacent tab exists", () => {
    expect(resolveSwipedContentMode("info", 80, 0)).toBeNull();
    expect(resolveSwipedContentMode("highlights", -80, 0)).toBeNull();
  });
});
