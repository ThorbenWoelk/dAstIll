<script lang="ts">
  import { page } from "$app/stores";
  import { DOCS_URL } from "$lib/app-config";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import {
    getSectionNavigationItems,
    goHintKeyForSection,
  } from "$lib/section-navigation";
  import type { SectionNavigationItem } from "$lib/section-navigation";
  import { resolveCurrentSectionFromPathname } from "$lib/mobile-navigation/resolveCurrentSectionFromPathname";

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
</script>

<nav
  id="app-section-nav-mobile"
  class="mobile-tab-bar lg:hidden"
  aria-label="App navigation"
  data-go-hint-key="M"
>
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
</nav>
