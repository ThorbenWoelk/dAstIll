import { buildChannelViewCacheKey } from "$lib/workspace/channel-view-cache";
import type { AcknowledgedFilter } from "$lib/workspace/types";
import type { VideoTypeFilter } from "$lib/types";

export function buildHomeWorkspaceChannelViewCacheKey(params: {
  channelId: string;
  workspaceCacheScopeKey: string;
  videoTypeFilter: VideoTypeFilter;
  acknowledgedFilter: AcknowledgedFilter;
}) {
  return buildChannelViewCacheKey(
    params.channelId,
    params.workspaceCacheScopeKey,
    params.videoTypeFilter,
    params.acknowledgedFilter,
  );
}
