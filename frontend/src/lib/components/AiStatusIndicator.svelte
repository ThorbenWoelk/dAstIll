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
</div>

{#if open}
  <div
    class="fixed inset-0 z-[9999] flex items-center justify-center p-4 sm:p-6"
    role="dialog"
    aria-modal="true"
    aria-label="AI engine status"
  >
    <button
      type="button"
      class="absolute inset-0 bg-[var(--overlay)]"
      aria-label="Close AI engine status"
      onclick={close}
    ></button>

    <div
      use:clickOutside={{ enabled: open, onClickOutside: close }}
      class="relative w-full max-w-sm overflow-hidden rounded-[var(--radius-lg)] border border-[var(--accent-border-soft)] bg-[var(--surface-strong)] shadow-2xl"
    >
      <div class="flex items-start justify-between gap-3 px-4 py-4 sm:px-5">
        <div class="min-w-0 space-y-2">
          <div class="flex min-w-0 items-center gap-2">
            <span class={`h-2.5 w-2.5 rounded-full ${dotClass}`}></span>
            <p
              class="truncate text-[13px] font-semibold text-[var(--foreground)]"
            >
              {title}
            </p>
          </div>
          <p class="text-[13px] leading-6 text-[var(--soft-foreground)]">
            {detail}
          </p>
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
    </div>
  </div>
{/if}
