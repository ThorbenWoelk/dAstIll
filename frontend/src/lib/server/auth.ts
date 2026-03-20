import { dev } from "$app/environment";
import type { Cookies } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { createHash, createHmac, timingSafeEqual } from "node:crypto";

export type SessionRole = "operator";

export interface Session {
  role: SessionRole;
  expiresAt: number;
}

interface SessionPayload {
  role: SessionRole;
  exp: number;
}

export interface AuthRuntimeConfig {
  appPassword: string;
  sessionSecret: string;
  backendApiBase: string;
  backendProxyToken: string;
  backendIdentityAudience?: string;
}

const LOCAL_DEV_APP_PASSWORD = "local-dev-password";
const LOCAL_DEV_SESSION_SECRET = "local-dev-session-secret";
const LOCAL_DEV_BACKEND_API_BASE = "http://localhost:3544";
const LOCAL_DEV_BACKEND_PROXY_TOKEN = "local-dev-backend-proxy-token";
const SESSION_TTL_SECONDS = 60 * 60 * 24 * 30;

export const SESSION_COOKIE_NAME = "dastill_session";

function normalizeConfiguredValue(
  value: string | undefined,
): string | undefined {
  const normalized = value?.trim();
  return normalized ? normalized : undefined;
}

function requiredPrivateEnv(key: string, localDevFallback?: string): string {
  const configuredValue = normalizeConfiguredValue(env[key]);
  if (configuredValue) {
    return configuredValue;
  }

  if (dev && localDevFallback) {
    return localDevFallback;
  }

  throw new Error(`${key} must be set`);
}

function hashComparableValue(value: string): Buffer {
  return createHash("sha256").update(value).digest();
}

function secureEquals(left: string, right: string): boolean {
  return timingSafeEqual(hashComparableValue(left), hashComparableValue(right));
}

function signSessionPayload(encodedPayload: string, secret: string): string {
  return createHmac("sha256", secret)
    .update(encodedPayload)
    .digest("base64url");
}

export function getAuthRuntimeConfig(): AuthRuntimeConfig {
  const backendIdentityAudience = normalizeConfiguredValue(
    env.BACKEND_IDENTITY_AUDIENCE,
  );

  return {
    appPassword: requiredPrivateEnv(
      "APP_AUTH_PASSWORD",
      LOCAL_DEV_APP_PASSWORD,
    ),
    sessionSecret: requiredPrivateEnv(
      "APP_SESSION_SECRET",
      LOCAL_DEV_SESSION_SECRET,
    ),
    backendApiBase: requiredPrivateEnv(
      "BACKEND_API_BASE",
      LOCAL_DEV_BACKEND_API_BASE,
    ),
    backendProxyToken: requiredPrivateEnv(
      "BACKEND_PROXY_TOKEN",
      LOCAL_DEV_BACKEND_PROXY_TOKEN,
    ),
    ...(backendIdentityAudience ? { backendIdentityAudience } : {}),
  };
}

export function verifyAppPassword(
  candidatePassword: string,
  expectedPassword: string,
): boolean {
  const normalizedCandidate = candidatePassword.trim();
  return (
    normalizedCandidate.length > 0 &&
    secureEquals(normalizedCandidate, expectedPassword)
  );
}

export function createSessionToken(
  role: SessionRole,
  secret: string,
  now = Date.now(),
): string {
  const payload: SessionPayload = {
    role,
    exp: now + SESSION_TTL_SECONDS * 1000,
  };
  const encodedPayload = Buffer.from(JSON.stringify(payload)).toString(
    "base64url",
  );
  const signature = signSessionPayload(encodedPayload, secret);
  return `${encodedPayload}.${signature}`;
}

export function parseSessionToken(
  token: string | undefined,
  secret: string,
  now = Date.now(),
): Session | null {
  const [encodedPayload, signature, ...rest] = token?.split(".") ?? [];
  if (!encodedPayload || !signature || rest.length > 0) {
    return null;
  }

  const expectedSignature = signSessionPayload(encodedPayload, secret);
  if (!secureEquals(signature, expectedSignature)) {
    return null;
  }

  let payload: SessionPayload;
  try {
    payload = JSON.parse(
      Buffer.from(encodedPayload, "base64url").toString("utf8"),
    ) as SessionPayload;
  } catch {
    return null;
  }

  if (payload.role !== "operator" || payload.exp <= now) {
    return null;
  }

  return {
    role: payload.role,
    expiresAt: payload.exp,
  };
}

export function readSessionCookie(
  cookies: Cookies,
  sessionSecret: string,
): Session | null {
  return parseSessionToken(cookies.get(SESSION_COOKIE_NAME), sessionSecret);
}

export function setSessionCookie(
  cookies: Cookies,
  sessionSecret: string,
  role: SessionRole = "operator",
) {
  cookies.set(SESSION_COOKIE_NAME, createSessionToken(role, sessionSecret), {
    httpOnly: true,
    maxAge: SESSION_TTL_SECONDS,
    path: "/",
    sameSite: "lax",
    secure: !dev,
  });
}

export function clearSessionCookie(cookies: Cookies) {
  cookies.delete(SESSION_COOKIE_NAME, {
    httpOnly: true,
    path: "/",
    sameSite: "lax",
    secure: !dev,
  });
}

export function sanitizeNextPath(nextPath: string | null | undefined): string {
  if (!nextPath || !nextPath.startsWith("/") || nextPath.startsWith("//")) {
    return "/";
  }

  if (nextPath === "/login" || nextPath.startsWith("/login?")) {
    return "/";
  }

  return nextPath;
}

const PUBLIC_FILE_PATTERN =
  /\.(?:css|gif|ico|jpg|jpeg|js|json|map|png|svg|txt|webmanifest|woff2?)$/i;

export function isPublicPath(pathname: string): boolean {
  return (
    pathname === "/login" ||
    pathname.startsWith("/_app/") ||
    pathname.startsWith("/@fs/") ||
    pathname.startsWith("/@id/") ||
    pathname.startsWith("/vite-dev/") ||
    PUBLIC_FILE_PATTERN.test(pathname)
  );
}
