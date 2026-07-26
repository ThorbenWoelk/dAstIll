import type { AuthContext } from "$lib/auth";
import { getAuthStorageScopeKey } from "$lib/auth/storage";
import type { UserPreferences } from "$lib/types";

export type PreferencesHydrationOutcome =
  | { status: "skipped"; reason: "unauthenticated" | "auth-not-ready" }
  | { status: "failed" }
  | {
      status: "loaded";
      preferences: UserPreferences;
      scopeKey: string;
    };

/**
 * Load server preferences only for an authenticated, ready session.
 * Anonymous GETs return empty defaults; applying those under an authenticated
 * UI would wipe vocabulary/channel prefs on the next PUT.
 */
export async function hydrateAuthenticatedPreferences(options: {
  authReady: boolean;
  auth: Pick<AuthContext, "authState" | "userId">;
  getPreferences: () => Promise<UserPreferences>;
}): Promise<PreferencesHydrationOutcome> {
  if (!options.authReady) {
    return { status: "skipped", reason: "auth-not-ready" };
  }
  if (options.auth.authState !== "authenticated" || !options.auth.userId) {
    return { status: "skipped", reason: "unauthenticated" };
  }

  const scopeKey = getAuthStorageScopeKey(options.auth);
  try {
    const preferences = await options.getPreferences();
    return { status: "loaded", preferences, scopeKey };
  } catch {
    return { status: "failed" };
  }
}

export function canPersistServerPreferences(options: {
  preferencesHydrated: boolean;
  preferencesScopeKey: string | null;
  auth: Pick<AuthContext, "authState" | "userId">;
}): boolean {
  if (!options.preferencesHydrated || !options.preferencesScopeKey) {
    return false;
  }
  if (options.auth.authState !== "authenticated" || !options.auth.userId) {
    return false;
  }
  return getAuthStorageScopeKey(options.auth) === options.preferencesScopeKey;
}
