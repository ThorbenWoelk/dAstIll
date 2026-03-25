import type { ChatClientConfig } from "$lib/bindings/ChatClientConfig";
import type {
  ChatConversation,
  ChatConversationSummary,
  ChatMessage,
  ChatSource,
  ChatStreamStatus,
  CreateConversationRequest,
  SendChatMessageRequest,
} from "$lib/types";
import { createAbortError, request, resolveApiUrl } from "$lib/api-client";

type ChatStreamHandlers = {
  onStatus?: (status: ChatStreamStatus) => void;
  onSources?: (sources: ChatSource[]) => void;
  onToken?: (token: string) => void;
  onDone?: (message: ChatMessage) => void;
  onError?: (message: string) => void;
};

export function getChatClientConfig() {
  return request<ChatClientConfig>("/api/chat/config");
}

export function listConversations() {
  return request<ChatConversationSummary[]>("/api/chat/conversations");
}

export function createConversation(payload: CreateConversationRequest = {}) {
  return request<ChatConversation>("/api/chat/conversations", {
    method: "POST",
    body: JSON.stringify(payload),
  });
}

export function getConversation(conversationId: string) {
  return request<ChatConversation>(`/api/chat/conversations/${conversationId}`);
}

export function renameConversation(conversationId: string, title: string) {
  return request<ChatConversation>(
    `/api/chat/conversations/${conversationId}`,
    {
      method: "PUT",
      body: JSON.stringify({ title }),
    },
  );
}

export function deleteConversation(conversationId: string) {
  return request<void>(`/api/chat/conversations/${conversationId}`, {
    method: "DELETE",
  });
}

export function cancelConversationGeneration(conversationId: string) {
  return request<void>(`/api/chat/conversations/${conversationId}/cancel`, {
    method: "POST",
  });
}

export async function sendConversationMessage(
  conversationId: string,
  payload: SendChatMessageRequest,
  handlers: ChatStreamHandlers,
  options?: { signal?: AbortSignal },
) {
  return consumeChatStream(
    `/api/chat/conversations/${conversationId}/messages`,
    {
      method: "POST",
      body: JSON.stringify(payload),
      signal: options?.signal,
    },
    handlers,
  );
}

export async function reconnectConversationStream(
  conversationId: string,
  handlers: ChatStreamHandlers,
  options?: { signal?: AbortSignal },
) {
  return consumeChatStream(
    `/api/chat/conversations/${conversationId}/stream`,
    {
      method: "GET",
      signal: options?.signal,
    },
    handlers,
  );
}

async function consumeChatStream(
  path: string,
  init: RequestInit,
  handlers: ChatStreamHandlers,
) {
  const response = await fetch(resolveApiUrl(path), {
    headers: {
      Accept: "text/event-stream",
      ...(init.body ? { "Content-Type": "application/json" } : {}),
    },
    ...init,
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `Request failed (${response.status})`);
  }

  if (!response.body) {
    throw new Error("Streaming response body is unavailable.");
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  let currentEvent = "message";
  let currentData: string[] = [];

  const dispatch = () => {
    if (currentData.length === 0) {
      currentEvent = "message";
      return;
    }

    const payload = currentData.join("\n");
    currentData = [];
    const data = payload ? JSON.parse(payload) : null;

    switch (currentEvent) {
      case "status":
        handlers.onStatus?.({
          stage: data?.stage ?? "retrieving",
          label: data?.label ?? null,
          detail: data?.detail ?? null,
          decision: data?.decision ?? null,
          plan: data?.plan ?? null,
        } satisfies ChatStreamStatus);
        break;
      case "sources":
        handlers.onSources?.((data?.sources ?? []) as ChatSource[]);
        break;
      case "token":
        handlers.onToken?.(data?.token ?? "");
        break;
      case "done":
        handlers.onDone?.(data?.message as ChatMessage);
        break;
      case "error":
        handlers.onError?.(data?.message ?? "Stream failed.");
        break;
      default:
        break;
    }

    currentEvent = "message";
  };

  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) {
        buffer += decoder.decode();
        break;
      }

      buffer += decoder.decode(value, { stream: true });
      buffer = consumeBufferedEvents(buffer, (line) => {
        if (!line) {
          dispatch();
          return;
        }
        if (line.startsWith(":")) {
          return;
        }
        if (line.startsWith("event:")) {
          currentEvent = line.slice(6).trim() || "message";
          return;
        }
        if (line.startsWith("data:")) {
          currentData.push(line.slice(5).trimStart());
        }
      });
    }

    buffer = consumeBufferedEvents(buffer, (line) => {
      if (!line) {
        dispatch();
        return;
      }
      if (line.startsWith("event:")) {
        currentEvent = line.slice(6).trim() || "message";
        return;
      }
      if (line.startsWith("data:")) {
        currentData.push(line.slice(5).trimStart());
      }
    });

    if (currentData.length > 0) {
      dispatch();
    }
  } catch (error) {
    if ((error as Error).name === "AbortError") {
      throw createAbortError();
    }
    throw error;
  } finally {
    reader.releaseLock();
  }
}

function consumeBufferedEvents(buffer: string, onLine: (line: string) => void) {
  let nextBuffer = buffer;

  for (;;) {
    const newlineIndex = nextBuffer.indexOf("\n");
    if (newlineIndex === -1) {
      return nextBuffer;
    }

    const line = nextBuffer.slice(0, newlineIndex).replace(/\r$/, "");
    onLine(line);
    nextBuffer = nextBuffer.slice(newlineIndex + 1);
  }
}
