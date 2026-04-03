import { tick } from "svelte";

import { authState } from "$lib/auth-state.svelte";
import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
import { isAnonymousChatQuotaError } from "$lib/chat/anonymous-quota";
import type { ChatStreamTiming } from "$lib/chat/conversation-meta";
import { deriveToolCalls } from "$lib/chat/tool-calls";
import { resumeConversationReply } from "$lib/chat/requests";
import type {
  ChatConversation,
  ChatMessage,
  ChatStreamStatus,
  ChatToolCall,
} from "$lib/types";

export type TimedStatus = ChatStreamStatus & { receivedAt: number };

type StreamConnect = (
  signal: AbortSignal,
  handlers: {
    onStatus: (status: ChatStreamStatus) => void;
    onSources: (sources: ChatMessage["sources"]) => void;
    onToken: (token: string) => void;
    onDone: (message: ChatMessage) => void;
    onError: (message: string) => void;
  },
) => Promise<void>;

export function createChatStreamController(options: {
  getActiveConversation: () => ChatConversation | null;
  setActiveConversation: (conversation: ChatConversation | null) => void;
  getCurrentMessageCount: () => number;
  setErrorMessage: (message: string | null) => void;
  setAnonymousQuotaMessage: (message: string | null) => void;
  refreshConversation: (conversationId: string) => Promise<void>;
}) {
  let streamStatuses = $state<TimedStatus[]>([]);
  let streamStartedAt = $state<number | null>(null);
  let streamGenerationStartedAt = $state<number | null>(null);
  let streamDoneAt = $state<number | null>(null);
  let streamingConversationId = $state<string | null>(null);
  let streamingMessageId = $state<string | null>(null);
  let pendingReconnectConversationId = $state<string | null>(null);
  let messagesViewport = $state<HTMLDivElement | null>(null);
  let stickyScroll = $state(true);
  let nearBottom = $state(true);
  let streamController: AbortController | null = null;

  const streamToolCalls = $derived.by((): ChatToolCall[] =>
    deriveToolCalls(streamStatuses),
  );
  const streamTimings = $derived.by((): ChatStreamTiming[] => {
    if (!streamStartedAt) return [];
    const retrievalComplete = [...streamStatuses]
      .reverse()
      .find((s) => s.stage === "retrieving_complete");
    if (!retrievalComplete) return [];
    const timings: ChatStreamTiming[] = [
      {
        label: "Retrieval",
        durationMs: retrievalComplete.receivedAt - streamStartedAt,
      },
    ];
    if (streamGenerationStartedAt) {
      timings.push({
        label: "Synthesis",
        durationMs: streamGenerationStartedAt - retrievalComplete.receivedAt,
      });
      if (streamDoneAt) {
        timings.push({
          label: "Generation",
          durationMs: streamDoneAt - streamGenerationStartedAt,
        });
        timings.push({
          label: "Total",
          durationMs: streamDoneAt - streamStartedAt,
        });
      }
    }
    return timings;
  });
  const showJumpToLatest = $derived(
    !nearBottom && options.getCurrentMessageCount() > 0,
  );

  function buildOptimisticMessage(
    role: ChatMessage["role"],
    content: string,
    status: ChatMessage["status"] = "completed",
  ): ChatMessage {
    return {
      id: `local-${role}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      role,
      content,
      sources: [],
      status,
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- timestamp metadata, not reactive state
      created_at: new Date().toISOString(),
    };
  }

  function pinToBottom() {
    stickyScroll = true;
    nearBottom = true;
  }

  function beginOptimisticStream(conversationId: string, messageId: string) {
    clearStreamState();
    pinToBottom();
    streamStartedAt = Date.now();
    streamingMessageId = messageId;
    streamingConversationId = conversationId;
    pendingReconnectConversationId = null;
  }

  function abortActiveChatStream() {
    streamController?.abort();
    streamController = null;
    streamingConversationId = null;
    streamingMessageId = null;
    pendingReconnectConversationId = null;
  }

  function streamingMessage() {
    const activeConversation = options.getActiveConversation();
    if (!activeConversation || !streamingMessageId) {
      return null;
    }

    return (
      activeConversation.messages.find(
        (message) => message.id === streamingMessageId,
      ) ?? null
    );
  }

  function patchStreamingMessage(patch: Partial<ChatMessage>) {
    const activeConversation = options.getActiveConversation();
    if (!activeConversation || !streamingMessageId) {
      return;
    }

    options.setActiveConversation({
      ...activeConversation,
      messages: activeConversation.messages.map((message) =>
        message.id === streamingMessageId ? { ...message, ...patch } : message,
      ),
    });
  }

  function replaceStreamingMessage(message: ChatMessage) {
    const activeConversation = options.getActiveConversation();
    if (!activeConversation || !streamingMessageId) {
      return;
    }

    options.setActiveConversation({
      ...activeConversation,
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- timestamp metadata, not reactive state
      updated_at: new Date().toISOString(),
      messages: activeConversation.messages.map((candidate) =>
        candidate.id === streamingMessageId ? message : candidate,
      ),
    });
  }

  function handleMessagesScroll() {
    const el = messagesViewport;
    if (!el) {
      return;
    }
    const threshold = 80;
    const distance = el.scrollHeight - el.scrollTop - el.clientHeight;
    const atBottom = distance <= threshold;
    nearBottom = atBottom;
    stickyScroll = atBottom;
  }

  function scrollToBottomIfPinned() {
    if (!stickyScroll) {
      return;
    }
    void scrollToBottom("auto");
  }

  async function scrollToBottom(behavior: "auto" | "smooth" = "smooth") {
    await tick();
    const el = messagesViewport;
    if (!el) {
      return;
    }
    const reduceMotion =
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const effectiveBehavior: "auto" | "smooth" =
      reduceMotion && behavior === "smooth" ? "auto" : behavior;
    el.scrollTo({
      top: el.scrollHeight,
      behavior: effectiveBehavior,
    });
    await tick();
    requestAnimationFrame(() => handleMessagesScroll());
  }

  async function jumpToLatest() {
    pinToBottom();
    await scrollToBottom("smooth");
  }

  function setMessagesViewport(value: HTMLDivElement | null) {
    messagesViewport = value;
  }

  function clearStreamState() {
    streamStatuses = [];
    streamStartedAt = null;
    streamGenerationStartedAt = null;
    streamDoneAt = null;
  }

  function appendStreamStatus(status: ChatStreamStatus) {
    const timed: TimedStatus = { ...status, receivedAt: Date.now() };
    const key = JSON.stringify({
      stage: status.stage,
      label: status.label ?? null,
      detail: status.detail ?? null,
      decision: status.decision ?? null,
      plan: status.plan
        ? {
            intent: status.plan.intent,
            label: status.plan.label,
            budget: status.plan.budget,
            max_per_video: status.plan.max_per_video,
            queries: status.plan.queries,
            expansion_queries: status.plan.expansion_queries,
          }
        : null,
    });
    if (
      streamStatuses.some((existing) => {
        const existingKey = JSON.stringify({
          stage: existing.stage,
          label: existing.label ?? null,
          detail: existing.detail ?? null,
          decision: existing.decision ?? null,
          plan: existing.plan
            ? {
                intent: existing.plan.intent,
                label: existing.plan.label,
                budget: existing.plan.budget,
                max_per_video: existing.plan.max_per_video,
                queries: existing.plan.queries,
                expansion_queries: existing.plan.expansion_queries,
              }
            : null,
        });
        return existingKey === key;
      })
    ) {
      return;
    }
    streamStatuses = [...streamStatuses, timed];
  }

  async function startStream(
    conversationId: string,
    connect: StreamConnect,
    opts?: { resetStreamingMessage?: boolean },
  ) {
    const controller = new AbortController();
    streamController = controller;
    streamingConversationId = conversationId;
    pendingReconnectConversationId = null;

    if (opts?.resetStreamingMessage) {
      patchStreamingMessage({ content: "", sources: [], status: "streaming" });
    }

    try {
      await connect(controller.signal, {
        onStatus: (status) => {
          appendStreamStatus(status);
        },
        onSources: (sources) => {
          patchStreamingMessage({ sources });
          void scrollToBottomIfPinned();
        },
        onToken: (token) => {
          if (!streamGenerationStartedAt) {
            streamGenerationStartedAt = Date.now();
          }
          patchStreamingMessage({
            content: `${streamingMessage()?.content ?? ""}${token}`,
          });
          void scrollToBottomIfPinned();
        },
        onDone: (message) => {
          streamDoneAt = Date.now();
          replaceStreamingMessage(message);
          if (
            authState.current.authState === "anonymous" &&
            options.getActiveConversation()?.title_status === "generating"
          ) {
            options.setActiveConversation({
              ...options.getActiveConversation()!,
              title_status: "idle",
            });
          }
        },
        onError: (message) => {
          if (presentAuthRequiredNoticeIfNeeded(new Error(message))) {
            patchStreamingMessage({
              content: "",
              status: "failed",
            });
            return;
          }
          patchStreamingMessage({
            content: message,
            status: "failed",
          });
          options.setErrorMessage(message);
        },
      });

      await options.refreshConversation(conversationId);
    } catch (error) {
      if ((error as Error).name === "AbortError") {
        return;
      }

      const message = (error as Error).message;
      if (message.includes("Active chat not found")) {
        await options.refreshConversation(conversationId);
        return;
      }
      if (isAnonymousChatQuotaError(message)) {
        options.setAnonymousQuotaMessage(message);
        options.setErrorMessage(null);
        return;
      }
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        options.setErrorMessage(message);
      }
    } finally {
      if (pendingReconnectConversationId !== conversationId) {
        streamController = null;
        streamingConversationId = null;
        streamingMessageId = null;
      }
    }
  }

  function pauseStreamForReconnect() {
    if (!streamingConversationId || !streamController) {
      return;
    }

    pendingReconnectConversationId = streamingConversationId;
    streamController.abort();
    streamController = null;
  }

  async function resumeStreamAfterReconnect() {
    const conversationId = pendingReconnectConversationId;
    if (!conversationId) {
      return;
    }

    await startStream(
      conversationId,
      (signal, handlers) =>
        resumeConversationReply(conversationId, handlers, { signal }),
      { resetStreamingMessage: true },
    );
  }

  return {
    get streamStatuses() {
      return streamStatuses;
    },
    get streamToolCalls() {
      return streamToolCalls;
    },
    get streamTimings() {
      return streamTimings;
    },
    get streamingConversationId() {
      return streamingConversationId;
    },
    get streamingMessageId() {
      return streamingMessageId;
    },
    get showJumpToLatest() {
      return showJumpToLatest;
    },
    buildOptimisticMessage,
    pinToBottom,
    beginOptimisticStream,
    abortActiveChatStream,
    startStream,
    pauseStreamForReconnect,
    resumeStreamAfterReconnect,
    setMessagesViewport,
    handleMessagesScroll,
    scrollToBottom,
    jumpToLatest,
    clearStreamState,
  };
}
