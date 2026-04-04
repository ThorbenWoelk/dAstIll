import type { PageLoad } from "./$types";
import { loadWorkspaceBootstrapPageData } from "$lib/load-workspace-bootstrap";

export const load: PageLoad = async (event) => {
  const selectedChannelId = event.url.searchParams.get("channel")?.trim();
  const selectedVideoId = event.url.searchParams.get("video")?.trim();

  if (!selectedChannelId && !selectedVideoId) {
    return {
      bootstrap: null,
      channelPreviews: {},
      channelPreviewsFilterKey: "all:all:default",
      selectedChannelId: null,
      selectedVideoId: null,
      contentMode: null,
      videoTypeFilter: null,
      acknowledgedFilter: null,
    };
  }

  return loadWorkspaceBootstrapPageData(event);
};
