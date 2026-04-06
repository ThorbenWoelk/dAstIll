<script lang="ts">
  import { goto } from "$app/navigation";

  import AiStatusIndicator from "$lib/components/AiStatusIndicator.svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import { DOCS_URL } from "$lib/app-config";
  import type { AiIndicatorPresentation } from "$lib/ai-status";
  import {
    getSectionNavigationItems,
    goHintKeyForSection,
    type SectionNavigationSection,
  } from "$lib/section-navigation";
  import { authState } from "$lib/auth-state.svelte";
  import WorkspaceUserMenu from "./WorkspaceUserMenu.svelte";

  let {
    currentSection = "workspace" as SectionNavigationSection,
    aiIndicator = null,
    collapsed = false,
    width = 180,
    onOpenGuide = () => {},
    onOpenShortcuts = () => {
      window.dispatchEvent(new CustomEvent("dastill:open-shortcuts"));
    },
    onToggleCollapse = () => {},
  }: {
    currentSection?: SectionNavigationSection;
    aiIndicator?: AiIndicatorPresentation | null;
    collapsed?: boolean;
    width?: number;
    onOpenGuide?: () => void;
    onOpenShortcuts?: () => void;
    onToggleCollapse?: () => void;
  } = $props();

  let navItems = $derived(getSectionNavigationItems(currentSection, DOCS_URL));

  $effect(() => {
    if (typeof window === "undefined") return;
    const onOpenGuideEvent = () => {
      onOpenGuide();
    };
    window.addEventListener("dastill:open-guide", onOpenGuideEvent);
    return () =>
      window.removeEventListener("dastill:open-guide", onOpenGuideEvent);
  });

  function navIcon(section: string): { viewBox: string; paths: string[] } {
    switch (section) {
      case "workspace":
        return {
          viewBox: "0 0 24 24",
          paths: ["M3 4h6v16H3z", "M10 4h5v16h-5z", "M16 4h5v16h-5z"],
        };
      case "queue":
        return {
          viewBox: "0 0 24 24",
          paths: ["M4 7h16", "M4 12h12", "M4 17h9"],
        };
      case "highlights":
        return {
          viewBox: "0 0 24 24",
          paths: [
            "M7 4h10l2 4v10a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V8z",
            "M9 12h6",
            "M9 16h4",
            "M9 4v4h6V4",
          ],
        };
      case "vocabulary":
        return {
          viewBox: "0 0 24 24",
          paths: ["M4 6h16", "M4 12h10", "M4 18h7", "M18 10l2 2-4 4-2-2z"],
        };
      case "chat":
        return {
          viewBox: "0 0 24 24",
          paths: [
            "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z",
            "M8 9h8",
            "M8 13h5",
          ],
        };
      case "docs":
        return {
          viewBox: "0 0 24 24",
          paths: [
            "M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z",
            "M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z",
          ],
        };
      default:
        return { viewBox: "0 0 24 24", paths: [] };
    }
  }

  async function handleSignIn() {
    await goto("/login");
  }

  async function handleSignOut() {
    await authState.signOut();
    window.location.href = "/";
  }
</script>

<aside
  class="relative z-50 hidden h-full shrink-0 flex-col bg-[var(--panel-surface)] lg:flex"
  style="width: {width}px;"
>
  {#if collapsed}
    <div class="flex items-center justify-center px-1.5 pt-3 pb-1">
      <button
        type="button"
        class="inline-flex h-7 w-7 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-60 transition-all hover:bg-[var(--accent-wash)] hover:opacity-100"
        onclick={onToggleCollapse}
        aria-label="Expand sidebar"
      >
        <ChevronIcon direction="right" />
      </button>
    </div>
  {:else}
    <div class="flex items-center justify-between gap-3 px-4 pt-3 pb-1">
      <div class="flex min-w-0 flex-1 items-center gap-2">
        <a
          href="/"
          data-sveltekit-preload-code="viewport"
          data-sveltekit-preload-data="tap"
          class="font-serif min-w-0 text-xl font-bold tracking-[-0.03em] text-[var(--color-swatch)] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)]"
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
      </div>

      <button
        type="button"
        class="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-55 transition-all hover:bg-[var(--accent-wash)] hover:opacity-100"
        onclick={onToggleCollapse}
        aria-label="Collapse sidebar"
      >
        <ChevronIcon direction="left" />
      </button>
    </div>
  {/if}

  <nav
    id="app-section-nav-rail"
    class={`space-y-0.5 ${collapsed ? "mt-1 px-1.5" : "mt-3 px-2"}`}
    aria-label="Sections"
  >
    {#each navItems as item (item.section)}
      {@const icon = navIcon(item.section)}
      <a
        href={item.href}
        target={item.external ? "_blank" : undefined}
        rel={item.external ? "noopener noreferrer" : undefined}
        data-sveltekit-preload-code={item.external ? undefined : "viewport"}
        data-sveltekit-preload-data={item.external ? undefined : "tap"}
        data-tour-target={item.section === "chat" ? "nav-chat" : undefined}
        data-go-hint-key={goHintKeyForSection(item.section)}
        id={item.section === "docs"
          ? "nav-docs-link"
          : item.section === "chat"
            ? "nav-chat-link"
            : item.section === "workspace"
              ? "nav-workspace-link"
              : undefined}
        class={`flex items-center gap-2 rounded-[var(--radius-sm)] transition-colors ${
          collapsed ? "justify-center px-0 py-2" : "px-3 py-2"
        } ${
          item.active
            ? "bg-[var(--accent-wash)] text-[var(--accent-strong)] font-semibold"
            : "text-[var(--soft-foreground)] hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
        }`}
        aria-current={item.active ? "page" : undefined}
        data-tooltip={collapsed ? item.label : undefined}
        data-tooltip-placement={collapsed ? "right" : undefined}
      >
        <svg
          width="16"
          height="16"
          viewBox={icon.viewBox}
          fill="none"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="shrink-0"
          aria-hidden="true"
        >
          {#each icon.paths as d}
            <path {d} />
          {/each}
        </svg>
        {#if !collapsed}
          <span class="min-w-0 truncate text-[14px] font-medium"
            >{item.label}</span
          >
          {#if item.external}
            <ExternalLinkIcon
              size={12}
              className="ml-auto shrink-0 opacity-50"
            />
          {/if}
        {/if}
      </a>
    {/each}
  </nav>

  <div class="mt-auto flex flex-col gap-3 pb-3 {collapsed ? 'px-1.5' : 'px-2'}">
    <WorkspaceUserMenu {collapsed} {onOpenGuide} {onOpenShortcuts} />
  </div>

  <div class="min-w-0 px-3 pb-2">
    <span
      class="inline-block shrink-0 whitespace-nowrap text-[10px] font-medium leading-snug text-[var(--soft-foreground)] opacity-40 {collapsed
        ? 'sr-only'
        : ''}"
    >
      &copy; {new Date().getFullYear()} Thorben Woelk.
    </span>
  </div>
</aside>
