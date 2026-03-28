import { redirect } from "@sveltejs/kit";
import type { PageServerLoad } from "./$types";

import {
  clearAuthSessionCookies,
  normalizeRedirectTarget,
  revokeAuthSessions,
} from "$lib/server/auth";

export const load: PageServerLoad = async ({ cookies, url, locals }) => {
  if (locals.auth.userId) {
    await revokeAuthSessions(locals.auth.userId);
  }

  clearAuthSessionCookies(cookies);
  const redirectTo = normalizeRedirectTarget(
    url.searchParams.get("redirectTo"),
  );
  throw redirect(303, redirectTo === "/logout" ? "/" : redirectTo);
};
