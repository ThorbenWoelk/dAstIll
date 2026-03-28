import type { AuthContext } from "$lib/auth";
import { getAuthStorageScopeKey, getScopedStorageKey } from "$lib/auth-storage";
import type { Highlight } from "$lib/types";

const SESSION_HIGHLIGHTS_BASE = "dastill.session-highlights.v1";

function storageKeyForScope(scopeKey: string): string {
  return getScopedStorageKey(SESSION_HIGHLIGHTS_BASE, scopeKey);
}

/** Highlights for signed-out visitors: keyed by video id, stored in sessionStorage (tab session). */
export function loadSessionHighlightsMap(
  scopeKey: string,
): Record<string, Highlight[]> {
  if (typeof sessionStorage === "undefined") return {};
  try {
    const raw = sessionStorage.getItem(storageKeyForScope(scopeKey));
    if (!raw) return {};
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return {};
    }
    return parsed as Record<string, Highlight[]>;
  } catch {
    return {};
  }
}

export function saveSessionHighlightsMap(
  scopeKey: string,
  map: Record<string, Highlight[]>,
): void {
  if (typeof sessionStorage === "undefined") return;
  try {
    sessionStorage.setItem(storageKeyForScope(scopeKey), JSON.stringify(map));
  } catch {
    // Quota or private mode; best-effort only.
  }
}

export function shouldUseSessionHighlights(
  auth: Pick<AuthContext, "authState"> | null | undefined,
): boolean {
  return auth?.authState !== "authenticated";
}

export function resolveHighlightsScopeKey(
  auth: Pick<AuthContext, "authState" | "userId"> | null | undefined,
): string {
  return getAuthStorageScopeKey(auth);
}
