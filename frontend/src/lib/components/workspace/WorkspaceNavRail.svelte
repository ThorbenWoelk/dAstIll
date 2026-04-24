<script lang="ts">
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import { DOCS_URL } from "$lib/app-config";
  import {
    getSectionNavigationItems,
    goHintKeyForSection,
    type SectionNavigationSection,
  } from "$lib/section-navigation";
  import { sectionIcon } from "$lib/section-navigation-icons";
  import WorkspaceUserMenu from "./WorkspaceUserMenu.svelte";

  let {
    currentSection = "workspace" as SectionNavigationSection,
    collapsed = false,
    width = 200,
    onOpenGuide = () => {},
    onOpenShortcuts = () => {
      window.dispatchEvent(new CustomEvent("dastill:open-shortcuts"));
    },
    onToggleCollapse = () => {},
  }: {
    currentSection?: SectionNavigationSection;
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
</script>

<aside
  class="workspace-nav-rail relative z-50 hidden h-full shrink-0 flex-col border-r border-[var(--border-soft)] bg-[var(--panel-surface)] lg:flex"
  style="width: {width}px;"
>
  {#if collapsed}
    <div class="hidden flex-col items-center gap-2 px-1.5 pt-4 pb-2 lg:flex">
      <a
        href="/"
        data-sveltekit-preload-code="viewport"
        data-sveltekit-preload-data="tap"
        class="font-serif text-xl font-bold tracking-[-0.03em] text-[var(--color-swatch)] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)]"
        aria-label="Go to dAstIll home"
      >
        d<span style="color:var(--soft-foreground);">A</span>
      </a>
      <button
        type="button"
        class="mt-1 inline-flex h-6 w-6 items-center justify-center rounded-md text-[var(--soft-foreground)] opacity-55 transition-all hover:bg-[var(--surface)] hover:text-[var(--foreground)] hover:opacity-100"
        onclick={onToggleCollapse}
        aria-label="Expand sidebar"
      >
        <ChevronIcon direction="right" />
      </button>
    </div>
  {:else}
    <div class="flex items-center justify-between gap-3 px-4 pt-4 pb-2">
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
      </div>

      <button
        type="button"
        class="hidden h-6 w-6 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-55 transition-all hover:bg-[var(--accent-wash)] hover:opacity-100 lg:inline-flex"
        onclick={onToggleCollapse}
        aria-label="Collapse sidebar"
      >
        <ChevronIcon direction="left" />
      </button>
    </div>
  {/if}

  <nav
    id="app-section-nav-rail"
    class={`space-y-1 ${collapsed ? "mt-1 px-1.5" : "mt-4 px-2.5"}`}
    aria-label="Sections"
  >
    {#each navItems as item (item.section)}
      {@const icon = sectionIcon(item.section)}
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
        class={`flex items-center gap-2.5 rounded-md transition-colors ${
          collapsed ? "mx-auto h-9 w-9 justify-center p-0" : "px-3 py-2"
        } ${
          item.active
            ? "bg-[var(--surface-strong)] font-semibold text-[var(--foreground)]"
            : "text-[var(--soft-foreground)] hover:bg-[var(--surface)] hover:text-[var(--foreground)]"
        }`}
        aria-current={item.active ? "page" : undefined}
        data-tooltip={collapsed ? item.label : undefined}
        data-tooltip-placement={collapsed ? "right" : undefined}
      >
        <svg
          width={collapsed ? 18 : 16}
          height={collapsed ? 18 : 16}
          viewBox={icon.viewBox}
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
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
          <span class="min-w-0 truncate text-[13px] font-medium"
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

  <div
    class="mt-auto flex flex-col gap-3 pb-3 {collapsed ? 'px-1.5' : 'px-2.5'}"
  >
    <WorkspaceUserMenu {collapsed} {onOpenGuide} {onOpenShortcuts} />
  </div>

  {#if !collapsed}
    <div class="min-w-0 px-3 pb-3">
      <span
        class="inline-block shrink-0 whitespace-nowrap text-[10px] font-medium uppercase tracking-widest leading-snug text-[var(--soft-foreground)] opacity-35"
      >
        &copy; {new Date().getFullYear()} Thorben Woelk
      </span>
    </div>
  {/if}
</aside>

<style>
  .workspace-nav-rail {
    --background: #121417;
    --foreground: #f5efe8;
    --soft-foreground: #b7ada4;
    --surface: color-mix(in srgb, white 10%, #121417);
    --surface-strong: color-mix(in srgb, white 14%, #121417);
    --panel-surface: #121417;
    --panel-surface-strong: color-mix(in srgb, white 8%, #121417);
    --border: color-mix(in srgb, white 18%, #121417);
    --border-soft: color-mix(in srgb, white 10%, #121417);
    --muted: color-mix(in srgb, white 8%, #121417);
    --color-swatch: color-mix(in srgb, var(--accent) 88%, white);
    --accent-wash: color-mix(in srgb, var(--accent) 20%, #121417);
    --accent-wash-strong: color-mix(in srgb, var(--accent) 28%, #121417);
  }

  :global(.dark) .workspace-nav-rail {
    --background: #f4efe8;
    --foreground: #17181a;
    --soft-foreground: #6b6259;
    --surface: color-mix(in srgb, black 4%, #f4efe8);
    --surface-strong: color-mix(in srgb, black 8%, #f4efe8);
    --panel-surface: #f4efe8;
    --panel-surface-strong: color-mix(in srgb, black 3%, #f4efe8);
    --border: color-mix(in srgb, black 16%, #f4efe8);
    --border-soft: color-mix(in srgb, black 10%, #f4efe8);
    --muted: color-mix(in srgb, black 4%, #f4efe8);
    --color-swatch: color-mix(in srgb, var(--accent) 88%, black);
    --accent-wash: color-mix(in srgb, var(--accent) 16%, #f4efe8);
    --accent-wash-strong: color-mix(in srgb, var(--accent) 22%, #f4efe8);
  }
</style>
