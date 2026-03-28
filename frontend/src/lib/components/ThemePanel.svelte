<script lang="ts">
  import { onMount } from "svelte";
  import { clickOutside } from "$lib/actions/click-outside";
  import { authState } from "$lib/auth-state.svelte";
  import {
    getAuthStorageScopeKey,
    getScopedStorageKey,
  } from "$lib/auth-storage";
  import {
    applyColorScheme,
    applyThemeState,
    COLOR_SCHEMES,
    DEFAULT_COLOR,
    readColorScheme,
    readThemeMode,
    resolveModePreference,
    resolveThemeState,
    writeColorScheme,
    writeThemeMode,
    type ColorScheme,
    type ThemeMode,
  } from "$lib/theme";

  const THEME_MODE_OPTIONS: Array<{ id: ThemeMode; label: string }> = [
    { id: "light", label: "Light" },
    { id: "dark", label: "Dark" },
    { id: "system", label: "System" },
  ];

  let {
    className = "",
    variant = "default",
  }: {
    className?: string;
    /** Inline block for drawers (no popover trigger). */
    variant?: "default" | "inline";
  } = $props();

  let open = $state(false);
  let mode = $state<ThemeMode>("system");
  let color = $state<ColorScheme>(DEFAULT_COLOR);
  let isDark = $state(false);
  let mediaQueryList = $state<MediaQueryList | null>(null);
  let triggerEl = $state<HTMLButtonElement | null>(null);
  let themeStorageKey = $derived(
    getScopedStorageKey(
      "dastill-theme-appearance",
      getAuthStorageScopeKey(authState.current),
    ),
  );
  let colorStorageKey = $derived(
    getScopedStorageKey(
      "dastill-theme-color",
      getAuthStorageScopeKey(authState.current),
    ),
  );

  function systemPrefersDark(): boolean {
    return (
      mediaQueryList?.matches ??
      (typeof window !== "undefined" &&
        window.matchMedia("(prefers-color-scheme: dark)").matches)
    );
  }

  function syncTheme() {
    if (typeof window === "undefined") return;
    const preference = resolveModePreference(mode, systemPrefersDark());
    const state = resolveThemeState(preference, systemPrefersDark());
    isDark = state.isDark;
    applyThemeState(document, state);
    applyColorScheme(document, color);
  }

  function setMode(m: ThemeMode) {
    mode = m;
    writeThemeMode(window.localStorage, m, themeStorageKey);
    syncTheme();
  }

  function setColor(c: ColorScheme) {
    color = c;
    writeColorScheme(window.localStorage, c, colorStorageKey);
    syncTheme();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && open) {
      open = false;
      triggerEl?.focus();
    }
  }

  onMount(() => {
    mediaQueryList = window.matchMedia("(prefers-color-scheme: dark)");
    mode = readThemeMode(window.localStorage, themeStorageKey);
    color = readColorScheme(window.localStorage, colorStorageKey);
    syncTheme();

    const handleChange = () => {
      if (mode === "system") syncTheme();
    };
    mediaQueryList.addEventListener("change", handleChange);
    document.addEventListener("keydown", handleKeydown);

    return () => {
      mediaQueryList?.removeEventListener("change", handleChange);
      document.removeEventListener("keydown", handleKeydown);
    };
  });

  $effect(() => {
    if (typeof window === "undefined") {
      return;
    }

    mode = readThemeMode(window.localStorage, themeStorageKey);
    color = readColorScheme(window.localStorage, colorStorageKey);
    syncTheme();
  });
</script>

