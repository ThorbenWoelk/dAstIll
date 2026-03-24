<script lang="ts">
  import { clickOutside } from "$lib/actions/click-outside";
  import { tick } from "svelte";
  import { fade, scale } from "svelte/transition";

  import {
    buildShortcutManual,
    primaryModifierLabel,
    type ShortcutManualGroup,
  } from "$lib/utils/keyboard-shortcuts";

  let {
    open = false,
    onClose = () => {},
  }: {
    open?: boolean;
    onClose?: () => void;
  } = $props();

  let closeButtonEl = $state<HTMLButtonElement | null>(null);

  let modLabel = $derived(primaryModifierLabel());
  let groups = $derived(buildShortcutManual(modLabel));

  $effect(() => {
    if (!open) {
      return;
    }
    void tick().then(() => {
      closeButtonEl?.focus();
    });
  });

  function handlePanelKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      onClose();
    }
  }
</script>

{#if open}
  <div
    class="fixed inset-0 z-[110] flex items-center justify-center p-4 sm:p-6"
    role="presentation"
  >
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="absolute inset-0 bg-[var(--overlay)]"
      transition:fade={{ duration: 200 }}
      role="presentation"
      onclick={onClose}
    ></div>

    <div
      use:clickOutside={{ enabled: open, onClickOutside: onClose }}
      class="relative flex max-h-[min(32rem,85vh)] w-full max-w-lg flex-col overflow-hidden rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--surface)] shadow-2xl"
      transition:scale={{ duration: 200, start: 0.97, opacity: 0 }}
      role="dialog"
      aria-modal="true"
      aria-labelledby="keyboard-shortcuts-title"
      tabindex="-1"
      onkeydown={handlePanelKeydown}
    >
      <div
        class="flex shrink-0 items-center justify-between gap-4 border-b border-[var(--border-soft)]/60 px-5 py-4"
      >
        <h2
          id="keyboard-shortcuts-title"
          class="font-serif text-[20px] font-semibold tracking-[-0.02em] text-[var(--foreground)]"
        >
          Keyboard shortcuts
        </h2>
        <button
          bind:this={closeButtonEl}
          type="button"
          class="inline-flex h-8 min-w-[4.5rem] items-center justify-center rounded-full px-3 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)]"
          onclick={onClose}
        >
          Close
        </button>
      </div>

      <div
        class="custom-scrollbar min-h-0 flex-1 overflow-y-auto px-5 py-4 text-[13px] leading-relaxed text-[var(--foreground)]"
      >
        {#each groups as group (group.title)}
          <section class="mb-6 last:mb-0">
            <h3
              class="mb-3 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-75"
            >
              {group.title}
            </h3>
            <ul class="space-y-2.5">
              {#each group.rows as row (row.keys + row.description)}
                <li
                  class="flex flex-col gap-1 sm:flex-row sm:items-baseline sm:justify-between sm:gap-6"
                >
                  <span class="text-[var(--soft-foreground)] opacity-90"
                    >{row.description}</span
                  >
                  <kbd
                    class="shrink-0 rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--muted)] px-2 py-1 text-left text-[11px] font-semibold tracking-wide text-[var(--foreground)] sm:text-right"
                    >{row.keys}</kbd
                  >
                </li>
              {/each}
            </ul>
          </section>
        {/each}
      </div>
    </div>
  </div>
{/if}
