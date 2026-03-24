import type { PageServerLoad } from "./$types";
import { loadWorkspaceBootstrapPageData } from "$lib/server/load-workspace-bootstrap";

/**
 * Client-side navigations (isDataRequest) skip the blocking API call so section
 * switches are instant. The page's onMount handles data loading independently.
 */
export const load: PageServerLoad = async (event) => {
  if (event.isDataRequest) {
    return { bootstrap: null, channelPreviews: {}, channelPreviewsFilterKey: "all:all:transcripts" };
  }
  return loadWorkspaceBootstrapPageData(event, { ssrQueueTabDefault: "transcripts" });
};
