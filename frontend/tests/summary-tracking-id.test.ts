import { describe, expect, it } from "bun:test";
import type { Summary } from "../src/lib/types";
import {
  deriveSummaryTrackingId,
  hashSummarySignature,
} from "../src/lib/workspace/summary-tracking-id";

describe("hashSummarySignature", () => {
  it("is deterministic for the same input", () => {
    expect(hashSummarySignature("alpha")).toBe(hashSummarySignature("alpha"));
  });

  it("differs when input changes", () => {
    expect(hashSummarySignature("a")).not.toBe(hashSummarySignature("b"));
  });
});

describe("deriveSummaryTrackingId", () => {
  it("embeds video_id and stable hash from model and content", () => {
    const summary: Summary = {
      video_id: "v1",
      model_used: "m1",
      content: "body",
    };
    expect(deriveSummaryTrackingId(summary)).toMatch(/^v1:[0-9a-f]{8}$/);
    expect(deriveSummaryTrackingId(summary)).toBe(
      deriveSummaryTrackingId(summary),
    );
  });

  it("changes when summary content changes", () => {
    const a: Summary = {
      video_id: "v1",
      model_used: "m1",
      content: "one",
    };
    const b: Summary = { ...a, content: "two" };
    expect(deriveSummaryTrackingId(a)).not.toBe(deriveSummaryTrackingId(b));
  });
});
