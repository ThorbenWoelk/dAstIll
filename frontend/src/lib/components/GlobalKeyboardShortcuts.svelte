<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount, tick, untrack } from "svelte";
  import { get } from "svelte/store";
  import { fade } from "svelte/transition";

  import { DOCS_URL } from "$lib/app-config";
  import KeyboardShortcutsModal from "$lib/components/KeyboardShortcutsModal.svelte";
  import {
    computeGoHintBadgeStyles,
    DASTILL_SET_WORKSPACE_CONTENT_MODE_EVENT,
    shouldIgnoreGlobalShortcutNavigation,
    type GoHintBadge,
  } from "$lib/utils/keyboard-shortcuts";

  let showManual = $state(false);
  let showGoHints = $state(false);
  let goHintPositions = $state<GoHintBadge[]>([]);

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

    // Manual / Help
    if (
      ((event.metaKey || event.ctrlKey) &&
        !event.altKey &&
        event.key === "/") ||
      (event.key === "?" && !event.metaKey && !event.ctrlKey && !event.altKey)
    ) {
      event.preventDefault();
      showManual = true;
      return;
    }

    const pathname = get(page).url.pathname;

    // Chat New Conversation (Shift+Cmd+N)
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

    // Search or Chat focus (/)
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

    // Numerical Sections (Cmd+1..6)
    if ((event.metaKey || event.ctrlKey) && !event.altKey && !event.shiftKey) {
      const num = event.key;
      if (num >= "1" && num <= "9") {
        const actions: Record<string, () => void> = {
          "1": () => {
            void goto("/");
          },
          "2": () => {
            void goto("/download-queue");
          },
          "3": () => {
            void goto("/highlights");
          },
          "4": () => {
            void goto("/vocabulary");
          },
          "5": () => {
            void goto("/chat");
          },
          "6": () => {
            window.open(DOCS_URL, "_blank", "noopener,noreferrer");
          },
          "7": () => {
            window.dispatchEvent(
              new CustomEvent(DASTILL_SET_WORKSPACE_CONTENT_MODE_EVENT, {
                detail: { mode: "info" },
              }),
            );
          },
          "8": () => {
            window.dispatchEvent(
              new CustomEvent(DASTILL_SET_WORKSPACE_CONTENT_MODE_EVENT, {
                detail: { mode: "summary" },
              }),
            );
          },
          "9": () => {
            window.dispatchEvent(
              new CustomEvent(DASTILL_SET_WORKSPACE_CONTENT_MODE_EVENT, {
                detail: { mode: "transcript" },
              }),
            );
          },
        };

        const action = actions[num];
        if (action) {
          event.preventDefault();
          action();
        }
      }

      if (event.key === ",") {
        event.preventDefault();
        window.dispatchEvent(new CustomEvent("dastill:open-settings"));
      }
    }
  }

  onMount(() => {
    const openManual = () => {
      showManual = true;
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Meta" || e.key === "Control") {
        showGoHints = true;
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.key === "Meta" || e.key === "Control") {
        showGoHints = false;
      }
    };

    const handleBlur = () => {
      showGoHints = false;
    };

    window.addEventListener("keydown", handleWindowKeydown);
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    window.addEventListener("blur", handleBlur);
    window.addEventListener("dastill:open-shortcuts", openManual);
    return () => {
      window.removeEventListener("keydown", handleWindowKeydown);
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      window.removeEventListener("blur", handleBlur);
      window.removeEventListener("dastill:open-shortcuts", openManual);
    };
  });
</script>

{#if showGoHints}
  <div
    class="pointer-events-none fixed inset-0 z-[105]"
    transition:fade={{ duration: 160 }}
    role="status"
    aria-live="polite"
    aria-label="Shortcut hints: press Cmd and a number"
  >
    {#each goHintPositions as hint, i (`${hint.key}-${i}`)}
      <kbd
        class="fixed z-[106] inline-flex min-h-[1.5rem] min-w-[1.5rem] items-center justify-center rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] bg-[var(--surface-frost)] px-1.5 py-0.5 text-[11px] font-bold tabular-nums text-[var(--accent-strong)] shadow-[var(--shadow-soft)] backdrop-blur-[10px]"
        style={hint.style}
        transition:fade={{ duration: 140 }}>{hint.key}</kbd
      >
    {/each}
  </div>
{/if}

<KeyboardShortcutsModal
  open={showManual}
  onClose={() => (showManual = false)}
/>
