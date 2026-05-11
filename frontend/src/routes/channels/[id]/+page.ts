import type { PageLoad } from "./$types";
import { loadWorkspaceBootstrapPageData } from "$lib/workspace/load-workspace-bootstrap";

export const load: PageLoad = async (event) =>
  loadWorkspaceBootstrapPageData(event, {
    selectedChannelIdOverride: event.params.id ?? null,
  });
