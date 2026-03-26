import type { Handle, HandleServerError } from "@sveltejs/kit";

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

export const handleError: HandleServerError = ({ error, event }) => {
  const errorId = crypto.randomUUID();

  // In a real app, you'd send this to a service like Sentry
  console.error(`[Server Error ${errorId}]`, {
    error,
    url: event.url.pathname,
    method: event.request.method,
  });

  return {
    message: "Whoops! An unexpected error occurred.",
    errorId,
  };
};
