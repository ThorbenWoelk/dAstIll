import { Window as HappyWindow } from "happy-dom";
import { afterAll, beforeAll, describe, expect, it } from "bun:test";
import {
  armGoSequence,
  buildShortcutManual,
  clearGoSequence,
  computeGoHintBadgeStyles,
  GO_SEQUENCE_HINTS,
  isApplePlatform,
  isEditableShortcutTarget,
  isInsideModalDialog,
  shouldIgnoreGlobalShortcutNavigation,
  type GoSequenceState,
} from "../src/lib/utils/keyboard-shortcuts";

beforeAll(() => {
  const w = new HappyWindow();
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

describe("go sequence helpers", () => {
  it("arms and clears pending state", () => {
    const state: GoSequenceState = { pending: false, timeoutId: null };
    armGoSequence(state);
    expect(state.pending).toBe(true);
    expect(state.timeoutId).not.toBeNull();
    clearGoSequence(state);
    expect(state.pending).toBe(false);
    expect(state.timeoutId).toBeNull();
  });
});

describe("computeGoHintBadgeStyles", () => {
  it("returns no badges when there are no marked elements", () => {
    document.body.replaceChildren();
    expect(computeGoHintBadgeStyles()).toEqual([]);
  });
});

describe("GO_SEQUENCE_HINTS", () => {
  it("stays aligned with G-then-letter navigation (GlobalKeyboardShortcuts)", () => {
    expect(GO_SEQUENCE_HINTS.map((h) => h.key.toLowerCase())).toEqual([
      "w",
      "q",
      "h",
      "c",
      "d",
      "m",
      "u",
    ]);
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
    expect(everywhere?.rows.some((r) => r.keys.includes("G W"))).toBe(true);
    expect(everywhere?.rows.some((r) => r.keys.includes("G U"))).toBe(true);
    const guideTour = groups.find((g) => g.title === "Feature guide tour");
    expect(guideTour?.rows[0]?.keys).toBe("G U");
  });
});

describe("isApplePlatform", () => {
  it("returns a boolean without throwing", () => {
    expect(typeof isApplePlatform()).toBe("boolean");
  });
});
