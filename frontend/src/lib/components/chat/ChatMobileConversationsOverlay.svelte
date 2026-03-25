<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    open,
    onClose,
    children,
  }: {
    open: boolean;
    onClose: () => void;
    children: Snippet;
  } = $props();
</script>

{#if open}
  <div
    class="fixed inset-0 z-[80] lg:hidden"
    role="dialog"
    aria-modal="true"
    aria-label="Conversations"
  >
    <button
      type="button"
      class="absolute inset-0 bg-[var(--overlay)]"
      onclick={onClose}
      aria-label="Close conversations"
    ></button>
    <div
      class="relative z-10 h-full w-[min(85vw,20rem)] overflow-hidden border-r border-[var(--accent-border-soft)] bg-[var(--surface-strong)] shadow-2xl"
    >
      {@render children()}
    </div>
  </div>
{/if}
