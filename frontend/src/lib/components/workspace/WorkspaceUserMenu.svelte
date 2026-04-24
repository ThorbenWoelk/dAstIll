<script lang="ts">
  import { authState } from "$lib/auth-state.svelte";
  import { clickOutside } from "$lib/actions/click-outside";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import CloseIcon from "$lib/components/icons/CloseIcon.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import { signOutAndReloadHome } from "$lib/logout";
  import { fade } from "svelte/transition";

  let {
    collapsed = false,
    onOpenGuide,
    onOpenShortcuts,
  } = $props<{
    collapsed?: boolean;
    onOpenGuide?: () => void;
    onOpenShortcuts?: () => void;
  }>();

  let menuOpen = $state(false);
  let view = $state<"main" | "help">("main");
  let settingsOpen = $state(false);

  function handleSignOut() {
    menuOpen = false;
    view = "main";
    void signOutAndReloadHome();
  }

  function handleSignIn() {
    menuOpen = false;
    view = "main";
    window.location.href = "/login";
  }

  function closeAll() {
    menuOpen = false;
    view = "main";
  }

  function openSettings() {
    menuOpen = false;
    view = "main";
    settingsOpen = true;
  }

  function closeSettings() {
    settingsOpen = false;
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    if (settingsOpen) {
      event.preventDefault();
      closeSettings();
      return;
    }
    if (menuOpen) {
      event.preventDefault();
      closeAll();
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#snippet mainView()}
  <div class="flex flex-col gap-0.5 text-[var(--foreground)]">
    <button
      class="group flex w-full items-center justify-between rounded-[var(--radius-md)] px-3 py-2 text-[14px] font-medium transition-colors hover:bg-[var(--accent-wash)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
      onclick={openSettings}
      role="menuitem"
    >
      <div class="flex items-center gap-3">
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="opacity-70"
        >
          <circle cx="12" cy="12" r="3" />
          <path
            d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h0a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h0a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v0a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
          />
        </svg>
        <span>Settings</span>
      </div>
      <ChevronIcon
        direction="right"
        size={14}
        strokeWidth={2}
        className="opacity-30 group-hover:opacity-60"
      />
    </button>

    <button
      class="group flex w-full items-center justify-between rounded-[var(--radius-md)] px-3 py-2 text-[14px] font-medium transition-colors hover:bg-[var(--accent-wash)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
      onclick={() => (view = "help")}
      role="menuitem"
    >
      <div class="flex items-center gap-3">
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="opacity-70"
        >
          <circle cx="12" cy="12" r="10" /><path
            d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"
          /><path d="M12 17h.01" />
        </svg>
        <span>Help</span>
      </div>
      <ChevronIcon
        direction="right"
        size={14}
        strokeWidth={2}
        className="opacity-30 group-hover:opacity-60"
      />
    </button>

    <a
      href="/mini"
      class="flex w-full items-center gap-3 rounded-[var(--radius-md)] px-3 py-2 text-[14px] font-medium transition-colors hover:bg-[var(--accent-wash)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
      data-sveltekit-preload-data="tap"
      data-sveltekit-preload-code="viewport"
      role="menuitem"
      onclick={closeAll}
    >
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="opacity-70"
      >
        <rect x="4" y="7" width="16" height="10" rx="2" />
        <path d="M9 21h6" />
        <path d="M12 17v4" />
      </svg>
      <span>Switch to Mini mode</span>
    </a>

    <div class="my-1 mx-2 h-px bg-[var(--border-soft)] opacity-40"></div>

    <button
      class="flex w-full items-center gap-3 rounded-[var(--radius-md)] px-3 py-2 text-[14px] font-medium text-[var(--foreground)] transition-colors hover:bg-[var(--accent-wash)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
      onclick={authState.current.authState === "authenticated"
        ? handleSignOut
        : handleSignIn}
      role="menuitem"
    >
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="opacity-70"
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
      <span
        >{authState.current.authState === "authenticated"
          ? "Log Out"
          : "Sign In"}</span
      >
    </button>
  </div>
{/snippet}

{#snippet mainViewMobile()}
  <div class="flex flex-col gap-0.5">
    <div class="flex items-center justify-between gap-3 px-3 pt-2 pb-1.5">
      <p
        class="font-serif text-[18px] font-semibold tracking-[-0.02em] text-[var(--foreground)]"
      >
        Account
      </p>
      <button
        type="button"
        class="inline-flex h-8 w-8 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface-strong)]"
        aria-label="Close menu"
        onclick={closeAll}
      >
        <CloseIcon size={16} strokeWidth={2.2} />
      </button>
    </div>
    {@render mainView()}
  </div>
{/snippet}

{#snippet helpView()}
  <div class="flex flex-col gap-1 p-1 text-[var(--foreground)]">
    <div class="mb-1 flex items-center gap-2 px-2 py-1.5">
      <button
        type="button"
        class="inline-flex h-8 w-8 items-center justify-center rounded-full transition-colors hover:bg-[var(--accent-wash)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
        onclick={() => (view = "main")}
        aria-label="Back to settings"
      >
        <ChevronIcon direction="left" size={16} strokeWidth={2.5} />
      </button>
      <p class="text-[14px] font-bold">Help</p>
    </div>
    <button
      class="flex w-full items-center gap-3 rounded-[var(--radius-md)] px-3 py-1.5 text-[14px] font-medium transition-colors hover:bg-[var(--accent-wash)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
      onclick={() => {
        menuOpen = false;
        onOpenShortcuts?.();
      }}
    >
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="opacity-50"
        ><rect x="2" y="6" width="20" height="12" rx="2" /><path
          d="M6 10h.01M10 10h.01M14 10h.01"
        /></svg
      >
      Keyboard shortcuts
    </button>
    <button
      class="flex w-full items-center gap-3 rounded-[var(--radius-md)] px-3 py-1.5 text-[14px] font-medium transition-colors hover:bg-[var(--accent-wash)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
      id="guide-trigger"
      onclick={() => {
        menuOpen = false;
        onOpenGuide?.();
      }}
    >
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="opacity-50"
        ><circle cx="12" cy="12" r="10" /><path
          d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"
        /><line x1="12" y1="17" x2="12.01" y2="17" /></svg
      >
      User guide
    </button>
    <a
      href="https://github.com/ThorbenWoelk/dAstIll"
      target="_blank"
      rel="noopener noreferrer"
      class="flex w-full items-center gap-3 rounded-[var(--radius-md)] px-3 py-1.5 text-[14px] font-medium transition-colors hover:bg-[var(--accent-wash)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
      onclick={() => (menuOpen = false)}
    >
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="currentColor"
        class="opacity-50"
        ><path
          d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"
        /></svg
      >
      GitHub repo
    </a>
  </div>
{/snippet}

{#snippet menuContent()}
  {#if view === "main"}
    {@render mainView()}
  {:else if view === "help"}
    {@render helpView()}
  {/if}
{/snippet}

{#snippet mobileMenuContent()}
  {#if view === "main"}
    {@render mainViewMobile()}
  {:else if view === "help"}
    {@render helpView()}
  {/if}
{/snippet}

<div
  class="relative flex w-full items-center gap-0.5 px-0.5"
  use:clickOutside={{
    enabled: menuOpen,
    onClickOutside: closeAll,
  }}
>
  <!-- User Profile Section -->
  <div class="relative flex min-w-0 flex-1">
    <button
      class="flex min-w-0 flex-1 items-center gap-2.5 rounded-[var(--radius-md)] p-1.5 text-left transition-colors hover:bg-[var(--accent-wash)] focus-visible:outline-none {collapsed
        ? 'justify-center'
        : ''}"
      onclick={() => {
        if (menuOpen) {
          closeAll();
        } else {
          menuOpen = true;
        }
      }}
      aria-haspopup="menu"
      aria-expanded={menuOpen}
    >
      <!-- Avatar -->
      <div
        class="h-9 w-9 shrink-0 overflow-hidden rounded-full border border-[var(--border-soft)] bg-[var(--surface-strong)] shadow-sm"
      >
        <div
          class="flex h-full w-full items-center justify-center bg-[var(--accent-strong)] text-[14px] font-bold text-white uppercase"
        >
          {authState.current.email
            ? authState.current.email.charAt(0)
            : authState.current.authState === "authenticated"
              ? "A"
              : "G"}
        </div>
      </div>

      {#if !collapsed}
        <div
          class="flex min-w-0 flex-1 flex-col overflow-hidden leading-[1.25]"
        >
          <span
            class="truncate text-[13.5px] font-bold text-[var(--foreground)]"
          >
            {authState.current.authState === "authenticated"
              ? authState.current.email?.split("@")[0] || "Account"
              : "Guest"}
          </span>
          <span
            class="truncate text-[11.5px] font-medium text-[var(--soft-foreground)] opacity-50"
          >
            {authState.current.authState === "authenticated"
              ? "Pro Plan"
              : "Not signed in"}
          </span>
        </div>
      {/if}
    </button>

    {#if menuOpen}
      <div
        class="fixed inset-0 z-[100] flex items-end lg:hidden"
        role="presentation"
      >
        <button
          type="button"
          class="absolute inset-0 bg-[var(--overlay)]"
          aria-label="Close settings"
          onclick={closeAll}
        ></button>
        <div
          class="relative z-10 flex max-h-[min(36rem,85vh)] w-full flex-col overflow-hidden rounded-t-[calc(var(--radius-lg)+0.25rem)] border-t border-[var(--border-soft)] bg-[var(--surface-strong)] px-2 pt-2 text-[var(--foreground)] shadow-2xl"
          role="dialog"
          aria-modal="true"
          aria-label="Settings"
        >
          <div
            class="custom-scrollbar min-h-0 flex-1 overflow-y-auto pb-[max(env(safe-area-inset-bottom),0.75rem)]"
          >
            {@render mobileMenuContent()}
          </div>
        </div>
      </div>

      <div
        class="absolute bottom-full left-0 z-[55] mb-2 hidden w-72 flex-col overflow-hidden rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--surface-strong)] p-1.5 text-[var(--foreground)] shadow-[var(--shadow-soft)] lg:flex"
        role="menu"
      >
        {@render menuContent()}
      </div>
    {/if}
  </div>
</div>

{#if settingsOpen}
  <div
    class="fixed inset-0 z-[200] flex items-center justify-center p-0 sm:p-4"
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
