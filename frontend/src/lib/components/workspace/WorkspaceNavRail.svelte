<script lang="ts">
  import { onMount } from "svelte";
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

  onMount(() => {
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
  class="hidden h-full shrink-0 flex-col bg-[var(--panel-surface)] lg:flex"
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
    data-go-hint-key="M"
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
    <div class="flex flex-col gap-1">
      <button
        type="button"
        id="guide-trigger"
        data-go-hint-key="U"
        class={`inline-flex items-center gap-2 rounded-[var(--radius-sm)] text-[var(--soft-foreground)] opacity-60 transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${collapsed ? "justify-center px-0 py-2" : "px-3 py-2"}`}
        onclick={onOpenGuide}
        aria-label="Feature guide"
        data-tooltip={collapsed ? "Guide" : undefined}
        data-tooltip-placement={collapsed ? "right" : undefined}
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="shrink-0"
          aria-hidden="true"
        >
          <circle cx="12" cy="12" r="10" />
          <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" />
          <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
        {#if !collapsed}
          <span class="text-[12px] font-medium">Guide</span>
        {/if}
      </button>
      <button
        type="button"
        id="keyboard-shortcuts-trigger"
        class={`inline-flex items-center gap-2 rounded-[var(--radius-sm)] text-[var(--soft-foreground)] opacity-60 transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${collapsed ? "justify-center px-0 py-2" : "px-3 py-2"}`}
        onclick={onOpenShortcuts}
        aria-label="Keyboard shortcuts"
        data-tooltip={collapsed ? "Shortcuts" : undefined}
        data-tooltip-placement={collapsed ? "right" : undefined}
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="shrink-0"
          aria-hidden="true"
        >
          <rect x="2" y="6" width="20" height="12" rx="2" />
          <path d="M6 10h.01M10 10h.01M14 10h.01" />
        </svg>
        {#if !collapsed}
          <span class="text-[12px] font-medium">Shortcuts</span>
        {/if}
      </button>
      <a
        href="https://github.com/ThorbenWoelk/dAstIll"
        target="_blank"
        rel="noopener noreferrer"
        class={`inline-flex shrink-0 items-center gap-2 rounded-[var(--radius-sm)] text-[var(--soft-foreground)] opacity-60 transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${collapsed ? "justify-center px-0 py-2" : "px-3 py-2"}`}
        aria-label={collapsed ? "GitHub" : undefined}
        data-tooltip={collapsed ? "GitHub" : undefined}
        data-tooltip-placement={collapsed ? "right" : undefined}
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="currentColor"
          class="shrink-0"
          aria-hidden="true"
        >
          <path
            d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"
          />
        </svg>
        {#if !collapsed}
          <span class="min-w-0 truncate text-[12px] font-medium">GitHub</span>
        {/if}
      </a>
    </div>

    {#if collapsed}
      {#if authState.error}
        <button
          type="button"
          class="inline-flex h-7 w-7 items-center justify-center rounded-full text-[var(--danger)] opacity-80 transition hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
          onclick={handleSignIn}
          aria-label="Auth error - retry"
          data-tooltip="Auth error"
          data-tooltip-placement="right"
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="shrink-0"
            aria-hidden="true"
          >
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
        </button>
      {:else if authState.current.authState === "authenticated"}
        <button
          type="button"
          class="inline-flex h-7 w-7 items-center justify-center rounded-full bg-[var(--accent-wash)] text-[var(--accent-strong)] transition hover:bg-[var(--accent)]/30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
          onclick={handleSignOut}
          aria-label="Sign out"
          data-tooltip="Sign out"
          data-tooltip-placement="right"
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="shrink-0"
            aria-hidden="true"
          >
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
            <polyline points="16 17 21 12 16 7" />
            <line x1="21" y1="12" x2="9" y2="12" />
          </svg>
        </button>
      {:else}
        <button
          type="button"
          class="inline-flex h-7 w-7 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-60 transition hover:bg-[var(--accent-wash)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
          onclick={handleSignIn}
          aria-label="Sign in"
          data-tooltip="Sign in"
          data-tooltip-placement="right"
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="shrink-0"
            aria-hidden="true"
          >
            <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
            <polyline points="10 17 15 12 10 7" />
            <line x1="15" y1="12" x2="3" y2="12" />
          </svg>
        </button>
      {/if}
    {:else}
      <div class="flex flex-col gap-1">
        {#if authState.error}
          <div
            class="flex flex-col gap-1 rounded-[var(--radius-sm)] bg-[var(--danger)]/10 px-3 py-2"
          >
            <span class="truncate text-[11px] font-medium text-[var(--danger)]">
              Auth error
            </span>
            <span
              class="text-[10px] leading-tight text-[var(--soft-foreground)]"
            >
              {authState.error}
            </span>
            <button
              type="button"
              class="mt-1 text-[10px] font-medium text-[var(--accent-strong)] hover:underline"
              onclick={handleSignIn}
            >
              Retry sign in
            </button>
          </div>
        {:else if authState.current.authState === "authenticated"}
          <div
            class="flex items-center gap-2 rounded-[var(--radius-sm)] bg-[var(--accent-wash)] px-3 py-2"
          >
            <div class="flex min-w-0 flex-1 flex-col">
              <span
                class="truncate text-[12px] font-medium text-[var(--accent-strong)]"
              >
                {authState.current.email ?? "Signed in"}
              </span>
            </div>
            <button
              type="button"
              class="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded text-[var(--soft-foreground)] transition hover:bg-[var(--background)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
              onclick={handleSignOut}
              aria-label="Sign out"
            >
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
              >
                <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
                <polyline points="16 17 21 12 16 7" />
                <line x1="21" y1="12" x2="9" y2="12" />
              </svg>
            </button>
          </div>
        {:else}
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-[var(--radius-sm)] text-[var(--soft-foreground)] opacity-60 transition hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] hover:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 px-3 py-2"
            onclick={handleSignIn}
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="shrink-0"
              aria-hidden="true"
            >
              <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
              <polyline points="10 17 15 12 10 7" />
              <line x1="15" y1="12" x2="3" y2="12" />
            </svg>
            <span class="text-[12px] font-medium">Sign in</span>
          </button>
        {/if}
      </div>
    {/if}

    {#if collapsed}
      <span class="sr-only">
        &copy; {new Date().getFullYear()} Thorben Woelk.
      </span>
    {:else}
      <div class="min-w-0 px-3 py-2">
        <span
          class="inline-block shrink-0 whitespace-nowrap text-[12px] font-medium leading-snug text-[var(--soft-foreground)] opacity-60"
        >
          &copy; {new Date().getFullYear()} Thorben Woelk.
        </span>
      </div>
    {/if}
  </div>
</aside>
