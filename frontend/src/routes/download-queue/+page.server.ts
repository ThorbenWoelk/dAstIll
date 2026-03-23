import type { PageServerLoad } from "./$types";
import { loadWorkspaceBootstrapPageData } from "$lib/server/load-workspace-bootstrap";

export const load: PageServerLoad = async (event) =>
  loadWorkspaceBootstrapPageData(event, { ssrQueueTabDefault: "transcripts" });
