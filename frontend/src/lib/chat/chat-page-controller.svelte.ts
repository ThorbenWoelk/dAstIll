import { goto } from "$app/navigation";
import { page } from "$app/state";
import { onMount, tick } from "svelte";
import { SvelteURLSearchParams } from "svelte/reactivity";

import { authState } from "$lib/auth-state.svelte";
import { getAuthStorageScopeKey, getScopedStorageKey } from "$lib/auth-storage";
import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
import { resolveAiIndicatorPresentation } from "$lib/ai-status";
import type { ChatClientConfig } from "$lib/bindings/ChatClientConfig";
import type {
  AiStatus,
  ChatConversation,
  ChatConversationSummary,
} from "$lib/types";
import {
  clearEphemeralThreads,
  conversationToSummary,
  createEmptyEphemeralConversation,
  loadEphemeralThreads,
  saveEphemeralThreads,
} from "$lib/chat/ephemeral-session";
import {
  cancelConversationReply,
  createConversation,
  deleteAllConversations,
  deleteConversation,
  getChatClientConfig,
  getConversation,
  listConversations,
  renameConversation,
  sendConversationMessage,
  sendEphemeralConversationMessage,
} from "$lib/chat/requests";
import { createAiStatusPoller } from "$lib/utils/ai-poller";

import { createChatStreamController } from "$lib/chat/chat-stream-controller.svelte";

function pickInitialChatModelId(
  cfg: ChatClientConfig,
  storageKey: string,
): string {
  try {
    const stored = localStorage.getItem(storageKey)?.trim();
    if (stored && cfg.models.some((entry) => entry.id === stored)) {
      return stored;
    }
  } catch {
    /* ignore */
  }
  if (cfg.models.some((entry) => entry.id === cfg.default_model)) {
    return cfg.default_model;
  }
  return cfg.models[0]?.id ?? "";
}

