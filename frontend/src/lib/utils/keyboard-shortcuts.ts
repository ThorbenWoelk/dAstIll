import {
  SECTION_NAVIGATION_ITEMS,
  goHintKeyForSection,
  type AppNavigationSection,
} from "$lib/section-navigation";
import {
  WORKSPACE_CONTENT_MODE_ORDER,
  goHintKeyForWorkspaceContentMode,
} from "$lib/workspace/navigation";
import type { WorkspaceContentMode } from "$lib/workspace/types";

/**
 * Shared helpers for app-wide keyboard shortcuts and the shortcuts reference modal.
 */

/** Dispatched on the window with `detail.mode` to switch workspace video content tab (home route). */
export const DASTILL_SET_WORKSPACE_CONTENT_MODE_EVENT =
  "dastill:set-workspace-content-mode" as const;

export type GlobalSectionShortcut =
  | "/"
  | "/highlights"
  | "/vocabulary"
  | "/chat"
  | "docs";

export type WorkspaceContentModeShortcut = WorkspaceContentMode;

type PrimaryModifierShortcut = {
  key: string;
  description: string;
};

type GlobalSectionShortcutDefinition = PrimaryModifierShortcut & {
  destination: GlobalSectionShortcut;
  hintLabel: string;
};

type WorkspaceContentShortcutDefinition = PrimaryModifierShortcut & {
  mode: WorkspaceContentModeShortcut;
  hintLabel: string;
};

type InlineActionShortcutDefinition = {
  eventKey: string;
  hintKey: string;
  description: string;
  hintLabel: string;
};

const INLINE_ACTION_SHORTCUTS: readonly InlineActionShortcutDefinition[] = [
  {
    eventKey: "*",
    hintKey: "*",
    description: "Run the primary summary/transcript action when available",
    hintLabel: "Primary action",
  },
  {
    eventKey: "]",
    hintKey: "]",
    description: "Open the current item on YouTube when available",
    hintLabel: "Open on YouTube",
  },
  {
    eventKey: "[",
    hintKey: "[",
    description: "Edit the current summary or transcript when available",
    hintLabel: "Edit action",
  },
  {
    eventKey: "Enter",
    hintKey: "↵",
    description: "Delete / reset the current item when available",
    hintLabel: "Delete / reset action",
  },
  {
    eventKey: ".",
    hintKey: ".",
    description: "Toggle the read check when available",
    hintLabel: "Read check action",
  },
] as const;

const GLOBAL_SECTION_SHORTCUT_DESCRIPTIONS: Record<
  AppNavigationSection,
  string
> = {
  workspace: "Go to Workspace",
  highlights: "Go to Highlights",
  vocabulary: "Go to Vocabulary",
  chat: "Go to Chat",
  docs: "Open documentation in a new tab",
};

const WORKSPACE_CONTENT_MODE_SHORTCUT_DESCRIPTIONS: Record<
  WorkspaceContentModeShortcut,
  string
> = {
  info: "Switch video panel to Info",
  summary: "Switch video panel to Summary",
  highlights: "Switch video panel to Highlights",
  transcript: "Switch video panel to Transcript",
};

const WORKSPACE_CONTENT_MODE_SHORTCUT_HINT_LABELS: Record<
  WorkspaceContentModeShortcut,
  string
> = {
  info: "Info (video tab)",
  summary: "Summary (video tab)",
  highlights: "Highlights (video tab)",
  transcript: "Transcript (video tab)",
};

const EDITABLE_SELECTORS =
  "input:not([type='hidden']):not([disabled]), textarea:not([disabled]), select:not([disabled]), [contenteditable='true'], [contenteditable='']";

function withPrimaryModifier(
  mod: "Cmd" | "Ctrl",
  rows: readonly PrimaryModifierShortcut[],
): ShortcutManualRow[] {
  return rows.map(({ key, description }) => ({
    keys: `${mod} + ${key}`,
    description,
  }));
}

export function resolveGlobalSectionShortcut(
  key: string,
): GlobalSectionShortcut | null {
  const normalized = key.trim();
  return (
    GLOBAL_SECTION_SHORTCUTS.find((entry) => entry.key === normalized)
      ?.destination ?? null
  );
}

export function resolveWorkspaceContentModeShortcut(
  key: string,
): WorkspaceContentModeShortcut | null {
  const normalized = key.trim().toLowerCase();
  return (
    WORKSPACE_CONTENT_MODE_SHORTCUTS.find(
      (entry) => entry.key.toLowerCase() === normalized,
    )?.mode ?? null
  );
}

export function resolveInlineActionHintKey(key: string): string | null {
  return (
    INLINE_ACTION_SHORTCUTS.find((entry) => entry.eventKey === key)?.hintKey ??
    null
  );
}

