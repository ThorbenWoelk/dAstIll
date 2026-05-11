import { Window as HappyWindow } from "happy-dom";
import { afterAll, beforeAll, describe, expect, it } from "bun:test";
import {
  SECTION_NAVIGATION_ITEMS,
  goHintKeyForSection,
} from "../src/lib/navigation/section-navigation";
import {
  WORKSPACE_CONTENT_MODE_ORDER,
  goHintKeyForWorkspaceContentMode,
} from "../src/lib/workspace/navigation";
import {
  buildShortcutManual,
  computeGoHintBadgeStyles,
  GO_SEQUENCE_HINTS,
  isApplePlatform,
  isEditableShortcutTarget,
  isInsideModalDialog,
  resolveGlobalSectionShortcut,
  resolveInlineActionHintKey,
  resolveWorkspaceContentModeShortcut,
  shouldIgnoreGlobalShortcutNavigation,
} from "../src/lib/utils/keyboard-shortcuts";

beforeAll(() => {
  const w = new HappyWindow();
  (
    w as unknown as {
      SyntaxError?: typeof SyntaxError;
    }
  ).SyntaxError = SyntaxError;
  globalThis.window = w as unknown as typeof globalThis.window;
  globalThis.document = w.document as unknown as Document;
  globalThis.Element = w.Element;
  globalThis.HTMLElement = w.HTMLElement;
});

afterAll(() => {
  Reflect.deleteProperty(globalThis, "document");
  Reflect.deleteProperty(globalThis, "window");
  Reflect.deleteProperty(globalThis, "Element");
  Reflect.deleteProperty(globalThis, "HTMLElement");
});

describe("isEditableShortcutTarget", () => {
  it("detects textarea and input", () => {
    expect(isEditableShortcutTarget(document.createElement("textarea"))).toBe(
      true,
    );
    const input = document.createElement("input");
    input.type = "text";
    expect(isEditableShortcutTarget(input)).toBe(true);
  });

  it("ignores hidden inputs", () => {
    const input = document.createElement("input");
    input.type = "hidden";
    expect(isEditableShortcutTarget(input)).toBe(false);
  });

  it("detects contenteditable hosts", () => {
    const div = document.createElement("div");
    div.setAttribute("contenteditable", "true");
    expect(isEditableShortcutTarget(div)).toBe(true);
  });

  it("detects typing inside nested editable content", () => {
    const host = document.createElement("div");
    host.setAttribute("contenteditable", "true");
    const inner = document.createElement("span");
    host.appendChild(inner);
    expect(isEditableShortcutTarget(inner)).toBe(true);
  });

  it("returns false for inert targets", () => {
    expect(isEditableShortcutTarget(document.body)).toBe(false);
    expect(isEditableShortcutTarget(null)).toBe(false);
  });
});

describe("isInsideModalDialog", () => {
  it("detects dialog ancestors", () => {
    const dialog = document.createElement("div");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    const button = document.createElement("button");
    dialog.appendChild(button);
    document.body.appendChild(dialog);
    expect(isInsideModalDialog(button)).toBe(true);
    dialog.remove();
  });
});

describe("shouldIgnoreGlobalShortcutNavigation", () => {
  it("combines editable and modal checks", () => {
    const ta = document.createElement("textarea");
    expect(shouldIgnoreGlobalShortcutNavigation(ta)).toBe(true);
  });
});

describe("computeGoHintBadgeStyles", () => {
  it("returns no badges when there are no marked elements", () => {
    document.body.replaceChildren();
    expect(computeGoHintBadgeStyles()).toEqual([]);
  });
});

describe("GO_SEQUENCE_HINTS", () => {
  it("stays aligned with modifier-based navigation hints", () => {
    expect(GO_SEQUENCE_HINTS.map((h) => h.key)).toEqual([
      "1",
      "2",
      "3",
      "4",
      "5",
      "I",
      "S",
      "H",
      "T",
      "*",
      "]",
      "[",
      "↵",
      ".",
    ]);
  });

  it("reuses section and content-mode hint contracts", () => {
    expect(GO_SEQUENCE_HINTS.slice(0, SECTION_NAVIGATION_ITEMS.length)).toEqual(
      SECTION_NAVIGATION_ITEMS.map((item) => ({
        key: goHintKeyForSection(item.section),
        label: item.label,
      })),
    );

    expect(
      GO_SEQUENCE_HINTS.slice(
        SECTION_NAVIGATION_ITEMS.length,
        SECTION_NAVIGATION_ITEMS.length + WORKSPACE_CONTENT_MODE_ORDER.length,
      ).map((hint) => hint.key),
    ).toEqual(
      WORKSPACE_CONTENT_MODE_ORDER.map((mode) =>
        goHintKeyForWorkspaceContentMode(mode),
      ),
    );
  });
});

