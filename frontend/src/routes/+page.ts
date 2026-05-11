import type { PageLoad } from "./$types";
import { loadWorkspaceBootstrapPageData } from "$lib/workspace/load-workspace-bootstrap";

export const load: PageLoad = async (event) => {
  const selectedSourceId =
    event.url.searchParams.get("source")?.trim() ??
    event.url.searchParams.get("channel")?.trim();
  const selectedItemId =
    event.url.searchParams.get("item")?.trim() ??
    event.url.searchParams.get("video")?.trim();

  if (!selectedSourceId && !selectedItemId) {
    return {
      bootstrap: null,
      channelPreviews: {},
      channelPreviewsFilterKey: "all:all:default",
      selectedSourceId: null,
      selectedChannelId: null,
      selectedItemId: null,
      selectedVideoId: null,
      contentMode: null,
      videoTypeFilter: null,
      acknowledgedFilter: null,
    };
  }

  return loadWorkspaceBootstrapPageData(event);
};