function buildGlobalSectionShortcuts(): readonly GlobalSectionShortcutDefinition[] {
  return SECTION_NAVIGATION_ITEMS.map((item) => ({
    key: goHintKeyForSection(item.section),
    description: GLOBAL_SECTION_SHORTCUT_DESCRIPTIONS[item.section],
    destination:
      item.section === "docs"
        ? "docs"
        : (item.href as Exclude<GlobalSectionShortcut, "docs">),
    hintLabel: item.label,
  }));
}

function buildWorkspaceContentModeShortcuts(): readonly WorkspaceContentShortcutDefinition[] {
  return WORKSPACE_CONTENT_MODE_ORDER.map((mode) => {
    const key = goHintKeyForWorkspaceContentMode(mode);
    if (!key) {
      throw new Error(
        `Missing workspace content shortcut key for mode: ${mode}`,
      );
    }

    return {
      key,
      description: WORKSPACE_CONTENT_MODE_SHORTCUT_DESCRIPTIONS[mode],
      mode,
      hintLabel: WORKSPACE_CONTENT_MODE_SHORTCUT_HINT_LABELS[mode],
    };
  });
}

const GLOBAL_SECTION_SHORTCUTS = buildGlobalSectionShortcuts();
const WORKSPACE_CONTENT_MODE_SHORTCUTS = buildWorkspaceContentModeShortcuts();

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
        ...withPrimaryModifier(mod, GLOBAL_SECTION_SHORTCUTS),
        {
          keys: `${mod} + ,`,
          description: "Open settings",
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
        ...withPrimaryModifier(mod, WORKSPACE_CONTENT_MODE_SHORTCUTS),
        ...INLINE_ACTION_SHORTCUTS.map(({ eventKey, description }) => ({
          keys: `${mod} + ${eventKey === "Enter" ? "Return" : eventKey}`,
          description,
        })),
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
      title: "Summary Audio",
      rows: [
        {
          keys: "Space",
          description: "Play / Pause",
        },
        {
          keys: "Arrow Left",
          description: "Back 10 seconds",
        },
        {
          keys: "Arrow Right",
          description: "Forward 10 seconds",
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
          keys: "Enter, Arrow right, or Arrow down",
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

/** Visible shortcut hint badges shown while holding Cmd/Ctrl. */
export const GO_SEQUENCE_HINTS: readonly { key: string; label: string }[] = [
  ...GLOBAL_SECTION_SHORTCUTS.map(({ key, hintLabel }) => ({
    key,
    label: hintLabel,
  })),
  ...WORKSPACE_CONTENT_MODE_SHORTCUTS.map(({ key, hintLabel }) => ({
    key,
    label: hintLabel,
  })),
  ...INLINE_ACTION_SHORTCUTS.map(({ hintKey, hintLabel }) => ({
    key: hintKey,
    label: hintLabel,
  })),
] as const;

export type GoHintBadge = {
  key: string;
  style: string;
};

/** One badge per visible `[data-go-hint-key]` target, rendered over the target itself. */
export function computeGoHintBadgeStyles(): GoHintBadge[] {
  if (typeof document === "undefined") {
    return [];
  }

  const nodes = document.querySelectorAll<HTMLElement>("[data-go-hint-key]");
  const out: GoHintBadge[] = [];

  for (const el of nodes) {
    const key = el.dataset.goHintKey?.trim();
    if (!key) continue;
    if (el.getClientRects().length === 0) continue;

    const r = el.getBoundingClientRect();
    const top = Math.round(r.top + r.height * 0.86);
    const isCompactTarget = r.width <= 44;
    const rightInset = isCompactTarget ? -8 : 6;
    const left = Math.round(r.right + rightInset);
    const style = `left:${left}px;top:${top}px;transform:translate(-100%,-50%)`;

    out.push({ key, style });
  }

  return out;
}

/** Focus the visible section navigation control (mobile section picker or desktop rail). */
export function focusSectionTabsNav(): void {
  if (typeof document === "undefined") {
    return;
  }

  const pickTab = (root: HTMLElement | null): HTMLElement | null => {
    if (!root || root.getClientRects().length === 0) {
      return null;
    }
    const current = root.querySelector<HTMLElement>("a[aria-current='page']");
    if (current) {
      return current;
    }
    return root.querySelector<HTMLElement>("a[href]");
  };

  const mobile = document.getElementById("app-section-nav-mobile");
  const mobileTab = pickTab(mobile);
  if (mobileTab) {
    mobileTab.focus({ preventScroll: false });
    mobile?.scrollIntoView({ block: "nearest", behavior: "auto" });
    return;
  }

  const rail = document.getElementById("app-section-nav-rail");
  const railTab = pickTab(rail);
  if (railTab) {
    railTab.focus({ preventScroll: false });
    return;
  }

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
