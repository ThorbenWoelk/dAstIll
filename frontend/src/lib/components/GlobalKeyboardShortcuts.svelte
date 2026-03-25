<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount, tick, untrack } from "svelte";
  import { get } from "svelte/store";
  import { fade } from "svelte/transition";

  import { DOCS_URL } from "$lib/app-config";
  import KeyboardShortcutsModal from "$lib/components/KeyboardShortcutsModal.svelte";
  import {
    armGoSequence,
    clearGoSequence,
    computeGoHintBadgeStyles,
    focusSectionTabsNav,
    shouldIgnoreGlobalShortcutNavigation,
    type GoHintBadge,
    type GoSequenceState,
  } from "$lib/utils/keyboard-shortcuts";

  let showManual = $state(false);
  let showGoHints = $state(false);
  let goHintPositions = $state<GoHintBadge[]>([]);
  const goState: GoSequenceState = { pending: false, timeoutId: null };

  $effect(() => {
    if (!showGoHints || typeof document === "undefined") {
      untrack(() => {
        goHintPositions = [];
      });
      return;
    }

    const sync = () => {
      untrack(() => {
        goHintPositions = computeGoHintBadgeStyles();
      });
    };

    sync();
    void tick().then(sync);
    const id = requestAnimationFrame(sync);
    window.addEventListener("resize", sync);
    window.addEventListener("scroll", sync, true);
    const mq = window.matchMedia("(min-width: 1024px)");
    mq.addEventListener("change", sync);

    return () => {
      cancelAnimationFrame(id);
      window.removeEventListener("resize", sync);
      window.removeEventListener("scroll", sync, true);
      mq.removeEventListener("change", sync);
    };
  });

  function dismissGoSequence() {
    clearGoSequence(goState);
    showGoHints = false;
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (showManual) {
      if (event.key === "Escape") {
        event.preventDefault();
        showManual = false;
      }
      return;
    }

    const target = event.target;
    if (shouldIgnoreGlobalShortcutNavigation(target)) {
      return;
    }

    if (
      (event.metaKey || event.ctrlKey) &&
      !event.altKey &&
      event.key === "/"
    ) {
      event.preventDefault();
      showManual = true;
      return;
    }

    if (
      event.key === "?" &&
      !event.metaKey &&
      !event.ctrlKey &&
      !event.altKey
    ) {
      event.preventDefault();
      showManual = true;
      return;
    }

    const pathname = get(page).url.pathname;

    if (
      pathname.startsWith("/chat") &&
      event.shiftKey &&
      (event.metaKey || event.ctrlKey) &&
      event.key.toLowerCase() === "n"
    ) {
      event.preventDefault();
      window.dispatchEvent(new CustomEvent("dastill:chat-new-conversation"));
      return;
    }

    if (
      event.key === "/" &&
      !event.metaKey &&
      !event.ctrlKey &&
      !event.altKey &&
      !event.shiftKey
    ) {
      if (pathname === "/") {
        event.preventDefault();
        window.dispatchEvent(
          new CustomEvent("dastill:focus-search", {
            detail: { mode: "search" as const },
          }),
        );
        return;
      }
      if (pathname.startsWith("/chat")) {
        event.preventDefault();
        window.dispatchEvent(new CustomEvent("dastill:chat-focus-composer"));
        return;
      }
    }

    const k = event.key.toLowerCase();

    if (goState.pending) {
      if (event.key === "Escape") {
        event.preventDefault();
        dismissGoSequence();
        return;
      }
      const goActions: Record<string, () => void> = {
        w: () => {
          void goto("/");
        },
        q: () => {
          void goto("/download-queue");
        },
        h: () => {
          void goto("/highlights");
        },
        c: () => {
          void goto("/chat");
        },
        d: () => {
          window.open(DOCS_URL, "_blank", "noopener,noreferrer");
        },
        m: () => {
          focusSectionTabsNav();
        },
        u: () => {
          window.dispatchEvent(new CustomEvent("dastill:open-guide"));
        },
      };
      const action = goActions[k];
      dismissGoSequence();
      if (action) {
        event.preventDefault();
        action();
      }
      return;
    }

    if (
      k === "g" &&
      !event.metaKey &&
      !event.ctrlKey &&
      !event.altKey &&
      !event.shiftKey
    ) {
      event.preventDefault();
      armGoSequence(goState, () => {
        showGoHints = false;
      });
      showGoHints = true;
      return;
    }

    if (event.key === "Escape") {
      dismissGoSequence();
    }
  }

  onMount(() => {
    const openManual = () => {
      showManual = true;
    };
    window.addEventListener("keydown", handleWindowKeydown);
    window.addEventListener("dastill:open-shortcuts", openManual);
    return () => {
      window.removeEventListener("keydown", handleWindowKeydown);
      window.removeEventListener("dastill:open-shortcuts", openManual);
      dismissGoSequence();
    };
  });
</script>

{#if showGoHints}
  <div
    class="pointer-events-none fixed inset-0 z-[105]"
    transition:fade={{ duration: 160 }}
    role="status"
    aria-live="polite"
    aria-label="Go navigation: press a highlighted letter"
  >
    {#each goHintPositions as hint, i (`${hint.key}-${i}`)}
      <kbd
        class="fixed z-[106] inline-flex min-h-[1.5rem] min-w-[1.5rem] items-center justify-center rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] bg-[var(--surface-frost)] px-1.5 py-0.5 text-[11px] font-bold tabular-nums text-[var(--accent-strong)] shadow-[var(--shadow-soft)] backdrop-blur-[10px]"
        style={hint.style}
        transition:fade={{ duration: 140 }}>{hint.key}</kbd
      >
    {/each}
    <p
      class="fixed bottom-[max(1rem,calc(var(--mobile-tab-bar-height,0px)+0.75rem))] left-1/2 z-[106] -translate-x-1/2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] opacity-70"
    >
      Esc to cancel
    </p>
  </div>
{/if}

<KeyboardShortcutsModal
  open={showManual}
  onClose={() => (showManual = false)}
/>
