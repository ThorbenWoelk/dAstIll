import type { Handle } from "@sveltejs/kit";
import { redirect } from "@sveltejs/kit";

import {
  getAuthRuntimeConfig,
  isPublicPath,
  readSessionCookie,
  sanitizeNextPath,
} from "$lib/server/auth";

export const handle: Handle = async ({ event, resolve }) => {
  const authConfig = getAuthRuntimeConfig();
  event.locals.session = readSessionCookie(
    event.cookies,
    authConfig.sessionSecret,
  );

  const pathname = event.url.pathname;
  if (isPublicPath(pathname)) {
    if (pathname === "/login" && event.locals.session) {
      throw redirect(303, sanitizeNextPath(event.url.searchParams.get("next")));
    }

    return resolve(event);
  }

  if (event.locals.session) {
    return resolve(event);
  }

  if (pathname.startsWith("/api/")) {
    return new Response("Unauthorized", { status: 401 });
  }

  const nextPath = sanitizeNextPath(`${pathname}${event.url.search}`);
  throw redirect(303, `/login?next=${encodeURIComponent(nextPath)}`);
};
