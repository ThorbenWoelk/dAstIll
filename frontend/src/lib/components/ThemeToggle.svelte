<script lang="ts">
  import { onMount } from "svelte";
  import {
    applyThemeState,
    readThemePreference,
    resolveNextThemePreference,
    resolveThemeState,
    writeThemePreference,
  } from "$lib/theme";

  let { className = "" }: { className?: string } = $props();

  let isDark = $state(false);
  let mediaQueryList = $state<MediaQueryList | null>(null);
  let label = $derived(
    isDark ? "Switch to light theme" : "Switch to dark theme",
  );

  function syncTheme() {
    if (typeof window === "undefined") return;

    const preference = readThemePreference(window.localStorage);
    const state = resolveThemeState(
      preference,
      mediaQueryList?.matches ??
        window.matchMedia("(prefers-color-scheme: dark)").matches,
    );

    isDark = state.isDark;
    applyThemeState(document, state);
  }

  function toggleTheme() {
    if (typeof window === "undefined") return;

    writeThemePreference(
      window.localStorage,
      resolveNextThemePreference(isDark),
    );
    syncTheme();
  }

  onMount(() => {
    mediaQueryList = window.matchMedia("(prefers-color-scheme: dark)");
    syncTheme();

    const handleChange = () => {
      if (readThemePreference(window.localStorage) === null) {
        syncTheme();
      }
    };

    mediaQueryList.addEventListener("change", handleChange);

    return () => {
      mediaQueryList?.removeEventListener("change", handleChange);
    };
  });
</script>

<button
  type="button"
  role="switch"
  aria-checked={isDark}
  aria-label={label}
  class={`theme-toggle ${className}`}
  class:theme-toggle--dark={isDark}
  onclick={toggleTheme}
>
  <span class="theme-toggle__track">
    <span class="theme-toggle__thumb"></span>
    <span class="theme-toggle__icon theme-toggle__icon--sun" aria-hidden="true">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
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
    </span>
    <span
      class="theme-toggle__icon theme-toggle__icon--moon"
      aria-hidden="true"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
        <path d="M20.5 14.1A8.5 8.5 0 0 1 9.9 3.5a8.5 8.5 0 1 0 10.6 10.6Z"
        ></path>
      </svg>
    </span>
  </span>
</button>

<style>
  .theme-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: none;
    background: none;
    cursor: pointer;
  }

  .theme-toggle__track {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: space-between;
    width: 4rem;
    padding: 0.25rem;
    border: 1px solid var(--border-soft);
    border-radius: 999px;
    background: var(--surface-frost);
    box-shadow: 0 10px 30px var(--shadow-soft);
    transition:
      border-color 180ms ease,
      background-color 180ms ease,
      box-shadow 180ms ease;
  }

  .theme-toggle:hover .theme-toggle__track {
    border-color: var(--border);
  }

  .theme-toggle:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: 999px;
  }

  .theme-toggle__thumb {
    position: absolute;
    top: 0.25rem;
    left: 0.25rem;
    width: 1.5rem;
    height: 1.5rem;
    border-radius: 999px;
    background: var(--surface);
    box-shadow: 0 8px 18px var(--shadow-soft);
    transition:
      transform 180ms cubic-bezier(0.16, 1, 0.3, 1),
      background-color 180ms ease,
      box-shadow 180ms ease;
  }

  .theme-toggle--dark .theme-toggle__thumb {
    transform: translateX(1.75rem);
    background: color-mix(in srgb, var(--surface) 75%, var(--foreground));
  }

  .theme-toggle__icon {
    position: relative;
    z-index: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    color: var(--soft-foreground);
    opacity: 0.55;
    transition:
      color 180ms ease,
      opacity 180ms ease;
  }

  .theme-toggle__icon svg {
    width: 0.95rem;
    height: 0.95rem;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .theme-toggle:not(.theme-toggle--dark) .theme-toggle__icon--sun,
  .theme-toggle--dark .theme-toggle__icon--moon {
    color: var(--foreground);
    opacity: 1;
  }
</style>
