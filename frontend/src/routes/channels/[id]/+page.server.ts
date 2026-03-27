import type { PageServerLoad } from "./$types";
import { loadWorkspaceBootstrapPageData } from "$lib/server/load-workspace-bootstrap";

export const load: PageServerLoad = async (event) => {
  if (event.isDataRequest) {
    return {
      bootstrap: null,
      channelPreviews: {},
      channelPreviewsFilterKey: "all:all:default",
    };
  }

  return loadWorkspaceBootstrapPageData(event, {
    selectedChannelIdOverride: event.params.id ?? null,
  });
};
