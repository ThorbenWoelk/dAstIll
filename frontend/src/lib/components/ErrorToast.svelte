<script lang="ts">
  import CloseIcon from "$lib/components/icons/CloseIcon.svelte";

  let {
    message,
    onDismiss = () => {},
  }: {
    message: string;
    onDismiss?: () => void;
  } = $props();
</script>

<div
  class="mobile-bottom-stack-offset fixed bottom-6 left-1/2 z-[80] w-[min(90vw,420px)] -translate-x-1/2 rounded-[var(--radius-md)] border border-[var(--danger-border)] bg-[var(--surface)] px-4 py-3 shadow-lg fade-in"
  role="status"
  aria-live="polite"
>
  <div class="flex items-start gap-3">
    <p class="flex-1 text-[13px] font-medium text-[var(--danger-foreground)]">
      {message}
    </p>
    <button
      type="button"
      onclick={onDismiss}
      class="shrink-0 text-[var(--soft-foreground)] opacity-40 hover:opacity-80"
      aria-label="Dismiss"
    >
      <CloseIcon />
    </button>
  </div>
</div>
