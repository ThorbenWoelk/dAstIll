import type { VideoTypeFilter } from "$lib/types";
import type {
  AcknowledgedFilter,
  ChannelSortMode,
  WorkspaceContentMode,
} from "$lib/workspace/types";

export interface WorkspaceScopeResetTarget {
  setChannels(channels: []): void;
  clearChannelSelectionState(): void;
  setLoadingVideos(value: boolean): void;
}

export interface WorkspaceScopeRestoreTarget {
  setSelectedChannel(channelId: string | null): void;
  setSelectedVideoId(videoId: string | null): void;
  setChannelOrder(channelOrder: string[]): void;
  setChannelSortMode(channelSortMode: ChannelSortMode): void;
  setAcknowledgedFilter(acknowledgedFilter: AcknowledgedFilter): void;
  setVideoTypeFilter(videoTypeFilter: VideoTypeFilter): void;
}

export interface WorkspaceScopeContentTarget {
  setMode(mode: WorkspaceContentMode): void;
}

export type RestoredWorkspaceScopeState = {
  selectedChannelId?: string | null;
  selectedVideoId?: string | null;
  contentMode?: WorkspaceContentMode;
  channelOrder?: string[];
  channelSortMode?: ChannelSortMode;
  acknowledgedFilter?: AcknowledgedFilter;
  videoTypeFilter?: VideoTypeFilter;
};

export function clearWorkspaceForScopeChange(
  sidebarState: WorkspaceScopeResetTarget,
) {
  sidebarState.setChannels([]);
  sidebarState.clearChannelSelectionState();
  sidebarState.setLoadingVideos(false);
}

export function applyWorkspaceStateForScopeChange(
  sidebarState: WorkspaceScopeRestoreTarget,
  content: WorkspaceScopeContentTarget,
  restored: RestoredWorkspaceScopeState,
) {
  if ("selectedChannelId" in restored) {
    sidebarState.setSelectedChannel(restored.selectedChannelId ?? null);
  }
  if ("selectedVideoId" in restored) {
    sidebarState.setSelectedVideoId(restored.selectedVideoId ?? null);
  }
  if (restored.contentMode) {
    content.setMode(restored.contentMode);
  }
  if (restored.channelOrder) {
    sidebarState.setChannelOrder(restored.channelOrder);
  }
  if (restored.channelSortMode) {
    sidebarState.setChannelSortMode(restored.channelSortMode);
  }
  if (restored.acknowledgedFilter) {
    sidebarState.setAcknowledgedFilter(restored.acknowledgedFilter);
  }
  if (restored.videoTypeFilter) {
    sidebarState.setVideoTypeFilter(restored.videoTypeFilter);
  }
}
