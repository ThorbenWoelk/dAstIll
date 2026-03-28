import type { ChatConversation, ChatConversationSummary } from "$lib/types";

const STORAGE_KEY = "dastill.chat.ephemeralThreads.v1";

export function loadEphemeralThreads(): ChatConversation[] {
  if (typeof sessionStorage === "undefined") {
    return [];
  }
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY);
    if (!raw?.trim()) {
      return [];
    }
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) {
      return [];
    }
    return parsed as ChatConversation[];
  } catch {
    return [];
  }
}

export function saveEphemeralThreads(threads: ChatConversation[]): void {
  if (typeof sessionStorage === "undefined") {
    return;
  }
  try {
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify(threads));
  } catch {
    /* quota or private mode */
  }
}

export function clearEphemeralThreads(): void {
  if (typeof sessionStorage === "undefined") {
    return;
  }
  try {
    sessionStorage.removeItem(STORAGE_KEY);
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
