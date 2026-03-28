import { describe, expect, it } from "bun:test";

import {
  getAdjacentContentMode,
  goHintKeyForWorkspaceContentMode,
  resolveDefaultContentMode,
  resolveSwipedContentMode,
  WORKSPACE_CONTENT_MODE_ORDER,
} from "../src/lib/workspace/navigation";

describe("WORKSPACE_CONTENT_MODE_ORDER", () => {
  it("keeps the mobile tab order stable", () => {
    expect(WORKSPACE_CONTENT_MODE_ORDER).toEqual([
      "info",
      "summary",
      "highlights",
      "transcript",
    ]);
  });
});

describe("goHintKeyForWorkspaceContentMode", () => {
  it("maps each tab to a G-chord letter (L for tab Highlights; G H is the Highlights page)", () => {
    expect(goHintKeyForWorkspaceContentMode("info")).toBe("I");
    expect(goHintKeyForWorkspaceContentMode("summary")).toBe("S");
    expect(goHintKeyForWorkspaceContentMode("highlights")).toBe("L");
    expect(goHintKeyForWorkspaceContentMode("transcript")).toBe("T");
  });
});

describe("getAdjacentContentMode", () => {
  it("moves backward and forward through the content tabs", () => {
    expect(getAdjacentContentMode("transcript", "previous")).toBe("highlights");
    expect(getAdjacentContentMode("summary", "next")).toBe("highlights");
  });

  it("returns null when swiping beyond the first or last tab", () => {
    expect(getAdjacentContentMode("info", "previous")).toBeNull();
    expect(getAdjacentContentMode("transcript", "next")).toBeNull();
  });
});

describe("resolveDefaultContentMode", () => {
  it("opens the info tab first regardless of available content", () => {
    expect(
      resolveDefaultContentMode({
        hasHighlights: true,
        hasSummary: true,
        hasTranscript: true,
      }),
    ).toBe("info");

    expect(
      resolveDefaultContentMode({
        hasHighlights: false,
        hasSummary: true,
        hasTranscript: true,
      }),
    ).toBe("info");

    expect(
      resolveDefaultContentMode({
        hasHighlights: false,
        hasSummary: false,
        hasTranscript: true,
      }),
    ).toBe("info");

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
    expect(resolveSwipedContentMode("transcript", 80, 8)).toBe("highlights");
  });

  it("ignores short or mostly vertical swipes", () => {
    expect(resolveSwipedContentMode("summary", 40, 4)).toBeNull();
    expect(resolveSwipedContentMode("summary", 80, 90)).toBeNull();
  });

  it("stays on the edge tabs when no adjacent tab exists", () => {
    expect(resolveSwipedContentMode("info", 80, 0)).toBeNull();
    expect(resolveSwipedContentMode("transcript", -80, 0)).toBeNull();
  });
});
