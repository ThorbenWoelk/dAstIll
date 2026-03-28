/**
 * Composable for vocabulary replacement modal state and save flow.
 *
 * Owns the modal's open/close lifecycle, the in-progress flag, and delegates
 * the actual preferences write to a caller-supplied `onSave` callback so this
 * module has no knowledge of channel order or other preference fields.
 */

import { upsertVocabularyReplacement } from "$lib/vocabulary";
import type { VocabularyReplacement } from "$lib/bindings/VocabularyReplacement";

export type VocabularyControllerParams = {
  /** Returns the current list of replacements (read from prefs after hydration). */
  getReplacements: () => VocabularyReplacement[];
  /** Called with the updated list after a successful save. */
  setReplacements: (r: VocabularyReplacement[]) => void;
  /** Called when an error occurs; pass null to clear. */
  onError: (msg: string | null) => void;
  /**
   * Performs the actual preferences write. Receives the next replacements list
   * so the caller can merge it with other preference fields (channel_order, etc.).
   */
  onSave: (replacements: VocabularyReplacement[]) => Promise<void>;
};

export function createVocabularyController(params: VocabularyControllerParams) {
  const { getReplacements, setReplacements, onError, onSave } = params;

  let modalSource = $state<string | null>(null);
  let modalValue = $state("");
  let creating = $state(false);

  /** Opens the vocabulary modal pre-filled with `selectedText`. No-op if empty. */
  function open(selectedText: string) {
    const source = selectedText.trim();
    if (!source) return;
    modalSource = source;
    modalValue = source;
  }

  /** Closes the modal without saving. Blocked while a save is in progress. */
  function close() {
    if (creating) return;
    modalSource = null;
    modalValue = "";
  }

  /** Validates, upserts, and persists the replacement via `onSave`. */
  async function confirm() {
    const source = modalSource?.trim();
    const replacement = modalValue.trim();
    if (!source || !replacement) return;

    const current = getReplacements();
    const next = upsertVocabularyReplacement(current, {
      from: source,
      to: replacement,
      // Transient timestamp for persistence, not reactive state
      // eslint-disable-next-line svelte/prefer-svelte-reactivity
      added_at: new Date().toISOString(),
    });

    // No change - nothing to persist.
    if (next === current) return;

    creating = true;
    onError(null);

    try {
      setReplacements(next);
      await onSave(next);
      modalSource = null;
      modalValue = "";
    } catch (error) {
      onError((error as Error).message);
    } finally {
      creating = false;
    }
  }

  return {
    get replacements() {
      return getReplacements();
    },
    set replacements(v: VocabularyReplacement[]) {
      setReplacements(v);
    },
    get modalSource() {
      return modalSource;
    },
    get modalValue() {
      return modalValue;
    },
    set modalValue(v: string) {
      modalValue = v;
    },
    get creating() {
      return creating;
    },
    open,
    close,
    confirm,
  };
}
