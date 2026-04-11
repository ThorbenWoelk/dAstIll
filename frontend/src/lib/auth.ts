export type AuthState = "anonymous" | "authenticated";
export type AccessRole = "anonymous" | "user" | "operator";

export interface AuthContext {
  userId: string | null;
  authState: AuthState;
  accessRole: AccessRole;
  email: string | null;
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
  accessRole: AccessRole = "user",
): AuthContext {
  return {
    userId,
    authState: "authenticated",
    accessRole,
    email,
  };
}

export function cloneAuthContext(auth: AuthContext): AuthContext {
  return { ...auth };
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
