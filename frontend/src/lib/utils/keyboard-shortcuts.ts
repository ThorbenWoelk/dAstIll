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
        {
          keys: "G U",
          description: "Open feature guide tour",
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
          keys: "G U",
          description:
            "Open feature guide (press G for hints, then U within a second)",
        },
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
          description: "Next step (same as clicking outside the card)",
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
  { key: "U", label: "Feature guide" },
] as const;

export type GoSequenceState = {
  pending: boolean;
  timeoutId: ReturnType<typeof setTimeout> | null;
};

export type GoHintBadge = {
  key: string;
  style: string;
};

/**
 * One badge per visible `[data-go-hint-key]` target: beside rail rows (desktop),
 * above tab items (mobile), top-left on main, or fallback U above the tab bar.
 */
export function computeGoHintBadgeStyles(): GoHintBadge[] {
  if (typeof document === "undefined") {
    return [];
  }

  const isLg = window.matchMedia("(min-width: 1024px)").matches;
  const nodes = document.querySelectorAll<HTMLElement>("[data-go-hint-key]");
  const out: GoHintBadge[] = [];
  const seen = new Set<string>();

  for (const el of nodes) {
    const key = el.dataset.goHintKey?.trim();
    if (!key) continue;
    if (el.getClientRects().length === 0) continue;

    const r = el.getBoundingClientRect();
    const inMobileNav = Boolean(el.closest("#app-section-nav-mobile"));
    const isMain = el.id === "main-content";

    let style: string;
    if (inMobileNav) {
      const top = Math.max(6, r.top - 20);
      const cx = r.left + r.width / 2;
      style = `left:${Math.round(cx)}px;top:${Math.round(top)}px;transform:translateX(-50%)`;
    } else if (isMain) {
      style = `left:${Math.round(r.left + 12)}px;top:${Math.round(r.top + 12)}px`;
    } else {
      const gap = 8;
      style = `left:${Math.round(r.right + gap)}px;top:${Math.round(r.top + r.height / 2)}px;transform:translateY(-50%)`;
    }

    out.push({ key, style });
    seen.add(key);
  }

  if (!isLg && !seen.has("U")) {
    const mobile = document.getElementById("app-section-nav-mobile");
    const r = mobile?.getBoundingClientRect();
    if (r && r.width > 0 && r.height > 0) {
      const cx = r.left + r.width * 0.9;
      const top = Math.max(6, r.top - 20);
      out.push({
        key: "U",
        style: `left:${Math.round(cx)}px;top:${Math.round(top)}px;transform:translateX(-50%)`,
      });
    }
  }

  return out;
}

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
