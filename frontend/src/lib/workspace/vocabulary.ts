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
