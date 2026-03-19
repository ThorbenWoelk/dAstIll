<script lang="ts">
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";
  import type { AiIndicatorPresentation } from "$lib/ai-status";
  import type { SectionNavigationSection } from "$lib/section-navigation";
  import WorkspaceNavRail from "$lib/components/workspace/WorkspaceNavRail.svelte";

  const NAV_DEFAULT = 180;
  const NAV_MIN = 52;
  const NAV_SNAP = 100;
  const SIDEBAR_DEFAULT = 280;
  const SIDEBAR_MIN = 52;
  const SIDEBAR_SNAP = 140;
  const STORAGE_KEY = "dastill:shell-layout";

  let {
    currentSection = "workspace" as SectionNavigationSection,
    aiIndicator = null,
    onOpenGuide = () => {},
    topBar,
    sidebar,
    children,
  }: {
    currentSection?: SectionNavigationSection;
    aiIndicator?: AiIndicatorPresentation | null;
    onOpenGuide?: () => void;
    topBar?: Snippet;
    sidebar?: Snippet<
      [{ collapsed: boolean; toggle: () => void; width: number }]
    >;
    children: Snippet;
  } = $props();

  let navWidth = $state(NAV_DEFAULT);
  let sidebarWidth = $state(SIDEBAR_DEFAULT);
  let dragging = $state<"nav" | "sidebar" | null>(null);

  let navCollapsed = $derived(navWidth <= NAV_MIN);
  let sidebarCollapsed = $derived(sidebarWidth <= SIDEBAR_MIN);

  onMount(() => {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw) {
        const saved = JSON.parse(raw);
        if (typeof saved.navWidth === "number") navWidth = saved.navWidth;
        if (typeof saved.sidebarWidth === "number")
          sidebarWidth = saved.sidebarWidth;
      }
    } catch {
      // ignore
    }
  });

  function persist() {
    try {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({ navWidth, sidebarWidth }),
      );
    } catch {
      // ignore
    }
  }

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
</script>

<div class="flex h-full">
  <a
    href="#main-content"
    class="skip-link absolute left-4 top-4 z-50 rounded-full bg-[var(--accent)] px-4 py-2 text-sm font-semibold text-white"
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

  <div class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
    {#if topBar}
      <header
        class="hidden shrink-0 items-center justify-between gap-4 border-b border-[var(--border-soft)]/50 bg-[var(--surface)] px-6 py-2 lg:flex"
      >
        {@render topBar()}
      </header>
    {/if}

    <main id="main-content" class="min-h-0 flex-1 overflow-hidden">
      {@render children()}
    </main>
  </div>
</div>

<style>
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
</style>
