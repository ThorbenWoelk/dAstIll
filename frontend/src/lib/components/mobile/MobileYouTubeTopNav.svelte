<script lang="ts">
  import type { Snippet } from "svelte";
  import { page } from "$app/stores";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import CloseIcon from "$lib/components/icons/CloseIcon.svelte";
  import {
    getSectionNavigationItems,
    type SectionNavigationItem,
  } from "$lib/section-navigation";
  import { DOCS_URL } from "$lib/app-config";
  import { tick } from "svelte";
  import { resolveCurrentSectionFromPathname } from "$lib/mobile-navigation/resolveCurrentSectionFromPathname";
  import { shouldCloseDrawerForKey } from "$lib/mobile-navigation/drawerKeyboard";
  import { mobileWorkspaceBrowseIntent } from "$lib/mobile-navigation/mobileWorkspaceBrowseIntent";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import ThemePanel from "$lib/components/ThemePanel.svelte";

  let {
    trailing,
    showBackInsteadOfMenu = false,
    onBack,
  }: {
    trailing?: Snippet;
    /** When true, left control is back (same slot as the hamburger). */
    showBackInsteadOfMenu?: boolean;
    onBack?: () => void;
  } = $props();

  let currentSection = $derived(
    resolveCurrentSectionFromPathname($page.url.pathname),
  );
  let items = $derived(getSectionNavigationItems(currentSection, DOCS_URL));

  const bottomOrder: Array<SectionNavigationItem["section"]> = [
    "workspace",
    "highlights",
    "vocabulary",
    "queue",
    "chat",
    "docs",
  ];

  let orderedItems = $derived(
    [...items].sort(
      (a, b) => bottomOrder.indexOf(a.section) - bottomOrder.indexOf(b.section),
    ),
  );

  let firstItem = $derived(orderedItems[0] ?? null);
  let remainingItems = $derived(firstItem ? orderedItems.slice(1) : []);

  let open = $state(false);
  let triggerEl = $state<HTMLButtonElement | null>(null);
  let firstSectionLinkEl = $state<HTMLAnchorElement | null>(null);
  let drawerFooterGitHubEl = $state<HTMLAnchorElement | null>(null);

  let drawerId = "mobile-section-drawer";
  let drawerMenuId = "mobile-section-drawer-menu";

  function closeDrawer() {
    open = false;
    // Return focus to the trigger after DOM update.
    void tick().then(() => triggerEl?.focus({ preventScroll: false }));
  }

  async function openDrawer() {
    open = true;
    await tick();
    firstSectionLinkEl?.focus({ preventScroll: false });
  }

  function handleKeydown(event: KeyboardEvent) {
    if (shouldCloseDrawerForKey(open, event.key)) closeDrawer();
  }

  $effect(() => {
    if (typeof document === "undefined") return;
    document.body.dataset.mobileDrawerOpen = open ? "true" : "";
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="grid w-full grid-cols-[auto_1fr_auto] items-center gap-2">
  <div class="flex justify-start">
    {#if showBackInsteadOfMenu}
      <button
        type="button"
        class="inline-flex h-10 w-10 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-80 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
        aria-label="Back"
        onclick={() => onBack?.()}
      >
        <ChevronIcon direction="left" size={20} strokeWidth={2.2} />
      </button>
    {:else}
      <button
        type="button"
        bind:this={triggerEl}
        class="inline-flex h-10 w-10 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-80 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
        aria-label="Open navigation menu"
        aria-expanded={open}
        aria-controls={drawerId}
        onclick={() => void openDrawer()}
      >
        <!-- Hamburger -->
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.1"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M4 7h16" />
          <path d="M4 12h16" />
          <path d="M4 17h16" />
        </svg>
      </button>
    {/if}
  </div>

  <div class="flex min-w-0 justify-center">
    <a
      href="/"
      class="min-w-0 text-base font-bold tracking-tighter text-[var(--color-swatch)] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)]"
      data-sveltekit-preload-data="tap"
      data-sveltekit-preload-code="viewport"
      aria-label="Go to dAstIll home"
      onclick={() => mobileWorkspaceBrowseIntent.set(true)}
    >
      d<span style="color:var(--soft-foreground);">A</span>st<span
        style="color:var(--soft-foreground);">I</span
      >ll
    </a>
  </div>

  <div class="flex min-w-0 justify-end">
    {#if trailing}
      {@render trailing()}
    {:else}
      <div class="w-10" aria-hidden="true"></div>
    {/if}
  </div>
</div>

{#if open}
  <div
    id={drawerId}
    class="fixed inset-0 z-[95] lg:hidden"
    role="dialog"
    aria-modal="true"
    aria-label="Navigation menu"
  >
    <button
      type="button"
      class="absolute inset-0 bg-[var(--overlay)]"
      onclick={closeDrawer}
      aria-label="Close navigation menu"
    ></button>

    <div
      class="relative flex h-full min-h-0 w-[min(85vw,20rem)] flex-col overflow-hidden border-r border-[var(--accent-border-soft)] bg-[var(--surface-strong)] shadow-2xl"
    >
      <div
        class="flex shrink-0 items-center justify-end border-b border-[var(--accent-border-soft)]/70 px-3 py-2"
      >
        <button
          type="button"
          class="inline-flex h-9 w-9 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-80 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
          aria-label="Close navigation menu"
          onclick={closeDrawer}
        >
          <CloseIcon size={18} strokeWidth={2.4} />
        </button>
      </div>

      <nav
        id={drawerMenuId}
        class="min-h-0 flex-1 overflow-y-auto p-3 pb-2"
        aria-label="Sections"
      >
        <div class="px-2 pb-2 pt-1">
          <div
            class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-60"
          >
            Sections
          </div>
        </div>

        {#if firstItem}
          <a
            bind:this={firstSectionLinkEl}
            href={firstItem.href}
            target={firstItem.external ? "_blank" : undefined}
            rel={firstItem.external ? "noopener noreferrer" : undefined}
            role="menuitem"
            class={`flex items-center justify-between gap-2 rounded-[var(--radius-sm)] px-3 py-2 text-[13px] font-semibold transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface-strong)] ${
              firstItem.active
                ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]"
                : "text-[var(--soft-foreground)] opacity-80 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
            }`}
            aria-current={firstItem.active ? "page" : undefined}
            onclick={() => {
              if (firstItem.section === "workspace") {
                mobileWorkspaceBrowseIntent.set(true);
              }
              closeDrawer();
            }}
            data-tour-target={firstItem.section === "chat"
              ? "nav-chat"
              : undefined}
            id={firstItem.section === "docs"
              ? "mobile-nav-docs-link"
              : firstItem.section === "chat"
                ? "mobile-nav-chat-link"
                : firstItem.section === "workspace"
                  ? "mobile-nav-workspace-link"
                  : undefined}
            data-sveltekit-preload-code={firstItem.external
              ? undefined
              : "viewport"}
            data-sveltekit-preload-data={firstItem.external ? undefined : "tap"}
          >
            <span class="min-w-0 truncate">{firstItem.label}</span>
            {#if firstItem.external}
              <ExternalLinkIcon size={14} className="shrink-0 opacity-70" />
            {/if}
          </a>
        {/if}

        {#each remainingItems as item (item.section)}
          <a
            href={item.href}
            target={item.external ? "_blank" : undefined}
            rel={item.external ? "noopener noreferrer" : undefined}
            role="menuitem"
            class={`flex items-center justify-between gap-2 rounded-[var(--radius-sm)] px-3 py-2 text-[13px] font-semibold transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface-strong)] ${
              item.active
                ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]"
                : "text-[var(--soft-foreground)] opacity-80 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
            }`}
            aria-current={item.active ? "page" : undefined}
            onclick={() => {
              if (item.section === "workspace") {
                mobileWorkspaceBrowseIntent.set(true);
              }
              closeDrawer();
            }}
            data-tour-target={item.section === "chat" ? "nav-chat" : undefined}
            id={item.section === "docs"
              ? "mobile-nav-docs-link"
              : item.section === "chat"
                ? "mobile-nav-chat-link"
                : item.section === "workspace"
                  ? "mobile-nav-workspace-link"
                  : undefined}
            data-sveltekit-preload-code={item.external ? undefined : "viewport"}
            data-sveltekit-preload-data={item.external ? undefined : "tap"}
          >
            <span class="min-w-0 truncate">{item.label}</span>
            {#if item.external}
              <ExternalLinkIcon size={14} className="shrink-0 opacity-70" />
            {/if}
          </a>
        {/each}
      </nav>

      <div
        class="shrink-0 border-t border-[var(--accent-border-soft)]/50 px-3 py-3"
        aria-label="Appearance settings"
      >
        <div class="pb-2">
          <div
            class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-60"
          >
            Settings
          </div>
        </div>
        <ThemePanel variant="inline" className="w-full" />
      </div>

      <div
        class="shrink-0 border-t border-[var(--accent-border-soft)]/50 px-3 py-3"
      >
        <a
          bind:this={drawerFooterGitHubEl}
          href="https://github.com/ThorbenWoelk/dAstIll"
          target="_blank"
          rel="noopener noreferrer"
          class="inline-flex w-full items-center gap-2 rounded-[var(--radius-sm)] px-3 py-2 text-[var(--soft-foreground)] opacity-60 transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
          onclick={closeDrawer}
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="currentColor"
            class="shrink-0"
            aria-hidden="true"
          >
            <path
              d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"
            />
          </svg>
          <span class="min-w-0 truncate text-[12px] font-medium">GitHub</span>
        </a>
        <div class="mt-2">
          <span
            class="text-[12px] font-medium leading-snug text-[var(--soft-foreground)] opacity-60"
          >
            &copy; {new Date().getFullYear()} Thorben Woelk.
          </span>
        </div>
      </div>
    </div>
  </div>
{/if}
