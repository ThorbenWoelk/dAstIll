<script lang="ts">
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import type { ChatMessage } from "$lib/types";
  import {
    hasCitationMarkers,
    linkifyCitationMarkers,
    sourceTooltipTitle,
    sourceWorkspaceHref,
  } from "$lib/utils/chat-citations";
  import { renderMarkdownForChat } from "$lib/utils/markdown";
  import { formatAssistantResponseStats } from "$lib/utils/chat-response-stats";

  let {
    message,
    loading = false,
  }: {
    message: ChatMessage;
    loading?: boolean;
  } = $props();

  let isAssistant = $derived(message.role === "assistant");
  let citedInText = $derived(
    isAssistant ? hasCitationMarkers(message.content) : false,
  );
  let contentHtml = $derived(
    isAssistant && message.content
      ? linkifyCitationMarkers(
          renderMarkdownForChat(message.content),
          message.sources,
        )
      : "",
  );
  let copyState = $state<"idle" | "copied" | "error">("idle");
  let copyReset = $state<ReturnType<typeof setTimeout> | null>(null);

  let canCopy = $derived(
    isAssistant && Boolean(message.content.trim()) && !loading,
  );

  let showCompactSources = $derived(
    isAssistant && message.sources.length > 0 && !citedInText && !loading,
  );

  let responseStatsLine = $derived(
    formatAssistantResponseStats(message, { loading }),
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
  class={`flex w-full max-w-[95%] ${isAssistant ? "justify-start" : "ml-auto justify-end"}`}
>
  <div
    class={`max-w-[min(100%,48rem)] space-y-3 ${isAssistant ? "w-full min-w-0" : "max-w-[36rem]"}`}
  >
    <div
      class={`relative rounded-[var(--radius-lg)] px-4 py-3 shadow-sm group/copy ${isAssistant ? "bg-[var(--panel-surface)] text-[var(--foreground)]" : "bg-[var(--accent-wash-strong)] text-[var(--foreground)]"}`}
    >
      {#if isAssistant && canCopy}
        <div
          class="pointer-events-none absolute right-2 top-2 z-10 opacity-0 transition-opacity duration-200 group-hover/copy:pointer-events-auto group-hover/copy:opacity-100 group-focus-within/copy:pointer-events-auto group-focus-within/copy:opacity-100 motion-reduce:pointer-events-auto motion-reduce:opacity-100"
        >
          <button
            type="button"
            class="pointer-events-auto inline-flex h-8 w-8 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--panel-surface)]"
            data-tooltip={copyState === "copied"
              ? "Copied"
              : copyState === "error"
                ? "Copy failed"
                : "Copy"}
            aria-label={copyState === "copied"
              ? "Copied"
              : copyState === "error"
                ? "Copy failed"
                : "Copy message"}
            onclick={copyContent}
          >
            {#if copyState === "copied"}
              <CheckIcon
                size={16}
                strokeWidth={2}
                className="text-[var(--accent)]"
              />
            {:else if copyState === "error"}
              <svg
                viewBox="0 0 24 24"
                class="h-4 w-4 text-[var(--danger)]"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                aria-hidden="true"
              >
                <path d="M18 6 6 18M6 6l12 12" />
              </svg>
            {:else}
              <svg
                viewBox="0 0 24 24"
                class="h-4 w-4"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
              >
                <path
                  d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"
                />
                <path
                  d="M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1Z"
                />
              </svg>
            {/if}
          </button>
        </div>
      {/if}
      {#if isAssistant}
        {#if message.content}
          <div
            class={`prose prose-sm max-w-none text-[var(--foreground)] prose-headings:text-[var(--foreground)] prose-p:text-[var(--foreground)] prose-strong:text-[var(--foreground)] prose-li:text-[var(--foreground)] ${canCopy ? "pr-8" : ""}`}
          >
            {@html contentHtml}
          </div>
        {:else if loading}
          <div
            class="flex items-center gap-2 text-[12px] text-[var(--soft-foreground)]"
          >
            <span class="h-2 w-2 animate-pulse rounded-full bg-[var(--accent)]"
            ></span>
            <span>Working through the retrieval plan…</span>
          </div>
        {/if}
      {:else}
        <p class="whitespace-pre-wrap text-[14px] leading-relaxed">
          {message.content}
        </p>
      {/if}
    </div>

    {#if responseStatsLine}
      <p
        class="pl-0.5 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
      >
        {responseStatsLine}
      </p>
    {/if}

    {#if showCompactSources}
      <div
        class="flex flex-wrap items-start gap-x-3 gap-y-1 pl-0 text-[11px] leading-relaxed text-[var(--soft-foreground)]"
      >
        <span
          class="text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
          >Sources</span
        >
        {#each message.sources as source, index (`${source.video_id}-${source.source_kind}-${source.section_title ?? ""}-${index}`)}
          <a
            href={sourceWorkspaceHref(source)}
            data-tooltip={sourceTooltipTitle(source)}
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex max-w-[14rem] items-baseline gap-1 truncate text-[var(--accent)] font-medium no-underline hover:underline"
          >
            <sup class="chat-cite-sup shrink-0">[{index + 1}]</sup>
            <span class="truncate">{source.video_title}</span>
          </a>
        {/each}
      </div>
    {/if}
  </div>
</article>