{#snippet themeFields()}
  <p class="theme-panel-label">Accent</p>
  <div class="theme-color-options">
    {#each COLOR_SCHEMES as scheme}
      <button
        type="button"
        class="theme-color-btn"
        class:is-active={color === scheme.id}
        style="--swatch: {scheme.swatch}"
        aria-label={scheme.label}
        title={scheme.label}
        onclick={() => setColor(scheme.id)}
      ></button>
    {/each}
  </div>

  <p class="theme-panel-label theme-panel-label-mode">Mode</p>
  <div class="theme-mode-options">
    {#each THEME_MODE_OPTIONS as opt}
      <button
        type="button"
        class="theme-mode-btn"
        class:is-active={mode === opt.id}
        onclick={() => setMode(opt.id)}
      >
        {opt.label}
      </button>
    {/each}
  </div>
{/snippet}

{#if variant === "inline"}
  <div
    class="theme-panel-inline {className}"
    role="group"
    aria-label="Appearance"
  >
    {@render themeFields()}
  </div>
{:else}
  <div
    class="theme-panel-wrapper {className}"
    use:clickOutside={{
      enabled: open,
      onClickOutside: () => {
        open = false;
      },
    }}
  >
    <button
      bind:this={triggerEl}
      type="button"
      class="theme-panel-trigger"
      aria-label="Appearance settings"
      aria-expanded={open}
      onclick={() => (open = !open)}
    >
      {#if isDark}
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M20.5 14.1A8.5 8.5 0 0 1 9.9 3.5a8.5 8.5 0 1 0 10.6 10.6Z"
          ></path>
        </svg>
      {:else}
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="4.25"></circle>
          <path d="M12 2.75v2.5"></path>
          <path d="M12 18.75v2.5"></path>
          <path d="m5.47 5.47 1.77 1.77"></path>
          <path d="m16.76 16.76 1.77 1.77"></path>
          <path d="M2.75 12h2.5"></path>
          <path d="M18.75 12h2.5"></path>
          <path d="m5.47 18.53 1.77-1.77"></path>
          <path d="m16.76 7.24 1.77-1.77"></path>
        </svg>
      {/if}
      <span class="theme-panel-swatch" aria-hidden="true"></span>
    </button>

    <div
      class="theme-panel"
      class:is-open={open}
      role="dialog"
      aria-label="Appearance"
    >
      {@render themeFields()}
    </div>
  </div>
{/if}

<style>
  .theme-panel-inline {
    width: 100%;
  }

  .theme-panel-wrapper {
    position: relative;
  }

  .theme-panel-trigger {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border: none;
    background: none;
    color: var(--soft-foreground);
    cursor: pointer;
    transition: color 180ms ease;
  }

  .theme-panel-trigger[aria-expanded="true"] {
    color: var(--accent-strong);
  }

  .theme-panel-trigger:hover {
    color: var(--foreground);
  }

  .theme-panel-swatch {
    position: absolute;
    right: 0.3rem;
    bottom: 0.3rem;
    width: 0.45rem;
    height: 0.45rem;
    border-radius: 50%;
    background: var(--color-swatch);
    box-shadow: 0 0 0 2px var(--surface-strong);
  }

  .theme-panel-trigger:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .theme-panel {
    position: absolute;
    top: calc(100% + 10px);
    right: 0;
    min-width: 200px;
    padding: 0.85rem 1rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-soft);
    background: var(--surface-strong);
    box-shadow: 0 8px 32px var(--shadow-soft);
    opacity: 0;
    visibility: hidden;
    transform: translateY(-6px);
    transition:
      opacity 180ms ease,
      visibility 180ms ease,
      transform 180ms cubic-bezier(0.16, 1, 0.3, 1);
    z-index: 100;
  }

  .theme-panel.is-open {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
  }

  .theme-panel-label {
    font-size: 0.65rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--soft-foreground);
    margin: 0 0 0.5rem;
  }

  .theme-panel-label-mode {
    margin-top: 0.85rem;
  }

  .theme-color-options {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .theme-color-btn {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 2px solid var(--border-soft);
    background: var(--swatch);
    cursor: pointer;
    position: relative;
    transition:
      transform 150ms ease,
      border-color 150ms ease;
    padding: 0;
  }

  .theme-color-btn:hover {
    transform: scale(1.15);
  }

  .theme-color-btn.is-active {
    border-color: var(--accent-strong);
    transform: scale(1.08);
  }

  .theme-color-btn.is-active::after {
    content: "";
    position: absolute;
    inset: 2px;
    border-radius: 50%;
    border: 2px solid var(--surface-strong);
  }

  .theme-color-btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .theme-mode-options {
    display: flex;
    gap: 0.3rem;
  }

  .theme-mode-btn {
    flex: 1;
    padding: 0.35rem 0.5rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-soft);
    background: var(--surface);
    color: var(--soft-foreground);
    font-size: 0.7rem;
    font-weight: 600;
    cursor: pointer;
    transition:
      color 150ms ease,
      background 150ms ease,
      border-color 150ms ease;
    font-family: inherit;
  }

  .theme-mode-btn:hover {
    color: var(--foreground);
    border-color: var(--border);
  }

  .theme-mode-btn.is-active {
    background: var(--accent-soft);
    color: var(--accent-strong);
    border-color: color-mix(in srgb, var(--accent) 28%, var(--border-soft));
  }

  .theme-mode-btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