export function createChatPageController() {
  let conversations = $state<ChatConversationSummary[]>([]);
  /** Anonymous-only: full threads kept in sessionStorage, never listed from the API. */
  let ephemeralThreads = $state<ChatConversation[]>([]);
  let activeConversation = $state<ChatConversation | null>(null);
  let loadingConversations = $state(true);
  let loadingConversation = $state(false);
  let creatingConversation = $state(false);
  let errorMessage = $state<string | null>(null);
  let anonymousQuotaMessage = $state<string | null>(null);
  let draft = $state("");
  let deepResearch = $state(false);
  let aiStatus = $state<AiStatus | null>(null);
  let mobileTab = $state<"conversations" | "content">("content");
  let hydratedConversationId = $state<string | null>(null);
  let hydratedConversationScopeKey = $state<string | null>(null);
  let handledPromptKey = $state<string | null>(null);
  let deleteConversationId = $state<string | null>(null);
  let confirmDeleteAll = $state(false);
  let deletingAllConversations = $state(false);
  /** Incremented when starting a new conversation so the prompt bar receives focus. */
  let chatInputFocusSignal = $state(0);
  let chatClientConfig = $state<ChatClientConfig | null>(null);
  let selectedChatModelId = $state("");
  const chatModelStorageKey = $derived(
    getScopedStorageKey(
      "dastill.chat.cloudModel",
      getAuthStorageScopeKey(authState.current),
    ),
  );
  const chatStorageScopeKey = $derived(
    getAuthStorageScopeKey(authState.current),
  );
  const ephemeralThreadsStorageKey = $derived(
    getScopedStorageKey(
      "dastill.chat.ephemeralThreads.v1",
      chatStorageScopeKey,
    ),
  );

  const requestedConversationId = $derived(page.url.searchParams.get("id"));
  const promptFromUrl = $derived(
    page.url.searchParams.get("prompt")?.trim() ?? "",
  );
  const isAuthenticated = $derived(
    authState.current.authState === "authenticated",
  );
  const aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  const currentMessages = $derived.by(() => {
    const messages = activeConversation?.messages ?? [];
    return [...messages].sort((left, right) =>
      left.created_at.localeCompare(right.created_at),
    );
  });

  const stream = createChatStreamController({
    getActiveConversation: () => activeConversation,
    setActiveConversation: (conversation) => {
      activeConversation = conversation;
    },
    getCurrentMessageCount: () => currentMessages.length,
    setErrorMessage: (message) => {
      errorMessage = message;
    },
    setAnonymousQuotaMessage: (message) => {
      anonymousQuotaMessage = message;
    },
    refreshConversation: async (conversationId) => {
      await refreshConversation(conversationId);
    },
  });

  const pendingDeleteConversation = $derived(
    deleteConversationId
      ? (conversations.find(
          (conversation) => conversation.id === deleteConversationId,
        ) ?? null)
      : null,
  );
  const showDeleteConfirmation = $derived(
    Boolean(deleteConversationId || confirmDeleteAll),
  );
  const deleteConfirmationTitle = $derived.by(() => {
    if (confirmDeleteAll) {
      return "Delete all conversations?";
    }
    return `Delete ${pendingDeleteConversation?.title ? `“${pendingDeleteConversation.title}”` : "conversation"}?`;
  });
  const deleteConfirmationMessage = $derived(
    confirmDeleteAll
      ? "Every chat and its message history will be permanently removed."
      : "This chat and its message history will be permanently removed.",
  );
  const deleteConfirmationConfirmLabel = $derived(
    confirmDeleteAll ? "Delete all" : "Delete",
  );
  const deleteConfirmationCancelLabel = $derived(
    confirmDeleteAll ? "Keep chats" : "Keep",
  );
  /** URL points at a thread that is not yet reflected in activeConversation (during fetch). */
  const showThreadPlaceholderLoading = $derived(
    Boolean(
      requestedConversationId &&
      loadingConversation &&
      (!activeConversation ||
        activeConversation.id !== requestedConversationId),
    ),
  );
  const headerConversationTitle = $derived.by(() => {
    if (activeConversation?.id === requestedConversationId) {
      return activeConversation.title ?? "New conversation";
    }
    if (requestedConversationId) {
      const summary = conversations.find(
        (conversation) => conversation.id === requestedConversationId,
      );
      return summary?.title ?? "Conversation";
    }
    return "New conversation";
  });

  const showConversationMeta = $derived(
    Boolean(
      stream.streamStatuses.length > 0 ||
      stream.streamToolCalls.length > 0 ||
      errorMessage,
    ),
  );
  const conversationMetaInsertMessageId = $derived.by(() => {
    if (
      !activeConversation ||
      currentMessages.length === 0 ||
      !showConversationMeta
    ) {
      return null;
    }

    if (
      stream.streamingMessageId &&
      currentMessages.some(
        (message) => message.id === stream.streamingMessageId,
      )
    ) {
      return stream.streamingMessageId;
    }

    const lastMessage = currentMessages[currentMessages.length - 1];
    return lastMessage?.role === "assistant" ? lastMessage.id : null;
  });
  const conversationMetaInsertIndex = $derived(
    conversationMetaInsertMessageId
      ? currentMessages.findIndex(
          (message) => message.id === conversationMetaInsertMessageId,
        )
      : -1,
  );
  const messagesBeforeConversationMeta = $derived(
    conversationMetaInsertIndex >= 0
      ? currentMessages.slice(0, conversationMetaInsertIndex)
      : currentMessages,
  );
  const messagesAfterConversationMeta = $derived(
    conversationMetaInsertIndex >= 0
      ? currentMessages.slice(conversationMetaInsertIndex)
      : [],
  );
  const showStarterSuggestions = $derived(
    !loadingConversation &&
      !creatingConversation &&
      currentMessages.length === 0 &&
      !anonymousQuotaMessage,
  );

  onMount(() => {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- transient URL for one-time guide redirect check
    const guideParam = new URL(window.location.href).searchParams.get("guide");
    if (guideParam !== null) {
      void goto(`/?guide=${guideParam}`, { replaceState: true });
      return () => {};
    }

    void loadConversations();
    void getChatClientConfig()
      .then((cfg) => {
        chatClientConfig = cfg;
        setSelectedChatModelId(
          pickInitialChatModelId(cfg, chatModelStorageKey),
        );
      })
      .catch(() => {
        chatClientConfig = null;
        setSelectedChatModelId("");
      });
    const stopAiPoller = createAiStatusPoller({
      onStatus: (status) => {
        aiStatus = status.status;
      },
    });

    const handleVisibilityChange = () => {
      if (authState.current.authState === "anonymous") {
        return;
      }
      if (document.visibilityState === "hidden") {
        stream.pauseStreamForReconnect();
      } else {
        void stream.resumeStreamAfterReconnect();
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);

    const onChatNewConversation = () => {
      if (creatingConversation) {
        return;
      }
      void handleCreateConversation();
    };
    const onChatFocusComposer = () => {
      chatInputFocusSignal += 1;
    };
    window.addEventListener(
      "dastill:chat-new-conversation",
      onChatNewConversation,
    );
    window.addEventListener("dastill:chat-focus-composer", onChatFocusComposer);

    return () => {
      window.removeEventListener(
        "dastill:chat-new-conversation",
        onChatNewConversation,
      );
      window.removeEventListener(
        "dastill:chat-focus-composer",
        onChatFocusComposer,
      );
      stopAiPoller();
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      stream.abortActiveChatStream();
    };
  });

  $effect(() => {
    const id = selectedChatModelId;
    if (!id) {
      return;
    }
    try {
      localStorage.setItem(chatModelStorageKey, id);
    } catch {
      /* ignore */
    }
  });

  $effect(() => {
    const conversationId = requestedConversationId;
    if (!conversationId) {
      stream.abortActiveChatStream();
      stream.clearStreamState();
      activeConversation = null;
      hydratedConversationId = null;
      return;
    }

    if (hydratedConversationId === conversationId) {
      return;
    }

    hydratedConversationId = conversationId;
    void loadConversation(conversationId);
  });

  $effect(() => {
    if (!promptFromUrl || loadingConversations || creatingConversation) {
      return;
    }

    const requestedId = requestedConversationId;
    if (requestedId && activeConversation?.id !== requestedId) {
      return;
    }

    const promptKey = `url-prompt:${promptFromUrl}`;
    if (handledPromptKey === promptKey) {
      return;
    }

    if (
      activeConversation?.id &&
      stream.streamingConversationId === activeConversation.id
    ) {
      return;
    }

    handledPromptKey = promptKey;
    void handleSend(promptFromUrl);
  });

  $effect(() => {
    if (isAuthenticated && anonymousQuotaMessage) {
      anonymousQuotaMessage = null;
    }
  });

  $effect(() => {
    if (!isAuthenticated) {
      return;
    }
    const conversationId = activeConversation?.id;
    const isGeneratingTitle = activeConversation?.title_status === "generating";
    if (!conversationId || !isGeneratingTitle) {
      return;
    }

    const intervalId = window.setInterval(() => {
      void refreshConversation(conversationId);
    }, 2500);

    return () => window.clearInterval(intervalId);
  });

  function openGuide() {
    void goto("/?guide=0");
  }

  function openMobileConversations() {
    setMobileTab("conversations");
  }

  function closeMobileConversations() {
    setMobileTab("content");
  }

  function setMobileTab(value: "conversations" | "content") {
    mobileTab = value;
  }

  function setDraft(value: string) {
    draft = value;
  }

  function pickStarterPrompt(value: string) {
    setDraft(value);
  }

  function setDeepResearch(value: boolean) {
    deepResearch = value;
  }

  function setSelectedChatModelId(value: string) {
    selectedChatModelId = value;
  }

  function bindMessagesViewport(node: HTMLDivElement) {
    stream.setMessagesViewport(node);
    return {
      destroy() {
        stream.setMessagesViewport(null);
      },
    };
  }

  async function loadConversations(options?: { quiet?: boolean }) {
    if (!options?.quiet) {
      loadingConversations = true;
    }
    try {
      if (authState.current.authState === "anonymous") {
        ephemeralThreads = loadEphemeralThreads(chatStorageScopeKey);
        conversations = ephemeralThreads.map(conversationToSummary);
        const conversationId = requestedConversationId;
        if (!conversationId && !promptFromUrl && conversations[0]) {
          await navigateToConversation(conversations[0].id);
        }
      } else {
        conversations = await listConversations();
        const conversationId = requestedConversationId;
        if (!conversationId && !promptFromUrl && conversations[0]) {
          await navigateToConversation(conversations[0].id);
        }
      }
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    } finally {
      if (!options?.quiet) {
        loadingConversations = false;
      }
    }
  }

  async function loadConversation(
    conversationId: string,
    options?: { quiet?: boolean },
  ) {
    if (authState.current.authState === "anonymous") {
      if (!options?.quiet) {
        loadingConversation = true;
      }
      const found = ephemeralThreads.find(
        (thread) => thread.id === conversationId,
      );
      if (requestedConversationId === conversationId) {
        if (found) {
          activeConversation = structuredClone(found);
          stream.clearStreamState();
          upsertConversationSummary(conversationToSummary(found));
          setMobileTab("content");
          stream.pinToBottom();
          await stream.scrollToBottom("auto");
        } else {
          activeConversation = null;
          errorMessage = "Conversation not found.";
        }
      }
      loadingConversation = false;
      return;
    }

    if (!options?.quiet) {
      loadingConversation = true;
      if (
        requestedConversationId === conversationId &&
        activeConversation !== null &&
        activeConversation.id !== conversationId
      ) {
        activeConversation = null;
      }
    }

    try {
      const conversation = await getConversation(conversationId);
      if (requestedConversationId === conversationId) {
        activeConversation = conversation;
        stream.clearStreamState();
        upsertConversationSummary(conversation);
        setMobileTab("content");
        stream.pinToBottom();
        await stream.scrollToBottom("auto");
      }
    } catch (error) {
      if (requestedConversationId === conversationId) {
        activeConversation = null;
        if (!presentAuthRequiredNoticeIfNeeded(error)) {
          errorMessage = (error as Error).message;
        }
      }
    } finally {
      loadingConversation = false;
    }
  }

  async function refreshConversation(conversationId: string) {
    if (authState.current.authState === "anonymous") {
      if (activeConversation?.id === conversationId) {
        persistEphemeralFromActive();
      }
      return;
    }
    try {
      const conversation = await getConversation(conversationId);
      if (activeConversation?.id === conversationId) {
        activeConversation = conversation;
        upsertConversationSummary(conversation);
      }
      await loadConversations({ quiet: true });
    } catch {
      // Keep optimistic UI if the refresh fails.
    }
  }

  function persistEphemeralFromActive() {
    if (authState.current.authState !== "anonymous" || !activeConversation) {
      return;
    }
    const id = activeConversation.id;
    const idx = ephemeralThreads.findIndex((thread) => thread.id === id);
    const merged: ChatConversation[] =
      idx === -1
        ? [activeConversation, ...ephemeralThreads]
        : ephemeralThreads.map((thread) =>
            thread.id === id
              ? (activeConversation as ChatConversation)
              : thread,
          );
    ephemeralThreads = merged;
    saveEphemeralThreads(chatStorageScopeKey, merged);
    conversations = merged.map(conversationToSummary);
  }

  async function handleCreateConversation() {
    creatingConversation = true;
    errorMessage = null;
    stream.abortActiveChatStream();
    stream.clearStreamState();
    try {
      if (authState.current.authState === "anonymous") {
        const conversation = createEmptyEphemeralConversation();
        ephemeralThreads = [conversation, ...ephemeralThreads];
        saveEphemeralThreads(chatStorageScopeKey, ephemeralThreads);
        conversations = ephemeralThreads.map(conversationToSummary);
        upsertConversationSummary(conversationToSummary(conversation));
        activeConversation = conversation;
        setMobileTab("content");
        hydratedConversationId = conversation.id;
        await navigateToConversation(conversation.id);
        chatInputFocusSignal += 1;
        await tick();
      } else {
        const conversation = await createConversation();
        upsertConversationSummary(conversation);
        activeConversation = conversation;
        setMobileTab("content");
        hydratedConversationId = conversation.id;
        await navigateToConversation(conversation.id);
        chatInputFocusSignal += 1;
        await tick();
      }
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    } finally {
      creatingConversation = false;
    }
  }

  async function handleRenameConversation(
    conversationId: string,
    title: string,
  ) {
    try {
      if (authState.current.authState === "anonymous") {
        const next = ephemeralThreads.map((thread) =>
          thread.id === conversationId
            ? {
                ...thread,
                title,
                title_status: "manual" as const,
                // eslint-disable-next-line svelte/prefer-svelte-reactivity -- timestamp metadata, not reactive state
                updated_at: new Date().toISOString(),
              }
            : thread,
        );
        ephemeralThreads = next;
        saveEphemeralThreads(chatStorageScopeKey, next);
        conversations = next.map(conversationToSummary);
        if (activeConversation?.id === conversationId) {
          activeConversation = {
            ...activeConversation,
            title,
            title_status: "manual",
            // eslint-disable-next-line svelte/prefer-svelte-reactivity -- timestamp metadata, not reactive state
            updated_at: new Date().toISOString(),
          };
        }
        return;
      }
      const conversation = await renameConversation(conversationId, title);
      if (activeConversation?.id === conversationId) {
        activeConversation = conversation;
      }
      upsertConversationSummary(conversation);
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    }
  }

  function handleDeleteConversation(conversationId: string) {
    confirmDeleteAll = false;
    deleteConversationId = conversationId;
  }

  function handleDeleteAllConversations() {
    deleteConversationId = null;
    confirmDeleteAll = true;
  }

  function cancelDeleteConversation() {
    deleteConversationId = null;
    confirmDeleteAll = false;
  }

  async function confirmDeleteConversation() {
    if (confirmDeleteAll) {
      deletingAllConversations = true;
      confirmDeleteAll = false;

      try {
        if (authState.current.authState === "anonymous") {
          clearEphemeralThreads(chatStorageScopeKey);
          ephemeralThreads = [];
          conversations = [];
          activeConversation = null;
          hydratedConversationId = null;
          stream.abortActiveChatStream();
          stream.clearStreamState();
          setMobileTab("content");
          await navigateToConversation(null);
        } else {
          await deleteAllConversations();
          conversations = [];
          activeConversation = null;
          hydratedConversationId = null;
          stream.abortActiveChatStream();
          stream.clearStreamState();
          setMobileTab("content");
          await navigateToConversation(null);
        }
      } catch (error) {
        if (!presentAuthRequiredNoticeIfNeeded(error)) {
          errorMessage = (error as Error).message;
        }
      } finally {
        deletingAllConversations = false;
      }
      return;
    }

    if (!deleteConversationId) {
      return;
    }

    const conversationId = deleteConversationId;
    deleteConversationId = null;

    try {
      if (authState.current.authState === "anonymous") {
        const nextThreads = ephemeralThreads.filter(
          (conversation) => conversation.id !== conversationId,
        );
        ephemeralThreads = nextThreads;
        saveEphemeralThreads(chatStorageScopeKey, nextThreads);
        conversations = nextThreads.map(conversationToSummary);
        if (activeConversation?.id === conversationId) {
          activeConversation = null;
          hydratedConversationId = null;
          stream.abortActiveChatStream();
          stream.clearStreamState();
          const nextConversation = conversations[0];
          await navigateToConversation(nextConversation?.id ?? null);
        }
      } else {
        await deleteConversation(conversationId);
        conversations = conversations.filter(
          (conversation) => conversation.id !== conversationId,
        );

        if (activeConversation?.id === conversationId) {
          activeConversation = null;
          hydratedConversationId = null;
          stream.abortActiveChatStream();
          stream.clearStreamState();
          const nextConversation = conversations[0];
          await navigateToConversation(nextConversation?.id ?? null);
        }
      }
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    }
  }

  async function handleSelectConversation(conversationId: string) {
    errorMessage = null;
    stream.abortActiveChatStream();
    stream.clearStreamState();
    setMobileTab("content");
    await navigateToConversation(conversationId);
  }

  async function handleSend(rawValue: string) {
    const content = rawValue.trim();
    if (!content || (!isAuthenticated && anonymousQuotaMessage)) {
      return;
    }

    errorMessage = null;

    let conversation = activeConversation;
    if (!conversation) {
      creatingConversation = true;
      try {
        if (authState.current.authState === "anonymous") {
          conversation = createEmptyEphemeralConversation();
          ephemeralThreads = [conversation, ...ephemeralThreads];
          saveEphemeralThreads(chatStorageScopeKey, ephemeralThreads);
          conversations = ephemeralThreads.map(conversationToSummary);
          activeConversation = conversation;
          setMobileTab("content");
          hydratedConversationId = conversation.id;
          upsertConversationSummary(conversationToSummary(conversation));
          await navigateToConversation(conversation.id);
        } else {
          conversation = await createConversation();
          activeConversation = conversation;
          setMobileTab("content");
          hydratedConversationId = conversation.id;
          upsertConversationSummary(conversation);
          await navigateToConversation(conversation.id);
        }
      } catch (error) {
        creatingConversation = false;
        if (!presentAuthRequiredNoticeIfNeeded(error)) {
          errorMessage = (error as Error).message;
        }
        return;
      }
      creatingConversation = false;
    }

    if (!conversation) {
      return;
    }

    const ephemeralRequestBase =
      authState.current.authState === "anonymous"
        ? structuredClone(conversation)
        : null;

    setDraft("");
    await navigateToConversation(conversation.id);

    const userMessage = stream.buildOptimisticMessage("user", content);
    const assistantMessage = stream.buildOptimisticMessage(
      "assistant",
      "",
      "streaming",
    );
    stream.beginOptimisticStream(conversation.id, assistantMessage.id);

    activeConversation = {
      ...conversation,
      title: conversation.title ?? content.slice(0, 80),
      title_status:
        conversation.messages.filter((message) => message.role === "user")
          .length === 0
          ? "generating"
          : conversation.title_status,
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- timestamp metadata, not reactive state
      updated_at: new Date().toISOString(),
      messages: [...conversation.messages, userMessage, assistantMessage],
    };
    upsertConversationSummary(
      authState.current.authState === "anonymous"
        ? conversationToSummary(activeConversation)
        : activeConversation,
    );
    stream.pinToBottom();
    await stream.scrollToBottom();

    await stream.startStream(
      conversation.id,
      (signal, handlers) =>
        authState.current.authState === "anonymous" && ephemeralRequestBase
          ? sendEphemeralConversationMessage(
              {
                conversation: ephemeralRequestBase,
                content,
                deep_research: deepResearch,
                ...(selectedChatModelId ? { model: selectedChatModelId } : {}),
              },
              handlers,
              {
                signal,
              },
            )
          : sendConversationMessage(
              conversation.id,
              {
                content,
                deep_research: deepResearch,
                ...(selectedChatModelId ? { model: selectedChatModelId } : {}),
              },
              handlers,
              {
                signal,
              },
            ),
      { resetStreamingMessage: false },
    );
  }

  $effect(() => {
    if (typeof window === "undefined") {
      return;
    }

    if (hydratedConversationScopeKey === null) {
      hydratedConversationScopeKey = ephemeralThreadsStorageKey;
      return;
    }

    if (hydratedConversationScopeKey === ephemeralThreadsStorageKey) {
      return;
    }

    hydratedConversationScopeKey = ephemeralThreadsStorageKey;
    activeConversation = null;
    hydratedConversationId = null;
    stream.abortActiveChatStream();
    stream.clearStreamState();
    void loadConversations({ quiet: true });
  });

  async function handleCancel() {
    if (!stream.streamingConversationId) {
      return;
    }

    try {
      await cancelConversationReply(stream.streamingConversationId);
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    }
  }

  async function navigateToConversation(
    conversationId: string | null,
    options?: { prompt?: string | null },
  ) {
    const params = new SvelteURLSearchParams(page.url.searchParams);

    if (conversationId) {
      params.set("id", conversationId);
    } else {
      params.delete("id");
    }

    if (options?.prompt) {
      params.set("prompt", options.prompt);
    } else {
      params.delete("prompt");
    }

    const query = params.toString();
    await goto(query ? `/chat?${query}` : "/chat", {
      replaceState: true,
      noScroll: true,
      keepFocus: true,
    });
  }

  function upsertConversationSummary(conversation: ChatConversationSummary) {
    conversations = [
      conversation,
      ...conversations.filter((candidate) => candidate.id !== conversation.id),
    ].sort((left, right) => right.updated_at.localeCompare(left.updated_at));
  }

  return {
    get aiIndicator() {
      return aiIndicator;
    },
    openGuide,
    get isMobileConversationsOpen() {
      return mobileTab === "conversations";
    },
    openMobileConversations,
    closeMobileConversations,
    get conversations() {
      return conversations;
    },
    get requestedConversationId() {
      return requestedConversationId;
    },
    get loadingConversations() {
      return loadingConversations;
    },
    get creatingConversation() {
      return creatingConversation;
    },
    get deletingAllConversations() {
      return deletingAllConversations;
    },
    handleCreateConversation,
    handleSelectConversation,
    handleRenameConversation,
    handleDeleteConversation,
    handleDeleteAllConversations,
    get headerConversationTitle() {
      return headerConversationTitle;
    },
    get activeConversation() {
      return activeConversation;
    },
    get currentMessages() {
      return currentMessages;
    },
    get loadingConversation() {
      return loadingConversation;
    },
    get showThreadPlaceholderLoading() {
      return showThreadPlaceholderLoading;
    },
    get showConversationMeta() {
      return showConversationMeta;
    },
    get conversationMetaInsertIndex() {
      return conversationMetaInsertIndex;
    },
    get messagesBeforeConversationMeta() {
      return messagesBeforeConversationMeta;
    },
    get messagesAfterConversationMeta() {
      return messagesAfterConversationMeta;
    },
    get errorMessage() {
      return errorMessage;
    },
    get showStarterSuggestions() {
      return showStarterSuggestions;
    },
    get draft() {
      return draft;
    },
    setDraft,
    pickStarterPrompt,
    get deepResearch() {
      return deepResearch;
    },
    setDeepResearch,
    get selectedChatModelId() {
      return selectedChatModelId;
    },
    setSelectedChatModelId,
    get chatClientConfig() {
      return chatClientConfig;
    },
    get chatInputFocusSignal() {
      return chatInputFocusSignal;
    },
    get anonymousQuotaMessage() {
      return anonymousQuotaMessage;
    },
    get isAuthenticated() {
      return isAuthenticated;
    },
    bindMessagesViewport,
    handleMessagesViewportScroll: stream.handleMessagesScroll,
    handleSend,
    handleCancel,
    get showDeleteConfirmation() {
      return showDeleteConfirmation;
    },
    get deleteConfirmationTitle() {
      return deleteConfirmationTitle;
    },
    get deleteConfirmationMessage() {
      return deleteConfirmationMessage;
    },
    get deleteConfirmationConfirmLabel() {
      return deleteConfirmationConfirmLabel;
    },
    get deleteConfirmationCancelLabel() {
      return deleteConfirmationCancelLabel;
    },
    confirmDeleteConversation,
    cancelDeleteConversation,
    stream,
  };
}
