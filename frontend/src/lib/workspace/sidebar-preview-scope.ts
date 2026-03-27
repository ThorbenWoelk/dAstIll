import type { QueueTab, VideoTypeFilter } from "$lib/types";
import type { WorkspaceSidebarPreviewScope } from "$lib/workspace/component-props";
import type { AcknowledgedFilter } from "$lib/workspace/types";

export function resolveSidebarPreviewQueueSegment(
  previewScope: WorkspaceSidebarPreviewScope,
): QueueTab | "unified" | "default" {
  if (previewScope.kind === "queue_tab" && previewScope.queueTab) {
    return previewScope.queueTab;
  }

  if (previewScope.kind === "unified") {
    return "unified";
  }

  return "default";
}

export function resolveSidebarPreviewFilterKey(
  videoTypeFilter: VideoTypeFilter,
  acknowledgedFilter: AcknowledgedFilter,
  previewScope: WorkspaceSidebarPreviewScope,
): string {
  return `${videoTypeFilter}:${acknowledgedFilter}:${resolveSidebarPreviewQueueSegment(previewScope)}`;
}

export function resolveSidebarPreviewQueueRequest(
  previewScope: WorkspaceSidebarPreviewScope,
): {
  queueOnly: boolean;
  queueTab: QueueTab | undefined;
} {
  return {
    queueOnly: previewScope.kind !== "default",
    queueTab:
      previewScope.kind === "queue_tab" ? previewScope.queueTab : undefined,
  };
}
