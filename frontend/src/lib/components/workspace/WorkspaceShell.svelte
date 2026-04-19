<script lang="ts">
  import { replaceState } from "$app/navigation";
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";
  import { authState } from "$lib/auth-state.svelte";
  import {
    getAuthStorageScopeKey,
    getScopedStorageKey,
  } from "$lib/auth-storage";
  import type { AiIndicatorPresentation } from "$lib/ai-status";
  import type { SectionNavigationSection } from "$lib/section-navigation";
  import AiStatusIndicator from "$lib/components/AiStatusIndicator.svelte";
  import WorkspaceNavRail from "$lib/components/workspace/WorkspaceNavRail.svelte";

  const NAV_DEFAULT = 200;
  const NAV_MIN = 52;
  const NAV_SNAP = 100;
  const SIDEBAR_DEFAULT = 280;
  const SIDEBAR_MIN = 52;
  const SIDEBAR_SNAP = 140;

  let {
    currentSection = "workspace" as SectionNavigationSection,
    aiIndicator = null,
    onOpenGuide = () => {},
    mobileTopBar,
    topBar,
    tabNav,
    sidebar,
    children,
  }: {
    currentSection?: SectionNavigationSection;
    aiIndicator?: AiIndicatorPresentation | null;
    onOpenGuide?: () => void;
    mobileTopBar?: Snippet;
    topBar?: Snippet;
    tabNav?: Snippet;
    sidebar?: Snippet<
      [
        {
          collapsed: boolean;
          toggle: () => void;
          width: number;
          mobileVisible?: boolean;
        },
      ]
    >;
    children: Snippet;
  } = $props();

  let navWidth = $state(NAV_DEFAULT);
  let sidebarWidth = $state(SIDEBAR_DEFAULT);
  let dragging = $state<"nav" | "sidebar" | null>(null);

  let navCollapsed = $derived(navWidth <= NAV_MIN);
  let sidebarCollapsed = $derived(sidebarWidth <= SIDEBAR_MIN);
  let shellLayoutStorageKey = $derived(
    getScopedStorageKey(
      "dastill:shell-layout",
      getAuthStorageScopeKey(authState.current),
    ),
  );

  onMount(() => {
    try {
      const raw = localStorage.getItem(shellLayoutStorageKey);
      if (raw) {
        const saved = JSON.parse(raw);
        if (typeof saved.navWidth === "number") navWidth = saved.navWidth;
        if (typeof saved.sidebarWidth === "number")
          sidebarWidth = saved.sidebarWidth;
      }
    } catch {
      // ignore
    }
    if (window.innerWidth < 1024) navWidth = NAV_MIN;
  });

  function persist() {
    try {
      localStorage.setItem(
        shellLayoutStorageKey,
        JSON.stringify({ navWidth, sidebarWidth }),
      );
    } catch {
      // ignore
    }
  }

  $effect(() => {
    if (typeof localStorage === "undefined") {
      return;
    }

    try {
      const raw = localStorage.getItem(shellLayoutStorageKey);
      if (!raw) {
        return;
      }

      const saved = JSON.parse(raw);
      if (typeof saved.navWidth === "number") navWidth = saved.navWidth;
      if (typeof saved.sidebarWidth === "number")
        sidebarWidth = saved.sidebarWidth;
    } catch {
      // ignore
    }
    if (window.innerWidth < 1024) navWidth = NAV_MIN;
  });

  function handleResizeStart(target: "nav" | "sidebar", event: PointerEvent) {
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = target === "nav" ? navWidth : sidebarWidth;
    const min = target === "nav" ? NAV_MIN : SIDEBAR_MIN;
    const snap = target === "nav" ? NAV_SNAP : SIDEBAR_SNAP;
    dragging = target;

    function onMove(e: PointerEvent) {
      const width = Math.max(min, startWidth + (e.clientX - startX));
      if (target === "nav") {
        navWidth = width < snap ? NAV_MIN : width;
      } else {
        sidebarWidth = width < snap ? SIDEBAR_MIN : width;
      }
    }

    function onUp() {
      dragging = null;
      persist();
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }

    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  }

  function handleDblClick(target: "nav" | "sidebar") {
    if (target === "nav") {
      navWidth = navCollapsed ? NAV_DEFAULT : NAV_MIN;
    } else {
      sidebarWidth = sidebarCollapsed ? SIDEBAR_DEFAULT : SIDEBAR_MIN;
    }
    persist();
  }

  function toggleNav() {
    navWidth = navCollapsed ? NAV_DEFAULT : NAV_MIN;
    persist();
  }

  function toggleSidebar() {
    sidebarWidth = sidebarCollapsed ? SIDEBAR_DEFAULT : SIDEBAR_MIN;
    persist();
  }

  /** Fragment links update the hash but do not move focus: `<main>` is not focusable by default, and the app shell uses overflow-hidden so the window does not scroll. */
  function skipToMainContent(event: Event) {
    event.preventDefault();
    const main = document.getElementById("main-content");
    if (!main) return;
    const { pathname, search } = window.location;
    replaceState(`${pathname}${search}#main-content`, window.history.state);
    main.focus({ preventScroll: false });
    main.scrollIntoView({ block: "nearest", behavior: "auto" });
  }
