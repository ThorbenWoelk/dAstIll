<script lang="ts">
  import { fade, slide } from "svelte/transition";
  import type { ChatStreamTiming } from "$lib/chat/conversation-meta";
  import type { ChatStreamStatus, ChatToolCall } from "$lib/types";

  type StatusWithTime = ChatStreamStatus & { receivedAt: number };

  let {
    statuses = [],
    streamTimings = [],
    toolCalls = [],
    errorMessage = null,
  }: {
    statuses: StatusWithTime[];
    streamTimings: ChatStreamTiming[];
    toolCalls: ChatToolCall[];
    errorMessage: string | null;
  } = $props();

  /**
   * Filter and group statuses into meaningful "steps" for the decision tree.
   * We want to show:
   * 1. Stage transitions (when label or stage changes)
   * 2. Decisions made (rationale)
   * 3. Tool calls / Actions
   */
  let steps = $derived.by(() => {
    const result: {
      id: string;
      label: string;
      detail?: string | null;
      decision?: string | null;
      type: "info" | "action" | "decision";
      active: boolean;
      timestamp: number;
    }[] = [];

    let lastLabel = "";
    let lastStage = "";

    statuses.forEach((status, index) => {
      const isLast = index === statuses.length - 1;
      const label = status.label || status.stage;

      // New stage or label
      if (
        label !== lastLabel ||
        status.stage !== lastStage ||
        status.decision
      ) {
        result.push({
          id: `${status.stage}-${index}`,
          label: status.label || status.stage.replace(/_/g, " "),
          detail: status.detail,
          decision: status.decision,
          type: status.decision ? "decision" : status.tool ? "action" : "info",
          active: isLast,
          timestamp: status.receivedAt,
        });
        lastLabel = label;
        lastStage = status.stage;
      }
    });

    return result;
  });
</script>

<div class="relative space-y-6 py-2 pl-4">
  <!-- Vertical Timeline Rail -->
  <div
    class="absolute bottom-4 left-[5px] top-4 w-[1px] bg-[var(--accent-border-soft)] opacity-50"
  ></div>

  {#if steps.length > 0}
    <div class="space-y-6">
      {#each steps as step, i (step.id)}
        <div
          class="relative pl-6"
          in:slide={{ duration: 300 }}
          out:fade={{ duration: 200 }}
        >
          <!-- Node Dot -->
          <div
            class={`absolute left-[-2px] top-1.5 h-2.5 w-2.5 rounded-full border-2 border-[var(--background)] transition-all duration-500 ${
              step.active
                ? "bg-[var(--accent)] ring-4 ring-[var(--accent-wash)]"
                : "bg-[var(--soft-foreground)] opacity-40"
            }`}
          >
            {#if step.active}
              <div
                class="absolute inset-0 animate-ping rounded-full bg-[var(--accent)] opacity-40"
              ></div>
            {/if}
          </div>

          <!-- Step Content -->
          <div
            class={`space-y-1 ${step.active ? "opacity-100" : "opacity-70"}`}
          >
            <div class="flex items-center gap-2">
              <p
                class={`text-[11px] font-bold uppercase tracking-[0.1em] ${
                  step.active
                    ? "text-[var(--accent-strong)]"
                    : "text-[var(--soft-foreground)]"
                }`}
              >
                {step.label}
              </p>
              {#if !step.active && i < steps.length - 1}
                <span
                  class="text-[10px] text-[var(--soft-foreground)] opacity-50"
                  >•</span
                >
              {/if}
            </div>

            {#if step.decision}
              <div
                class="mt-1.5 rounded-[var(--radius-sm)] bg-[var(--accent-wash)]/30 px-3 py-2 border-l-2 border-[var(--accent-soft)]"
              >
                <p
                  class="text-[10px] font-bold uppercase tracking-[0.05em] text-[var(--accent-strong)] opacity-70"
                >
                  Decision
                </p>
                <p
                  class="mt-0.5 text-[12px] leading-relaxed text-[var(--foreground)]"
                >
                  {step.decision}
                </p>
              </div>
            {:else if step.detail}
              <p
                class="text-[12px] leading-relaxed text-[var(--soft-foreground)]"
              >
                {step.detail}
              </p>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if toolCalls.length > 0}
    <div class="mt-4 space-y-2 pl-6">
      <p
        class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-70"
      >
        Tools invoked
      </p>
      {#each toolCalls as tool (`${tool.name}:${tool.input}`)}
        <div
          class="flex items-center gap-2.5 rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--surface)] px-3 py-2 shadow-sm transition-all hover:border-[var(--accent-border-soft)]"
        >
          <svg
            viewBox="0 0 24 24"
            class="h-3.5 w-3.5 shrink-0 text-[var(--accent)]"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path
              d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"
            />
          </svg>
          <p class="min-w-0 truncate text-[12px] text-[var(--soft-foreground)]">
            <span class="font-semibold text-[var(--foreground)]"
              >{tool.label}</span
            >
            <span class="opacity-50 mx-1">—</span>
            {tool.input}
          </p>
        </div>
      {/each}
    </div>
  {/if}

  {#if streamTimings.length > 0}
    <div
      class="mt-6 border-t border-[var(--accent-border-soft)] opacity-40 pt-4 pl-6"
    >
      <div class="flex flex-wrap gap-x-6 gap-y-2">
        {#each streamTimings as timing}
          <div class="flex flex-col gap-0.5">
            <span
              class="text-[9px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)]"
            >
              {timing.label}
            </span>
            <span class="text-[11px] font-medium text-[var(--foreground)]">
              {(timing.durationMs / 1000).toFixed(2)}s
            </span>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  {#if errorMessage}
    <div
      class="mt-4 rounded-[var(--radius-md)] border border-[var(--danger)]/20 bg-[var(--danger)]/5 px-3 py-2 text-[12px] text-[var(--danger)] pl-6"
    >
      <p class="font-bold uppercase tracking-wider text-[9px]">Error</p>
      <p class="mt-0.5">{errorMessage}</p>
    </div>
  {/if}
</div>
