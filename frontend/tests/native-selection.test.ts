import { afterEach, beforeEach, describe, expect, it } from "bun:test";

import { registerNativeSelectionHandlers } from "../src/lib/platform/native-selection";
import { isTauriRuntime } from "../src/lib/platform/tauri-runtime";

const originalWindow = globalThis.window;

beforeEach(() => {
  Object.defineProperty(globalThis, "window", {
    value: {},
    configurable: true,
  });
});

afterEach(() => {
  if (originalWindow === undefined) {
    delete (globalThis as typeof globalThis & { window?: unknown }).window;
  } else {
    Object.defineProperty(globalThis, "window", {
      value: originalWindow,
      configurable: true,
    });
  }
});

describe("native selection bridge", () => {
  it("registers and restores the native selection callbacks", () => {
    let highlightCount = 0;
    let correctCount = 0;

    const cleanup = registerNativeSelectionHandlers(
      () => {
        highlightCount += 1;
      },
      () => {
        correctCount += 1;
      },
    );

    window.__tauri_selection_highlight?.();
    window.__tauri_selection_correct?.();

    expect(highlightCount).toBe(1);
    expect(correctCount).toBe(1);

    cleanup();

    expect(window.__tauri_selection_highlight).toBeUndefined();
    expect(window.__tauri_selection_correct).toBeUndefined();
  });

  it("detects the Tauri runtime from the global internals marker", () => {
    expect(isTauriRuntime()).toBe(false);
    window.__TAURI_INTERNALS__ = {};
    expect(isTauriRuntime()).toBe(true);
    delete window.__TAURI_INTERNALS__;
  });
});
