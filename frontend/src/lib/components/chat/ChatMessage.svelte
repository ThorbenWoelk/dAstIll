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
  let copyState = $state<"idle" | "copied" | "error">("idle");
  let copyReset = $state<ReturnType<typeof setTimeout> | null>(null);

  let canCopy = $derived(
    isAssistant && Boolean(message.content.trim()) && !loading,
  );

  async function copyContent() {
    if (!canCopy) return;
    try {
      await navigator.clipboard.writeText(message.content);
      copyState = "copied";
      if (copyReset) clearTimeout(copyReset);
      copyReset = setTimeout(() => {
        copyState = "idle";
        copyReset = null;
      }, 2000);
    } catch {
      copyState = "error";
      if (copyReset) clearTimeout(copyReset);
      copyReset = setTimeout(() => {
        copyState = "idle";
        copyReset = null;
      }, 2000);
    }
  }
</script>

<article
  class={`group flex w-full max-w-[95%] ${isAssistant ? "justify-start" : "ml-auto justify-end"}`}
>
  <div
    class={`max-w-[min(100%,48rem)] space-y-3 ${isAssistant ? "w-full min-w-0" : "max-w-[36rem]"}`}
  >
    <div
      class={`rounded-[var(--radius-lg)] px-4 py-3 shadow-sm ${isAssistant ? "bg-[var(--panel-surface)] text-[var(--foreground)]" : "bg-[var(--accent-wash-strong)] text-[var(--foreground)]"}`}
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

    {#if isAssistant && canCopy}
      <div
        class="flex justify-start pl-0.5 opacity-100 motion-reduce:opacity-100 md:opacity-0 md:transition-opacity md:duration-200 md:group-hover:opacity-100 md:group-focus-within:opacity-100"
      >
        <button
          type="button"
          class="inline-flex h-8 items-center rounded-full px-2.5 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
          onclick={copyContent}
        >
          {#if copyState === "copied"}
            Copied
          {:else if copyState === "error"}
            Copy failed
          {:else}
            Copy
          {/if}
        </button>
      </div>
    {/if}

    {#if isAssistant && message.sources.length > 0}
      <div class="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
        {#each message.sources as source, index (`${source.video_id}-${source.source_kind}-${source.section_title ?? ""}-${index}`)}
          <ChatSourceChip {source} />
        {/each}
      </div>
    {/if}
  </div>
</article>
