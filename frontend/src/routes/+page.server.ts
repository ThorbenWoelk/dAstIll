import type { PageServerLoad } from "./$types";
import { loadWorkspaceBootstrapPageData } from "$lib/server/load-workspace-bootstrap";

/**
 * Server-side load for the workspace root route. Delegates to
 * loadWorkspaceBootstrapPageData (see that module for VAL-DATA-001/002 docs).
 */
export const load: PageServerLoad = async (event) =>
  loadWorkspaceBootstrapPageData(event);
