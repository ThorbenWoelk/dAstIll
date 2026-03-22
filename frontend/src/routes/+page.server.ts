import type { PageServerLoad } from "./$types";
import type { WorkspaceBootstrap } from "$lib/types";

/**
 * Server-side load function for the workspace root route.
 *
 * Fetches the consolidated workspace bootstrap data (channels, AI status,
 * search status, and initial channel snapshot) in a single server-side
 * request so the initial HTML response contains meaningful workspace state
 * before JavaScript hydrates (VAL-DATA-001, VAL-DATA-002).
 *
 * The server-side fetch goes through the existing /api/[...path] proxy,
 * which forwards auth headers and the operator role from locals.
 *
 * Returns { bootstrap: null } on any error so the page falls back to
 * the IndexedDB warm-start path in onMount.
 */
export const load: PageServerLoad = async ({ fetch, url }) => {
  const selectedChannelId = url.searchParams.get("channel") ?? null;

  try {
    const params = new URLSearchParams();
    if (selectedChannelId) {
      params.set("selected_channel_id", selectedChannelId);
    }
    // Match the page-level limit constant (20 videos per snapshot page)
    params.set("limit", "20");

    const response = await fetch(
      `/api/workspace/bootstrap?${params.toString()}`,
    );

    if (!response.ok) {
      return { bootstrap: null };
    }

    const bootstrap = (await response.json()) as WorkspaceBootstrap;
    return { bootstrap };
  } catch {
    return { bootstrap: null };
  }
};
