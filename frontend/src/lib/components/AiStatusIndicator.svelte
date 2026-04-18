<script lang="ts">
  import { clickOutside } from "$lib/actions/click-outside";
  import CloseIcon from "$lib/components/icons/CloseIcon.svelte";

  type Props = {
    detail: string;
    dotClass: string;
    title: string;
    showLabel?: boolean;
  };

  let { detail, dotClass, title, showLabel = false }: Props = $props();

  let open = $state(false);

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
  {#if showLabel}
    <button
      type="button"
      id="ai-status-pill"
      class={`inline-flex h-8 cursor-pointer items-center gap-2 rounded-md px-2 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${open ? "bg-[var(--surface-strong)] text-[var(--foreground)]" : "text-[var(--soft-foreground)] hover:bg-[var(--surface)] hover:text-[var(--foreground)]"}`}
      aria-expanded={open}
      aria-haspopup="dialog"
      aria-label={`AI engine status: ${title}`}
      onclick={toggle}
    >
      <span class={`h-1.5 w-1.5 rounded-full ${dotClass}`}></span>
      <span class="truncate">{title}</span>
    </button>
  {:else}
    <button
      type="button"
      id="ai-status-pill"
      class={`inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-full text-[var(--soft-foreground)] transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${open ? "bg-[var(--accent-wash)] text-[var(--accent-strong)]" : "hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"}`}
      aria-expanded={open}
      aria-haspopup="dialog"
      aria-label={`AI engine status: ${title}`}
      onclick={toggle}
    >
      <span class={`h-3 w-3 rounded-full ${dotClass}`}></span>
    </button>
  {/if}

  {#if open}
    <div
      role="dialog"
      aria-label="AI engine status"
      class="absolute left-0 top-full z-50 mt-2 w-72 max-w-[calc(100vw-2rem)] overflow-hidden rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-strong)] shadow-[var(--shadow-soft)]"
    >
      <div class="flex items-start justify-between gap-3 p-4">
        <div class="min-w-0 space-y-2">
          <div class="flex min-w-0 items-center gap-2">
            <span class={`h-2 w-2 rounded-full ${dotClass}`}></span>
            <p
              class="truncate text-[13px] font-semibold text-[var(--foreground)]"
            >
              {title}
            </p>
          </div>
          <p class="text-[12px] leading-5 text-[var(--soft-foreground)]">
            {detail}
          </p>
        </div>
        <button
          type="button"
          class="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-[var(--soft-foreground)] opacity-55 transition-colors hover:bg-[var(--surface)] hover:text-[var(--foreground)]"
          aria-label="Close"
          onclick={close}
        >
          <CloseIcon size={12} />
        </button>
      </div>
    </div>
  {/if}
</div>
