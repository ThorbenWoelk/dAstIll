<script lang="ts">
  import { DOCS_URL } from "$lib/app-config";
  import type { AiIndicatorPresentation } from "$lib/ai-status";
  import AiStatusIndicator from "$lib/components/AiStatusIndicator.svelte";
  import SectionNavigation from "$lib/components/SectionNavigation.svelte";
  import type { SectionNavigationSection } from "$lib/section-navigation";
  import type { SearchResult, SearchStatus } from "$lib/types";

  import WorkspaceSearchBar from "$lib/components/workspace/WorkspaceSearchBar.svelte";

  let {
    currentSection = "workspace",
    aiIndicator = null,
    initialSearchStatus = null,
    onOpenGuide = () => {},
    onSearchResultSelect = async () => {},
  }: {
    currentSection?: SectionNavigationSection;
    aiIndicator?: AiIndicatorPresentation | null;
    initialSearchStatus?: SearchStatus | null;
    onOpenGuide?: () => void;
    onSearchResultSelect?: (
      result: SearchResult,
      mode: "transcript" | "summary",
    ) => Promise<void> | void;
  } = $props();
</script>

<div class="space-y-3 lg:space-y-0">
  <header
    class="mx-auto w-full max-w-[1440px] min-w-0 px-4 pb-2 sm:px-2 lg:px-5 lg:pb-6"
  >
    <div
      class="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-3 lg:grid-cols-[minmax(0,1fr)_auto_minmax(24rem,30rem)] lg:gap-6"
    >
      <div class="flex min-w-0 items-center gap-3 lg:justify-self-start">
        <a
          href="/"
          class="text-xl font-bold tracking-tighter text-[var(--color-swatch)] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] sm:text-2xl"
          aria-label="Go to dAstIll home"
        >
          d<span style="color:var(--soft-foreground);">A</span>st<span
            style="color:var(--soft-foreground);">I</span
          >ll
        </a>
        {#if aiIndicator}
          <AiStatusIndicator
            detail={aiIndicator.detail}
            dotClass={aiIndicator.dotClass}
            title={aiIndicator.title}
          />
        {/if}
        <button
          type="button"
          id="guide-trigger"
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
      </div>

      <div class="hidden lg:flex lg:justify-center">
        <SectionNavigation
          {currentSection}
          docsUrl={DOCS_URL}
          showMobile={false}
        />
      </div>

      <WorkspaceSearchBar {initialSearchStatus} {onSearchResultSelect} />
    </div>
  </header>
</div>
