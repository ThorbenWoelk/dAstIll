import { describe, expect, it } from "bun:test";
import {
  formatDuration,
  formatPublishedAt,
  formatSyncDate,
  hasKnownDuration,
  resolveSummaryQualityPresentation,
  resolveTranscriptPresentation,
  stripContentPrefix,
} from "../src/lib/workspace/content";
import type { Summary, Transcript } from "../src/lib/types";

function createTranscript(overrides: Partial<Transcript> = {}): Transcript {
  return {
    video_id: "video-1",
    raw_text: null,
    formatted_markdown: null,
    render_mode: "plain_text",
    ...overrides,
  };
}

function createSummary(overrides: Partial<Summary> = {}): Summary {
  return {
    video_id: "video-1",
    content: "Summary",
    quality_score: null,
    quality_note: null,
    model_used: null,
    quality_model_used: null,
    ...overrides,
  };
}

describe("stripContentPrefix", () => {
  it("removes transcript and summary prefixes", () => {
    expect(stripContentPrefix("Transcript: Hello")).toBe("Hello");
    expect(stripContentPrefix("summary: Hello")).toBe("Hello");
  });
});

describe("resolveTranscriptPresentation", () => {
  it("prefers markdown content when the transcript render mode is markdown", () => {
    const presentation = resolveTranscriptPresentation(
      createTranscript({
        raw_text: "Transcript: Plain text",
        formatted_markdown: "Transcript: **Markdown**",
        render_mode: "markdown",
      }),
    );

    expect(presentation).toEqual({
      content: "**Markdown**",
      originalText: "Plain text",
      renderMode: "markdown",
    });
  });

  it("falls back to raw text when markdown is absent", () => {
    const presentation = resolveTranscriptPresentation(
      createTranscript({
        raw_text: "Transcript: Plain text",
        formatted_markdown: "",
        render_mode: "markdown",
      }),
    );

    expect(presentation.content).toBe("Plain text");
    expect(presentation.originalText).toBe("Plain text");
  });

  it("uses an unavailable fallback when no transcript text exists", () => {
    expect(resolveTranscriptPresentation(createTranscript())).toEqual({
      content: "Transcript unavailable.",
      originalText: "Transcript unavailable.",
      renderMode: "plain_text",
    });
  });
});

describe("resolveSummaryQualityPresentation", () => {
  it("rounds and clamps the score while trimming note text", () => {
    const presentation = resolveSummaryQualityPresentation(
      createSummary({
        quality_score: 10.8,
        quality_note: "  **Faithfulness**: \\n- Dense and accurate.  ",
        model_used: "summary-model",
        quality_model_used: "eval-model",
      }),
    );

    expect(presentation).toEqual({
      score: 10,
      note: "**Faithfulness**: \\n- Dense and accurate.",
      modelUsed: "summary-model",
      qualityModelUsed: "eval-model",
    });
  });

  it("returns null quality fields when the summary has none", () => {
    expect(resolveSummaryQualityPresentation(createSummary())).toEqual({
      score: null,
      note: null,
      modelUsed: null,
      qualityModelUsed: null,
    });
  });
});

describe("workspace content formatters", () => {
  it("formats valid publish dates and preserves invalid values", () => {
    expect(formatPublishedAt("2026-03-01T10:15:00.000Z")).toBeTruthy();
    expect(formatPublishedAt("not-a-date")).toBe("not-a-date");
    expect(formatPublishedAt(null)).toBe("Unknown");
  });

  it("formats sync dates with unknown fallbacks", () => {
    expect(formatSyncDate("2026-03-01T10:15:00.000Z")).toBeTruthy();
    expect(formatSyncDate("not-a-date")).toBe("Unknown");
    expect(formatSyncDate(undefined)).toBe("Unknown");
  });

  it("detects duration from seconds or iso strings", () => {
    expect(hasKnownDuration(0, null)).toBe(true);
    expect(hasKnownDuration(null, "PT1M")).toBe(true);
    expect(hasKnownDuration(null, null)).toBe(false);
  });

  it("formats duration from seconds before falling back to iso text", () => {
    expect(formatDuration(3723, null)).toBe("1h 2m 3s");
    expect(formatDuration(123, null)).toBe("2m 3s");
    expect(formatDuration(null, "PT5M")).toBe("PT5M");
    expect(formatDuration(null, null)).toBe("Unknown");
  });
});
