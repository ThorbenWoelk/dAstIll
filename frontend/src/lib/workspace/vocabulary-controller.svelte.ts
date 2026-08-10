/**
 * Composable for vocabulary replacement modal state and save flow.
 *
 * Owns the modal's open/close lifecycle, the in-progress flag, and delegates
 * the actual preferences write to a caller-supplied `onSave` callback so this
 * module has no knowledge of channel order or other preference fields.
 */

import { prepareVocabularyReplacementSave } from "$lib/workspace/vocabulary";
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
  /**
   * Refresh in-memory replacements from the authenticated server document
   * before upsert. Without this, Correct can build `[newRule]` from empty
   * defaults and wipe existing vocabulary via a full-document PUT.
   */
  ensureReplacementsLoaded?: () => Promise<void>;
};

export function createVocabularyController(params: VocabularyControllerParams) {
  const {
    getReplacements,
    setReplacements,
    onError,
    onSave,
    ensureReplacementsLoaded,
  } = params;

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

  function setModalValue(value: string) {
    modalValue = value;
  }

  /** Validates, upserts, and persists the replacement via `onSave`. */
  async function confirm() {
    const source = modalSource?.trim();
    const replacement = modalValue.trim();
    if (!source || !replacement) return;

    creating = true;
    onError(null);

    try {
      const { next, changed } = await prepareVocabularyReplacementSave({
        getReplacements,
        ensureReplacementsLoaded,
        candidate: {
          from: source,
          to: replacement,
          // Transient timestamp for persistence, not reactive state
          // eslint-disable-next-line svelte/prefer-svelte-reactivity
          added_at: new Date().toISOString(),
        },
      });

      // No change - nothing to persist.
      if (!changed) {
        modalSource = null;
        modalValue = "";
        return;
      }

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
    get creating() {
      return creating;
    },
    open,
    close,
    setModalValue,
    confirm,
  };
}
