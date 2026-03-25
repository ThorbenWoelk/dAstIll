<script lang="ts">
  import type { ChatTitleStatus } from "$lib/types";

  let {
    onOpenConversationsMobile,
    streamingConversationId,
    conversationTitle,
    titleStatus,
  }: {
    onOpenConversationsMobile: () => void;
    streamingConversationId: string | null;
    conversationTitle: string;
    titleStatus: ChatTitleStatus | undefined;
  } = $props();
</script>

<div class="flex flex-col gap-3 px-4 max-lg:pb-1 max-lg:pt-3 sm:px-6 lg:px-0">
  <div class="flex items-center justify-between gap-3">
    <div class="flex items-center gap-2">
      <button
        type="button"
        class="inline-flex h-8 items-center justify-center gap-2 rounded-full px-3 text-[12px] font-semibold text-[var(--soft-foreground)] transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 lg:hidden"
        onclick={onOpenConversationsMobile}
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
      <h2 class="text-base font-bold tracking-tight text-[var(--foreground)]">
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
        {conversationTitle}
      </p>
      <p
        class="mt-2 max-w-[34rem] text-[14px] leading-6 text-[var(--soft-foreground)]"
      >
        {titleStatus === "generating"
          ? "AI is naming this chat while the conversation stays available in the background."
          : "Ask questions grounded in indexed transcripts and summaries, with source-backed answers streamed into this pane."}
      </p>
    </div>
  </div>
</div>
