import { describe, expect, it } from "bun:test";

import { hasActiveOverlay } from "../src/lib/workspace/overlays";
import type { WorkspaceOverlaysState } from "../src/lib/workspace/component-props";

function makeOverlaysState(
  overrides: Partial<WorkspaceOverlaysState> = {},
): WorkspaceOverlaysState {
  return {
    errorMessage: null,
    showDeleteConfirmation: false,
    showDeleteAccessPrompt: false,
    ...overrides,
  };
}

describe("hasActiveOverlay", () => {
  it("returns false when no overlay is active", () => {
    const state = makeOverlaysState();
    expect(hasActiveOverlay(state)).toBe(false);
  });

  it("returns true when errorMessage is set", () => {
    const state = makeOverlaysState({ errorMessage: "Something went wrong" });
    expect(hasActiveOverlay(state)).toBe(true);
  });

  it("returns true when delete confirmation is shown", () => {
    const state = makeOverlaysState({ showDeleteConfirmation: true });
    expect(hasActiveOverlay(state)).toBe(true);
  });

  it("returns true when admin access prompt is shown", () => {
    const state = makeOverlaysState({ showDeleteAccessPrompt: true });
    expect(hasActiveOverlay(state)).toBe(true);
  });

  it("returns true when multiple overlays are active simultaneously", () => {
    const state = makeOverlaysState({
      errorMessage: "Error occurred",
      showDeleteConfirmation: true,
    });
    expect(hasActiveOverlay(state)).toBe(true);
  });

  it("returns false when errorMessage is empty string", () => {
    const state = makeOverlaysState({ errorMessage: null });
    expect(hasActiveOverlay(state)).toBe(false);
  });
});
