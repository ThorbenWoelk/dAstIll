import type { PageServerLoad } from "./$types";
import { loadWorkspaceBootstrapPageData } from "$lib/server/load-workspace-bootstrap";

/**
 * Server-side load for the workspace root route. Delegates to
 * loadWorkspaceBootstrapPageData (see that module for VAL-DATA-001/002 docs).
 *
 * Client-side navigations normally skip the blocking API call so section
 * switches are instant. When navigation targets a specific channel/video, keep
 * the bootstrap path enabled so the workspace can hydrate the destination
 * state immediately instead of cold-starting on mount.
 */
export const load: PageServerLoad = async (event) => {
  const selectedChannelId = event.url.searchParams.get("channel")?.trim();
  const selectedVideoId = event.url.searchParams.get("video")?.trim();

  if (event.isDataRequest && !selectedChannelId && !selectedVideoId) {
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
