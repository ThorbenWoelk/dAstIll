<script lang="ts">
  import CloseIcon from "$lib/components/icons/CloseIcon.svelte";
  import type { AddSourceFeedback } from "$lib/workspace/add-source-feedback";

  let {
    feedback,
    onDismiss = () => {},
    onAction = async () => {},
  }: {
    feedback: AddSourceFeedback;
    onDismiss?: () => void;
    onAction?: () => Promise<void> | void;
  } = $props();

  const loading = $derived(feedback.status === "loading");
</script>

<div
  class="mobile-bottom-stack-offset fixed bottom-6 left-1/2 z-[80] flex w-[min(92vw,430px)] -translate-x-1/2 items-start gap-3 rounded-[var(--radius-md)] bg-[var(--surface-strong)] px-4 py-3 shadow-lg popover-rise"
  role="status"
  aria-live="polite"
>
  <div class="mt-0.5 flex h-2.5 w-2.5 shrink-0 items-center justify-center">
    <span
      class={`h-2.5 w-2.5 rounded-full ${loading ? "animate-pulse bg-[var(--accent)]" : feedback.status === "failed" ? "bg-[var(--danger)]" : "bg-[var(--accent-strong)]"}`}
      aria-hidden="true"
    ></span>
  </div>

  <div class="min-w-0 flex-1">
    <p class="text-[13px] font-semibold text-[var(--foreground)]">
      {feedback.title}
    </p>
    <p class="mt-1 text-[13px] leading-5 text-[var(--soft-foreground)]">
      {feedback.message}
    </p>
    {#if feedback.actionLabel}
      <button
        type="button"
        class="mt-3 inline-flex items-center justify-center rounded-full bg-[var(--accent-wash)] px-3 py-1.5 text-[11px] font-bold uppercase tracking-[0.08em] text-[var(--accent)] transition-colors hover:bg-[var(--accent-soft)]"
        onclick={() => void onAction()}
      >
        {feedback.actionLabel}
      </button>
    {/if}
  </div>

  <button
    type="button"
    class="shrink-0 text-[var(--soft-foreground)] opacity-45 transition-opacity hover:opacity-80"
    aria-label="Dismiss"
    onclick={onDismiss}
  >
    <CloseIcon />
  </button>
</div>
