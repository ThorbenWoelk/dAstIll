import { dev } from "$app/environment";
import { env } from "$env/dynamic/private";
import type { AuthContext, AccessRole } from "$lib/auth";
import type { Cookies } from "@sveltejs/kit";
import {
  getApps as getAdminApps,
  initializeApp as initializeAdminApp,
  type App as FirebaseAdminApp,
} from "firebase-admin/app";
import {
  getAuth as getFirebaseAdminAuth,
  type DecodedIdToken,
  type DecodedIdToken as DecodedSessionCookie,
} from "firebase-admin/auth";

export interface AuthRuntimeConfig {
  backendApiBase: string;
  backendProxyToken: string;
  backendIdentityAudience?: string;
}

const LOCAL_DEV_BACKEND_API_BASE = "http://localhost:3544";
const LOCAL_DEV_BACKEND_PROXY_TOKEN = "local-dev-backend-proxy-token";
const LOCAL_DEV_FIREBASE_PROJECT_ID = "demo-dastill";
const LOCAL_DEV_FIREBASE_AUTH_EMULATOR_HOST = "127.0.0.1:9099";
const SESSION_DURATION_MS = 7 * 24 * 60 * 60 * 1000;
const RECENT_SIGN_IN_WINDOW_MS = 5 * 60 * 1000;

export const AUTH_SESSION_MAX_AGE_SECONDS = Math.floor(
  SESSION_DURATION_MS / 1000,
);
export const SESSION_COOKIE_NAME = "__session";
export const LEGACY_SESSION_COOKIE_NAME = "dastill-session";

export class AuthSessionError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "AuthSessionError";
  }
}

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

function readConfiguredFirebaseProjectId(): string {
  const projectId =
    normalizeConfiguredValue(env.FIREBASE_PROJECT_ID) ??
    normalizeConfiguredValue(process.env.FIREBASE_PROJECT_ID) ??
    normalizeConfiguredValue(process.env.PUBLIC_FIREBASE_PROJECT_ID) ??
    normalizeConfiguredValue(process.env.GCLOUD_PROJECT) ??
    normalizeConfiguredValue(process.env.GOOGLE_CLOUD_PROJECT);

  if (projectId) {
    return projectId;
  }

  if (dev) {
    return LOCAL_DEV_FIREBASE_PROJECT_ID;
  }

  throw new Error(
    "FIREBASE_PROJECT_ID or PUBLIC_FIREBASE_PROJECT_ID must be set",
  );
}

function readOperatorEmailAllowlist(): Set<string> {
  const configuredValue =
    normalizeConfiguredValue(env.OPERATOR_EMAIL_ALLOWLIST) ??
    normalizeConfiguredValue(process.env.OPERATOR_EMAIL_ALLOWLIST);

  return new Set(
    (configuredValue ?? "")
      .split(",")
      .map((email) => email.trim().toLowerCase())
      .filter(Boolean),
  );
}

function resolveAuthenticatedAccessRole(email: string | null): AccessRole {
  if (!email) {
    return "user";
  }

  return readOperatorEmailAllowlist().has(email.trim().toLowerCase())
    ? "operator"
    : "user";
}

export function buildAnonymousAuthContext(
  userId: string | null = null,
): AuthContext {
  return {
    userId,
    authState: "anonymous",
    accessRole: "anonymous",
    email: null,
  };
}

export function buildAuthenticatedAuthContext(
  userId: string,
  email: string | null,
): AuthContext {
  return {
    userId,
    authState: "authenticated",
    accessRole: resolveAuthenticatedAccessRole(email),
    email,
  };
}

function buildAuthContextFromDecodedToken(
  decodedToken: DecodedIdToken | DecodedSessionCookie,
): AuthContext {
  const providerId = decodedToken.firebase?.sign_in_provider ?? null;
  if (providerId === "anonymous") {
    return buildAnonymousAuthContext(decodedToken.uid);
  }

  return buildAuthenticatedAuthContext(
    decodedToken.uid,
    decodedToken.email ?? null,
  );
}

