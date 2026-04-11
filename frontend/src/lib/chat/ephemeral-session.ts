import type { ChatConversation, ChatConversationSummary } from "$lib/types";
import { getScopedStorageKey } from "$lib/auth-storage";

const STORAGE_KEY = "dastill.chat.ephemeralThreads.v1";
const ANONYMOUS_BOOTSTRAP_SCOPE_KEY = "anonymous:bootstrap";

function storageKeyForScope(scopeKey: string): string {
  return getScopedStorageKey(STORAGE_KEY, scopeKey);
}

function parseStoredThreads(raw: string | null): ChatConversation[] {
  if (!raw?.trim()) {
    return [];
  }
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) {
      return [];
    }
    return parsed as ChatConversation[];
  } catch {
    return [];
  }
}

export function loadEphemeralThreads(scopeKey: string): ChatConversation[] {
  if (typeof sessionStorage === "undefined") {
    return [];
  }

  const scopedKey = storageKeyForScope(scopeKey);
  const scopedThreads = parseStoredThreads(sessionStorage.getItem(scopedKey));
  if (scopedThreads.length > 0) {
    return scopedThreads;
  }

  const shouldMigrateBootstrapThreads =
    scopeKey.startsWith("anonymous:") &&
    scopeKey !== ANONYMOUS_BOOTSTRAP_SCOPE_KEY;
  if (shouldMigrateBootstrapThreads) {
    const bootstrapKey = storageKeyForScope(ANONYMOUS_BOOTSTRAP_SCOPE_KEY);
    const bootstrapThreads = parseStoredThreads(
      sessionStorage.getItem(bootstrapKey),
    );
    if (bootstrapThreads.length > 0) {
      try {
        sessionStorage.setItem(scopedKey, JSON.stringify(bootstrapThreads));
        sessionStorage.removeItem(bootstrapKey);
      } catch {
        // Best effort only. Even if migration persistence fails, return the data.
      }

      return bootstrapThreads;
    }
  }

  const legacyThreads = parseStoredThreads(sessionStorage.getItem(STORAGE_KEY));
  if (legacyThreads.length === 0) {
    return [];
  }

  try {
    sessionStorage.setItem(scopedKey, JSON.stringify(legacyThreads));
    sessionStorage.removeItem(STORAGE_KEY);
  } catch {
    // Best effort only. Even if migration persistence fails, return the data.
  }

  return legacyThreads;
}

export function saveEphemeralThreads(
  scopeKey: string,
  threads: ChatConversation[],
): void {
  if (typeof sessionStorage === "undefined") {
    return;
  }
  try {
    sessionStorage.setItem(
      storageKeyForScope(scopeKey),
      JSON.stringify(threads),
    );
  } catch {
    /* quota or private mode */
  }
}

export function clearEphemeralThreads(scopeKey: string): void {
  if (typeof sessionStorage === "undefined") {
    return;
  }
  try {
    sessionStorage.removeItem(storageKeyForScope(scopeKey));
  } catch {
    /* ignore */
  }
}

export function conversationToSummary(
  conversation: ChatConversation,
): ChatConversationSummary {
  return {
    id: conversation.id,
    title: conversation.title,
    title_status: conversation.title_status,
    created_at: conversation.created_at,
    updated_at: conversation.updated_at,
  };
}

export function createEmptyEphemeralConversation(): ChatConversation {
  const now = new Date().toISOString();
  return {
    id: `conv_${Date.now()}_${Math.random().toString(36).slice(2, 10)}`,
    title: null,
    title_status: "idle",
    created_at: now,
    updated_at: now,
    messages: [],
  };
}
