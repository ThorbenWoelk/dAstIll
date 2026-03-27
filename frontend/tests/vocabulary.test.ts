import { describe, expect, it } from "bun:test";

import type { VocabularyReplacement } from "../src/lib/types";
import {
  formatVocabularyAddedAt,
  normalizeVocabularyReplacement,
  upsertVocabularyReplacement,
} from "../src/lib/vocabulary";

describe("normalizeVocabularyReplacement", () => {
  it("trims both sides and rejects empty values", () => {
    expect(
      normalizeVocabularyReplacement({
        from: "  Open A I  ",
        to: "  OpenAI  ",
        added_at: "2026-03-27T10:00:00.000Z",
      }),
    ).toEqual({
      from: "Open A I",
      to: "OpenAI",
      added_at: "2026-03-27T10:00:00.000Z",
    });

    expect(
      normalizeVocabularyReplacement({
        from: "   ",
        to: "OpenAI",
        added_at: "2026-03-27T10:00:00.000Z",
      }),
    ).toBeNull();
  });

  it("rejects identity replacements after trimming", () => {
    expect(
      normalizeVocabularyReplacement({
        from: "OpenAI",
        to: " OpenAI ",
        added_at: "2026-03-27T10:00:00.000Z",
      }),
    ).toBeNull();
  });
});

describe("upsertVocabularyReplacement", () => {
  it("replaces an existing rule by normalized source phrase", () => {
    const current: VocabularyReplacement[] = [
      {
        from: "Open A I",
        to: "OpenAI",
        added_at: "2026-03-27T10:00:00.000Z",
      },
      {
        from: "San Franciso",
        to: "San Francisco",
        added_at: "2026-03-27T11:00:00.000Z",
      },
    ];

    expect(
      upsertVocabularyReplacement(current, {
        from: "  Open A I ",
        to: "OpenAI Inc.",
        added_at: "2026-03-27T12:00:00.000Z",
      }),
    ).toEqual([
      {
        from: "Open A I",
        to: "OpenAI Inc.",
        added_at: "2026-03-27T10:00:00.000Z",
      },
      {
        from: "San Franciso",
        to: "San Francisco",
        added_at: "2026-03-27T11:00:00.000Z",
      },
    ]);
  });

  it("appends a new normalized rule when no match exists", () => {
    expect(
      upsertVocabularyReplacement([], {
        from: "Lang Chan",
        to: "LangChain",
        added_at: "2026-03-27T12:00:00.000Z",
      }),
    ).toEqual([
      {
        from: "Lang Chan",
        to: "LangChain",
        added_at: "2026-03-27T12:00:00.000Z",
      },
    ]);
  });
});

describe("formatVocabularyAddedAt", () => {
  it("renders a compact month-day label for valid timestamps", () => {
    expect(formatVocabularyAddedAt("2026-03-27T12:00:00.000Z")).toBe(
      "Mar 27, 2026",
    );
  });

  it("falls back to a neutral label for invalid timestamps", () => {
    expect(formatVocabularyAddedAt("not-a-date")).toBe("Unknown date");
  });
});
