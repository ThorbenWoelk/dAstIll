import { writable } from "svelte/store";
import type { VideoTypeFilter } from "$lib/types";
import type { AcknowledgedFilter } from "$lib/workspace/types";

/** Props for `WorkspaceSidebarVideoFilterControl` when hosted in the mobile footer. */
export type MobileFooterVideoFilterPayload = {
  videoTypeFilter: VideoTypeFilter;
  acknowledgedFilter: AcknowledgedFilter;
  disabled: boolean;
  onSelectVideoType: (value: VideoTypeFilter) => void | Promise<void>;
  onSelectAcknowledged: (value: AcknowledgedFilter) => void | Promise<void>;
  onClearAllFilters: () => void | Promise<void>;
};

/** Mobile bottom chrome: section links, optional library filter row, video actions, or hidden. */
export type MobileBottomBarState =
  | { kind: "sections" }
  | { kind: "sectionsWithVideoFilter"; filter: MobileFooterVideoFilterPayload }
  | { kind: "hidden" }
  | {
      kind: "videoActions";
      youtubeUrl: string | null;
      showRegenerate: boolean;
      regenerating: boolean;
      aiAvailable: boolean;
      onRegenerate: () => void;
      showFormatAction: boolean;
      formatting: boolean;
      onFormat: () => void;
      showRevertAction: boolean;
      reverting: boolean;
      canRevert: boolean;
      onRevert: () => void;
      busy: boolean;
      onRequestResetVideo: () => void;
      resetting: boolean;
      showAcknowledgeToggle: boolean;
      acknowledged: boolean;
      onAcknowledgeToggle: () => void;
      showEditAction: boolean;
      onEdit: () => void;
    };

export const mobileBottomBar = writable<MobileBottomBarState>({
  kind: "sections",
});
