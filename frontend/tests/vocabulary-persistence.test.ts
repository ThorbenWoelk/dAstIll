import { describe, expect, it, mock } from "bun:test";

import type { UserPreferences, VocabularyReplacement } from "../src/lib/types";
import { prepareVocabularyReplacementSave } from "../src/lib/workspace/vocabulary";
import { saveVocabularyReplacements } from "../src/lib/workspace/vocabulary-persistence";

function prefs(
  vocabulary: VocabularyReplacement[],
  channelOrder: string[] = ["channel-a"],
): UserPreferences {
  return {
    channel_order: channelOrder,
    channel_sort_mode: "custom",
    vocabulary_replacements: vocabulary,
  };
}

describe("saveVocabularyReplacements", () => {
  it("merges vocabulary into the latest server preferences document", async () => {
    const saved: UserPreferences[] = [];
    const nextVocabulary: VocabularyReplacement[] = [
      {
        from: "Open A I",
        to: "OpenAI",
        added_at: "2026-03-27T10:00:00.000Z",
      },
      {
        from: "San Franciso",
        to: "San Francisco",
        added_at: "2026-08-10T11:00:00.000Z",
      },
    ];

    const result = await saveVocabularyReplacements({
      getPreferences: async () =>
        prefs(
          [
            {
              from: "Open A I",
              to: "OpenAI",
              added_at: "2026-03-27T10:00:00.000Z",
            },
          ],
          ["channel-a", "channel-b"],
        ),
      savePreferences: async (preferences) => {
        saved.push(preferences);
      },
      replacements: nextVocabulary,
    });

    expect(saved).toEqual([prefs(nextVocabulary, ["channel-a", "channel-b"])]);
    expect(result.channel_order).toEqual(["channel-a", "channel-b"]);
    expect(result.vocabulary_replacements).toEqual(nextVocabulary);
  });
});

describe("prepareVocabularyReplacementSave", () => {
  it("loads server replacements before upsert so Correct cannot wipe existing rules", async () => {
    let replacements: VocabularyReplacement[] = [];
    const ensureReplacementsLoaded = mock(async () => {
      replacements = [
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
    });

    const prepared = await prepareVocabularyReplacementSave({
      getReplacements: () => replacements,
      ensureReplacementsLoaded,
      candidate: {
        from: "dAstIl",
        to: "dAstIll",
        added_at: "2026-08-10T12:00:00.000Z",
      },
    });

    expect(ensureReplacementsLoaded).toHaveBeenCalledTimes(1);
    expect(prepared.changed).toBe(true);
    expect(prepared.next.map((entry) => entry.from)).toEqual([
      "Open A I",
      "San Franciso",
      "dAstIl",
    ]);
  });

  it("propagates ensureReplacementsLoaded failures before building a save payload", async () => {
    const replacements: VocabularyReplacement[] = [];

    await expect(
      prepareVocabularyReplacementSave({
        getReplacements: () => replacements,
        ensureReplacementsLoaded: async () => {
          throw new Error("Sign-in required to save vocabulary.");
        },
        candidate: {
          from: "Open A I",
          to: "OpenAI",
          added_at: "2026-08-10T12:00:00.000Z",
        },
      }),
    ).rejects.toThrow("Sign-in required to save vocabulary.");
    expect(replacements).toEqual([]);
  });
});
