import type { WorkspaceOverlaysState } from "$lib/workspace/component-props";

/**
 * Returns true when any workspace overlay (error toast or modal) is currently
 * visible. Useful for accessibility concerns such as focus trapping.
 */
export function hasActiveOverlay(state: WorkspaceOverlaysState): boolean {
  return (
    state.errorMessage !== null ||
    state.showDeleteConfirmation ||
    state.showDeleteAccessPrompt
  );
}
