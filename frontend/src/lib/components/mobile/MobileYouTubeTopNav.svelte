<script lang="ts">
  import type { Snippet } from "svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import MenuIcon from "$lib/components/icons/MenuIcon.svelte";

  let {
    trailing,
    showBackInsteadOfMenu = false,
    onBack,
  }: {
    trailing?: Snippet;
    /** When true, left control is a back arrow instead of hamburger. */
    showBackInsteadOfMenu?: boolean;
    onBack?: () => void;
  } = $props();

  function openSectionDrawer() {
    window.dispatchEvent(new CustomEvent("dastill:open-section-drawer"));
  }
</script>

<div class="grid w-full grid-cols-[auto_1fr_auto] items-center gap-2">
  <div class="flex justify-start">
    {#if showBackInsteadOfMenu}
      <button
        type="button"
        class="inline-flex h-9 w-9 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-80 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
        aria-label="Back"
        onclick={() => onBack?.()}
      >
        <ChevronIcon direction="left" size={18} strokeWidth={2.2} />
      </button>
    {:else}
      <button
        type="button"
        class="inline-flex h-9 w-9 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-80 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
        aria-label="Open menu"
        onclick={openSectionDrawer}
      >
        <MenuIcon size={18} strokeWidth={2} />
      </button>
    {/if}
  </div>

  <div class="flex min-w-0 justify-center">
    <a
      href="/"
      class="font-serif min-w-0 text-base font-bold tracking-[-0.03em] text-[var(--color-swatch)] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)]"
      data-sveltekit-preload-data="tap"
      data-sveltekit-preload-code="viewport"
      aria-label="Go to dAstIll home"
    >
      d<span style="color:var(--soft-foreground);">A</span>st<span
        style="color:var(--soft-foreground);">I</span
      >ll
    </a>
  </div>

  <div class="flex min-w-0 justify-end">
    <a
      href="/mini"
      class="inline-flex h-8 items-center justify-center rounded-full border border-[var(--border-soft)] px-3 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] transition hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
      data-sveltekit-preload-data="tap"
      data-sveltekit-preload-code="viewport"
      aria-label="Open dastill-mini"
    >
      Mini
    </a>
    {#if trailing}
      {@render trailing()}
    {/if}
  </div>
</div>
