<script lang="ts">
  import { authState } from "$lib/auth-state.svelte";
  import { clickOutside } from "$lib/actions/click-outside";
  import ThemePanel from "$lib/components/ThemePanel.svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";

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
  let filterOpen = $state(false);
  let view = $state<"main" | "appearance" | "help">("main");

  function handleSignOut() {
    menuOpen = false;
    view = "main";
    void authState.signOut();
  }

  function handleSignIn() {
    menuOpen = false;
    view = "main";
    window.location.href = "/login";
  }

  function closeAll() {
    menuOpen = false;
    filterOpen = false;
    view = "main";
  }

  function openSettings() {
    closeAll();
    window.dispatchEvent(new CustomEvent("dastill:open-settings"));
  }
</script>

<div
  class="relative flex w-full items-center gap-0.5 px-0.5"
  use:clickOutside={{
    enabled: menuOpen || filterOpen,
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
        menuOpen = !menuOpen;
        filterOpen = false;
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
        class="absolute bottom-full left-0 mb-2 w-72 rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--surface-strong)] p-1.5 shadow-[0_8px_30px_rgb(0,0,0,0.12)] z-55 flex flex-col overflow-hidden"
        role="menu"
      >
        {#if view === "main"}
          <div class="flex flex-col gap-0.5">
            <button
              class="group flex w-full items-center justify-between rounded-[var(--radius-md)] px-3 py-2 text-[14px] font-medium transition-colors hover:bg-[var(--accent-wash)]"
              onclick={() => (view = "appearance")}
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
                  <circle cx="12" cy="12" r="10" /><path d="M12 2v20" /><path
                    d="M12 2a10 10 0 0 1 0 20z"
                    fill="currentColor"
                    fill-opacity="0.15"
                  />
                </svg>
                <span>Appearance</span>
              </div>
              <ChevronIcon
                direction="right"
                size={14}
                strokeWidth={2}
                className="opacity-30 group-hover:opacity-60"
              />
            </button>

            <button
              class="group flex w-full items-center justify-between rounded-[var(--radius-md)] px-3 py-2 text-[14px] font-medium transition-colors hover:bg-[var(--accent-wash)]"
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

            <div
              class="my-1 mx-2 h-px bg-[var(--border-soft)] opacity-40"
            ></div>

            <button
              class="flex w-full items-center gap-3 rounded-[var(--radius-md)] px-3 py-2 text-[14px] font-medium text-[var(--foreground)] transition-colors hover:bg-[var(--accent-wash)]"
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
                  <path
                    d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"
                  /><polyline points="10 17 15 12 10 7" /><line
                    x1="15"
                    y1="12"
                    x2="3"
                    y2="12"
                  />
                {/if}
              </svg>
              <span
                >{authState.current.authState === "authenticated"
                  ? "Log Out"
                  : "Sign In"}</span
              >
            </button>
          </div>
        {:else if view === "appearance"}
          <div class="flex flex-col gap-3 p-3">
            <div class="flex items-center gap-2 mb-1">
              <button
                class="inline-flex h-8 w-8 items-center justify-center rounded-full transition-colors hover:bg-[var(--accent-wash)]"
                onclick={() => (view = "main")}
              >
                <ChevronIcon direction="left" size={16} strokeWidth={2.5} />
              </button>
              <p class="text-[14px] font-bold">Appearance</p>
            </div>
            <ThemePanel variant="inline" className="w-full" />
          </div>
        {:else if view === "help"}
          <div class="flex flex-col gap-1 p-1">
            <div class="flex items-center gap-2 px-2 py-1.5 mb-1">
              <button
                class="inline-flex h-8 w-8 items-center justify-center rounded-full transition-colors hover:bg-[var(--accent-wash)]"
                onclick={() => (view = "main")}
              >
                <ChevronIcon direction="left" size={16} strokeWidth={2.5} />
              </button>
              <p class="text-[14px] font-bold">Help</p>
            </div>
            <button
              class="flex w-full items-center gap-3 rounded-[var(--radius-md)] px-3 py-1.5 text-[14px] font-medium transition-colors hover:bg-[var(--accent-wash)]"
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
              class="flex w-full items-center gap-3 rounded-[var(--radius-md)] px-3 py-1.5 text-[14px] font-medium transition-colors hover:bg-[var(--accent-wash)]"
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
              class="flex w-full items-center gap-3 rounded-[var(--radius-md)] px-3 py-1.5 text-[14px] font-medium transition-colors hover:bg-[var(--accent-wash)]"
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
        {/if}
      </div>
    {/if}
  </div>

  {#if !collapsed}
    <div class="flex shrink-0 items-center gap-0.5 pr-1">
      <!-- Filter Toggle -->
      <div class="relative">
        <button
          class="flex h-8 w-8 items-center justify-center rounded-[var(--radius-md)] text-[var(--soft-foreground)] opacity-40 transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] hover:opacity-100 {filterOpen
            ? 'bg-[var(--accent-wash)] text-[var(--foreground)] opacity-100'
            : ''}"
          onclick={() => {
            filterOpen = !filterOpen;
            menuOpen = false;
          }}
          aria-haspopup="menu"
          aria-expanded={filterOpen}
          aria-label="View options"
        >
          <svg
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"><path d="M3 6h18M7 12h10M10 18h4" /></svg
          >
        </button>

        {#if filterOpen}
          <div
            class="absolute bottom-full left-[-130px] mb-2 w-64 rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--surface-strong)] p-1 shadow-[0_8px_30px_rgb(0,0,0,0.12)] z-55 flex flex-col overflow-hidden"
            role="menu"
          >
            <div class="flex flex-col p-1">
              <div
                class="px-3 py-1.5 text-[10px] font-bold text-[var(--soft-foreground)] opacity-50 uppercase tracking-wider"
              >
                Group by
              </div>
              <button
                class="flex w-full items-center justify-between rounded-[var(--radius-md)] px-3 py-1.5 text-[13.5px] font-medium hover:bg-[var(--accent-wash)]"
              >
                <div class="flex items-center gap-2.5">
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="opacity-50"
                    ><rect x="3" y="3" width="7" height="7" rx="1" /><rect
                      x="14"
                      y="3"
                      width="7"
                      height="7"
                      rx="1"
                    /><rect x="14" y="14" width="7" height="7" rx="1" /><rect
                      x="3"
                      y="14"
                      width="7"
                      height="7"
                      rx="1"
                    /></svg
                  >
                  <span>Workspace</span>
                </div>
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="3"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  class="text-[var(--accent-strong)]"
                  ><polyline points="20 6 9 17 4 12" /></svg
                >
              </button>
              <button
                class="flex w-full items-center gap-2.5 rounded-[var(--radius-md)] px-3 py-1.5 text-[13.5px] font-medium hover:bg-[var(--accent-wash)]"
              >
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  class="opacity-50"
                  ><circle cx="12" cy="12" r="10" /><polyline
                    points="12 6 12 12 16 14"
                  /></svg
                >
                <span>Updated</span>
              </button>
            </div>
            <div
              class="h-px bg-[var(--border-soft)] opacity-30 mx-1 my-0.5"
            ></div>
            <div class="flex flex-col p-1">
              <div
                class="px-3 py-1.5 text-[10px] font-bold text-[var(--soft-foreground)] opacity-50 uppercase tracking-wider"
              >
                Show
              </div>
              <button
                class="flex w-full items-center justify-between rounded-[var(--radius-md)] px-3 py-1.5 text-[13.5px] font-medium hover:bg-[var(--accent-wash)]"
              >
                <div class="flex items-center gap-2.5">
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="opacity-50"><circle cx="12" cy="12" r="10" /></svg
                  >
                  <span>Status</span>
                </div>
                <ChevronIcon
                  direction="right"
                  size={12}
                  strokeWidth={2.5}
                  className="opacity-20"
                />
              </button>
            </div>
            <div
              class="h-px bg-[var(--border-soft)] opacity-30 mx-1 my-0.5"
            ></div>
            <div class="p-1">
              <button
                class="flex w-full items-center rounded-[var(--radius-md)] px-3 py-1.5 text-[13.5px] font-medium hover:bg-[var(--accent-wash)]"
                >Mark All Read</button
              >
            </div>
          </div>
        {/if}
      </div>

      <!-- Settings Button -->
      <button
        class="flex h-8 w-8 items-center justify-center rounded-[var(--radius-md)] text-[var(--soft-foreground)] opacity-40 transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] hover:opacity-100"
        onclick={openSettings}
        aria-label="Settings"
        data-go-hint-key=","
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><path
            d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
          /><circle cx="12" cy="12" r="3" /></svg
        >
      </button>
    </div>
  {/if}
</div>
