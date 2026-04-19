<script lang="ts">
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import ChatAnonymousQuotaNotice from "$lib/components/chat/ChatAnonymousQuotaNotice.svelte";
  import ChatContentSectionHeader from "$lib/components/chat/ChatContentSectionHeader.svelte";
  import ChatConversationMeta from "$lib/components/chat/ChatConversationMeta.svelte";
  import ChatInput from "$lib/components/chat/ChatInput.svelte";
  import ChatMessageBubble from "$lib/components/chat/ChatMessage.svelte";
  import ChatMessageList from "$lib/components/chat/ChatMessageList.svelte";
  import ChatMobileConversationsOverlay from "$lib/components/chat/ChatMobileConversationsOverlay.svelte";
  import ChatSidebar from "$lib/components/chat/ChatSidebar.svelte";
  import ChatSuggestions from "$lib/components/chat/ChatSuggestions.svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import MobileYouTubeTopNav from "$lib/components/mobile/MobileYouTubeTopNav.svelte";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import { CHAT_STARTER_PROMPTS } from "$lib/chat/starter-prompts";
  import { createChatPageController } from "$lib/chat/chat-page-controller.svelte";

  const chat = createChatPageController();
  const bindMessagesViewport = chat.bindMessagesViewport;
</script>

<WorkspaceShell
  currentSection="chat"
  aiIndicator={chat.aiIndicator}
  onOpenGuide={chat.openGuide}
