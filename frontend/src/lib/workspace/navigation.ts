import type { WorkspaceContentMode } from "$lib/workspace/types";

export const WORKSPACE_CONTENT_MODE_ORDER: WorkspaceContentMode[] = [
  "info",
  "transcript",
  "summary",
  "highlights",
];

export function resolveDefaultContentMode(options: {
  hasHighlights: boolean;
  hasSummary: boolean;
  hasTranscript: boolean;
}): WorkspaceContentMode {
  if (options.hasHighlights) {
    return "highlights";
  }

  if (options.hasSummary) {
    return "summary";
  }

  if (options.hasTranscript) {
    return "transcript";
  }

  return "info";
}

export function getAdjacentContentMode(
  currentMode: WorkspaceContentMode,
  direction: "previous" | "next",
): WorkspaceContentMode | null {
  const currentIndex = WORKSPACE_CONTENT_MODE_ORDER.indexOf(currentMode);
  if (currentIndex === -1) {
    return null;
  }

  const targetIndex =
    direction === "previous" ? currentIndex - 1 : currentIndex + 1;

  return WORKSPACE_CONTENT_MODE_ORDER[targetIndex] ?? null;
}

export function resolveSwipedContentMode(
  currentMode: WorkspaceContentMode,
  deltaX: number,
  deltaY: number,
  threshold = 56,
): WorkspaceContentMode | null {
  if (
    Math.abs(deltaX) < threshold ||
    Math.abs(deltaX) <= Math.abs(deltaY) * 1.25
  ) {
    return null;
  }

  return getAdjacentContentMode(currentMode, deltaX > 0 ? "previous" : "next");
}
