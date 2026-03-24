<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount, tick } from "svelte";

  import { CONTACT_EMAIL } from "$lib/app-config";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import {
    cancelConversationGeneration,
    createConversation,
    deleteConversation,
    getConversation,
    listConversations,
    reconnectConversationStream,
    renameConversation,
    sendConversationMessage,
  } from "$lib/chat-api";
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import ChatInput from "$lib/components/chat/ChatInput.svelte";
  import ChatMessageBubble from "$lib/components/chat/ChatMessage.svelte";
  import ChatMessageList from "$lib/components/chat/ChatMessageList.svelte";
  import ChatSidebar from "$lib/components/chat/ChatSidebar.svelte";
  import ChatSuggestions from "$lib/components/chat/ChatSuggestions.svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import { createAiStatusPoller } from "$lib/utils/ai-poller";
  import { buildWorkspaceViewHref } from "$lib/view-url";
  import type {
    AiStatus,
    ChatConversation,
    ChatConversationSummary,
    ChatMessage,
    ChatStreamStatus,
    SearchResult,
  } from "$lib/types";

  type TimedStatus = ChatStreamStatus & { receivedAt: number };
  type StreamTiming = { label: string; durationMs: number };

  let conversations = $state<ChatConversationSummary[]>([]);
  let activeConversation = $state<ChatConversation | null>(null);
  let loadingConversations = $state(true);
  let loadingConversation = $state(false);
  let creatingConversation = $state(false);
  let errorMessage = $state<string | null>(null);
  let anonymousQuotaMessage = $state<string | null>(null);
  let draft = $state("");
  let streamStage = $state("idle");
  let streamStatuses = $state<TimedStatus[]>([]);
  let streamStartedAt = $state<number | null>(null);
  let streamGenerationStartedAt = $state<number | null>(null);
  let streamDoneAt = $state<number | null>(null);
  let streamingConversationId = $state<string | null>(null);
  let streamingMessageId = $state<string | null>(null);
  let pendingReconnectConversationId = $state<string | null>(null);
  let aiStatus = $state<AiStatus | null>(null);
  let mobileTab = $state<"conversations" | "content">("content");
  let hydratedConversationId = $state<string | null>(null);
  let handledPromptKey = $state<string | null>(null);
  let deleteConversationId = $state<string | null>(null);
  /** Incremented when starting a new conversation so the prompt bar receives focus. */
  let chatInputFocusSignal = $state(0);
  let messagesViewport = $state<HTMLDivElement | null>(null);
  let streamController: AbortController | null = null;

  /** When true, new tokens and layout growth keep the viewport pinned to the bottom. */
  let stickyScroll = $state(true);
  let nearBottom = $state(true);

  const CHAT_STARTER_PROMPTS = [
    "What topics come up most across my library?",
    "Summarize recent additions I should watch first.",
    "Find mentions of a topic I care about",
  ] as const;

  let requestedConversationId = $derived($page.url.searchParams.get("id"));
  let promptFromUrl = $derived(
    $page.url.searchParams.get("prompt")?.trim() ?? "",
  );
  let isOperator = $derived(Boolean($page.data.isOperator));
  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  let currentMessages = $derived.by(() => {
    const messages = activeConversation?.messages ?? [];
    return [...messages].sort((left, right) =>
      left.created_at.localeCompare(right.created_at),
    );
  });
  let latestStreamStatus = $derived(
    streamStatuses[streamStatuses.length - 1] ?? null,
  );
  let visibleStreamPlan = $derived(
    [...streamStatuses].reverse().find((status) => status.plan)?.plan ?? null,
  );
  let streamPlanLabel = $derived(visibleStreamPlan?.label ?? null);
  let streamUsedExpansionQueries = $derived(
    streamStatuses.some(
      (status) =>
        status.stage === "retrieving_pass_2" ||
        status.stage === "retrieving_pass_3",
    ),
  );
  let streamDisplayedQueries = $derived(
    visibleStreamPlan
      ? Array.from(
          new Set([
            ...visibleStreamPlan.queries,
            ...(streamUsedExpansionQueries
              ? visibleStreamPlan.expansion_queries
              : []),
          ]),
        )
      : [],
  );
  let streamCoverageSummary = $derived(
    [...streamStatuses]
      .reverse()
      .find((status) => status.stage === "retrieving_complete")?.detail ?? null,
  );
  let streamPrimaryDecision = $derived(
    [...streamStatuses]
      .reverse()
      .find(
        (status) =>
          (status.stage === "retrieving_complete" ||
            status.stage === "classifying") &&
          status.decision,
      )?.decision ??
      latestStreamStatus?.decision ??
      null,
  );
  let pendingDeleteConversation = $derived(
    deleteConversationId
      ? (conversations.find(
          (conversation) => conversation.id === deleteConversationId,
        ) ?? null)
      : null,
  );
  let streamBanner = $derived(
    pendingReconnectConversationId
      ? "Reconnecting to active response…"
      : latestStreamStatus?.label
        ? latestStreamStatus.label
        : streamStage === "retrieving"
          ? "Searching knowledge base…"
          : streamStage === "generating"
            ? "Generating response…"
            : null,
  );
  let streamBannerDetail = $derived(
    pendingReconnectConversationId
      ? "Waiting to resume the live response stream."
      : (latestStreamStatus?.detail ?? null),
  );
  let streamTimings = $derived.by((): StreamTiming[] => {
    if (!streamStartedAt) return [];
    const retrievalComplete = [...streamStatuses]
      .reverse()
      .find((s) => s.stage === "retrieving_complete");
    if (!retrievalComplete) return [];
    const timings: StreamTiming[] = [
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

  let streamTraceVisible = $derived(
    Boolean(
      streamPlanLabel ||
      streamDisplayedQueries.length ||
      streamCoverageSummary ||
      streamPrimaryDecision ||
      streamTimings.length > 0,
    ),
  );
  let showConversationMeta = $derived(
    Boolean(streamBanner || streamTraceVisible || errorMessage),
  );
  let conversationMetaInsertMessageId = $derived.by(() => {
    if (
      !activeConversation ||
      currentMessages.length === 0 ||
      !showConversationMeta
    ) {
      return null;
    }

    if (
      streamingMessageId &&
      currentMessages.some((message) => message.id === streamingMessageId)
    ) {
      return streamingMessageId;
    }

    const lastMessage = currentMessages[currentMessages.length - 1];
    return lastMessage?.role === "assistant" ? lastMessage.id : null;
  });
  let conversationMetaInsertIndex = $derived(
    conversationMetaInsertMessageId
      ? currentMessages.findIndex(
          (message) => message.id === conversationMetaInsertMessageId,
        )
      : -1,
  );
  let messagesBeforeConversationMeta = $derived(
    conversationMetaInsertIndex >= 0
      ? currentMessages.slice(0, conversationMetaInsertIndex)
      : currentMessages,
  );
  let messagesAfterConversationMeta = $derived(
    conversationMetaInsertIndex >= 0
      ? currentMessages.slice(conversationMetaInsertIndex)
      : [],
  );

  let showJumpToLatest = $derived(!nearBottom && currentMessages.length > 0);
  let showStarterSuggestions = $derived(
    !loadingConversation &&
      !creatingConversation &&
      currentMessages.length === 0 &&
      !anonymousQuotaMessage,
  );

  onMount(() => {
    const guideParam = new URL(window.location.href).searchParams.get("guide");
    if (guideParam !== null) {
      void goto(`/?guide=${guideParam}`, { replaceState: true });
      return () => {};
    }

    void loadConversations();
    const stopAiPoller = createAiStatusPoller({
      onStatus: (status) => {
        aiStatus = status.status;
      },
    });

    const handleVisibilityChange = () => {
      if (document.visibilityState === "hidden") {
        pauseStreamForReconnect();
      } else {
        void resumeStreamAfterReconnect();
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
      streamController?.abort();
    };
  });

  $effect(() => {
    const conversationId = requestedConversationId;
    if (!conversationId) {
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

    const promptTarget = activeConversation?.id ?? requestedId ?? "new";
    const promptKey = `${promptTarget}:${promptFromUrl}`;
    if (handledPromptKey === promptKey) {
      return;
    }

    if (
      activeConversation?.id &&
      streamingConversationId === activeConversation.id
    ) {
      return;
    }

    handledPromptKey = promptKey;
    void handleSend(promptFromUrl);
  });

  $effect(() => {
    if (isOperator && anonymousQuotaMessage) {
      anonymousQuotaMessage = null;
    }
  });

  $effect(() => {
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

  async function handleSearchResultSelect(
    result: SearchResult,
    mode: "transcript" | "summary",
  ) {
    mobileTab = "content";
    await goto(
      buildWorkspaceViewHref({
        selectedChannelId: result.channel_id,
        selectedVideoId: result.video_id,
        contentMode: mode,
        videoTypeFilter: "all",
        acknowledgedFilter: "all",
      }),
    );
  }

  async function loadConversations(options?: { quiet?: boolean }) {
    if (!options?.quiet) {
      loadingConversations = true;
    }
    try {
      conversations = await listConversations();
      const conversationId = requestedConversationId;
      if (!conversationId && !promptFromUrl && conversations[0]) {
        await navigateToConversation(conversations[0].id);
      }
    } catch (error) {
      errorMessage = (error as Error).message;
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
    if (!options?.quiet) {
      loadingConversation = true;
    }

    try {
      const conversation = await getConversation(conversationId);
      if (requestedConversationId === conversationId) {
        activeConversation = conversation;
        clearStreamState();
        upsertConversationSummary(conversation);
        mobileTab = "content";
        stickyScroll = true;
        await scrollToBottom("auto");
      }
    } catch (error) {
      if (requestedConversationId === conversationId) {
        activeConversation = null;
        errorMessage = (error as Error).message;
      }
    } finally {
      loadingConversation = false;
    }
  }

  async function refreshConversation(conversationId: string) {
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

  async function handleCreateConversation() {
    creatingConversation = true;
    errorMessage = null;
    clearStreamState();
    try {
      const conversation = await createConversation();
      upsertConversationSummary(conversation);
      activeConversation = conversation;
      mobileTab = "content";
      hydratedConversationId = conversation.id;
      await navigateToConversation(conversation.id);
      chatInputFocusSignal += 1;
      await tick();
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      creatingConversation = false;
    }
  }

  async function handleRenameConversation(
    conversationId: string,
    title: string,
  ) {
    try {
      const conversation = await renameConversation(conversationId, title);
      if (activeConversation?.id === conversationId) {
        activeConversation = conversation;
      }
      upsertConversationSummary(conversation);
    } catch (error) {
      errorMessage = (error as Error).message;
    }
  }

  function handleDeleteConversation(conversationId: string) {
    deleteConversationId = conversationId;
  }

  function cancelDeleteConversation() {
    deleteConversationId = null;
  }

  async function confirmDeleteConversation() {
    if (!deleteConversationId) {
      return;
    }

    const conversationId = deleteConversationId;
    deleteConversationId = null;

    try {
      await deleteConversation(conversationId);
      conversations = conversations.filter(
        (conversation) => conversation.id !== conversationId,
      );

      if (activeConversation?.id === conversationId) {
        activeConversation = null;
        hydratedConversationId = null;
        clearStreamState();
        const nextConversation = conversations[0];
        await navigateToConversation(nextConversation?.id ?? null);
      }
    } catch (error) {
      errorMessage = (error as Error).message;
    }
  }

  async function handleSelectConversation(conversationId: string) {
    errorMessage = null;
    clearStreamState();
    mobileTab = "content";
    await navigateToConversation(conversationId);
  }

  async function handleSend(rawValue: string) {
    const content = rawValue.trim();
    if (!content || (!isOperator && anonymousQuotaMessage)) {
      return;
    }

    errorMessage = null;

    let conversation = activeConversation;
    if (!conversation) {
      creatingConversation = true;
      try {
        conversation = await createConversation();
        activeConversation = conversation;
        mobileTab = "content";
        hydratedConversationId = conversation.id;
        upsertConversationSummary(conversation);
        await navigateToConversation(conversation.id);
      } catch (error) {
        creatingConversation = false;
        errorMessage = (error as Error).message;
        return;
      }
      creatingConversation = false;
    }

    if (!conversation) {
      return;
    }

    draft = "";
    await navigateToConversation(conversation.id);

    const userMessage = buildOptimisticMessage("user", content);
    const assistantMessage = buildOptimisticMessage(
      "assistant",
      "",
      "streaming",
    );
    clearStreamState();
    streamStartedAt = Date.now();
    streamingMessageId = assistantMessage.id;
    streamingConversationId = conversation.id;
    streamStage = "retrieving";

    activeConversation = {
      ...conversation,
      title: conversation.title ?? content.slice(0, 80),
      title_status:
        conversation.messages.filter((message) => message.role === "user")
          .length === 0
          ? "generating"
          : conversation.title_status,
      updated_at: new Date().toISOString(),
      messages: [...conversation.messages, userMessage, assistantMessage],
    };
    upsertConversationSummary(activeConversation);
    stickyScroll = true;
    await scrollToBottom();

    await startStream(
      conversation.id,
      (signal, handlers) =>
        sendConversationMessage(conversation.id, { content }, handlers, {
          signal,
        }),
      { resetStreamingMessage: false },
    );
  }

  async function handleCancel() {
    if (!streamingConversationId) {
      return;
    }

    try {
      await cancelConversationGeneration(streamingConversationId);
    } catch (error) {
      errorMessage = (error as Error).message;
    }
  }

  async function startStream(
    conversationId: string,
    connect: (
      signal: AbortSignal,
      handlers: {
        onStatus: (status: ChatStreamStatus) => void;
        onSources: (sources: ChatMessage["sources"]) => void;
        onToken: (token: string) => void;
        onDone: (message: ChatMessage) => void;
        onError: (message: string) => void;
      },
    ) => Promise<void>,
    options?: { resetStreamingMessage?: boolean },
  ) {
    const controller = new AbortController();
    streamController = controller;
    streamingConversationId = conversationId;
    pendingReconnectConversationId = null;

    if (options?.resetStreamingMessage) {
      patchStreamingMessage({ content: "", sources: [], status: "streaming" });
    }

    try {
      await connect(controller.signal, {
        onStatus: (status) => {
          streamStage = status.stage;
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
          streamStage = "generating";
          patchStreamingMessage({
            content: `${streamingMessage()?.content ?? ""}${token}`,
          });
          void scrollToBottomIfPinned();
        },
        onDone: (message) => {
          streamDoneAt = Date.now();
          replaceStreamingMessage(message);
        },
        onError: (message) => {
          patchStreamingMessage({
            content: message,
            status: "failed",
          });
          errorMessage = message;
        },
      });

      await refreshConversation(conversationId);
    } catch (error) {
      if ((error as Error).name === "AbortError") {
        return;
      }

      const message = (error as Error).message;
      if (message.includes("Active chat not found")) {
        await refreshConversation(conversationId);
        return;
      }
      if (isAnonymousChatQuotaError(message)) {
        anonymousQuotaMessage = message;
        errorMessage = null;
        return;
      }
      errorMessage = message;
    } finally {
      if (pendingReconnectConversationId !== conversationId) {
        streamController = null;
        streamingConversationId = null;
        streamingMessageId = null;
        streamStage = "idle";
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
        reconnectConversationStream(conversationId, handlers, { signal }),
      { resetStreamingMessage: true },
    );
  }

  async function navigateToConversation(
    conversationId: string | null,
    options?: { prompt?: string | null },
  ) {
    const params = new URLSearchParams($page.url.searchParams);

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
      created_at: new Date().toISOString(),
    };
  }

  function streamingMessage() {
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
    if (!activeConversation || !streamingMessageId) {
      return;
    }

    activeConversation = {
      ...activeConversation,
      messages: activeConversation.messages.map((message) =>
        message.id === streamingMessageId ? { ...message, ...patch } : message,
      ),
    };
  }

  function replaceStreamingMessage(message: ChatMessage) {
    if (!activeConversation || !streamingMessageId) {
      return;
    }

    activeConversation = {
      ...activeConversation,
      updated_at: new Date().toISOString(),
      messages: activeConversation.messages.map((candidate) =>
        candidate.id === streamingMessageId ? message : candidate,
      ),
    };
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
    stickyScroll = true;
    await scrollToBottom("smooth");
  }

  function clearStreamState() {
    streamStatuses = [];
    streamStartedAt = null;
    streamGenerationStartedAt = null;
    streamDoneAt = null;
  }

  function appendStreamStatus(status: ChatStreamStatus) {
    const timed: TimedStatus = { ...status, receivedAt: Date.now() };
    const key = streamStatusKey(timed);
    if (streamStatuses.some((existing) => streamStatusKey(existing) === key)) {
      return;
    }
    streamStatuses = [...streamStatuses, timed];
  }

  function streamStatusKey(status: ChatStreamStatus) {
    return JSON.stringify({
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
  }

  function isAnonymousChatQuotaError(message: string) {
    return message.includes("Anonymous chat quota exceeded");
  }
</script>

<WorkspaceShell currentSection="chat" {aiIndicator} onOpenGuide={openGuide}>
  <div class="flex h-full min-h-0 w-full">
    <div id="conversations-panel">
      {#if mobileTab === "conversations"}
        <div
          class="fixed inset-0 z-[80] lg:hidden"
          role="dialog"
          aria-modal="true"
          aria-label="Conversations"
        >
          <button
            type="button"
            class="absolute inset-0 bg-[var(--overlay)]"
            onclick={() => (mobileTab = "content")}
            aria-label="Close conversations"
          ></button>
          <div
            class="relative z-10 h-full w-[min(85vw,20rem)] overflow-hidden border-r border-[var(--accent-border-soft)] bg-[var(--surface-strong)] shadow-2xl"
          >
            <ChatSidebar
              mobileVisible={true}
              {conversations}
              activeConversationId={requestedConversationId}
              loading={loadingConversations}
              creating={creatingConversation}
              canDelete={true}
              onCreate={handleCreateConversation}
              onSelect={handleSelectConversation}
              onRename={handleRenameConversation}
              onDelete={handleDeleteConversation}
            />
          </div>
        </div>
      {/if}

      <div class="hidden lg:flex lg:h-full">
        <ChatSidebar
          {conversations}
          activeConversationId={requestedConversationId}
          loading={loadingConversations}
          creating={creatingConversation}
          canDelete={true}
          onCreate={handleCreateConversation}
          onSelect={handleSelectConversation}
          onRename={handleRenameConversation}
          onDelete={handleDeleteConversation}
        />
      </div>
    </div>

    <section
      id="content-view"
      class="fade-in stagger-3 relative z-10 flex min-h-0 min-w-0 flex-col overflow-visible lg:h-full lg:gap-4 lg:px-8 lg:pt-4 lg:pb-6"
    >
      <div
        class="flex flex-col gap-3 px-4 max-lg:pb-1 max-lg:pt-3 sm:px-6 lg:px-0"
      >
        <div class="flex items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <button
              type="button"
              class="inline-flex h-8 items-center justify-center gap-2 rounded-full px-3 text-[12px] font-semibold text-[var(--soft-foreground)] transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 lg:hidden"
              onclick={() => (mobileTab = "conversations")}
              aria-label="Open conversations"
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path
                  d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"
                />
                <path d="M8 9h8" />
                <path d="M8 13h5" />
              </svg>
              <span>History</span>
            </button>
            <h2
              class="text-base font-bold tracking-tight text-[var(--foreground)]"
            >
              Chat
            </h2>
          </div>
          {#if streamingConversationId}
            <span
              class="h-3 w-3 animate-spin rounded-full border-2 border-[var(--border)] border-t-[var(--accent)]"
              role="status"
              aria-label="Generating response"
            ></span>
          {/if}
        </div>

        <div class="border-b border-[var(--accent-border-soft)] pb-3">
          <div class="min-w-0">
            <p
              class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-55"
            >
              Grounded conversation
            </p>
            <p
              class="mt-1 truncate text-[20px] font-semibold tracking-tight text-[var(--foreground)]"
            >
              {activeConversation?.title ?? "New conversation"}
            </p>
            <p
              class="mt-2 max-w-[34rem] text-[14px] leading-6 text-[var(--soft-foreground)]"
            >
              {activeConversation?.title_status === "generating"
                ? "AI is naming this chat while the conversation stays available in the background."
                : "Ask questions grounded in indexed transcripts and summaries, with source-backed answers streamed into this pane."}
            </p>
          </div>
        </div>
      </div>

      {#snippet conversationMeta()}
        <div class="space-y-3">
          {#if streamBanner}
            <div
              class="flex items-start gap-2 rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-3 py-2 text-[12px] text-[var(--soft-foreground)]"
            >
              <span
                class="h-2 w-2 animate-pulse rounded-full bg-[var(--accent)]"
              ></span>
              <div class="min-w-0">
                <p class="font-medium text-[var(--foreground)]">
                  {streamBanner}
                </p>
                {#if streamBannerDetail}
                  <p
                    class="mt-1 text-[12px] leading-relaxed text-[var(--soft-foreground)]"
                  >
                    {streamBannerDetail}
                  </p>
                {/if}
              </div>
            </div>
          {/if}

          {#if streamTraceVisible}
            <div
              class="rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-3 py-3 text-[12px] text-[var(--soft-foreground)]"
            >
              <div class="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <p
                    class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-70"
                  >
                    How I checked
                  </p>
                  {#if streamPlanLabel}
                    <p
                      class="mt-1 text-[12px] font-semibold text-[var(--foreground)]"
                    >
                      {streamPlanLabel}
                    </p>
                  {/if}
                </div>
              </div>

              {#if streamDisplayedQueries.length > 0}
                <div class="mt-3">
                  <p
                    class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-70"
                  >
                    Searches run
                  </p>
                  <div class="mt-2 flex flex-wrap gap-2">
                    {#each streamDisplayedQueries as query}
                      <span
                        class="rounded-full border border-[var(--accent-border-soft)] bg-[var(--surface-strong)] px-2 py-1 text-[11px] text-[var(--foreground)]"
                      >
                        {query}
                      </span>
                    {/each}
                  </div>
                </div>
              {/if}

              {#if streamCoverageSummary}
                <div
                  class="mt-3 rounded-[var(--radius-sm)] bg-[var(--surface-strong)] px-3 py-2"
                >
                  <p
                    class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-70"
                  >
                    Coverage
                  </p>
                  <p
                    class="mt-1 text-[11px] leading-relaxed text-[var(--foreground)]"
                  >
                    {streamCoverageSummary}
                  </p>
                </div>
              {/if}

              {#if streamPrimaryDecision}
                <div
                  class="mt-3 rounded-[var(--radius-sm)] bg-[var(--surface-strong)] px-3 py-2"
                >
                  <p
                    class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-70"
                  >
                    Why this approach
                  </p>
                  <p
                    class="mt-1 text-[11px] leading-relaxed text-[var(--foreground)]"
                  >
                    {streamPrimaryDecision}
                  </p>
                </div>
              {/if}

              {#if streamTimings.length > 0}
                <div
                  class="mt-3 border-t border-[var(--accent-border-soft)] pt-3"
                >
                  <p
                    class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-70"
                  >
                    Timings
                  </p>
                  <div class="mt-2 flex flex-wrap gap-x-5 gap-y-1">
                    {#each streamTimings as timing}
                      <span class="text-[11px] text-[var(--soft-foreground)]">
                        <span class="font-semibold text-[var(--foreground)]"
                          >{timing.label}</span
                        >
                        {(timing.durationMs / 1000).toFixed(1)}s
                      </span>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          {/if}

          {#if errorMessage}
            <div
              class="rounded-[var(--radius-md)] border border-amber-500/20 bg-amber-500/8 px-3 py-2 text-[12px] text-amber-200"
            >
              {errorMessage}
            </div>
          {/if}
        </div>
      {/snippet}

      <div class="relative flex min-h-0 w-full flex-1 flex-col">
        <div
          bind:this={messagesViewport}
          class="custom-scrollbar mobile-bottom-stack-padding min-h-0 flex-1 overflow-y-auto px-4 max-lg:pt-4 sm:px-6 lg:px-0 lg:pr-4 lg:pb-0"
          role="region"
          aria-label="Chat conversation"
          onscroll={handleMessagesScroll}
        >
          {#if !activeConversation || currentMessages.length === 0}
            {#if showConversationMeta}
              <div class="mb-4">
                {@render conversationMeta()}
              </div>
            {/if}
            <ChatMessageList
              messages={currentMessages}
              loadingMessageId={streamingMessageId}
              empty={true}
            />
          {:else if conversationMetaInsertIndex >= 0}
            <div class="flex flex-col gap-8">
              {#each messagesBeforeConversationMeta as message (message.id)}
                <ChatMessageBubble
                  {message}
                  loading={streamingMessageId === message.id}
                />
              {/each}

              {@render conversationMeta()}

              {#each messagesAfterConversationMeta as message (message.id)}
                <ChatMessageBubble
                  {message}
                  loading={streamingMessageId === message.id}
                />
              {/each}
            </div>
          {:else}
            <ChatMessageList
              messages={currentMessages}
              loadingMessageId={streamingMessageId}
              empty={false}
            />
            {#if showConversationMeta}
              <div class="mt-4">
                {@render conversationMeta()}
              </div>
            {/if}
          {/if}
        </div>

        {#if showJumpToLatest}
          <button
            type="button"
            class="absolute bottom-4 left-1/2 z-10 inline-flex h-9 -translate-x-1/2 items-center gap-2 rounded-full border border-[var(--accent-border-soft)] bg-[var(--surface-strong)] px-4 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)] shadow-sm transition-colors hover:bg-[var(--accent-wash)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 motion-reduce:transition-none"
            onclick={() => void jumpToLatest()}
            aria-label="Jump to latest messages"
          >
            <ChevronIcon
              direction="down"
              size={14}
              className="text-[var(--accent)]"
            />
            Latest
          </button>
        {/if}
      </div>

      <div
        class="border-t border-[var(--accent-border-soft)] px-4 py-4 sm:px-6 lg:px-0 lg:pr-4"
      >
        {#if showStarterSuggestions}
          <ChatSuggestions
            suggestions={CHAT_STARTER_PROMPTS}
            disabled={Boolean(streamingConversationId) || loadingConversation}
            onPick={(value) => {
              draft = value;
            }}
          />
        {/if}
        <ChatInput
          bind:value={draft}
          focusSignal={chatInputFocusSignal}
          disabled={loadingConversation ||
            creatingConversation ||
            (!isOperator && Boolean(anonymousQuotaMessage))}
          busy={Boolean(streamingConversationId) || creatingConversation}
          canCancel={Boolean(streamingConversationId)}
          onSubmit={(value) => void handleSend(value)}
          onCancel={() => void handleCancel()}
        />
        {#if anonymousQuotaMessage && !isOperator}
          <div
            class="mt-3 rounded-[var(--radius-md)] bg-[var(--accent-wash)] px-4 py-3"
          >
            <p
              class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--accent-strong)]"
            >
              Preview limit reached
            </p>
            <p
              class="mt-2 text-[14px] leading-relaxed text-[var(--foreground)]"
            >
              This is a showcase - guest chat is limited to give everyone a
              chance to try it.
            </p>
            {#if CONTACT_EMAIL}
              <p
                class="mt-2 text-[12px] leading-relaxed text-[var(--soft-foreground)]"
              >
                Want to explore further?
                <a
                  href="mailto:{CONTACT_EMAIL}"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="text-[var(--accent)] hover:text-[var(--accent-strong)] transition-colors"
                  >{CONTACT_EMAIL}</a
                >
              </p>
            {:else}
              <p
                class="mt-2 text-[12px] leading-relaxed text-[var(--soft-foreground)]"
              >
                Feel free to reach out if you'd like to explore further.
              </p>
            {/if}
          </div>
        {/if}
      </div>
    </section>
  </div>

  <ConfirmationModal
    show={Boolean(deleteConversationId)}
    title={`Delete ${pendingDeleteConversation?.title ? `“${pendingDeleteConversation.title}”` : "conversation"}?`}
    message="This chat and its message history will be permanently removed."
    confirmLabel="Delete"
    cancelLabel="Keep"
    tone="danger"
    onConfirm={() => void confirmDeleteConversation()}
    onCancel={cancelDeleteConversation}
  />
</WorkspaceShell>
