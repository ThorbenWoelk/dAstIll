<script lang="ts">
  import { clickOutside } from "$lib/actions/click-outside";
  import TrashIcon from "$lib/components/icons/TrashIcon.svelte";
  import { fade, scale } from "svelte/transition";

  let {
    show = false,
    title = "Are you sure?",
    message = "This action cannot be undone.",
    confirmLabel = "Delete",
    cancelLabel = "Cancel",
    onConfirm,
    onCancel,
    tone = "danger",
  }: {
    show?: boolean;
    title?: string;
    message?: string;
    confirmLabel?: string;
    cancelLabel?: string;
    onConfirm: () => void;
    onCancel: () => void;
    tone?: "danger" | "info";
  } = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (!show) return;
    if (event.key === "Escape") {
      onCancel();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show}
  <div
    class="fixed inset-0 z-[100] flex items-center justify-center p-4 sm:p-6"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
  >
    <!-- Backdrop -->
    <div
      class="absolute inset-0 bg-[var(--overlay)]"
      transition:fade={{ duration: 200 }}
    ></div>

    <!-- Modal -->
    <div
      use:clickOutside={{ enabled: show, onClickOutside: onCancel }}
      class="relative w-full max-w-sm overflow-hidden rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--surface)] shadow-2xl transition-all"
      transition:scale={{ duration: 200, start: 0.95, opacity: 0 }}
    >
      <div class="p-6 sm:p-8">
        <div class="flex flex-col items-center text-center">
          {#if tone === "danger"}
            <div
              class="mb-5 flex h-12 w-12 items-center justify-center rounded-full bg-[var(--danger-soft)] text-[var(--danger)]"
            >
              <TrashIcon size={24} strokeWidth={2.5} />
            </div>
          {:else}
            <div
              class="mb-5 flex h-12 w-12 items-center justify-center rounded-full bg-[var(--accent-soft)] text-[var(--accent)]"
            >
              <svg
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="16" x2="12" y2="12"></line>
                <line x1="12" y1="8" x2="12.01" y2="8"></line>
              </svg>
            </div>
          {/if}

          <h3
            id="modal-title"
            class="text-xl font-bold tracking-tight text-[var(--foreground)]"
          >
            {title}
          </h3>
          <p
            class="mt-3 text-[14px] font-medium leading-relaxed text-[var(--soft-foreground)] opacity-70"
          >
            {message}
          </p>
        </div>

        <div class="mt-8 flex flex-col gap-2 sm:flex-row-reverse sm:gap-3">
          <button
            type="button"
            class={`inline-flex w-full items-center justify-center rounded-[var(--radius-md)] px-6 py-3 text-[12px] font-bold uppercase tracking-[0.1em] text-white transition-all sm:w-auto ${tone === "danger" ? "" : "bg-[var(--accent)] hover:bg-[var(--accent-strong)] shadow-lg"}`}
            class:tone-danger={tone === "danger"}
            onclick={onConfirm}
          >
            {confirmLabel}
          </button>
          <button
            type="button"
            class="inline-flex w-full items-center justify-center rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface)] px-6 py-3 text-[12px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] transition-all hover:bg-[var(--muted)] sm:w-auto"
            onclick={onCancel}
          >
            {cancelLabel}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .tone-danger {
    background: var(--danger);
    box-shadow: 0 18px 36px var(--shadow-strong);
  }

  .tone-danger:hover {
    background: var(--danger-foreground);
  }
</style>
