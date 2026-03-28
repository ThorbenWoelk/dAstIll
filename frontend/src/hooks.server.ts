import type { Handle, HandleServerError } from "@sveltejs/kit";

import {
  LEGACY_SESSION_COOKIE_NAME,
  SESSION_COOKIE_NAME,
  buildAnonymousAuthContext,
  clearAuthSessionCookies,
  readAuthSession,
} from "$lib/server/auth";

export const handle: Handle = async ({ event, resolve }) => {
  const sessionCookie = event.cookies.get(SESSION_COOKIE_NAME);
  const legacySessionCookie = event.cookies.get(LEGACY_SESSION_COOKIE_NAME);
  const auth =
    sessionCookie === undefined
      ? buildAnonymousAuthContext()
      : await readAuthSession(sessionCookie);

  if ((sessionCookie && !auth) || legacySessionCookie) {
    clearAuthSessionCookies(event.cookies);
  }

  event.locals.auth = auth ?? buildAnonymousAuthContext();
  event.locals.isOperator = event.locals.auth.accessRole === "operator";

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
