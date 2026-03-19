<script lang="ts">
  import ChatMessage from "$lib/components/chat/ChatMessage.svelte";
  import type { ChatMessage as ChatMessagePayload } from "$lib/types";

  let {
    messages,
    loadingMessageId = null,
    empty = false,
  }: {
    messages: ChatMessagePayload[];
    loadingMessageId?: string | null;
    empty?: boolean;
  } = $props();
</script>

{#if empty}
  <div
    class="flex min-h-[20rem] items-center justify-center rounded-[var(--radius-lg)] border border-dashed border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-6 py-10 text-center"
  >
    <div class="max-w-md space-y-2">
      <p
        class="text-[12px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)]"
      >
        RAG chat
      </p>
      <h2 class="text-lg font-semibold text-[var(--foreground)]">
        Ask questions grounded in your indexed library
      </h2>
      <p class="text-[13px] leading-relaxed text-[var(--soft-foreground)]">
        Answers stream from Ollama and include the transcript and summary
        sources they rely on.
      </p>
    </div>
  </div>
{:else}
  <div class="space-y-4">
    {#each messages as message (message.id)}
      <ChatMessage {message} loading={loadingMessageId === message.id} />
    {/each}
  </div>
{/if}
