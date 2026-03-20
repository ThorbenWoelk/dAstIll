import { redirect } from "@sveltejs/kit";
import type { PageServerLoad } from "./$types";

import {
  clearAdminSessionCookie,
  normalizeRedirectTarget,
} from "$lib/server/auth";

export const load: PageServerLoad = async ({ cookies, url }) => {
  clearAdminSessionCookie(cookies);
  const redirectTo = normalizeRedirectTarget(
    url.searchParams.get("redirectTo"),
  );
  throw redirect(303, redirectTo === "/logout" ? "/" : redirectTo);
};
