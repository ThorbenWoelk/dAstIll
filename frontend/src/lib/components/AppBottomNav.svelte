<script lang="ts">
  import { page } from "$app/stores";
  import { DOCS_URL } from "$lib/app-config";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import {
    getSectionNavigationItems,
    type SectionNavigationSection,
  } from "$lib/section-navigation";

  function resolveCurrentSection(pathname: string): SectionNavigationSection {
    if (pathname.startsWith("/download-queue")) return "queue";
    if (pathname.startsWith("/highlights")) return "highlights";
    if (pathname.startsWith("/chat")) return "chat";
    return "workspace";
  }

  let currentSection = $derived(resolveCurrentSection($page.url.pathname));
  let items = $derived(getSectionNavigationItems(currentSection, DOCS_URL));
</script>

<nav
  id="app-section-nav-mobile"
  class="mobile-tab-bar lg:hidden"
  aria-label="App navigation"
>
  {#each items as item}
    <a
      href={item.href}
      target={item.external ? "_blank" : undefined}
      rel={item.external ? "noopener noreferrer" : undefined}
      data-sveltekit-preload-code={item.external ? undefined : "viewport"}
      data-sveltekit-preload-data={item.external ? undefined : "tap"}
      data-tour-target={item.section === "chat" ? "nav-chat" : undefined}
      id={item.section === "docs"
        ? "mobile-nav-docs-link"
        : item.section === "chat"
          ? "mobile-nav-chat-link"
          : item.section === "workspace"
            ? "mobile-nav-workspace-link"
            : undefined}
      class={`mobile-tab-item ${!item.external && item.active ? "mobile-tab-item--active" : ""}`}
      aria-current={!item.external && item.active ? "page" : undefined}
    >
      {#if item.section === "workspace"}
        <svg
          class="h-5 w-5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="3" y="4" width="6" height="16" rx="1.5" />
          <rect x="10" y="4" width="5" height="16" rx="1.5" />
          <rect x="16" y="4" width="5" height="16" rx="1.5" />
        </svg>
      {:else if item.section === "queue"}
        <svg
          class="h-5 w-5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M4 7h16" />
          <path d="M4 12h12" />
          <path d="M4 17h9" />
          <circle cx="18" cy="17" r="2" />
        </svg>
      {:else if item.section === "highlights"}
        <svg
          class="h-5 w-5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M7 4h10l2 4v10a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V8z" />
          <path d="M9 12h6" />
          <path d="M9 16h4" />
          <path d="M9 4v4h6V4" />
        </svg>
      {:else if item.section === "chat"}
        <svg
          class="h-5 w-5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.7"
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
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
          <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
        </svg>
      {/if}
      <span class="inline-flex items-center gap-1">
        {item.label}
        {#if item.external}
          <ExternalLinkIcon size={11} className="opacity-70" />
        {/if}
      </span>
    </a>
  {/each}
</nav>
