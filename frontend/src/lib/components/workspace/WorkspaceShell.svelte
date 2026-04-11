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
    sidebar,
    children,
  }: {
    currentSection?: SectionNavigationSection;
    aiIndicator?: AiIndicatorPresentation | null;
    onOpenGuide?: () => void;
    mobileTopBar?: Snippet;
    topBar?: Snippet;
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
    {aiIndicator}
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
    {#if topBar}
      <header
        class="workspace-desktop-header hidden shrink-0 items-center justify-between gap-4 lg:flex"
      >
        {@render topBar()}
      </header>
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
    background:
      radial-gradient(
        140% 110% at 50% -8%,
        color-mix(in srgb, var(--background) 88%, var(--accent-soft)) 0%,
        var(--background) 60%
      ),
      var(--background);
  }

  .workspace-main {
    position: relative;
    background: linear-gradient(
      180deg,
      color-mix(in srgb, var(--surface) 94%, var(--accent-soft)) 0%,
      var(--background) 100%
    );
  }

  .workspace-mobile-header,
  .workspace-desktop-header {
    position: relative;
    z-index: 5;
    border-bottom: 1px solid
      color-mix(in srgb, var(--border-soft) 92%, var(--background));
    background: linear-gradient(
      180deg,
      color-mix(in srgb, var(--surface) 96%, var(--background)) 0%,
      color-mix(in srgb, var(--surface) 92%, var(--accent-soft)) 100%
    );
  }

  .workspace-desktop-header {
    min-height: 4.5rem;
    padding: 0 1.75rem;
  }

  .workspace-content-frame {
    background: linear-gradient(
      180deg,
      color-mix(in srgb, var(--surface) 96%, var(--background)) 0%,
      color-mix(in srgb, var(--background) 92%, var(--surface)) 100%
    );
  }

  .resize-handle {
    width: 4px;
    flex-shrink: 0;
    cursor: col-resize;
    position: relative;
    z-index: 20;
  }

  .resize-handle::after {
    content: "";
    position: absolute;
    inset: 0;
    width: 2px;
    margin: 0 auto;
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
      background: linear-gradient(
        180deg,
        color-mix(in srgb, var(--surface) 98%, var(--background)) 0%,
        color-mix(in srgb, var(--surface) 92%, var(--accent-soft)) 100%
      );
    }
  }
</style>
