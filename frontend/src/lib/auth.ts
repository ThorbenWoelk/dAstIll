export type AuthState = "anonymous" | "authenticated";
export type AccessRole = "anonymous" | "user" | "operator";

export interface AuthContext {
  userId: string | null;
  authState: AuthState;
  accessRole: AccessRole;
  email: string | null;
}

export function cloneAuthContext(auth: AuthContext): AuthContext {
  return { ...auth };
}
