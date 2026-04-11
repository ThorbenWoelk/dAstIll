export interface WorkspaceScopeResetTarget {
  setChannels(channels: []): void;
  clearChannelSelectionState(): void;
  setLoadingVideos(value: boolean): void;
}

export function clearWorkspaceForScopeChange(
  sidebarState: WorkspaceScopeResetTarget,
) {
  sidebarState.setChannels([]);
  sidebarState.clearChannelSelectionState();
  sidebarState.setLoadingVideos(false);
}
