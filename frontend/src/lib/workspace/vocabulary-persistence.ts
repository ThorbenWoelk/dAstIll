import type { UserPreferences, VocabularyReplacement } from "$lib/types";

/**
 * Persist vocabulary as a field merge against the latest server preferences.
 * Callers must build `replacements` from a freshly loaded server list so a
 * pre-hydration Correct cannot replace the full document with `[newRule]`.
 */
export async function saveVocabularyReplacements(options: {
  getPreferences: () => Promise<UserPreferences>;
  savePreferences: (preferences: UserPreferences) => Promise<void>;
  replacements: VocabularyReplacement[];
}): Promise<UserPreferences> {
  const current = await options.getPreferences();
  const next: UserPreferences = {
    ...current,
    vocabulary_replacements: options.replacements,
  };
  await options.savePreferences(next);
  return next;
}
