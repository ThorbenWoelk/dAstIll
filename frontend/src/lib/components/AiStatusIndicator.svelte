<script lang="ts">
  import { clickOutside } from "$lib/actions/click-outside";
  import CloseIcon from "$lib/components/icons/CloseIcon.svelte";

  type Props = {
    detail: string;
    dotClass: string;
    title: string;
  };

  let { detail, dotClass, title }: Props = $props();

  let open = $state(false);
  const toneClass = $derived(
    dotClass.includes("--status-error")
      ? "border-[var(--danger-border)] bg-[var(--danger-soft)] text-[var(--danger-foreground)]"
      : dotClass.includes("--status-warn")
        ? "border-[var(--border)] bg-[var(--muted)] text-[var(--soft-foreground)]"
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
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="relative"
  use:clickOutside={{ enabled: open, onClickOutside: close }}
>
  <button
    type="button"
    id="ai-status-pill"
    class={`inline-flex h-8 w-8 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${open ? "text-[var(--accent-strong)]" : "hover:text-[var(--foreground)]"}`}
    aria-expanded={open}
    aria-haspopup="dialog"
    aria-label={`AI engine status: ${title}`}
    onclick={toggle}
  >
    <span class={`h-3 w-3 rounded-full ${dotClass}`}></span>
  </button>

  {#if open}
    <div
      role="dialog"
      aria-label="AI engine status details"
      class="absolute right-0 top-full z-[9999] mt-2 w-[min(22rem,calc(100vw-2rem))] overflow-hidden rounded-[var(--radius-lg)] border border-[var(--accent-border-soft)] bg-[var(--surface-frost-strong)] shadow-2xl backdrop-blur"
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
          <CloseIcon size={12} />
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
