/**
 * Shared helpers for app-wide keyboard shortcuts and the shortcuts reference modal.
 */

const EDITABLE_SELECTORS =
  "input:not([type='hidden']):not([disabled]), textarea:not([disabled]), select:not([disabled]), [contenteditable='true'], [contenteditable='']";

export function isEditableShortcutTarget(target: EventTarget | null): boolean {
  if (!target || !(target instanceof Element)) {
    return false;
  }
  const el = target as HTMLElement;
  if (el.isContentEditable) {
    return true;
  }
  return Boolean(el.closest(EDITABLE_SELECTORS));
}

/** True when focus is inside an aria-modal dialog (confirm modals, feature tour, shortcuts sheet, etc.). */
export function isInsideModalDialog(target: EventTarget | null): boolean {
  if (!target || !(target instanceof Element)) {
    return false;
  }
  return Boolean(target.closest('[role="dialog"][aria-modal="true"]'));
}

export function shouldIgnoreGlobalShortcutNavigation(
  target: EventTarget | null,
): boolean {
  return isEditableShortcutTarget(target) || isInsideModalDialog(target);
}

export function isApplePlatform(): boolean {
  if (typeof navigator === "undefined") {
    return false;
  }
  const p = navigator.platform ?? "";
  const ua = navigator.userAgent ?? "";
  return /Mac|iPhone|iPad|iPod/i.test(p) || /Mac OS X/i.test(ua);
}

export function primaryModifierLabel(): "Cmd" | "Ctrl" {
  return isApplePlatform() ? "Cmd" : "Ctrl";
}

export type ShortcutManualRow = {
  keys: string;
  description: string;
};

export type ShortcutManualGroup = {
  title: string;
  rows: ShortcutManualRow[];
};

export function buildShortcutManual(
  mod: "Cmd" | "Ctrl",
): ShortcutManualGroup[] {
  return [
    {
      title: "Everywhere",
      rows: [
        {
          keys: `${mod} + /`,
          description: "Open this keyboard shortcuts reference",
        },
        {
          keys: "?",
          description: "Open shortcuts reference (when not typing in a field)",
        },
        {
          keys: "G W",
          description:
            "Go to Workspace (press G to see hints, then W within a second)",
        },
        {
          keys: "G Q",
          description: "Go to Queue",
        },
        {
          keys: "G H",
          description: "Go to Highlights",
        },
        {
          keys: "G C",
          description: "Go to Chat",
        },
        {
          keys: "G D",
          description: "Open documentation in a new tab",
        },
        {
          keys: "G M",
          description: "Move focus to main content region",
        },
      ],
    },
    {
      title: "Workspace home",
      rows: [
        {
          keys: `${mod} + K`,
          description: "Focus workspace search / ask bar",
        },
        {
          keys: "Ctrl + L",
          description: "Toggle Search vs Ask submit mode (Windows / Linux)",
        },
        {
          keys: "/",
          description: "Focus search bar (when not typing in a field)",
        },
      ],
    },
    {
      title: "Chat",
      rows: [
        {
          keys: `${mod} + Shift + N`,
          description: "Start a new conversation",
        },
        {
          keys: "/",
          description: "Focus message field (when not typing elsewhere)",
        },
      ],
    },
    {
      title: "Chat composer",
      rows: [
        {
          keys: "Enter",
          description: "Send message",
        },
        {
          keys: "Shift + Enter",
          description: "New line in the message",
        },
        {
          keys: "Arrow up or Ctrl + P",
          description: "Previous message in history (first line only)",
        },
        {
          keys: "Arrow down or Ctrl + N",
          description: "Next message in history (last line only)",
        },
      ],
    },
    {
      title: "Feature guide tour",
      rows: [
        {
          keys: "Arrow left or Arrow up",
          description: "Previous step",
        },
        {
          keys: "Arrow right or Arrow down",
          description: "Next step",
        },
        {
          keys: "Escape",
          description: "Close guide",
        },
      ],
    },
  ];
}

const GO_SEQUENCE_MS = 1000;

/** Second key in the G-then-key navigation chord; labels shown after pressing G. */
export const GO_SEQUENCE_HINTS: readonly { key: string; label: string }[] = [
  { key: "W", label: "Workspace" },
  { key: "Q", label: "Queue" },
  { key: "H", label: "Highlights" },
  { key: "C", label: "Chat" },
  { key: "D", label: "Docs" },
  { key: "M", label: "Main content" },
] as const;

export type GoSequenceState = {
  pending: boolean;
  timeoutId: ReturnType<typeof setTimeout> | null;
};

export function clearGoSequence(state: GoSequenceState): void {
  state.pending = false;
  if (state.timeoutId !== null) {
    clearTimeout(state.timeoutId);
    state.timeoutId = null;
  }
}

/** Arms the G-prefix sequence; `onExpire` runs when the window elapses without a second key. */
export function armGoSequence(
  state: GoSequenceState,
  onExpire?: () => void,
): void {
  state.pending = true;
  if (state.timeoutId !== null) {
    clearTimeout(state.timeoutId);
  }
  state.timeoutId = setTimeout(() => {
    state.pending = false;
    state.timeoutId = null;
    onExpire?.();
  }, GO_SEQUENCE_MS);
}

export function focusMainContentRegion(): void {
  const main = document.getElementById("main-content");
  if (!main) {
    return;
  }
  const { pathname, search } = window.location;
  window.history.replaceState(
    window.history.state,
    "",
    `${pathname}${search}#main-content`,
  );
  main.focus({ preventScroll: false });
  main.scrollIntoView({ block: "nearest", behavior: "auto" });
}
