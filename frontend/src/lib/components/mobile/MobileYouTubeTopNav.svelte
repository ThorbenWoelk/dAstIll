<script lang="ts">
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

  let currentSection = $derived(
    resolveCurrentSectionFromPathname($page.url.pathname),
  );
  let items = $derived(getSectionNavigationItems(currentSection, DOCS_URL));

  const bottomOrder: Array<SectionNavigationItem["section"]> = [
    "workspace",
    "highlights",
    "queue",
    "chat",
    "docs",
  ];

  let orderedItems = $derived(
    [...items].sort(
      (a, b) => bottomOrder.indexOf(a.section) - bottomOrder.indexOf(b.section),
    ),
  );

  let open = $state(false);
  let triggerEl = $state<HTMLButtonElement | null>(null);
  let firstLinkEl = $state<HTMLAnchorElement | null>(null);

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
    firstLinkEl?.focus({ preventScroll: false });
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

<div class="flex w-full items-center justify-between gap-3">
  <button
    type="button"
    bind:this={triggerEl}
    class="inline-flex h-10 w-10 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-80 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
    aria-label="Open navigation"
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

  <a
    href="/"
    class="min-w-0 text-base font-bold tracking-tighter text-[var(--color-swatch)] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)]"
    data-sveltekit-preload-data="tap"
    data-sveltekit-preload-code="viewport"
    aria-label="Go to dAstIll home"
  >
    d<span style="color:var(--soft-foreground);">A</span>st<span
      style="color:var(--soft-foreground);">I</span
    >ll
  </a>

  <div class="w-10" aria-hidden="true"></div>
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
      class="relative h-full w-[min(85vw,20rem)] overflow-hidden border-r border-[var(--accent-border-soft)] bg-[var(--surface-strong)] shadow-2xl"
    >
      <div
        class="flex items-center justify-between gap-3 border-b border-[var(--accent-border-soft)]/70 px-4 py-3"
      >
        <div
          class="min-w-0 text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-70"
        >
          Menu
        </div>
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
        class="h-full overflow-y-auto p-3"
        aria-label="Sections"
      >
        <!-- Home group -->
        <div class="px-2 pb-2 pt-1">
          <div
            class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-60"
          >
            Home
          </div>
        </div>

        {#each orderedItems.filter((i) => i.section === "workspace") as item}
          <a
            bind:this={firstLinkEl}
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
            onclick={closeDrawer}
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
          </a>
        {/each}

        <!-- Docs group -->
        <div class="px-2 pb-2 pt-5">
          <div
            class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-60"
          >
            Docs
          </div>
        </div>

        {#each orderedItems.filter((i) => i.section === "docs") as item}
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
            onclick={closeDrawer}
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
    </div>
  </div>
{/if}
