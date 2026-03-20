import { dev } from "$app/environment";
import { env } from "$env/dynamic/private";
import type { Cookies } from "@sveltejs/kit";
import {
  createHash,
  createHmac,
  randomBytes,
  timingSafeEqual,
} from "node:crypto";

export interface AuthRuntimeConfig {
  backendApiBase: string;
  backendProxyToken: string;
  backendIdentityAudience?: string;
  adminPassword: string;
}

const LOCAL_DEV_BACKEND_API_BASE = "http://localhost:3544";
const LOCAL_DEV_BACKEND_PROXY_TOKEN = "local-dev-backend-proxy-token";
const SESSION_COOKIE_NAME = "dastill-session";
const SESSION_DURATION_MS = 7 * 24 * 60 * 60 * 1000;

type SessionPayload = {
  sid: string;
  exp: number;
};

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

function secureEquals(left: string, right: string): boolean {
  const leftHash = createHash("sha256").update(left).digest();
  const rightHash = createHash("sha256").update(right).digest();
  return timingSafeEqual(leftHash, rightHash);
}

function signSessionPayload(payload: string, secret: string): Buffer {
  return createHmac("sha256", secret).update(payload).digest();
}

function decodeSessionPayload(value: string): SessionPayload | null {
  try {
    const payload = JSON.parse(
      Buffer.from(value, "base64url").toString("utf8"),
    ) as {
      sid?: unknown;
      exp?: unknown;
    };
    if (typeof payload.sid !== "string" || typeof payload.exp !== "number") {
      return null;
    }
    return {
      sid: payload.sid,
      exp: payload.exp,
    };
  } catch {
    return null;
  }
}

export function normalizeRedirectTarget(
  value: string | null | undefined,
): string {
  const normalized = value?.trim();
  if (
    !normalized ||
    !normalized.startsWith("/") ||
    normalized.startsWith("//")
  ) {
    return "/";
  }
  return normalized;
}

function loadAuthRuntimeConfig(): AuthRuntimeConfig {
  const backendIdentityAudience = normalizeConfiguredValue(
    env.BACKEND_IDENTITY_AUDIENCE,
  );

  return {
    backendApiBase: requiredPrivateEnv(
      "BACKEND_API_BASE",
      LOCAL_DEV_BACKEND_API_BASE,
    ),
    backendProxyToken: requiredPrivateEnv(
      "BACKEND_PROXY_TOKEN",
      LOCAL_DEV_BACKEND_PROXY_TOKEN,
    ),
    adminPassword: requiredPrivateEnv("ADMIN_PASSWORD"),
    ...(backendIdentityAudience ? { backendIdentityAudience } : {}),
  };
}

const authRuntimeConfig = loadAuthRuntimeConfig();

export function getAuthRuntimeConfig(): AuthRuntimeConfig {
  return authRuntimeConfig;
}

export function isValidAdminPassword(password: string): boolean {
  return secureEquals(password, getAuthRuntimeConfig().adminPassword);
}

export function createAdminSessionToken(): string {
  const payload = Buffer.from(
    JSON.stringify({
      sid: randomBytes(16).toString("hex"),
      exp: Date.now() + SESSION_DURATION_MS,
    } satisfies SessionPayload),
    "utf8",
  ).toString("base64url");
  const signature = signSessionPayload(
    payload,
    getAuthRuntimeConfig().adminPassword,
  ).toString("base64url");
  return `${payload}.${signature}`;
}

export function readAdminSession(
  token: string | undefined,
): SessionPayload | null {
  if (!token) {
    return null;
  }

  const [payload, signature, ...rest] = token.split(".");
  if (!payload || !signature || rest.length > 0) {
    return null;
  }

  const expectedSignature = signSessionPayload(
    payload,
    getAuthRuntimeConfig().adminPassword,
  );
  let providedSignature: Buffer;
  try {
    providedSignature = Buffer.from(signature, "base64url");
  } catch {
    return null;
  }

  if (
    expectedSignature.length !== providedSignature.length ||
    !timingSafeEqual(expectedSignature, providedSignature)
  ) {
    return null;
  }

  const session = decodeSessionPayload(payload);
  if (!session || !Number.isFinite(session.exp) || session.exp <= Date.now()) {
    return null;
  }

  return session;
}

export function setAdminSessionCookie(cookies: Cookies): void {
  cookies.set(SESSION_COOKIE_NAME, createAdminSessionToken(), {
    path: "/",
    httpOnly: true,
    sameSite: "strict",
    secure: !dev,
    maxAge: Math.floor(SESSION_DURATION_MS / 1000),
  });
}

export function clearAdminSessionCookie(cookies: Cookies): void {
  cookies.delete(SESSION_COOKIE_NAME, {
    path: "/",
    httpOnly: true,
    sameSite: "strict",
    secure: !dev,
  });
}

export { SESSION_COOKIE_NAME };
