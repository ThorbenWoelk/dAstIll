export function channelOverviewSyncSettingsHref(channelId: string): string {
  return `/channels/${encodeURIComponent(channelId)}#sync-boundary`;
}

export function shouldShowSelectedChannelSyncSettingsLink({
  videosCount,
  hasMore,
  historyExhausted,
  loadingVideos,
  backfillingHistory,
  isVirtualChannel,
}: {
  videosCount: number;
  hasMore: boolean;
  historyExhausted: boolean;
  loadingVideos: boolean;
  backfillingHistory: boolean;
  isVirtualChannel: boolean;
}): boolean {
  return (
    videosCount > 0 &&
    !isVirtualChannel &&
    !hasMore &&
    historyExhausted &&
    !loadingVideos &&
    !backfillingHistory
  );
}

export function shouldShowPagedCollectionSyncSettingsLink({
  videosCount,
  hasMore,
  loadingInitial,
  loadingMore,
  isVirtualChannel,
}: {
  videosCount: number;
  hasMore: boolean;
  loadingInitial: boolean;
  loadingMore: boolean;
  isVirtualChannel: boolean;
}): boolean {
  return (
    videosCount > 0 &&
    !isVirtualChannel &&
    !hasMore &&
    !loadingInitial &&
    !loadingMore
  );
}
