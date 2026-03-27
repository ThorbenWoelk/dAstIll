<script lang="ts">
  import { clickOutside } from "$lib/actions/click-outside";
  import { fade, scale } from "svelte/transition";

  let {
    show = false,
    source = "",
    value = "",
    busy = false,
    onValueChange = (_value: string) => {},
    onConfirm = () => {},
    onCancel = () => {},
  }: {
    show?: boolean;
    source?: string;
    value?: string;
    busy?: boolean;
    onValueChange?: (value: string) => void;
    onConfirm?: () => void;
    onCancel?: () => void;
  } = $props();

  let inputElement = $state<HTMLInputElement | null>(null);

  function handleKeydown(event: KeyboardEvent) {
    if (!show) return;
    if (event.key === "Escape" && !busy) {
      onCancel();
      return;
    }
    if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
      event.preventDefault();
      onConfirm();
    }
  }

  function handleInput(event: Event) {
    onValueChange((event.currentTarget as HTMLInputElement).value);
  }

  $effect(() => {
    if (!show || !inputElement) {
      return;
    }
    requestAnimationFrame(() => {
      inputElement?.focus();
      inputElement?.select();
    });
  });
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show}
  <div
    class="fixed inset-0 z-[100] flex items-center justify-center p-4 sm:p-6"
    role="dialog"
    aria-modal="true"
    aria-labelledby="vocabulary-modal-title"
  >
    <div
      class="absolute inset-0 bg-[var(--overlay)]"
      transition:fade={{ duration: 200 }}
    ></div>

    <div
      use:clickOutside={{ enabled: show && !busy, onClickOutside: onCancel }}
      class="relative w-full max-w-md rounded-[var(--radius-lg)] bg-[var(--surface)] shadow-2xl transition-all"
      transition:scale={{ duration: 200, start: 0.95, opacity: 0 }}
    >
      <div class="space-y-6 p-6 sm:p-8">
        <div class="space-y-3">
          <div
            class="inline-flex rounded-full bg-[var(--accent-wash)] px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--accent-strong)]"
          >
            Vocabulary
          </div>
          <div class="space-y-2">
            <h3
              id="vocabulary-modal-title"
              class="text-xl font-bold tracking-tight text-[var(--foreground)]"
            >
              Correct spelling
            </h3>
            <p
              class="text-[14px] leading-relaxed text-[var(--soft-foreground)] opacity-75"
            >
              Future summaries will replace this phrase with the canonical
              spelling you save here.
            </p>
          </div>
        </div>

        <div class="space-y-4">
          <div class="space-y-1.5">
            <label
              class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-60"
              for="vocabulary-source"
            >
              Selected text
            </label>
            <div
              id="vocabulary-source"
              class="rounded-[var(--radius-md)] bg-[var(--panel-surface)] px-4 py-3 text-[15px] font-medium text-[var(--foreground)]"
            >
              {source}
            </div>
          </div>

          <div class="space-y-1.5">
            <label
              class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-60"
              for="vocabulary-replacement"
            >
              Use this spelling
            </label>
            <input
              bind:this={inputElement}
              id="vocabulary-replacement"
              type="text"
              class="w-full rounded-[var(--radius-md)] bg-[var(--panel-surface)] px-4 py-3 text-[15px] font-medium text-[var(--foreground)] placeholder:text-[var(--soft-foreground)]/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
              {value}
              oninput={handleInput}
              placeholder="Canonical spelling"
              disabled={busy}
            />
          </div>
        </div>

        <div class="flex flex-col gap-2 sm:flex-row-reverse sm:gap-3">
          <button
            type="button"
            class="inline-flex w-full items-center justify-center rounded-[var(--radius-md)] bg-[var(--foreground)] px-6 py-3 text-[12px] font-bold uppercase tracking-[0.1em] text-[var(--background)] transition-all hover:bg-[var(--accent-strong)] disabled:opacity-40 sm:w-auto"
            onclick={onConfirm}
            disabled={busy || !value.trim() || value.trim() === source.trim()}
          >
            {busy ? "Saving..." : "Save"}
          </button>
          <button
            type="button"
            class="inline-flex w-full items-center justify-center rounded-[var(--radius-md)] bg-[var(--panel-surface)] px-6 py-3 text-[12px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] disabled:opacity-40 sm:w-auto"
            onclick={onCancel}
            disabled={busy}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
