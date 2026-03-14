<script lang="ts">
  import { onMount } from "svelte";

  import {
    getSectionNavigationItems,
    type SectionNavigationItem,
    type SectionNavigationSection,
  } from "$lib/section-navigation";

  type Props = {
    currentSection: SectionNavigationSection;
    docsUrl: string;
  };

  let { currentSection, docsUrl }: Props = $props();

  let open = $state(false);
  let button = $state<HTMLButtonElement | null>(null);
  let panel = $state<HTMLDivElement | null>(null);
  let items = $derived(getSectionNavigationItems(currentSection, docsUrl));
  let currentItem = $derived(
    items.find((item) => item.section === currentSection) ?? items[0],
  );

  function toggle() {
    open = !open;
  }

  function close() {
    open = false;
  }

  function itemClass(item: SectionNavigationItem): string {
    return item.active
      ? "bg-[var(--muted)] text-[var(--foreground)]"
      : "text-[var(--soft-foreground)] opacity-80 hover:bg-[var(--muted)]/55 hover:text-[var(--foreground)]";
  }

  function pillClass(item: SectionNavigationItem): string {
    return item.active
      ? "bg-[var(--muted)] text-[var(--foreground)]"
      : "text-[var(--soft-foreground)] opacity-50 hover:opacity-100";
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") close();
  }

  onMount(() => {
    const handlePointerDown = (event: PointerEvent) => {
      if (!open) return;

      const target = event.target as Node;
      if (button?.contains(target) || panel?.contains(target)) return;

      close();
    };

    document.addEventListener("pointerdown", handlePointerDown);
    return () => document.removeEventListener("pointerdown", handlePointerDown);
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="lg:hidden">
  <div
    class="mobile-bottom-stack-offset fixed left-1/2 z-[70] -translate-x-1/2"
  >
    {#if open}
      <div
        bind:this={panel}
        id="section-navigation-menu"
        role="menu"
        aria-label="Sections"
        class="absolute bottom-full left-1/2 z-[70] mb-2 flex w-[min(90vw,16rem)] -translate-x-1/2 flex-col gap-1 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-strong)] p-1.5 shadow-xl fade-in"
      >
        {#each items as item (item.section)}
          <a
            href={item.href}
            target={item.external ? "_blank" : undefined}
            rel={item.external ? "noopener noreferrer" : undefined}
            role="menuitem"
            class={`rounded-[var(--radius-sm)] px-3 py-2 text-[12px] font-semibold transition-colors ${itemClass(item)}`}
            aria-current={item.active ? "page" : undefined}
            onclick={close}
          >
            {item.label}
          </a>
        {/each}
      </div>
    {/if}

    <button
      bind:this={button}
      type="button"
      class="inline-flex h-10 min-w-[9.5rem] max-w-[calc(100vw-2rem)] items-center justify-center gap-2 rounded-full border border-[var(--border-soft)] bg-[var(--surface-strong)] px-4 text-[12px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)] shadow-lg transition-colors hover:border-[var(--border)]"
      aria-expanded={open}
      aria-haspopup="menu"
      aria-controls="section-navigation-menu"
      aria-label="Open section navigation"
      onclick={toggle}
    >
      <span class="min-w-0 truncate">{currentItem.label}</span>
      <svg
        class={`h-3 w-3 shrink-0 transition-transform ${open ? "rotate-180" : ""}`}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="3"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="m6 9 6 6 6-6" />
      </svg>
    </button>
  </div>
</div>

<nav
  class="hidden items-center gap-0.5 lg:flex"
  aria-label="Workspace sections"
>
  {#each items as item (item.section)}
    <a
      href={item.href}
      target={item.external ? "_blank" : undefined}
      rel={item.external ? "noopener noreferrer" : undefined}
      class={`rounded-full px-3.5 py-1.5 text-[11px] font-bold uppercase tracking-[0.1em] transition-all ${pillClass(item)}`}
      aria-current={item.active ? "page" : undefined}
    >
      {item.label}
    </a>
  {/each}
</nav>
