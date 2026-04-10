<script lang="ts">
  import { page } from "$app/state";
  import { sectionIcon } from "$lib/section-navigation-icons";
  import type { SectionNavigationSection } from "$lib/section-navigation";

  let { currentSection }: { currentSection: SectionNavigationSection } =
    $props();

  const tabs: {
    section: SectionNavigationSection;
    label: string;
    href: string;
  }[] = [
    { section: "workspace", label: "Workspace", href: "/" },
    { section: "queue", label: "Queue", href: "/download-queue" },
    { section: "highlights", label: "Highlights", href: "/highlights" },
    { section: "vocabulary", label: "Vocabulary", href: "/vocabulary" },
    { section: "chat", label: "Chat", href: "/chat" },
  ];

  let keyboardOpen = $state(false);

  $effect(() => {
    if (typeof window === "undefined") return;

    const vv = window.visualViewport;
    if (!vv) return;

    function check() {
      if (!vv) return;
      keyboardOpen = vv.height < window.innerHeight * 0.75;
    }

    check();
    vv.addEventListener("resize", check);
    return () => vv.removeEventListener("resize", check);
  });
</script>

{#if !keyboardOpen}
  <nav
    class="fixed bottom-0 left-0 right-0 z-[60] flex items-stretch border-t border-[var(--border-soft)]/50 bg-[var(--surface)] pb-[max(0.25rem,env(safe-area-inset-bottom))] lg:hidden"
    aria-label="App sections"
  >
    {#each tabs as tab (tab.section)}
      {@const icon = sectionIcon(tab.section)}
      {@const active = currentSection === tab.section}
      <a
        href={tab.href}
        data-sveltekit-preload-code="viewport"
        data-sveltekit-preload-data="tap"
        class={`flex min-h-[52px] flex-1 flex-col items-center justify-center gap-0.5 pt-1.5 transition-colors ${
          active
            ? "text-[var(--accent-strong)]"
            : "text-[var(--soft-foreground)] opacity-60"
        }`}
        aria-current={active ? "page" : undefined}
      >
        <svg
          width="20"
          height="20"
          viewBox={icon.viewBox}
          fill="none"
          stroke="currentColor"
          stroke-width={active ? "2" : "1.7"}
          stroke-linecap="round"
          stroke-linejoin="round"
          class="shrink-0"
          aria-hidden="true"
        >
          {#each icon.paths as d}
            <path {d} />
          {/each}
        </svg>
        <span
          class="text-[10px] font-bold uppercase leading-tight tracking-[0.06em]"
        >
          {tab.label}
        </span>
      </a>
    {/each}
  </nav>
{/if}
