import { json, type RequestHandler } from "@sveltejs/kit";

import {
  AuthSessionError,
  buildAnonymousAuthContext,
  clearAuthSessionCookies,
  createSessionCookieFromIdToken,
  revokeAuthSessions,
  setAuthSessionCookie,
} from "$lib/server/auth";

type SessionRequestPayload = {
  idToken?: unknown;
};

function jsonError(status: number, message: string) {
  return json({ message }, { status });
}

export const GET: RequestHandler = async ({ locals }) => json(locals.auth);

export const POST: RequestHandler = async ({ cookies, request }) => {
  let payload: SessionRequestPayload;

  try {
    payload = (await request.json()) as SessionRequestPayload;
  } catch {
    return jsonError(400, "idToken is required");
  }

  const idToken =
    typeof payload.idToken === "string" ? payload.idToken.trim() : "";
  if (!idToken) {
    return jsonError(400, "idToken is required");
  }

  try {
    const { auth, sessionCookie } =
      await createSessionCookieFromIdToken(idToken);
    setAuthSessionCookie(cookies, sessionCookie);
    return json(auth);
  } catch (cause) {
    if (cause instanceof AuthSessionError) {
      return jsonError(cause.status, cause.message);
    }
    throw cause;
  }
};

export const DELETE: RequestHandler = async ({ cookies, locals }) => {
  if (locals?.auth?.userId) {
    await revokeAuthSessions(locals.auth.userId);
  }

  clearAuthSessionCookies(cookies);
  return json(buildAnonymousAuthContext());
};
