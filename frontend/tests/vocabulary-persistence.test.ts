import { describe, expect, it, mock } from "bun:test";

import type { UserPreferences, VocabularyReplacement } from "../src/lib/types";
import { saveVocabularyReplacements } from "../src/lib/workspace/vocabulary-persistence";
import { createVocabularyController } from "../src/lib/workspace/vocabulary-controller.svelte";

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

    expect(saved).toEqual([
      prefs(nextVocabulary, ["channel-a", "channel-b"]),
    ]);
    expect(result.channel_order).toEqual(["channel-a", "channel-b"]);
    expect(result.vocabulary_replacements).toEqual(nextVocabulary);
  });
});

describe("createVocabularyController confirm hydration", () => {
  it("loads server replacements before upsert so Correct cannot wipe existing rules", async () => {
    let replacements: VocabularyReplacement[] = [];
    const saved: VocabularyReplacement[][] = [];
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

    const controller = createVocabularyController({
      getReplacements: () => replacements,
      setReplacements: (next) => {
        replacements = next;
      },
      onError: () => {},
      ensureReplacementsLoaded,
      onSave: async (next) => {
        saved.push(next);
      },
    });

    controller.open("dAstIl");
    controller.setModalValue("dAstIll");
    await controller.confirm();

    expect(ensureReplacementsLoaded).toHaveBeenCalledTimes(1);
    expect(saved).toHaveLength(1);
    expect(saved[0]?.map((entry) => entry.from)).toEqual([
      "Open A I",
      "San Franciso",
      "dAstIl",
    ]);
  });

  it("surfaces ensureReplacementsLoaded failures without writing preferences", async () => {
    let replacements: VocabularyReplacement[] = [];
    const saved: VocabularyReplacement[][] = [];
    let errorMessage: string | null = null;

    const controller = createVocabularyController({
      getReplacements: () => replacements,
      setReplacements: (next) => {
        replacements = next;
      },
      onError: (message) => {
        errorMessage = message;
      },
      ensureReplacementsLoaded: async () => {
        throw new Error("Sign-in required to save vocabulary.");
      },
      onSave: async (next) => {
        saved.push(next);
      },
    });

    controller.open("Open A I");
    controller.setModalValue("OpenAI");
    await controller.confirm();

    expect(saved).toEqual([]);
    expect(replacements).toEqual([]);
    expect(errorMessage).toBe("Sign-in required to save vocabulary.");
  });
});
