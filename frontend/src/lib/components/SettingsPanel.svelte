<script lang="ts">
  import { onMount } from "svelte";
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

  let { onClose = () => {} }: { onClose?: () => void } = $props();

  let mode = $state<ThemeMode>("light");
  let color = $state<ColorScheme>(DEFAULT_COLOR);
  let dyslexic = $state(false);

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
  let dyslexicStorageKey = $derived(
    getScopedStorageKey(
      "dastill-dyslexic-font",
      getAuthStorageScopeKey(authState.current),
    ),
  );

  function systemPrefersDark(): boolean {
    if (typeof window === "undefined") return false;
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  }

  function syncTheme() {
    if (typeof window === "undefined") return;
    const preference = resolveModePreference(mode, systemPrefersDark());
    const state = resolveThemeState(preference, systemPrefersDark());
    applyThemeState(document, state);
    applyColorScheme(document, color);
    document.documentElement.setAttribute(
      "data-dyslexic",
      dyslexic ? "true" : "false",
    );
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

  function toggleDyslexic() {
    dyslexic = !dyslexic;
    localStorage.setItem(dyslexicStorageKey, dyslexic ? "true" : "false");
    syncTheme();
  }

  onMount(() => {
    mode = readThemeMode(window.localStorage, themeStorageKey);
    color = readColorScheme(window.localStorage, colorStorageKey);
    dyslexic = localStorage.getItem(dyslexicStorageKey) === "true";
    syncTheme();
  });

  const VIEW_OPTIONS = [
    { id: "appearance", label: "Appearance", active: true },
    { id: "ai", label: "AI Models & Health", active: false },
    { id: "vocabulary", label: "Vocabulary Rules", active: false },
    { id: "account", label: "Account", active: false },
  ];

  const THEME_MODES: ThemeMode[] = ["light", "dark", "system"];
</script>

<div
  class="flex h-[min(600px,80vh)] w-full max-w-4xl flex-col overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--surface-strong)] shadow-2xl lg:flex-row"
>
  <!-- Settings Nav -->
  <div
    class="flex w-full flex-col gap-2 border-b border-[var(--border)] bg-[var(--surface)] p-6 lg:w-64 lg:border-b-0 lg:border-r"
  >
    <h3
      class="mb-2 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-60"
    >
      Preferences
    </h3>
    <div class="flex flex-row gap-1 lg:flex-col">
      {#each VIEW_OPTIONS as opt}
        <button
          class={`rounded-md px-3 py-2 text-left text-sm font-medium transition-colors ${
            opt.active
              ? "bg-[var(--muted)] text-[var(--foreground)]"
              : "text-[var(--soft-foreground)] hover:bg-[var(--muted)] hover:text-[var(--foreground)]"
          } ${!opt.active ? "opacity-50 grayscale cursor-not-allowed" : ""}`}
          disabled={!opt.active}
        >
          {opt.label}
        </button>
      {/each}
    </div>

    <div class="mt-auto hidden lg:block">
      <button
        class="text-[12px] font-medium text-[var(--soft-foreground)] hover:text-[var(--foreground)]"
        onclick={onClose}
      >
        Close Settings
      </button>
    </div>
  </div>

  <!-- Settings Content -->
  <div class="flex-1 overflow-y-auto p-8 lg:p-10">
    <div class="mb-8 flex items-center justify-between">
      <h1 class="text-2xl font-bold tracking-tight text-[var(--foreground)]">
        Appearance
      </h1>
      <button class="lg:hidden" onclick={onClose}>
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
      </button>
    </div>

    <div class="space-y-12">
      <!-- Theme Selection -->
      <div class="space-y-4">
        <h4 class="text-[13px] font-semibold text-[var(--foreground)]">
          Interface Theme
        </h4>
        <div
          class="inline-flex rounded-lg border border-[var(--border)] bg-[var(--surface)] p-1"
        >
          {#each THEME_MODES as m}
            <button
              class={`rounded-md px-6 py-2 text-sm font-medium transition-all ${
                mode === m
                  ? "bg-[var(--muted)] text-[var(--foreground)] shadow-sm"
                  : "text-[var(--soft-foreground)] hover:text-[var(--foreground)]"
              }`}
              onclick={() => setMode(m)}
            >
              <span class="capitalize">{m}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Accent Palette -->
      <div class="space-y-4">
        <div class="space-y-1">
          <h4 class="text-[13px] font-semibold text-[var(--foreground)]">
            Accent Palette
          </h4>
          <p
            class="max-w-md text-[12px] leading-relaxed text-[var(--soft-foreground)]"
          >
            Select the primary color used for focus states, highlights, and
            active indicators. The interface remains muted regardless of choice.
          </p>
        </div>

        <div class="flex flex-wrap items-center gap-4">
          {#each COLOR_SCHEMES as scheme}
            <button
              class="group relative flex h-8 w-8 items-center justify-center rounded-full transition-all ring-offset-2 ring-offset-[var(--surface-strong)] hover:ring-2 hover:ring-[var(--border)]"
              style="background-color: {scheme.swatch}"
              aria-label={scheme.label}
              onclick={() => setColor(scheme.id)}
            >
              {#if color === scheme.id}
                <div class="h-2 w-2 rounded-full bg-white shadow-sm"></div>
                <div
                  class="absolute -inset-1 rounded-full ring-2"
                  style="ring-color: {scheme.swatch}; --tw-ring-color: {scheme.swatch}"
                ></div>
              {/if}
            </button>
          {/each}
        </div>
      </div>

      <div class="h-px w-full bg-[var(--border-soft)] opacity-50"></div>

      <!-- Typography -->
      <div class="flex items-center justify-between max-w-md">
        <div class="space-y-0.5">
          <h4 class="text-[13px] font-semibold text-[var(--foreground)]">
            Dyslexia-friendly Font
          </h4>
          <p class="text-[12px] text-[var(--soft-foreground)]">
            Overrides editorial serif fonts with high-readability alternatives.
          </p>
        </div>
        <button
          class={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-[var(--accent)] focus:ring-offset-2 ${
            dyslexic ? "bg-[var(--accent)]" : "bg-[var(--muted)]"
          }`}
          role="switch"
          aria-checked={dyslexic}
          onclick={toggleDyslexic}
        >
          <span
            aria-hidden="true"
            class={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
              dyslexic ? "translate-x-5" : "translate-x-0"
            }`}
          ></span>
        </button>
      </div>
    </div>
  </div>
</div>
