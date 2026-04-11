<script lang="ts">
  import { page } from "$app/state";
  import CloseIcon from "$lib/components/icons/CloseIcon.svelte";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import { DOCS_URL } from "$lib/app-config";
  import {
    getSectionNavigationItems,
    type SectionNavigationSection,
  } from "$lib/section-navigation";
  import { sectionIcon } from "$lib/section-navigation-icons";
  import { resolveCurrentSectionFromPathname } from "$lib/mobile-navigation/resolveCurrentSectionFromPathname";

  let {
    open = false,
    onClose,
  }: {
    open: boolean;
    onClose: () => void;
  } = $props();

  let currentSection = $derived(
    resolveCurrentSectionFromPathname(page.url.pathname),
  );
  let navItems = $derived(getSectionNavigationItems(currentSection, DOCS_URL));

  function handleBackdropClick() {
    onClose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    }
  }
</script>

<svelte:window onkeydown={open ? handleKeydown : undefined} />

{#if open}
  <!-- Backdrop -->
  <button
    type="button"
    class="fixed inset-0 z-[70] bg-black/50 backdrop-blur-[2px] lg:hidden"
    aria-label="Close menu"
    onclick={handleBackdropClick}
    tabindex="-1"
  ></button>

  <!-- Drawer -->
  <nav
    class="fixed inset-y-0 left-0 z-[71] flex w-[280px] flex-col bg-[var(--surface)] shadow-xl lg:hidden"
    style="padding-top: max(0.75rem, env(safe-area-inset-top)); padding-bottom: max(0.75rem, env(safe-area-inset-bottom));"
    aria-label="App sections"
  >
    <div class="flex items-center justify-between px-4 pb-4">
      <a
        href="/"
        class="font-serif text-xl font-bold tracking-[-0.03em] text-[var(--color-swatch)]"
        onclick={onClose}
        aria-label="Go to dAstIll home"
      >
        d<span style="color:var(--soft-foreground);">A</span>st<span
          style="color:var(--soft-foreground);">I</span
        >ll
      </a>
      <button
        type="button"
        class="inline-flex h-8 w-8 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
        aria-label="Close menu"
        onclick={onClose}
      >
        <CloseIcon size={18} strokeWidth={2} />
      </button>
    </div>

    <div class="flex-1 space-y-1 overflow-y-auto px-3">
      {#each navItems as item (item.section)}
        {@const icon = sectionIcon(item.section)}
        <a
          href={item.href}
          target={item.external ? "_blank" : undefined}
          rel={item.external ? "noopener noreferrer" : undefined}
          data-sveltekit-preload-code={item.external ? undefined : "viewport"}
          data-sveltekit-preload-data={item.external ? undefined : "tap"}
          class={`flex items-center gap-3 rounded-[var(--radius-sm)] px-3 py-3 text-[15px] font-medium transition-colors ${
            item.active
              ? "bg-[var(--accent-wash)] text-[var(--accent-strong)] font-semibold"
              : "text-[var(--soft-foreground)] hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
          }`}
          aria-current={item.active ? "page" : undefined}
          onclick={onClose}
        >
          <svg
            width="20"
            height="20"
            viewBox={icon.viewBox}
            fill="none"
            stroke="currentColor"
            stroke-width="1.7"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="shrink-0"
            aria-hidden="true"
          >
            {#each icon.paths as d}
              <path {d} />
            {/each}
          </svg>
          <span class="min-w-0 truncate">{item.label}</span>
          {#if item.external}
            <ExternalLinkIcon
              size={13}
              className="ml-auto shrink-0 opacity-50"
            />
          {/if}
        </a>
      {/each}
    </div>

    <div class="mt-auto px-4 pt-3">
      <span
        class="text-[10px] font-medium text-[var(--soft-foreground)] opacity-40"
      >
        &copy; {new Date().getFullYear()} Thorben Woelk.
      </span>
    </div>
  </nav>
{/if}
