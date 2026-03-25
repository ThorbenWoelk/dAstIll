<script lang="ts">
  import { page } from "$app/stores";
  import { DOCS_URL } from "$lib/app-config";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import ContentActionButton from "$lib/components/ContentActionButton.svelte";
  import {
    getSectionNavigationItems,
    goHintKeyForSection,
  } from "$lib/section-navigation";
  import type { SectionNavigationItem } from "$lib/section-navigation";
  import { resolveCurrentSectionFromPathname } from "$lib/mobile-navigation/resolveCurrentSectionFromPathname";
  import { mobileBottomBar } from "$lib/mobile-navigation/mobileBottomBar";
  import WorkspaceSidebarVideoFilterControl from "$lib/components/workspace/WorkspaceSidebarVideoFilterControl.svelte";

  let currentSection = $derived(
    resolveCurrentSectionFromPathname($page.url.pathname),
  );
  let items = $derived(getSectionNavigationItems(currentSection, DOCS_URL));
  const centerSlotSection = "queue";

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

  function resolveYouTubeRoleLabel(section: SectionNavigationItem["section"]) {
    if (section === "workspace") return "Home";
    if (section === "highlights") return "Shorts";
    if (section === "queue") return "Create";
    if (section === "chat") return "Chat";
    return "Docs";
  }

  let bar = $derived($mobileBottomBar);

  $effect(() => {
    if (typeof document === "undefined") return;
    const hidden = bar.kind === "hidden";
    document.documentElement.dataset.mobileBottomBar = hidden
      ? "hidden"
      : "visible";
    let height = "0px";
    if (!hidden) {
      if (bar.kind === "sectionsWithVideoFilter") {
        height =
          "calc(var(--mobile-footer-toolbar-height) + var(--mobile-tab-bar-height))";
      } else if (bar.kind === "videoActions") {
        height = "var(--mobile-video-actions-bar-height)";
      } else {
        height = "var(--mobile-tab-bar-height)";
      }
    }
    document.documentElement.style.setProperty(
      "--mobile-bottom-nav-height",
      height,
    );
  });
</script>

