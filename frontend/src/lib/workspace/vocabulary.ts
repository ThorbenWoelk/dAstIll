import type { VocabularyReplacement } from "$lib/types";

export function normalizeVocabularyReplacement(
  replacement: VocabularyReplacement,
): VocabularyReplacement | null {
  const from = replacement.from.trim();
  const to = replacement.to.trim();

  if (!from || !to || from === to) {
    return null;
  }

  return { from, to, added_at: replacement.added_at };
}

export function upsertVocabularyReplacement(
  current: VocabularyReplacement[],
  candidate: VocabularyReplacement,
): VocabularyReplacement[] {
  const normalized = normalizeVocabularyReplacement(candidate);
  if (!normalized) {
    return current;
  }

  const matchIndex = current.findIndex(
    (entry) => entry.from.trim() === normalized.from,
  );

  if (matchIndex === -1) {
    return [...current, normalized];
  }

  return current.map((entry, index) =>
    index === matchIndex ? { ...normalized, added_at: entry.added_at } : entry,
  );
}

/**
 * Refresh replacements from the server (when provided), then upsert the
 * candidate. Used by Correct so empty pre-hydration defaults cannot become
 * the sole persisted vocabulary list.
 */
export async function prepareVocabularyReplacementSave(options: {
  getReplacements: () => VocabularyReplacement[];
  candidate: VocabularyReplacement;
  ensureReplacementsLoaded?: () => Promise<void>;
}): Promise<{
  current: VocabularyReplacement[];
  next: VocabularyReplacement[];
  changed: boolean;
}> {
  if (options.ensureReplacementsLoaded) {
    await options.ensureReplacementsLoaded();
  }
  const current = options.getReplacements();
  const next = upsertVocabularyReplacement(current, options.candidate);
  return {
    current,
    next,
    changed: next !== current,
  };
}

export function formatVocabularyAddedAt(value: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return "Unknown date";
  }

  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(parsed);
}
