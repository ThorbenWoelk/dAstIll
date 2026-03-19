<script lang="ts">
  import AiStatusIndicator from "$lib/components/AiStatusIndicator.svelte";
  import SectionNavigation from "$lib/components/SectionNavigation.svelte";
  import ThemePanel from "$lib/components/ThemePanel.svelte";
  import type { SectionNavigationSection } from "$lib/section-navigation";

  interface AiIndicatorPresentation {
    detail: string;
    dotClass: string;
    title: string;
  }

  let {
    currentSection,
    docsUrl,
    aiIndicator = null,
    showGuide = false,
    onOpenGuide = () => {},
    showSearchToggle = false,
    searchOpen = false,
    onSearchToggle = () => {},
    guideButtonId = undefined,
  }: {
    currentSection: SectionNavigationSection;
    docsUrl: string;
    aiIndicator?: AiIndicatorPresentation | null;
    showGuide?: boolean;
    onOpenGuide?: () => void;
    showSearchToggle?: boolean;
    searchOpen?: boolean;
    onSearchToggle?: () => void;
    guideButtonId?: string | undefined;
  } = $props();
</script>

<header class="mx-auto w-full max-w-[1440px] min-w-0 px-4 pb-2 sm:px-2 lg:px-5">
  <div
    class="flex min-w-0 items-center gap-3 lg:grid lg:grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] lg:gap-6"
  >
    <div class="flex min-w-0 items-center gap-3 lg:justify-self-start">
      <a
        href="/"
        class="text-xl font-bold tracking-tighter text-[var(--accent)] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] sm:text-2xl"
        aria-label="Go to dAstIll home"
      >
        dAstIll
      </a>
      {#if aiIndicator}
        <AiStatusIndicator
          detail={aiIndicator.detail}
          dotClass={aiIndicator.dotClass}
          title={aiIndicator.title}
        />
      {/if}
      {#if showGuide}
        <button
          type="button"
          id={guideButtonId}
          class="inline-flex h-8 w-8 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-70 transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
          onclick={onOpenGuide}
          aria-label="Feature guide"
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="12" cy="12" r="10"></circle>
            <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
          </svg>
        </button>
      {/if}
    </div>

    <div class="hidden lg:flex lg:justify-center">
      <SectionNavigation {currentSection} {docsUrl} showMobile={false} />
    </div>

    <div class="ml-auto flex shrink-0 items-center gap-2 lg:justify-self-end">
      {#if showSearchToggle}
        <button
          type="button"
          class={`inline-flex h-8 min-w-8 items-center justify-center gap-2 rounded-full border px-3 text-[11px] font-bold uppercase tracking-[0.08em] transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${
            searchOpen
              ? "border-[var(--accent)]/25 bg-[var(--accent-soft)]/70 text-[var(--accent-strong)]"
              : "border-[var(--accent-border-soft)] text-[var(--soft-foreground)] hover:border-[var(--accent)]/40 hover:text-[var(--foreground)]"
          }`}
          onclick={onSearchToggle}
          aria-expanded={searchOpen}
          aria-controls="workspace-search-panel"
          aria-label={searchOpen ? "Close search" : "Open search"}
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.4"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
          <span class="hidden sm:inline">Search</span>
        </button>
      {/if}

      <ThemePanel />
    </div>
  </div>
</header>
