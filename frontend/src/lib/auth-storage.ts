import type { AuthContext } from "$lib/auth";

export const LEGACY_THEME_STORAGE_KEY = "dastill-theme-appearance";
export const LEGACY_COLOR_STORAGE_KEY = "dastill-theme-color";
export const LEGACY_WORKSPACE_STATE_KEY = "dastill.workspace.state.v1";
export const LEGACY_SEARCH_SESSION_KEY = "workspace-search-session";
export const LEGACY_CHAT_MODEL_STORAGE_KEY = "dastill.chat.cloudModel";
export const LEGACY_SHELL_LAYOUT_STORAGE_KEY = "dastill:shell-layout";
export const LEGACY_WORKSPACE_CACHE_DB = "dastill-workspace-cache";

const STORAGE_CLEANUP_KEY = "dastill.storage.cleanup.v1";

export function getAuthStorageScopeKey(
  auth: Pick<AuthContext, "authState" | "userId"> | null | undefined,
): string {
  if (auth?.authState === "authenticated" && auth.userId) {
    return `user:${auth.userId}`;
  }

  if (auth?.userId) {
    return `anonymous:${auth.userId}`;
  }

  return "anonymous:bootstrap";
}

export function getScopedStorageKey(baseKey: string, scopeKey: string): string {
  return `${baseKey}:${scopeKey}`;
}

export async function cleanupLegacyClientStorage(): Promise<void> {
  if (typeof window === "undefined") {
    return;
  }

  try {
    if (window.localStorage.getItem(STORAGE_CLEANUP_KEY) === "done") {
      return;
    }

    for (const key of [
      LEGACY_THEME_STORAGE_KEY,
      LEGACY_COLOR_STORAGE_KEY,
      LEGACY_WORKSPACE_STATE_KEY,
      LEGACY_CHAT_MODEL_STORAGE_KEY,
      LEGACY_SHELL_LAYOUT_STORAGE_KEY,
    ]) {
      window.localStorage.removeItem(key);
    }

    window.sessionStorage.removeItem(LEGACY_SEARCH_SESSION_KEY);

    if (typeof indexedDB !== "undefined") {
      await new Promise<void>((resolve) => {
        const request = indexedDB.deleteDatabase(LEGACY_WORKSPACE_CACHE_DB);
        request.onsuccess = () => resolve();
        request.onerror = () => resolve();
        request.onblocked = () => resolve();
      });
    }

    window.localStorage.setItem(STORAGE_CLEANUP_KEY, "done");
  } catch {
    // Best effort cleanup only.
  }
}
