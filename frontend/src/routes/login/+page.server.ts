import type { PageServerLoad } from "./$types";

import { normalizeRedirectTarget } from "$lib/server/auth";

export const load: PageServerLoad = async ({ url }) => {
  const redirectTo = normalizeRedirectTarget(
    url.searchParams.get("redirectTo"),
  );

  return { redirectTo };
};
