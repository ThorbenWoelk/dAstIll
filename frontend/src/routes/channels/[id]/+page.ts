import type { PageLoad } from "./$types";
import { loadWorkspaceBootstrapPageData } from "$lib/load-workspace-bootstrap";

export const load: PageLoad = async (event) =>
  loadWorkspaceBootstrapPageData(event, {
    selectedChannelIdOverride: event.params.id ?? null,
  });
