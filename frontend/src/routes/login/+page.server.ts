import { fail, redirect } from "@sveltejs/kit";
import type { Actions, PageServerLoad } from "./$types";

import {
  isValidAdminPassword,
  normalizeRedirectTarget,
  setAdminSessionCookie,
} from "$lib/server/auth";

export const load: PageServerLoad = async ({ locals, url }) => {
  const redirectTo = normalizeRedirectTarget(
    url.searchParams.get("redirectTo"),
  );

  if (locals.isOperator) {
    throw redirect(303, redirectTo === "/login" ? "/" : redirectTo);
  }

  return { redirectTo };
};

export const actions: Actions = {
  default: async ({ request, cookies }) => {
    const formData = await request.formData();
    const password = `${formData.get("password") ?? ""}`;
    const redirectTo = normalizeRedirectTarget(
      `${formData.get("redirectTo") ?? "/"}`,
    );

    if (!isValidAdminPassword(password)) {
      return fail(401, {
        message: "Invalid password.",
        redirectTo,
      });
    }

    setAdminSessionCookie(cookies);
    throw redirect(303, redirectTo === "/login" ? "/" : redirectTo);
  },
};
