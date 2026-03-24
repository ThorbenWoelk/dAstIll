<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount, tick } from "svelte";
  import { get } from "svelte/store";
  import { fade } from "svelte/transition";

  import { DOCS_URL } from "$lib/app-config";
  import KeyboardShortcutsModal from "$lib/components/KeyboardShortcutsModal.svelte";
  import {
    GO_SEQUENCE_HINTS,
    armGoSequence,
    clearGoSequence,
    focusMainContentRegion,
    shouldIgnoreGlobalShortcutNavigation,
    type GoSequenceState,
  } from "$lib/utils/keyboard-shortcuts";

  let showManual = $state(false);
  let showGoHints = $state(false);
  let goHintsLayoutStyle = $state("");
  const goState: GoSequenceState = { pending: false, timeoutId: null };

  /** Places the popover beside the desktop section rail or above the mobile tab bar. */
  function computeGoHintsLayoutStyle(): string {
    const lg = window.matchMedia("(min-width: 1024px)").matches;
    const rail = document.getElementById("app-section-nav-rail");
    const mobile = document.getElementById("app-section-nav-mobile");
    const railRect = rail?.getBoundingClientRect();
    const mobileRect = mobile?.getBoundingClientRect();
    const gap = 10;
    const vw = window.innerWidth;
    const vh = window.innerHeight;

    const railOk =
      Boolean(lg && railRect && railRect.width >= 32 && railRect.height >= 48);

    const mobileOk = Boolean(
      mobileRect && mobileRect.width >= 80 && mobileRect.height >= 16,
    );

    if (railOk && railRect) {
      const maxW = Math.max(200, vw - railRect.right - gap * 2);
      const centerY = railRect.top + railRect.height / 2;
      const top = Math.round(
        Math.min(Math.max(12, centerY), vh - 12),
      );
      return [
        `left:${Math.round(railRect.right + gap)}px`,
        `top:${top}px`,
        "transform:translateY(-50%)",
        `width:min(18rem,${Math.round(maxW)}px)`,
        "max-height:min(24rem,calc(100vh - 24px))",
      ].join(";");
    }

    if (mobileOk && mobileRect) {
      const bottom = Math.round(vh - mobileRect.top + gap);
      return [
        "left:50%",
        `bottom:${bottom}px`,
        "transform:translateX(-50%)",
        "width:min(22rem,calc(100vw - 2rem))",
      ].join(";");
    }

    return [
      "left:50%",
      "transform:translateX(-50%)",
      "bottom:calc(var(--mobile-tab-bar-height) + var(--space-md))",
      "width:min(22rem,calc(100vw - 2rem))",
    ].join(";");
  }

  $effect(() => {
    if (!showGoHints || typeof document === "undefined") {
      goHintsLayoutStyle = "";
      return;
    }

    const update = () => {
      goHintsLayoutStyle = computeGoHintsLayoutStyle();
    };

    update();
    void tick().then(update);
    window.addEventListener("resize", update);
    const mq = window.matchMedia("(min-width: 1024px)");
    mq.addEventListener("change", update);

    return () => {
      window.removeEventListener("resize", update);
      mq.removeEventListener("change", update);
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
          focusMainContentRegion();
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
    class="pointer-events-none fixed z-[105]"
    style={goHintsLayoutStyle}
    transition:fade={{ duration: 200 }}
    role="status"
    aria-live="polite"
    aria-atomic="true"
  >
    <div
      class="pointer-events-none max-h-[min(24rem,calc(100vh-24px))] overflow-y-auto rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-frost)] px-4 py-3 shadow-[var(--shadow-soft)] backdrop-blur-[10px]"
    >
      <p
        class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-80"
      >
        Go to
      </p>
      <ul class="mt-2.5 grid grid-cols-2 gap-x-4 gap-y-2 sm:grid-cols-3">
        {#each GO_SEQUENCE_HINTS as hint (hint.key)}
          <li class="flex items-center gap-2">
            <kbd
              class="inline-flex min-w-[1.75rem] justify-center rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--muted)] px-1.5 py-0.5 text-[11px] font-semibold tabular-nums text-[var(--foreground)]"
              >{hint.key}</kbd
            >
            <span class="text-[12px] font-medium text-[var(--foreground)]"
              >{hint.label}</span
            >
          </li>
        {/each}
      </ul>
      <p
        class="mt-3 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] opacity-55"
      >
        Esc to cancel
      </p>
    </div>
  </div>
{/if}

<KeyboardShortcutsModal
  open={showManual}
  onClose={() => (showManual = false)}
/>
