<script lang="ts">
  import { page } from "$app/state";
  import { fade } from "svelte/transition";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import CloseIcon from "$lib/components/icons/CloseIcon.svelte";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import { DOCS_URL } from "$lib/app-config";
  import { authState } from "$lib/auth-state.svelte";
  import { signOutAndReloadHome } from "$lib/logout";
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

  let settingsOpen = $state(false);

  let currentSection = $derived(
    resolveCurrentSectionFromPathname(page.url.pathname),
  );
  let navItems = $derived(getSectionNavigationItems(currentSection, DOCS_URL));

  function handleBackdropClick() {
    close();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    if (settingsOpen) {
      event.preventDefault();
      settingsOpen = false;
      return;
    }
    event.preventDefault();
    close();
  }

  function close() {
    onClose();
  }

  function openSettings() {
    close();
    settingsOpen = true;
  }

  function closeSettings() {
    settingsOpen = false;
  }

  function handleSignIn() {
    close();
    window.location.href = "/login";
  }

  function handleSignOut() {
    close();
    void signOutAndReloadHome();
  }
</script>

<svelte:window onkeydown={open || settingsOpen ? handleKeydown : undefined} />

{#if open}
  <!-- Backdrop -->
  <button
    type="button"
    class="fixed inset-0 bg-black/50 backdrop-blur-[2px] lg:hidden"
    style="z-index: var(--z-mobile-section-drawer-backdrop);"
    aria-label="Close menu"
    onclick={handleBackdropClick}
    tabindex="-1"
  ></button>

  <!-- Drawer -->
  <nav
    class="fixed inset-y-0 left-0 flex w-[280px] flex-col bg-[var(--surface)] shadow-xl lg:hidden"
    style="z-index: var(--z-mobile-section-drawer-panel); padding-top: max(0.75rem, env(safe-area-inset-top)); padding-bottom: max(0.75rem, env(safe-area-inset-bottom));"
    aria-label="App sections"
  >
    <div class="flex items-center justify-between px-4 pb-4">
      <a
        href="/"
        class="font-serif text-xl font-bold tracking-[-0.03em] text-[var(--color-swatch)]"
        onclick={close}
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
        onclick={close}
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
          onclick={close}
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

    <!-- Settings section -->
    <div class="mt-auto border-t border-[var(--border-soft)]/50 px-3 pt-3">
      <button
        type="button"
        class="flex w-full items-center gap-3 rounded-[var(--radius-sm)] px-3 py-2.5 text-[15px] font-medium text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
        onclick={openSettings}
      >
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="shrink-0"
          aria-hidden="true"
        >
          <circle cx="12" cy="12" r="3" />
          <path
            d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h0a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h0a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v0a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
          />
        </svg>
        <span>Settings</span>
        <ChevronIcon
          direction="right"
          size={14}
          strokeWidth={2}
          className="ml-auto shrink-0 opacity-30"
        />
      </button>

      <button
        type="button"
        class="flex w-full items-center gap-3 rounded-[var(--radius-sm)] px-3 py-2.5 text-[15px] font-medium text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
        onclick={authState.current.authState === "authenticated"
          ? handleSignOut
          : handleSignIn}
      >
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="shrink-0"
          aria-hidden="true"
        >
          {#if authState.current.authState === "authenticated"}
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" /><polyline
              points="16 17 21 12 16 7"
            /><line x1="21" y1="12" x2="9" y2="12" />
          {:else}
            <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" /><polyline
              points="10 17 15 12 10 7"
            /><line x1="15" y1="12" x2="3" y2="12" />
          {/if}
        </svg>
        <span>
          {authState.current.authState === "authenticated"
            ? "Sign Out"
            : "Sign In"}
        </span>
      </button>

      <!-- User info -->
      <div class="mt-2 flex items-center gap-2.5 px-3 pb-1">
        <div
          class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-[var(--accent-strong)] text-[11px] font-bold text-white uppercase"
        >
          {authState.current.email
            ? authState.current.email.charAt(0)
            : authState.current.authState === "authenticated"
              ? "A"
              : "G"}
        </div>
        <span
          class="min-w-0 truncate text-[12px] font-medium text-[var(--soft-foreground)] opacity-55"
        >
          {authState.current.authState === "authenticated"
            ? authState.current.email || "Account"
            : "Not signed in"}
        </span>
      </div>
    </div>
  </nav>
{/if}

{#if settingsOpen}
  <div
    class="fixed inset-0 flex items-center justify-center p-0 sm:p-4 lg:hidden"
    style="z-index: var(--z-mobile-section-drawer-panel);"
    transition:fade={{ duration: 150 }}
    role="presentation"
  >
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="absolute inset-0 bg-[var(--overlay)]"
      onclick={closeSettings}
    ></div>
    <div class="relative z-10 flex w-full max-w-4xl items-stretch">
      <SettingsPanel onClose={closeSettings} />
    </div>
  </div>
{/if}