{#if bar.kind === "hidden"}
  <!-- Bottom bar suppressed (e.g. channel overview on mobile). -->
{:else if bar.kind === "videoActions"}
  <nav
    id="app-section-nav-mobile"
    class="mobile-tab-bar mobile-tab-bar--actions mobile-tab-bar--video-actions lg:hidden"
    aria-label="Video actions"
  >
    <div
      class="custom-scrollbar flex w-full min-w-0 flex-nowrap items-center justify-center gap-2 overflow-x-auto px-2 py-1.5 [scrollbar-width:thin]"
    >
      {#if bar.showFormatAction}
        <ContentActionButton
          compact
          icon="format"
          loading={bar.formatting}
          disabled={bar.busy ||
            bar.formatting ||
            bar.reverting ||
            !bar.aiAvailable}
          label={bar.formatting
            ? "Formatting transcript"
            : bar.aiAvailable
              ? "Clean formatting"
              : "auto-format (AI engine required)"}
          tooltip={bar.formatting
            ? "Formatting…"
            : bar.aiAvailable
              ? "Clean formatting"
              : "auto-format (AI engine required)"}
          onClick={() => bar.onFormat()}
        />
        {#if bar.showRevertAction}
          <ContentActionButton
            compact
            icon="revert"
            loading={bar.reverting}
            disabled={bar.busy ||
              bar.formatting ||
              bar.reverting ||
              !bar.canRevert}
            label={bar.reverting
              ? "Reverting transcript"
              : "Revert to original transcript"}
            tooltip={bar.reverting
              ? "Reverting…"
              : "Revert to original transcript"}
            onClick={() => bar.onRevert()}
          />
        {/if}
      {/if}
      {#if bar.showRegenerate}
        <ContentActionButton
          compact
          icon="regenerate"
          loading={bar.regenerating}
          disabled={bar.busy ||
            bar.formatting ||
            bar.regenerating ||
            bar.reverting ||
            !bar.aiAvailable}
          label={bar.regenerating
            ? "Regenerating summary"
            : bar.aiAvailable
              ? "Regenerate summary"
              : "regenerate (AI engine required)"}
          tooltip={bar.regenerating
            ? "Regenerating…"
            : bar.aiAvailable
              ? "Regenerate summary"
              : "regenerate (AI engine required)"}
          onClick={() => bar.onRegenerate()}
        />
      {/if}
      {#if bar.youtubeUrl}
        <ContentActionButton
          compact
          icon="youtube"
          href={bar.youtubeUrl}
          label="Open video on YouTube"
          tooltip="Open on YouTube"
        />
      {/if}
      <button
        type="button"
        class="inline-flex h-8 w-8 items-center justify-center rounded-full text-[var(--danger)] opacity-40 transition-all hover:bg-[var(--danger)]/10 hover:opacity-100 disabled:pointer-events-none disabled:opacity-20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--danger)]/30"
        aria-label="Reset video - wipe transcript and summary"
        disabled={bar.busy || bar.resetting}
        onclick={() => bar.onRequestResetVideo()}
      >
        {#if bar.resetting}
          <svg
            width="13"
            height="13"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="animate-spin"><path d="M21 12a9 9 0 1 1-6.219-8.56" /></svg
          >
        {:else}
          <svg
            width="13"
            height="13"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><polyline points="3 6 5 6 21 6" /><path
              d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"
            /><path d="M10 11v6" /><path d="M14 11v6" /><path
              d="M9 6V4h6v2"
            /></svg
          >
        {/if}
      </button>
      {#if bar.showAcknowledgeToggle}
        <button
          type="button"
          id="mark-read-toggle-mobile-footer"
          class={`inline-flex h-9 items-center gap-2 rounded-full px-2 text-[11px] font-bold uppercase tracking-[0.08em] transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 disabled:cursor-not-allowed disabled:opacity-30 hover:bg-[var(--accent-wash)] ${
            bar.acknowledged
              ? "text-[var(--foreground)]"
              : "text-[var(--soft-foreground)] hover:text-[var(--foreground)]"
          }`}
          aria-label={bar.acknowledged ? "Mark as unread" : "Mark as read"}
          aria-pressed={bar.acknowledged}
          onclick={() => bar.onAcknowledgeToggle()}
          disabled={bar.busy}
        >
          <span
            class={`flex h-5 w-5 shrink-0 items-center justify-center rounded-full border transition-all ${
              bar.acknowledged
                ? "border-[var(--accent)] bg-[var(--accent)] text-white"
                : "border-[var(--border)] bg-transparent text-transparent"
            }`}
            aria-hidden="true"
          >
            <svg
              class={`h-3 w-3 transition-opacity ${bar.acknowledged ? "opacity-100" : "opacity-0"}`}
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="3.4"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </span>
          <span>{bar.acknowledged ? "Read" : "Unread"}</span>
        </button>
      {/if}
      {#if bar.showEditAction}
        <ContentActionButton
          compact
          icon="edit"
          disabled={bar.busy}
          label="Edit distillation"
          tooltip="Edit distillation"
          tooltipAnchor="end"
          onClick={() => bar.onEdit()}
        />
      {/if}
    </div>
  </nav>
{:else}
  {#snippet sectionNavLinks()}
    {#each orderedItems as item}
      <a
        href={item.href}
        target={item.external ? "_blank" : undefined}
        rel={item.external ? "noopener noreferrer" : undefined}
        data-sveltekit-preload-code={item.external ? undefined : "viewport"}
        data-sveltekit-preload-data={item.external ? undefined : "tap"}
        data-tour-target={item.section === "chat" ? "nav-chat" : undefined}
        data-go-hint-key={goHintKeyForSection(item.section)}
        id={item.section === "docs"
          ? "mobile-nav-docs-link"
          : item.section === "chat"
            ? "mobile-nav-chat-link"
            : item.section === "workspace"
              ? "mobile-nav-workspace-link"
              : undefined}
        class={`mobile-tab-item ${
          item.section === centerSlotSection ? "mobile-tab-item--center" : ""
        } ${!item.external && item.active ? "mobile-tab-item--active" : ""}`}
        aria-current={!item.external && item.active ? "page" : undefined}
        aria-label={item.external
          ? `${resolveYouTubeRoleLabel(item.section)} (external)`
          : resolveYouTubeRoleLabel(item.section)}
      >
        {#if item.section === "workspace"}
          <svg
            class="h-5 w-5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M3 10.5 12 3l9 7.5" />
            <path d="M6.5 10.5V21h11V10.5" />
            <path d="M10 21v-6h4v6" />
          </svg>
        {:else if item.section === "queue"}
          <svg
            class="h-5 w-5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="12" cy="12" r="9" />
            <path d="M8.5 12h7" />
            <path d="M12 8.5v7" />
          </svg>
        {:else if item.section === "highlights"}
          <svg
            class="h-5 w-5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M9 7h10v10H9z" />
            <path d="M5 7l4-2v14l-4-2z" />
            <path d="M12 10l2 2-2 2" />
          </svg>
        {:else if item.section === "chat"}
          <svg
            class="h-5 w-5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path
              d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"
            />
            <path d="M8 9h8" />
            <path d="M8 13h5" />
          </svg>
        {:else if item.section === "docs"}
          <svg
            class="h-5 w-5"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
            <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
          </svg>
        {/if}
        <span class="mobile-tab-item-label inline-flex items-center gap-1">
          {resolveYouTubeRoleLabel(item.section)}
          {#if item.external}
            <ExternalLinkIcon size={11} className="opacity-70" />
          {/if}
        </span>
      </a>
    {/each}
  {/snippet}

  {#if bar.kind === "sectionsWithVideoFilter"}
    <div class="mobile-footer-stack lg:hidden">
      <div class="mobile-footer-toolbar" aria-label="Video filters">
        <WorkspaceSidebarVideoFilterControl
          videoTypeFilter={bar.filter.videoTypeFilter}
          acknowledgedFilter={bar.filter.acknowledgedFilter}
          disabled={bar.filter.disabled}
          onSelectVideoType={bar.filter.onSelectVideoType}
          onSelectAcknowledged={bar.filter.onSelectAcknowledged}
          onClearAllFilters={bar.filter.onClearAllFilters}
        />
      </div>
      <nav
        id="app-section-nav-mobile"
        class="mobile-tab-bar mobile-tab-bar--in-stack"
        aria-label="App navigation"
        data-go-hint-key="M"
      >
        {@render sectionNavLinks()}
      </nav>
    </div>
  {:else}
    <nav
      id="app-section-nav-mobile"
      class="mobile-tab-bar lg:hidden"
      aria-label="App navigation"
      data-go-hint-key="M"
    >
      {@render sectionNavLinks()}
    </nav>
  {/if}
{/if}
