<script lang="ts">
  import { clickOutside } from "$lib/actions/click-outside";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";

  import {
    getSectionNavigationItems,
    type SectionNavigationItem,
    type SectionNavigationSection,
  } from "$lib/section-navigation";

  type Props = {
    currentSection: SectionNavigationSection;
    docsUrl: string;
    mobileMode?: "bottom" | "inline";
    showMobile?: boolean;
  };

  let {
    currentSection,
    docsUrl,
    mobileMode = "bottom",
    showMobile = true,
  }: Props = $props();

  let open = $state(false);
  let items = $derived(getSectionNavigationItems(currentSection, docsUrl));
  let currentItem = $derived(
    items.find((item) => item.section === currentSection) ?? items[0],
  );
  let mobileWrapperClass = $derived(
    mobileMode === "inline"
      ? "relative z-[70]"
      : "mobile-bottom-nav-offset fixed left-1/2 z-[70] -translate-x-1/2",
  );
  let mobilePanelClass = $derived(
    mobileMode === "inline"
      ? "absolute right-0 top-full z-[70] mt-2 flex w-[min(90vw,16rem)] flex-col gap-1 rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--surface-strong)] p-2 shadow-xl popover-rise"
      : "absolute bottom-full left-1/2 z-[70] mb-2 flex w-[min(90vw,16rem)] -translate-x-1/2 flex-col gap-1 rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--surface-strong)] p-2 shadow-xl popover-rise",
  );
  let mobileButtonClass = $derived(
    mobileMode === "inline"
      ? "inline-flex h-9 min-w-[8.5rem] max-w-[min(13rem,calc(100vw-7rem))] items-center justify-center gap-2 rounded-full border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-4 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)] shadow-sm transition-colors hover:border-[var(--accent)]/40"
      : "inline-flex h-10 min-w-[9.5rem] max-w-[calc(100vw-2rem)] items-center justify-center gap-2 rounded-full border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-4 text-[12px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)] shadow-lg transition-colors hover:border-[var(--accent)]/40",
  );

  function toggle() {
    open = !open;
  }

  function close() {
    open = false;
  }

  function itemClass(item: SectionNavigationItem): string {
    return item.active
      ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]"
      : "text-[var(--soft-foreground)] opacity-80 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]";
  }

  function pillClass(item: SectionNavigationItem): string {
    return item.active
      ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)] shadow-sm"
      : "text-[var(--soft-foreground)] opacity-65 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] hover:opacity-100";
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") close();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if showMobile}
  <div class="lg:hidden">
    <div
      class={mobileWrapperClass}
      use:clickOutside={{ enabled: open, onClickOutside: close }}
    >
      {#if open}
        <div
          id="section-navigation-menu"
          role="menu"
          aria-label="Sections"
          class={mobilePanelClass}
        >
          {#each items as item (item.section)}
            <a
              href={item.href}
              target={item.external ? "_blank" : undefined}
              rel={item.external ? "noopener noreferrer" : undefined}
              role="menuitem"
              class={`flex items-center justify-between gap-2 rounded-[var(--radius-sm)] px-3 py-2 text-[12px] font-semibold transition-colors ${itemClass(item)}`}
              aria-current={item.active ? "page" : undefined}
              onclick={close}
            >
              <span>{item.label}</span>
              {#if item.external}
                <ExternalLinkIcon size={14} className="shrink-0 opacity-70" />
              {/if}
            </a>
          {/each}
        </div>
      {/if}

      <button
        type="button"
        class={mobileButtonClass}
        aria-expanded={open}
        aria-haspopup="menu"
        aria-controls="section-navigation-menu"
        aria-label="Open section navigation"
        onclick={toggle}
      >
        <span class="min-w-0 truncate">{currentItem.label}</span>
        <svg
          class={`h-3 w-3 shrink-0 transition-transform ${open ? "rotate-180" : ""}`}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="3"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="m6 9 6 6 6-6" />
        </svg>
      </button>
    </div>
  </div>
{/if}

<nav class="hidden items-center gap-1 lg:flex" aria-label="Workspace sections">
  {#each items as item (item.section)}
    <a
      href={item.href}
      target={item.external ? "_blank" : undefined}
      rel={item.external ? "noopener noreferrer" : undefined}
      id={item.section === "docs" ? "nav-docs-link" : undefined}
      class={`inline-flex items-center gap-2 rounded-full px-4 py-2 text-[11px] font-bold uppercase tracking-[0.1em] transition-all ${pillClass(item)}`}
      aria-current={item.active ? "page" : undefined}
    >
      <span>{item.label}</span>
      {#if item.external}
        <ExternalLinkIcon size={12} className="shrink-0 opacity-70" />
      {/if}
    </a>
  {/each}
</nav>
