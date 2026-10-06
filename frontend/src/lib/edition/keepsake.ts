import type { Channel } from "$lib/api";
import type { SectionId, Story } from "$lib/edition/stories";

/**
 * The last printed edition, kept in this browser so the paper opens
 * instantly while a fresh copy loads. Per reader, best effort only.
 */
interface StoredEdition {
  version: 1;
  savedAt: number;
  channels: Channel[];
  stories: Story[];
}

const EDITION_KEY_PREFIX = "dastill.edition.v1.";
const SECTION_KEY_PREFIX = "dastill.section.v1.";
/** localStorage is ~5 MB per origin. Leave room for Firebase's own keys. */
const MAX_STORED_CHARS = 2_000_000;

function storage(): Storage | null {
  try {
    return typeof localStorage === "undefined" ? null : localStorage;
  } catch {
    return null;
  }
}

export function loadStoredEdition(
  uid: string,
): { channels: Channel[]; stories: Story[]; savedAt: number } | null {
  try {
    const raw = storage()?.getItem(EDITION_KEY_PREFIX + uid);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as StoredEdition;
    if (parsed?.version !== 1 || !Array.isArray(parsed.stories)) return null;
    return parsed;
  } catch {
    return null;
  }
}

export function storeEdition(
  uid: string,
  channels: Channel[],
  stories: Story[],
) {
  const target = storage();
  if (!target) return;
  const record: StoredEdition = {
    version: 1,
    savedAt: Date.now(),
    channels,
    stories,
  };
  try {
    const raw = JSON.stringify(record);
    if (raw.length > MAX_STORED_CHARS) {
      target.removeItem(EDITION_KEY_PREFIX + uid);
      return;
    }
    target.setItem(EDITION_KEY_PREFIX + uid, raw);
  } catch {
    // Quota or privacy mode: the paper still works, it just opens slower.
  }
}

export function forgetEdition(uid: string) {
  try {
    storage()?.removeItem(EDITION_KEY_PREFIX + uid);
  } catch {
    // ignore
  }
}

export function loadStoredSection(uid: string): SectionId | null {
  try {
    return storage()?.getItem(SECTION_KEY_PREFIX + uid) ?? null;
  } catch {
    return null;
  }
}

export function storeSection(uid: string, section: SectionId) {
  try {
    storage()?.setItem(SECTION_KEY_PREFIX + uid, section);
  } catch {
    // ignore
  }
}