>
  {#snippet mobileTopBar()}
    <MobileYouTubeTopNav />
  {/snippet}
  <div class="flex h-full min-h-0 w-full">
    <div id="conversations-panel">
      <ChatMobileConversationsOverlay
        open={chat.isMobileConversationsOpen}
        onClose={chat.closeMobileConversations}
      >
        <ChatSidebar
          mobileVisible={true}
          conversations={chat.conversations}
          activeConversationId={chat.requestedConversationId}
          loading={chat.loadingConversations}
          creating={chat.creatingConversation}
          deletingAll={chat.deletingAllConversations}
          canDelete={true}
          onCreate={chat.handleCreateConversation}
          onSelect={chat.handleSelectConversation}
          onRename={chat.handleRenameConversation}
          onDelete={chat.handleDeleteConversation}
          onDeleteAll={chat.handleDeleteAllConversations}
        />
      </ChatMobileConversationsOverlay>

      <div class="hidden w-64 shrink-0 lg:flex lg:h-full">
        <ChatSidebar
          conversations={chat.conversations}
          activeConversationId={chat.requestedConversationId}
          loading={chat.loadingConversations}
          creating={chat.creatingConversation}
          deletingAll={chat.deletingAllConversations}
          canDelete={true}
          onCreate={chat.handleCreateConversation}
          onSelect={chat.handleSelectConversation}
          onRename={chat.handleRenameConversation}
          onDelete={chat.handleDeleteConversation}
          onDeleteAll={chat.handleDeleteAllConversations}
        />
      </div>
    </div>

    <section
      id="content-view"
      class="fade-in stagger-3 relative z-10 flex min-h-0 min-w-0 flex-1 flex-col overflow-visible bg-[var(--surface-strong)] lg:h-full"
    >
      <div class="lg:hidden">
        <ChatContentSectionHeader
          onOpenConversationsMobile={chat.openMobileConversations}
          streamingConversationId={chat.stream.streamingConversationId}
          conversationTitle={chat.headerConversationTitle}
          titleStatus={chat.activeConversation?.id ===
          chat.requestedConversationId
            ? chat.activeConversation.title_status
            : undefined}
        />
      </div>

      <div class="relative flex min-h-0 w-full flex-1 flex-col">
        <div
          use:bindMessagesViewport
          class="custom-scrollbar mobile-bottom-stack-padding min-h-0 flex-1 overflow-y-auto px-4 max-lg:pt-4 sm:px-6 lg:px-8 lg:py-8"
          role="region"
          aria-label="Chat conversation"
          onscroll={chat.handleMessagesViewportScroll}
        >
          {#if chat.showThreadPlaceholderLoading}
            <div
              class="flex min-h-[12rem] flex-col items-center justify-center px-4 py-12"
              role="status"
              aria-live="polite"
            >
              <p
                class="text-[11px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
              >
                Loading conversation
              </p>
            </div>
          {:else if !chat.activeConversation || chat.currentMessages.length === 0}
            {#if chat.showConversationMeta}
              <div class="mb-4">
                <ChatConversationMeta
                  statuses={chat.stream.streamStatuses}
                  streamTimings={chat.stream.streamTimings}
                  toolCalls={chat.stream.streamToolCalls}
                  errorMessage={chat.errorMessage}
                />
              </div>
            {/if}
            <ChatMessageList
              messages={chat.currentMessages}
              loadingMessageId={chat.stream.streamingMessageId}
              empty={true}
            />
          {:else if chat.conversationMetaInsertIndex >= 0}
            <div class="flex flex-col gap-8">
              {#each chat.messagesBeforeConversationMeta as message (message.id)}
                <ChatMessageBubble
                  {message}
                  loading={chat.stream.streamingMessageId === message.id}
                />
              {/each}

              <ChatConversationMeta
                statuses={chat.stream.streamStatuses}
                streamTimings={chat.stream.streamTimings}
                toolCalls={chat.stream.streamToolCalls}
                errorMessage={chat.errorMessage}
              />

              {#each chat.messagesAfterConversationMeta as message (message.id)}
                <ChatMessageBubble
                  {message}
                  loading={chat.stream.streamingMessageId === message.id}
                />
              {/each}
            </div>
          {:else}
            <ChatMessageList
              messages={chat.currentMessages}
              loadingMessageId={chat.stream.streamingMessageId}
              empty={false}
            />
            {#if chat.showConversationMeta}
              <div class="mt-4">
                <ChatConversationMeta
                  statuses={chat.stream.streamStatuses}
                  streamTimings={chat.stream.streamTimings}
                  toolCalls={chat.stream.streamToolCalls}
                  errorMessage={chat.errorMessage}
                />
              </div>
            {/if}
          {/if}
        </div>

        {#if chat.stream.showJumpToLatest}
          <button
            type="button"
            class="absolute bottom-4 left-1/2 z-10 inline-flex h-9 -translate-x-1/2 items-center gap-2 rounded-full border border-[var(--accent-border-soft)] bg-[var(--surface-strong)] px-4 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)] shadow-sm transition-colors hover:bg-[var(--accent-wash)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 motion-reduce:transition-none"
            onclick={() => void chat.stream.jumpToLatest()}
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
        class="border-t border-[var(--border-soft)] px-4 py-4 sm:px-6 lg:px-8 lg:py-6"
      >
        <div class="w-full">
          {#if chat.showStarterSuggestions}
            <ChatSuggestions
              suggestions={CHAT_STARTER_PROMPTS}
              disabled={Boolean(chat.stream.streamingConversationId) ||
                chat.loadingConversation}
              onPick={chat.pickStarterPrompt}
            />
          {/if}
          <ChatInput
            value={chat.draft}
            deepResearch={chat.deepResearch}
            selectedModelId={chat.selectedChatModelId}
            modelOptions={chat.chatClientConfig?.models ?? []}
            focusSignal={chat.chatInputFocusSignal}
            disabled={chat.loadingConversation ||
              chat.creatingConversation ||
              (!chat.isAuthenticated && Boolean(chat.anonymousQuotaMessage))}
            busy={Boolean(chat.stream.streamingConversationId) ||
              chat.creatingConversation}
            canCancel={Boolean(chat.stream.streamingConversationId)}
            onValueChange={chat.setDraft}
            onDeepResearchChange={chat.setDeepResearch}
            onSelectedModelIdChange={chat.setSelectedChatModelId}
            onSubmit={(value) => void chat.handleSend(value)}
            onCancel={() => void chat.handleCancel()}
          />
          {#if chat.anonymousQuotaMessage && !chat.isAuthenticated}
            <ChatAnonymousQuotaNotice />
          {/if}
          <p class="mt-3 text-center text-[10px] text-[var(--soft-foreground)]">
            Synthesizing information across your indexed library.
          </p>
        </div>
      </div>
    </section>
  </div>

  <ConfirmationModal
    show={chat.showDeleteConfirmation}
    title={chat.deleteConfirmationTitle}
    message={chat.deleteConfirmationMessage}
    confirmLabel={chat.deleteConfirmationConfirmLabel}
    cancelLabel={chat.deleteConfirmationCancelLabel}
    tone="danger"
    onConfirm={() => void chat.confirmDeleteConversation()}
    onCancel={chat.cancelDeleteConversation}
  />
</WorkspaceShell>
