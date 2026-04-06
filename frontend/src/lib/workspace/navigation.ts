import type { WorkspaceContentMode } from "$lib/workspace/types";

export const WORKSPACE_CONTENT_MODE_ORDER: WorkspaceContentMode[] = [
  "info",
  "summary",
  "highlights",
  "transcript",
];

export function resolveDefaultContentMode(): WorkspaceContentMode {
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

/** Shortcut hint shown on content tabs while holding Cmd/Ctrl. */
export function goHintKeyForWorkspaceContentMode(
  mode: WorkspaceContentMode,
): string | undefined {
  const keys: Partial<Record<WorkspaceContentMode, string>> = {
    info: "7",
    summary: "8",
    transcript: "9",
  };
  return keys[mode];
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