describe("resolveGlobalSectionShortcut", () => {
  it("maps the global number shortcuts to sections and docs", () => {
    expect(resolveGlobalSectionShortcut("1")).toBe("/");
    expect(resolveGlobalSectionShortcut("2")).toBe("/highlights");
    expect(resolveGlobalSectionShortcut("3")).toBe("/vocabulary");
    expect(resolveGlobalSectionShortcut("4")).toBe("/chat");
    expect(resolveGlobalSectionShortcut("5")).toBe("docs");
    expect(resolveGlobalSectionShortcut("6")).toBeNull();
  });
});

describe("resolveWorkspaceContentModeShortcut", () => {
  it("accepts mnemonic keys case-insensitively", () => {
    expect(resolveWorkspaceContentModeShortcut("i")).toBe("info");
    expect(resolveWorkspaceContentModeShortcut("S")).toBe("summary");
    expect(resolveWorkspaceContentModeShortcut("h")).toBe("highlights");
    expect(resolveWorkspaceContentModeShortcut("T")).toBe("transcript");
    expect(resolveWorkspaceContentModeShortcut("x")).toBeNull();
  });
});

describe("resolveInlineActionHintKey", () => {
  it("normalizes inline action shortcut keys", () => {
    expect(resolveInlineActionHintKey("Enter")).toBe("↵");
    expect(resolveInlineActionHintKey(".")).toBe(".");
    expect(resolveInlineActionHintKey("*")).toBe("*");
    expect(resolveInlineActionHintKey("]")).toBe("]");
    expect(resolveInlineActionHintKey("[")).toBe("[");
    expect(resolveInlineActionHintKey("x")).toBeNull();
  });
});

describe("buildShortcutManual", () => {
  it("includes core sections for Cmd label", () => {
    const groups = buildShortcutManual("Cmd");
    const titles = groups.map((g) => g.title);
    expect(titles).toContain("Everywhere");
    expect(titles).toContain("Workspace home");
    expect(titles).toContain("Chat");
    const everywhere = groups.find((g) => g.title === "Everywhere");
    expect(everywhere?.rows.some((r) => r.keys === "Cmd + 1")).toBe(true);
    expect(everywhere?.rows.some((r) => r.keys === "Cmd + 4")).toBe(true);
    expect(everywhere?.rows.some((r) => r.keys === "Cmd + ,")).toBe(true);
    const workspaceHome = groups.find((g) => g.title === "Workspace home");
    expect(workspaceHome?.rows.some((r) => r.keys === "Cmd + I")).toBe(true);
    expect(workspaceHome?.rows.some((r) => r.keys === "Cmd + T")).toBe(true);
    expect(workspaceHome?.rows.some((r) => r.keys === "Cmd + *")).toBe(true);
    expect(workspaceHome?.rows.some((r) => r.keys === "Cmd + ]")).toBe(true);
    expect(workspaceHome?.rows.some((r) => r.keys === "Cmd + [")).toBe(true);
    expect(workspaceHome?.rows.some((r) => r.keys === "Cmd + Return")).toBe(
      true,
    );
    expect(workspaceHome?.rows.some((r) => r.keys === "Cmd + .")).toBe(true);
    const guideTour = groups.find((g) => g.title === "Feature guide tour");
    expect(guideTour?.rows[0]?.keys).toBe("Arrow left or Arrow up");
  });

  it("swaps the primary modifier label for Ctrl layouts", () => {
    const groups = buildShortcutManual("Ctrl");
    const everywhere = groups.find((g) => g.title === "Everywhere");
    const workspaceHome = groups.find((g) => g.title === "Workspace home");
    expect(everywhere?.rows.some((r) => r.keys === "Ctrl + 1")).toBe(true);
    expect(everywhere?.rows.some((r) => r.keys === "Ctrl + ,")).toBe(true);
    expect(workspaceHome?.rows.some((r) => r.keys === "Ctrl + K")).toBe(true);
    expect(workspaceHome?.rows.some((r) => r.keys === "Ctrl + H")).toBe(true);
  });
});

describe("isApplePlatform", () => {
  it("returns a boolean without throwing", () => {
    expect(typeof isApplePlatform()).toBe("boolean");
  });
});
