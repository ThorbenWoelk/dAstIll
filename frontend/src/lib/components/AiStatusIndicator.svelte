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

  const toneClass = $derived(
    dotClass.includes("bg-emerald")
      ? "border-emerald-500/20 bg-emerald-500/8 text-emerald-700 dark:text-emerald-300"
      : dotClass.includes("bg-amber")
        ? "border-amber-500/20 bg-amber-500/8 text-amber-700 dark:text-amber-300"
        : "border-[var(--accent-border-soft)] bg-[var(--accent-wash)] text-[var(--accent-strong)]",
  );

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

<div class="relative">
  <button
    bind:this={button}
    type="button"
    id="ai-status-pill"
    class={`inline-flex h-8 items-center justify-center gap-2 rounded-full border px-2.5 text-[10px] font-bold uppercase tracking-[0.12em] transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${open ? "border-[var(--accent)]/25 bg-[var(--accent-soft)]/70 text-[var(--accent-strong)]" : "border-[var(--accent-border-soft)] bg-[var(--panel-surface)] text-[var(--soft-foreground)] hover:border-[var(--accent)]/35 hover:text-[var(--foreground)]"}`}
    aria-expanded={open}
    aria-haspopup="dialog"
    aria-label={`AI engine status: ${title}`}
    onclick={toggle}
  >
    <span class={`h-2 w-2 rounded-full ${dotClass}`}></span>
    <span class="hidden sm:inline">AI</span>
  </button>

  {#if open}
    <div
      bind:this={panel}
      role="dialog"
      aria-label="AI engine status details"
      class="fixed left-4 right-4 top-[calc(env(safe-area-inset-top)+4.5rem)] z-[9999] overflow-hidden rounded-[var(--radius-lg)] border border-[var(--accent-border-soft)] bg-[var(--surface-frost-strong)] shadow-2xl backdrop-blur sm:left-auto sm:right-4 sm:top-[calc(env(safe-area-inset-top)+4rem)] sm:w-[22rem]"
    >
      <div
        class="flex items-start justify-between gap-3 border-b border-[var(--accent-border-soft)] px-4 py-3"
      >
        <div class="min-w-0">
          <p
            class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-60"
          >
            AI status
          </p>
          <div class="mt-2 flex min-w-0 items-center gap-2">
            <span class={`h-2.5 w-2.5 rounded-full ${dotClass}`}></span>
            <p
              class="truncate text-[13px] font-semibold text-[var(--foreground)]"
            >
              {title}
            </p>
          </div>
        </div>
        <button
          type="button"
          class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-55 transition-colors hover:bg-[var(--muted)] hover:text-[var(--foreground)]"
          aria-label="Close"
          onclick={close}
        >
          <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.4"
            stroke-linecap="round"
          >
            <path d="M6 6L18 18"></path>
            <path d="M18 6L6 18"></path>
          </svg>
        </button>
      </div>

      <div class="space-y-3 px-4 py-4">
        <span
          class={`inline-flex items-center rounded-full border px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.12em] ${toneClass}`}
        >
          {title}
        </span>
        <p class="text-[13px] leading-6 text-[var(--soft-foreground)]">
          {detail}
        </p>
      </div>
    </div>
  {/if}
</div>
