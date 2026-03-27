<script lang="ts">
  import type { ChatStreamTiming } from "$lib/chat/conversation-meta";
  import type { ChatToolCall } from "$lib/types";

  let {
    streamBanner,
    streamBannerDetail,
    streamTraceVisible,
    streamPlanLabel,
    streamDisplayedQueries,
    streamCoverageSummary,
    streamPrimaryDecision,
    streamTimings,
    toolCalls,
    errorMessage,
  }: {
    streamBanner: string | null;
    streamBannerDetail: string | null;
    streamTraceVisible: boolean;
    streamPlanLabel: string | null;
    streamDisplayedQueries: string[];
    streamCoverageSummary: string | null;
    streamPrimaryDecision: string | null;
    streamTimings: ChatStreamTiming[];
    toolCalls: ChatToolCall[];
    errorMessage: string | null;
  } = $props();
</script>

<div class="space-y-3">
  {#if streamBanner}
    <div
      class="flex items-start gap-2 rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-3 py-2 text-[12px] text-[var(--soft-foreground)]"
    >
      <span class="h-2 w-2 animate-pulse rounded-full bg-[var(--accent)]"
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

  {#if toolCalls.length > 0}
    <div class="space-y-1.5">
      {#each toolCalls as tool (`${tool.name}:${tool.input}`)}
        <div
          class="flex items-center gap-2.5 rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--surface)] px-3 py-2"
        >
          <svg
            viewBox="0 0 24 24"
            class="h-3.5 w-3.5 shrink-0 text-[var(--soft-foreground)]"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path
              d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"
            />
          </svg>
          <p class="min-w-0 truncate text-[12px] text-[var(--soft-foreground)]">
            <span class="font-semibold text-[var(--foreground)]">Tool call</span
            >
            &ndash; {tool.label}: {tool.input}
          </p>
        </div>
      {/each}
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
            Search strategy
          </p>
          {#if streamPlanLabel}
            <p class="mt-1 text-[12px] font-semibold text-[var(--foreground)]">
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
          <p class="mt-1 text-[11px] leading-relaxed text-[var(--foreground)]">
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
          <p class="mt-1 text-[11px] leading-relaxed text-[var(--foreground)]">
            {streamPrimaryDecision}
          </p>
        </div>
      {/if}

      {#if streamTimings.length > 0}
        <div class="mt-3 border-t border-[var(--accent-border-soft)] pt-3">
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