function resolveAuthTimeMillis(decodedToken: DecodedIdToken): number | null {
  const authTimeSeconds = Number(decodedToken.auth_time);
  if (!Number.isFinite(authTimeSeconds) || authTimeSeconds <= 0) {
    return null;
  }
  return authTimeSeconds * 1000;
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
    ...(backendIdentityAudience ? { backendIdentityAudience } : {}),
  };
}

let authRuntimeConfig: AuthRuntimeConfig | null = null;
let firebaseAdminApp: FirebaseAdminApp | null = null;

export function getAuthRuntimeConfig(): AuthRuntimeConfig {
  authRuntimeConfig ??= loadAuthRuntimeConfig();
  return authRuntimeConfig;
}

function getFirebaseAdminApp(): FirebaseAdminApp {
  if (firebaseAdminApp) {
    return firebaseAdminApp;
  }

  if (dev && !process.env.FIREBASE_AUTH_EMULATOR_HOST) {
    process.env.FIREBASE_AUTH_EMULATOR_HOST =
      LOCAL_DEV_FIREBASE_AUTH_EMULATOR_HOST;
  }

  const existingApp = getAdminApps()[0];
  if (existingApp) {
    firebaseAdminApp = existingApp;
    return existingApp;
  }

  firebaseAdminApp = initializeAdminApp({
    projectId: readConfiguredFirebaseProjectId(),
  });
  return firebaseAdminApp;
}

function getFirebaseAuthAdmin() {
  return getFirebaseAdminAuth(getFirebaseAdminApp());
}

export async function createSessionCookieFromIdToken(idToken: string): Promise<{
  auth: AuthContext;
  sessionCookie: string;
}> {
  const trimmedIdToken = idToken.trim();
  if (!trimmedIdToken) {
    throw new AuthSessionError(400, "idToken is required");
  }

  let decodedIdToken: DecodedIdToken;
  try {
    decodedIdToken = await getFirebaseAuthAdmin().verifyIdToken(trimmedIdToken);
  } catch {
    throw new AuthSessionError(401, "Invalid Firebase ID token.");
  }

  const authTimeMillis = resolveAuthTimeMillis(decodedIdToken);
  if (
    !authTimeMillis ||
    Date.now() - authTimeMillis > RECENT_SIGN_IN_WINDOW_MS
  ) {
    throw new AuthSessionError(401, "Recent sign-in required.");
  }

  try {
    const sessionCookie = await getFirebaseAuthAdmin().createSessionCookie(
      trimmedIdToken,
      { expiresIn: SESSION_DURATION_MS },
    );
    return {
      auth: buildAuthContextFromDecodedToken(decodedIdToken),
      sessionCookie,
    };
  } catch {
    throw new AuthSessionError(
      401,
      "Failed to create Firebase session cookie.",
    );
  }
}

export async function readAuthSession(
  sessionCookie: string | undefined,
): Promise<AuthContext | null> {
  if (!sessionCookie) {
    return null;
  }

  try {
    const decodedSessionCookie =
      await getFirebaseAuthAdmin().verifySessionCookie(sessionCookie, true);
    return buildAuthContextFromDecodedToken(decodedSessionCookie);
  } catch {
    return null;
  }
}

export async function revokeAuthSessions(userId: string): Promise<void> {
  const trimmedUserId = userId.trim();
  if (!trimmedUserId) {
    return;
  }
  await getFirebaseAuthAdmin().revokeRefreshTokens(trimmedUserId);
}

function authCookieOptions() {
  return {
    path: "/",
    httpOnly: true,
    sameSite: "strict" as const,
    secure: !dev,
  };
}

export function setAuthSessionCookie(
  cookies: Pick<Cookies, "set">,
  sessionCookie: string,
): void {
  cookies.set(SESSION_COOKIE_NAME, sessionCookie, {
    ...authCookieOptions(),
    maxAge: AUTH_SESSION_MAX_AGE_SECONDS,
  });
}

export function clearAuthSessionCookies(
  cookies: Pick<Cookies, "delete">,
): void {
  const options = authCookieOptions();
  cookies.delete(SESSION_COOKIE_NAME, options);
  cookies.delete(LEGACY_SESSION_COOKIE_NAME, options);
}
