<script lang="ts">
  import ChatSourceChip from "$lib/components/chat/ChatSourceChip.svelte";
  import type { ChatMessage } from "$lib/types";
  import { renderMarkdown } from "$lib/utils/markdown";

  let {
    message,
    loading = false,
  }: {
    message: ChatMessage;
    loading?: boolean;
  } = $props();

  let isAssistant = $derived(message.role === "assistant");
  let contentHtml = $derived(
    isAssistant && message.content ? renderMarkdown(message.content) : "",
  );
</script>

<article class={`flex ${isAssistant ? "justify-start" : "justify-end"}`}>
  <div
    class={`max-w-[min(100%,48rem)] space-y-3 ${isAssistant ? "w-full" : "max-w-[36rem]"}`}
  >
    <div
      class={`rounded-[var(--radius-lg)] border px-4 py-3 shadow-sm ${isAssistant ? "border-[var(--accent-border-soft)] bg-[var(--panel-surface)] text-[var(--foreground)]" : "border-[var(--accent)]/15 bg-[var(--accent-wash-strong)] text-[var(--foreground)]"}`}
    >
      {#if isAssistant}
        {#if message.content}
          <div
            class="prose prose-sm max-w-none text-[var(--foreground)] prose-headings:text-[var(--foreground)] prose-p:text-[var(--foreground)] prose-strong:text-[var(--foreground)] prose-li:text-[var(--foreground)]"
          >
            {@html contentHtml}
          </div>
        {:else if loading}
          <div
            class="flex items-center gap-2 text-[12px] text-[var(--soft-foreground)]"
          >
            <span
              class="h-2.5 w-2.5 animate-pulse rounded-full bg-[var(--accent)]"
            ></span>
            <span>Working through the retrieval plan…</span>
          </div>
        {/if}
      {:else}
        <p class="whitespace-pre-wrap text-[13px] leading-relaxed">
          {message.content}
        </p>
      {/if}
    </div>

    {#if isAssistant && message.sources.length > 0}
      <div class="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
        {#each message.sources as source, index (`${source.video_id}-${source.source_kind}-${source.section_title ?? ""}-${index}`)}
          <ChatSourceChip {source} />
        {/each}
      </div>
    {/if}
  </div>
</article>
