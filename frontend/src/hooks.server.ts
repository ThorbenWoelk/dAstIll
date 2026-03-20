import type { Handle } from "@sveltejs/kit";

import {
  SESSION_COOKIE_NAME,
  clearAdminSessionCookie,
  readAdminSession,
} from "$lib/server/auth";

export const handle: Handle = async ({ event, resolve }) => {
  const token = event.cookies.get(SESSION_COOKIE_NAME);
  const session = readAdminSession(token);

  event.locals.isOperator = Boolean(session);

  if (token && !session) {
    clearAdminSessionCookie(event.cookies);
  }

  return resolve(event);
};
