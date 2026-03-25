<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    open,
    sidebar,
    onClose,
  }: {
    open: boolean;
    sidebar?: Snippet<
      [
        {
          collapsed: boolean;
          toggle: () => void;
          width: number;
          mobileVisible?: boolean;
        },
      ]
    >;
    onClose: () => void;
  } = $props();
</script>

{#if open}
  <div
    class="fixed inset-0 z-[80] lg:hidden"
    role="dialog"
    aria-modal="true"
    aria-label="Browse channels"
  >
    <button
      type="button"
      class="absolute inset-0 bg-[var(--overlay)]"
      onclick={onClose}
      aria-label="Close sidebar"
    ></button>
    <div
      class="relative z-10 h-full w-[min(85vw,20rem)] overflow-hidden border-r border-[var(--accent-border-soft)] bg-[var(--surface-strong)] shadow-2xl"
    >
      {#if sidebar}
        {@render sidebar({
          collapsed: false,
          width: 0,
          toggle: onClose,
          mobileVisible: true,
        })}
      {/if}
    </div>
  </div>
{/if}
