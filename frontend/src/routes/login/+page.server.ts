import { fail, redirect } from "@sveltejs/kit";

import type { Actions, PageServerLoad } from "./$types";
import {
  getAuthRuntimeConfig,
  sanitizeNextPath,
  setSessionCookie,
  verifyAppPassword,
} from "$lib/server/auth";

export const load: PageServerLoad = ({ url }) => ({
  next: sanitizeNextPath(url.searchParams.get("next")),
});

export const actions: Actions = {
  default: async ({ cookies, request }) => {
    const authConfig = getAuthRuntimeConfig();
    const formData = await request.formData();
    const password = `${formData.get("password") ?? ""}`;
    const nextPath = sanitizeNextPath(`${formData.get("next") ?? "/"}`);

    if (!verifyAppPassword(password, authConfig.appPassword)) {
      return fail(401, {
        invalidPassword: true,
        next: nextPath,
      });
    }

    setSessionCookie(cookies, authConfig.sessionSecret);
    throw redirect(303, nextPath);
  },
};
