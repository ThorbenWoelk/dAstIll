<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
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
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import type { AiStatus, VocabularyReplacement } from "$lib/types";
  import { createAiStatusPoller, refreshAiStatus } from "$lib/utils/ai-poller";
  import { formatVocabularyAddedAt } from "$lib/vocabulary";
  import { getPreferences } from "$lib/api";
  import CloseIcon from "$lib/components/icons/CloseIcon.svelte";

  let { onClose = () => {} }: { onClose?: () => void } = $props();

  type SectionId = "appearance" | "ai" | "vocabulary" | "account";

  const SECTIONS: Array<{ id: SectionId; label: string }> = [
    { id: "appearance", label: "Appearance" },
    { id: "ai", label: "AI Models & Health" },
    { id: "vocabulary", label: "Vocabulary Rules" },
    { id: "account", label: "Account" },
  ];

  const THEME_MODES: ThemeMode[] = ["light", "dark", "system"];

  let activeSection = $state<SectionId>("appearance");

  let mode = $state<ThemeMode>("light");
  let color = $state<ColorScheme>(DEFAULT_COLOR);
  let dyslexic = $state(false);

  let aiStatus = $state<AiStatus | null>(null);
  let aiRefreshing = $state(false);

  let vocabulary = $state<VocabularyReplacement[]>([]);
  let vocabularyLoaded = $state(false);

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

  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );

  let sortedVocabulary = $derived(
    vocabulary.toSorted(
      (a, b) => new Date(b.added_at).getTime() - new Date(a.added_at).getTime(),
    ),
  );

  let accountEmail = $derived(authState.current.email ?? null);
  let isAuthenticated = $derived(
    authState.current.authState === "authenticated",
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

  async function loadVocabulary() {
    try {
      const preferences = await getPreferences();
      vocabulary = preferences.vocabulary_replacements ?? [];
    } catch {
      vocabulary = [];
    } finally {
      vocabularyLoaded = true;
    }
  }

  async function handleAiRefresh() {
    if (aiRefreshing) return;
    aiRefreshing = true;
    try {
      await refreshAiStatus((payload) => {
        aiStatus = payload.status;
      });
    } catch {
      // poller swallows errors; keep previous status
    } finally {
      aiRefreshing = false;
    }
  }

  function openVocabularyPage() {
    onClose();
    void goto("/vocabulary");
  }

  function handleSignOut() {
    onClose();
    void authState.signOut();
  }

  function handleSignIn() {
    onClose();
    window.location.href = "/login";
  }

  onMount(() => {
    mode = readThemeMode(window.localStorage, themeStorageKey);
    color = readColorScheme(window.localStorage, colorStorageKey);
    dyslexic = localStorage.getItem(dyslexicStorageKey) === "true";
    syncTheme();

    void loadVocabulary();

    return createAiStatusPoller({
      intervalMs: 30000,
      onStatus: (payload) => {
        aiStatus = payload.status;
      },
    });
  });
</script>

{#snippet appearanceSection()}
  <div class="space-y-10">
    <header class="space-y-1">
      <h3
        class="font-serif text-[20px] font-semibold tracking-[-0.02em] text-[var(--foreground)]"
      >
        Appearance
      </h3>
      <p class="text-[13px] leading-relaxed text-[var(--soft-foreground)]">
        Tune how the interface looks across your devices.
      </p>
    </header>

    <div class="space-y-3">
      <h4
        class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-75"
      >
        Interface Theme
      </h4>
      <div
        class="inline-flex rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-strong)] p-1"
      >
        {#each THEME_MODES as m}
          <button
            type="button"
            class={`rounded-[var(--radius-sm)] px-5 py-1.5 text-[12px] font-bold uppercase tracking-[0.08em] transition-colors ${
              mode === m
                ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]"
                : "text-[var(--soft-foreground)] hover:text-[var(--foreground)]"
            }`}
            onclick={() => setMode(m)}
          >
            {m}
          </button>
        {/each}
      </div>
    </div>

    <div class="space-y-3">
      <div class="space-y-1">
        <h4
          class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-75"
        >
          Accent Palette
        </h4>
        <p
          class="max-w-md text-[13px] leading-relaxed text-[var(--soft-foreground)]"
        >
          Sets the primary color used for focus states, highlights, and active
          indicators.
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-3">
        {#each COLOR_SCHEMES as scheme}
          <button
            type="button"
            class="group relative flex h-8 w-8 items-center justify-center rounded-full transition-transform hover:scale-110"
            style="background-color: {scheme.swatch}"
            aria-label={scheme.label}
            aria-pressed={color === scheme.id}
            onclick={() => setColor(scheme.id)}
          >
            {#if color === scheme.id}
              <span
                class="absolute inset-[-4px] rounded-full ring-2 ring-[var(--foreground)] ring-offset-2 ring-offset-[var(--surface)]"
              ></span>
            {/if}
          </button>
        {/each}
      </div>
    </div>

    <div
      class="flex items-start justify-between gap-6 border-t border-[var(--border-soft)]/60 pt-6 max-w-md"
    >
      <div class="space-y-1">
        <h4
          class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-75"
        >
          Dyslexia-friendly Font
        </h4>
        <p class="text-[13px] text-[var(--soft-foreground)]">
          Swaps the editorial serif for a high-readability alternative.
        </p>
      </div>
      <button
        type="button"
        class={`relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)] ${
          dyslexic ? "bg-[var(--accent)]" : "bg-[var(--muted)]"
        }`}
        role="switch"
        aria-checked={dyslexic}
        aria-label="Toggle dyslexia-friendly font"
        onclick={toggleDyslexic}
      >
        <span
          aria-hidden="true"
          class={`inline-block h-5 w-5 transform rounded-full bg-[var(--surface-strong)] shadow transition-transform ${
            dyslexic ? "translate-x-5" : "translate-x-0.5"
          }`}
        ></span>
      </button>
    </div>
  </div>
{/snippet}

{#snippet aiSection()}
  <div class="space-y-6">
    <header class="space-y-1">
      <h3
        class="font-serif text-[20px] font-semibold tracking-[-0.02em] text-[var(--foreground)]"
      >
        AI Models & Health
      </h3>
      <p class="text-[13px] leading-relaxed text-[var(--soft-foreground)]">
        dAstIll blends cloud and local models for summaries, chat, and
        highlights. When the cloud is unreachable, we fall back to local
        fallbacks automatically.
      </p>
    </header>

    <div
      class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-strong)] p-5"
    >
      {#if aiIndicator}
        <div class="flex items-start gap-3">
          <span
            class={`mt-1.5 h-2.5 w-2.5 shrink-0 rounded-full ${aiIndicator.dotClass}`}
          ></span>
          <div class="min-w-0 flex-1 space-y-1.5">
            <p class="text-[14px] font-semibold text-[var(--foreground)]">
              {aiIndicator.title}
            </p>
            <p
              class="text-[13px] leading-relaxed text-[var(--soft-foreground)]"
            >
              {aiIndicator.detail}
            </p>
          </div>
        </div>
      {:else}
        <p class="text-[13px] text-[var(--soft-foreground)]">
          Checking AI availability…
        </p>
      {/if}
    </div>

    <div class="flex items-center gap-3">
      <button
        type="button"
        class="inline-flex items-center justify-center rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-strong)] px-4 py-2 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)] transition-colors hover:bg-[var(--accent-wash)] disabled:opacity-40"
        onclick={handleAiRefresh}
        disabled={aiRefreshing}
      >
        {aiRefreshing ? "Checking…" : "Re-check now"}
      </button>
      <span class="text-[12px] text-[var(--soft-foreground)] opacity-70">
        Auto-refreshes every 30 seconds.
      </span>
    </div>
  </div>
{/snippet}

{#snippet vocabularySection()}
  <div class="space-y-6">
    <header class="space-y-1">
      <h3
        class="font-serif text-[20px] font-semibold tracking-[-0.02em] text-[var(--foreground)]"
      >
        Vocabulary Rules
      </h3>
      <p class="text-[13px] leading-relaxed text-[var(--soft-foreground)]">
        Exact replacements applied before summaries are generated. Add new
        entries by selecting a misspelled name in any transcript or summary.
      </p>
    </header>

    {#if !vocabularyLoaded}
      <p class="text-[13px] text-[var(--soft-foreground)]">Loading rules…</p>
    {:else if sortedVocabulary.length === 0}
      <div
        class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-soft)] bg-[var(--surface-strong)] p-6 text-center"
      >
        <p class="text-[13px] text-[var(--soft-foreground)]">
          No vocabulary rules saved yet. Select a misspelled name, place, or
          company in any transcript or summary to save a replacement.
        </p>
      </div>
    {:else}
      <div class="space-y-2">
        <div class="flex items-baseline justify-between">
          <span
            class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-75"
          >
            {sortedVocabulary.length} saved rule{sortedVocabulary.length === 1
              ? ""
              : "s"}
          </span>
          <span
            class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-55"
          >
            Most recent first
          </span>
        </div>
        <ul
          class="divide-y divide-[var(--border-soft)]/60 overflow-hidden rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-strong)]"
        >
          {#each sortedVocabulary.slice(0, 5) as rule (rule.from + rule.added_at)}
            <li class="flex items-baseline justify-between gap-4 px-4 py-3">
              <div class="min-w-0 flex-1">
                <p
                  class="truncate text-[13px] font-medium text-[var(--foreground)]"
                >
                  <span class="text-[var(--soft-foreground)]">{rule.from}</span>
                  <span
                    class="px-2 text-[var(--soft-foreground)] opacity-50"
                    aria-hidden="true">→</span
                  >
                  <span>{rule.to}</span>
                </p>
              </div>
              <span
                class="shrink-0 text-[11px] text-[var(--soft-foreground)] opacity-70"
              >
                {formatVocabularyAddedAt(rule.added_at)}
              </span>
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    <button
      type="button"
      class="inline-flex items-center justify-center rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-strong)] px-4 py-2 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)] transition-colors hover:bg-[var(--accent-wash)]"
      onclick={openVocabularyPage}
    >
      Manage all rules
    </button>
  </div>
{/snippet}

{#snippet accountSection()}
  <div class="space-y-6">
    <header class="space-y-1">
      <h3
        class="font-serif text-[20px] font-semibold tracking-[-0.02em] text-[var(--foreground)]"
      >
        Account
      </h3>
      <p class="text-[13px] leading-relaxed text-[var(--soft-foreground)]">
        Manage your sign-in and session.
      </p>
    </header>

    <div
      class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-strong)] p-5 space-y-4"
    >
      <div class="space-y-1">
        <p
          class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-75"
        >
          Signed in as
        </p>
        <p class="text-[14px] font-semibold text-[var(--foreground)]">
          {isAuthenticated ? (accountEmail ?? "Account") : "Guest"}
        </p>
      </div>

      <div class="space-y-1">
        <p
          class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-75"
        >
          Plan
        </p>
        <p class="text-[13px] text-[var(--soft-foreground)]">
          {isAuthenticated ? "Pro Plan" : "Not signed in"}
        </p>
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-3">
      {#if isAuthenticated}
        <button
          type="button"
          class="inline-flex items-center justify-center rounded-[var(--radius-md)] bg-[var(--foreground)] px-5 py-2 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--background)] transition-colors hover:bg-[var(--accent-strong)]"
          onclick={handleSignOut}
        >
          Sign out
        </button>
      {:else}
        <button
          type="button"
          class="inline-flex items-center justify-center rounded-[var(--radius-md)] bg-[var(--foreground)] px-5 py-2 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--background)] transition-colors hover:bg-[var(--accent-strong)]"
          onclick={handleSignIn}
        >
          Sign in
        </button>
      {/if}
    </div>
  </div>
{/snippet}
<div
  class="flex h-[min(640px,85vh)] w-full max-w-4xl flex-col overflow-hidden rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--surface)] shadow-2xl"
>
  <div
    class="flex shrink-0 items-center justify-between gap-4 border-b border-[var(--border-soft)]/60 px-5 py-4 lg:px-8"
  >
    <h2
      class="font-serif text-[22px] font-semibold tracking-[-0.02em] text-[var(--foreground)]"
    >
      Settings
    </h2>
    <button
      type="button"
      class="inline-flex h-8 w-8 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
      aria-label="Close settings"
      onclick={onClose}
    >
      <CloseIcon size={16} strokeWidth={2.2} />
    </button>
  </div>

  <div class="flex min-h-0 flex-1 flex-col lg:flex-row">
    <nav
      class="flex shrink-0 gap-1 overflow-x-auto border-b border-[var(--border-soft)]/60 bg-[var(--surface-strong)] px-3 py-3 lg:w-60 lg:flex-col lg:overflow-x-visible lg:overflow-y-auto lg:border-b-0 lg:border-r lg:px-4 lg:py-6"
      aria-label="Settings sections"
    >
      {#each SECTIONS as section}
        <button
          type="button"
          class={`shrink-0 rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors lg:shrink ${
            activeSection === section.id
              ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)]"
              : "text-[var(--soft-foreground)] hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
          }`}
          aria-current={activeSection === section.id ? "page" : undefined}
          onclick={() => (activeSection = section.id)}
        >
          {section.label}
        </button>
      {/each}
    </nav>

    <div class="min-h-0 flex-1 overflow-y-auto px-5 py-6 lg:px-10 lg:py-8">
      {#if activeSection === "appearance"}
        {@render appearanceSection()}
      {:else if activeSection === "ai"}
        {@render aiSection()}
      {:else if activeSection === "vocabulary"}
        {@render vocabularySection()}
      {:else if activeSection === "account"}
        {@render accountSection()}
      {/if}
    </div>
  </div>
</div>
