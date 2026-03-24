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
    class="flex min-h-[18rem] flex-col items-center justify-center px-4 py-12 text-center"
  >
    <div class="max-w-md space-y-3">
      <p
        class="text-[12px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)]"
      >
        RAG chat
      </p>
      <h2
        class="font-serif text-xl font-semibold tracking-[-0.02em] text-[var(--foreground)]"
        style="font-variation-settings: 'opsz' 72;"
      >
        Ask questions grounded in your indexed library
      </h2>
      <p class="text-[13px] leading-relaxed text-[var(--soft-foreground)]">
        Answers stream from the local model and cite transcript and summary
        sources they rely on.
      </p>
    </div>
  </div>
{:else}
  <div class="flex flex-col gap-8">
    {#each messages as message (message.id)}
      <ChatMessage {message} loading={loadingMessageId === message.id} />
    {/each}
  </div>
{/if}