</script>

<div class="workspace-shell flex h-full">
  <a
    href="#main-content"
    class="skip-link absolute left-4 top-4 z-50 rounded-full bg-[var(--accent)] px-4 py-2 text-sm font-semibold text-white"
    onclick={skipToMainContent}
  >
    Skip to Main Content
  </a>

  <WorkspaceNavRail
    {currentSection}
    collapsed={navCollapsed}
    width={navWidth}
    {onOpenGuide}
    onToggleCollapse={toggleNav}
  />

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="resize-handle hidden lg:block"
    class:active={dragging === "nav"}
    onpointerdown={(event) => handleResizeStart("nav", event)}
    ondblclick={() => handleDblClick("nav")}
  ></div>

  {#if sidebar}
    {@render sidebar({
      collapsed: sidebarCollapsed,
      toggle: toggleSidebar,
      width: sidebarWidth,
    })}

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="resize-handle hidden lg:block"
      class:active={dragging === "sidebar"}
      onpointerdown={(event) => handleResizeStart("sidebar", event)}
      ondblclick={() => handleDblClick("sidebar")}
    ></div>
  {/if}

  <div
    class="workspace-main flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden"
  >
    {#if mobileTopBar}
      <header
        class="workspace-mobile-header shrink-0 lg:hidden"
        style="padding-top: var(--safe-area-inset-top);"
      >
        <div class="flex h-12 items-center justify-between gap-4 px-4">
          {@render mobileTopBar()}
        </div>
      </header>
    {/if}
    {#if topBar || aiIndicator}
      <header
        class="workspace-desktop-header hidden shrink-0 items-center gap-4 lg:flex"
      >
        {#if aiIndicator}
          <div class="shrink-0">
            <AiStatusIndicator
              detail={aiIndicator.detail}
              dotClass={aiIndicator.dotClass}
              title={aiIndicator.title}
              showLabel
            />
          </div>
        {/if}
        {#if topBar}
          <div class="flex min-w-0 flex-1 items-center justify-between gap-4">
            {@render topBar()}
          </div>
        {/if}
      </header>
    {/if}
    {#if tabNav}
      <div class="workspace-desktop-tabs hidden shrink-0 lg:block">
        {@render tabNav()}
      </div>
    {/if}

    <main
      id="main-content"
      tabindex="-1"
      class="workspace-content-frame min-h-0 flex-1 overflow-hidden outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)]"
    >
      {@render children()}
    </main>
  </div>
</div>

<style>
  .workspace-shell {
    background: var(--background);
  }

  .workspace-main {
    position: relative;
    background: var(--surface-strong);
  }

  .workspace-mobile-header,
  .workspace-desktop-header {
    position: relative;
    z-index: var(--z-shell-header);
    border-bottom: 1px solid var(--border-soft);
    background: var(--surface-strong);
  }

  .workspace-mobile-header {
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

  :global(.dark) .workspace-mobile-header {
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

  .workspace-desktop-header {
    min-height: 3.5rem;
    padding: 0 1.5rem;
  }

  .workspace-desktop-tabs {
    padding: 1rem 1.5rem 0;
    border-bottom: 1px solid var(--border-soft);
    background: var(--surface-strong);
  }

  .workspace-content-frame {
    background: var(--surface-strong);
  }

  .resize-handle {
    width: 12px;
    margin-inline: -6px;
    flex-shrink: 0;
    cursor: col-resize;
    position: relative;
    z-index: var(--z-shell-resize-handle);
  }

  .resize-handle::after {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    left: 50%;
    width: 2px;
    transform: translateX(-50%);
    border-radius: 1px;
    background: transparent;
    transition: background 150ms ease;
  }

  .resize-handle:hover::after,
  .resize-handle.active::after {
    background: var(--accent);
    opacity: 0.5;
  }

  .resize-handle.active::after {
    opacity: 1;
  }

  @media (max-width: 1023px) {
    .workspace-mobile-header {
      min-height: calc(3rem + var(--safe-area-inset-top));
      background: var(--surface-strong);
      /* Remove the stacking context so position:fixed children (filter popup at
         z-[110]) participate in the root stacking context instead of being capped
         at this element's z-index. Without a stacking context the popup clears
         browse overlay and section drawer root layers. */
      z-index: auto;
    }
  }
</style>
