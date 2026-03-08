<script lang="ts">
  import { onMount } from "svelte";

  type Props = {
    detail: string;
    dotClass: string;
    title: string;
  };

  let { detail, dotClass, title }: Props = $props();

  let open = $state(false);
  let button = $state<HTMLButtonElement | null>(null);
  let panel = $state<HTMLDivElement | null>(null);

  function toggle() {
    open = !open;
  }

  function close() {
    open = false;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") close();
  }

  onMount(() => {
    const handlePointerDown = (event: PointerEvent) => {
      if (!open) return;
      const target = event.target as Node;
      if (button?.contains(target) || panel?.contains(target)) return;
      close();
    };
    document.addEventListener("pointerdown", handlePointerDown);
    return () => document.removeEventListener("pointerdown", handlePointerDown);
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<button
  bind:this={button}
  type="button"
  class="inline-flex h-7 w-7 items-center justify-center rounded-full transition-colors hover:bg-[var(--muted)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
  aria-expanded={open}
  aria-haspopup="dialog"
  aria-label={`AI engine status: ${title}`}
  onclick={toggle}
>
  <span class={`h-2.5 w-2.5 rounded-full ${dotClass}`}></span>
</button>

{#if open}
  <div
    bind:this={panel}
    role="dialog"
    aria-label="AI engine status details"
    class="fixed left-0 right-0 top-0 z-[9999] border-b border-[var(--border)] bg-[var(--surface)] p-3 shadow-lg"
  >
    <div class="mx-auto flex max-w-[1440px] items-start justify-between gap-3 px-4">
      <div>
        <div class="mb-1 flex items-center gap-2">
          <span class={`h-2.5 w-2.5 rounded-full ${dotClass}`}></span>
          <p class="text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--foreground)]">
            {title}
          </p>
        </div>
        <p class="text-[13px] leading-6 text-[var(--soft-foreground)]">
          {detail}
        </p>
      </div>
      <button
        type="button"
        class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--muted)] hover:text-[var(--foreground)]"
        aria-label="Close"
        onclick={close}
      >
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
          stroke-linecap="round"
        >
          <path d="M6 6L18 18"></path>
          <path d="M18 6L6 18"></path>
        </svg>
      </button>
    </div>
  </div>
{/if}
