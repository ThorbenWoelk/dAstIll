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

<div
  class="flex items-center justify-between gap-3 px-4 py-3 sm:px-6 lg:px-0 lg:py-2"
>
  <div class="flex min-w-0 items-center gap-2">
    <button
      type="button"
      class="inline-flex h-8 items-center justify-center gap-2 rounded-md px-2 text-[12px] font-semibold text-[var(--soft-foreground)] transition-colors hover:bg-[var(--muted)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 lg:hidden"
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
    </button>
    <p
      class="truncate text-[13px] font-semibold tracking-tight text-[var(--foreground)] lg:text-[14px]"
    >
      {conversationTitle}
    </p>
    {#if titleStatus === "generating"}
      <span
        class="inline-flex items-center gap-1 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--accent-strong)]"
      >
        <span class="h-1.5 w-1.5 animate-pulse rounded-full bg-[var(--accent)]"
        ></span>
        naming
      </span>
    {/if}
  </div>
  {#if streamingConversationId}
    <span
      class="h-3 w-3 shrink-0 animate-spin rounded-full border-2 border-[var(--border)] border-t-[var(--accent)]"
      role="status"
      aria-label="Generating response"
    ></span>
  {/if}
</div>
