export function resolveChannelOverviewMissingMessage(params: {
  overviewBusy: boolean;
  loadingChannels: boolean;
  channelsLength: number;
  hasSelectedChannel: boolean;
}) {
  if (params.overviewBusy || params.loadingChannels) {
    return null;
  }

  if (params.channelsLength === 0) {
    return "Follow a channel to start shaping your workspace.";
  }

  return params.hasSelectedChannel ? null : "Channel not found.";
}

export function shouldReloadChannelOverviewForAuthScope(params: {
  workspaceStateHydrated: boolean;
  authReady: boolean;
  loadedAuthScopeKey: string | null;
  loadingAuthScopeKey: string | null;
  authScopeKey: string;
}) {
  if (!params.workspaceStateHydrated || !params.authReady) {
    return false;
  }

  return (
    params.loadedAuthScopeKey !== params.authScopeKey &&
    params.loadingAuthScopeKey !== params.authScopeKey
  );
}
