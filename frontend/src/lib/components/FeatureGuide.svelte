<script lang="ts">
  import { tick } from "svelte";
  import { isEditableShortcutTarget } from "$lib/utils/keyboard-shortcuts";

  export type TourStep = {
    selector: string;
    title: string;
    body: string;
    placement?: "top" | "bottom" | "left" | "right";
    /** Runs before measuring targets; may be async (e.g. select a row so the anchor exists). */
    prepare?: () => void | Promise<void>;
    /** @deprecated Prefer `fallbackSelectors` */
    fallbackSelector?: string;
    /** Tried in order after `selector` until an element has a non-empty layout rect. */
    fallbackSelectors?: string[];
  };

  type Props = {
    open: boolean;
    step: number;
    steps: TourStep[];
    docsUrl?: string;
    onClose: () => void;
    onStep: (step: number) => void;
  };

  let { open, step, steps, docsUrl, onClose, onStep }: Props = $props();

  let cardEl = $state<HTMLDivElement | null>(null);
  let cardStyle = $state("");
  let placement = $state<"top" | "bottom" | "left" | "right">("bottom");
  /** Viewport hole (px) so scrim leaves the step target fully bright; null = full dim. */
  let tourSpotlight = $state<{
    top: number;
    left: number;
    right: number;
    bottom: number;
  } | null>(null);

  const PADDING = 8;
  const CARD_GAP = 12;
  const TARGET_CLASS = "tour-step-target";

  function clearTourTarget() {
    for (const node of document.querySelectorAll(`.${TARGET_CLASS}`)) {
      node.classList.remove(TARGET_CLASS);
    }
  }

  function nextStep() {
    if (step < steps.length - 1) {
      onStep(step + 1);
    } else {
      onClose();
    }
  }

  function prevStep() {
    if (step > 0) {
      onStep(step - 1);
    }
  }

  /**
   * Only skip tour Enter when (1) focus is in the card (Next/Back/pips handle Enter natively)
   * or (2) user is typing in a field. Do not treat page buttons/links/tabs as "interactive"
   * for this purpose — those often match the highlighted step and would block Enter otherwise.
   */
  function shouldSuppressEnterForTour(
    eventTarget: EventTarget | null,
  ): boolean {
    if (!(eventTarget instanceof Element)) return false;
    if (eventTarget.closest(".tour-card")) return true;
    if (isEditableShortcutTarget(eventTarget)) return true;
    return false;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    }
    if (event.key === "Enter" && !shouldSuppressEnterForTour(event.target)) {
      event.preventDefault();
      nextStep();
    }
    if (event.key === "ArrowRight" || event.key === "ArrowDown") {
      event.preventDefault();
      nextStep();
    }
    if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
      event.preventDefault();
      prevStep();
    }
  }

  function computePosition(
    targetRect: DOMRect,
    cardRect: DOMRect,
    preferredPlacement: "top" | "bottom" | "left" | "right",
  ): { style: string; placement: "top" | "bottom" | "left" | "right" } {
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    const padded = {
      top: targetRect.top - PADDING,
      left: targetRect.left - PADDING,
      bottom: targetRect.bottom + PADDING,
      right: targetRect.right + PADDING,
      width: targetRect.width + PADDING * 2,
      height: targetRect.height + PADDING * 2,
    };

    // Try preferred placement, then fallback
    const placements: Array<"top" | "bottom" | "left" | "right"> = [
      preferredPlacement,
      "bottom",
      "top",
      "right",
      "left",
    ];

    for (const p of placements) {
      let top: number;
      let left: number;

      if (p === "bottom") {
        top = padded.bottom + CARD_GAP;
        left = padded.left + padded.width / 2 - cardRect.width / 2;
      } else if (p === "top") {
        top = padded.top - CARD_GAP - cardRect.height;
        left = padded.left + padded.width / 2 - cardRect.width / 2;
      } else if (p === "right") {
        top = padded.top + padded.height / 2 - cardRect.height / 2;
        left = padded.right + CARD_GAP;
      } else {
        top = padded.top + padded.height / 2 - cardRect.height / 2;
        left = padded.left - CARD_GAP - cardRect.width;
      }

      // Clamp to viewport
      left = Math.max(12, Math.min(left, vw - cardRect.width - 12));
      top = Math.max(12, Math.min(top, vh - cardRect.height - 12));

      // Check if it fits without overlapping the target
      const cardBottom = top + cardRect.height;
      const cardRight = left + cardRect.width;
      const overlapsTarget =
        left < padded.right &&
        cardRight > padded.left &&
        top < padded.bottom &&
        cardBottom > padded.top;

      if (!overlapsTarget || p === placements[placements.length - 1]) {
        return {
          style: `top:${top}px;left:${left}px`,
          placement: p,
        };
      }
    }

    // Fallback: center
    return {
      style: `top:${vh / 2 - cardRect.height / 2}px;left:${vw / 2 - cardRect.width / 2}px`,
      placement: "bottom",
    };
  }

  function resolveTarget(s: TourStep): Element | null {
    const chain = [
      s.selector,
      s.fallbackSelector,
      ...(s.fallbackSelectors ?? []),
    ].filter((x): x is string => typeof x === "string" && x.length > 0);
    for (const sel of chain) {
      const matches = document.querySelectorAll(sel);
      for (const node of matches) {
        if (node instanceof Element && node.getClientRects().length > 0) {
          return node;
        }
      }
    }
    return null;
  }

  async function positionCard() {
    if (!open || !steps[step]) return;

    const s = steps[step];
    await Promise.resolve(s.prepare?.());
    await tick();
    await tick();
    await new Promise<void>((r) => requestAnimationFrame(() => r()));

    clearTourTarget();
    tourSpotlight = null;

    const el = resolveTarget(s);
    if (!el) {
      // No target found: center card
      if (cardEl) {
        const cr = cardEl.getBoundingClientRect();
        cardStyle = `top:${window.innerHeight / 2 - cr.height / 2}px;left:${window.innerWidth / 2 - cr.width / 2}px`;
      }
      return;
    }

    el.classList.add(TARGET_CLASS);

    // Snap element into view instantly (no smooth scroll lag)
    el.scrollIntoView({ behavior: "instant", block: "nearest" });

    const rect = el.getBoundingClientRect();
    if (cardEl) {
      const cr = cardEl.getBoundingClientRect();
      const result = computePosition(rect, cr, s.placement ?? "bottom");
      cardStyle = result.style;
      placement = result.placement;
    }

    const vw = window.innerWidth;
    const vh = window.innerHeight;
    const l = Math.max(0, rect.left - PADDING);
    const t = Math.max(0, rect.top - PADDING);
    const r = Math.min(vw, rect.right + PADDING);
    const b = Math.min(vh, rect.bottom + PADDING);
    if (r > l && b > t) {
      tourSpotlight = { top: t, left: l, right: r, bottom: b };
    }
  }

  $effect(() => {
    if (open && steps[step]) {
      void positionCard();
    } else if (!open) {
      clearTourTarget();
      tourSpotlight = null;
    }
  });

  $effect(() => {
    if (!open) return;
    const onResize = () => void positionCard();
    window.addEventListener("resize", onResize);
    window.addEventListener("scroll", onResize, true);
    return () => {
      window.removeEventListener("resize", onResize);
      window.removeEventListener("scroll", onResize, true);
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open && steps[step]}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div
    class="tour-root"
    role="dialog"
    aria-modal="true"
    aria-label="Feature guide"
    tabindex="-1"
    onkeydown={(e) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      }
      if (
        (e.key === "Enter" || e.key === " ") &&
        e.target === e.currentTarget
      ) {
        e.preventDefault();
        nextStep();
      }
    }}
  >
    <!-- Four scrim strips leave a true viewport hole so the target stays full brightness (not dimmed by opacity). -->
    {#if tourSpotlight}
      <div
        class="tour-scrim tour-scrim--top"
        style="height:{tourSpotlight.top}px"
        aria-hidden="true"
      ></div>
      <div
        class="tour-scrim tour-scrim--left"
        style="top:{tourSpotlight.top}px;width:{tourSpotlight.left}px;height:{tourSpotlight.bottom -
          tourSpotlight.top}px"
        aria-hidden="true"
      ></div>
      <div
        class="tour-scrim tour-scrim--right"
        style="top:{tourSpotlight.top}px;left:{tourSpotlight.right}px;height:{tourSpotlight.bottom -
          tourSpotlight.top}px"
        aria-hidden="true"
      ></div>
      <div
        class="tour-scrim tour-scrim--bottom"
        style="top:{tourSpotlight.bottom}px"
        aria-hidden="true"
      ></div>
    {:else}
      <div class="tour-scrim tour-scrim--full" aria-hidden="true"></div>
    {/if}

    <!-- Card: use Next / keyboard; backdrop clicks do not advance. -->
    <div class="tour-card" bind:this={cardEl} style={cardStyle}>
      <!-- Step counter + close -->
      <div class="tour-card-header">
        <div class="tour-card-counter">
          {#each steps as _, i}
            <button
              type="button"
              class="tour-pip"
              class:tour-pip--active={i === step}
              class:tour-pip--done={i < step}
              onclick={() => onStep(i)}
              aria-label="Step {i + 1}"
            ></button>
          {/each}
          <span class="tour-step-label">{step + 1}/{steps.length}</span>
        </div>
        <button
          type="button"
          class="tour-close"
          onclick={onClose}
          aria-label="Close guide"
        >
          <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
          >
            <path d="M6 6L18 18"></path>
            <path d="M18 6L6 18"></path>
          </svg>
        </button>
      </div>

      <!-- Content -->
      {#key step}
        <div class="tour-card-body">
          <h3 class="tour-title">{steps[step].title}</h3>
          <p class="tour-body">{steps[step].body}</p>
        </div>
      {/key}

      <!-- Nav -->
      <div class="tour-nav">
        {#if step > 0}
          <button type="button" class="tour-nav-back" onclick={prevStep}>
            Back
          </button>
        {:else}
          <div></div>
        {/if}
        <div class="flex items-center gap-4 ml-auto">
          {#if step === steps.length - 1 && docsUrl}
            <a
              href={docsUrl}
              target="_blank"
              rel="noopener noreferrer"
              class="tour-nav-github"
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
                <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"></path>
                <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"></path>
              </svg>
              Docs
            </a>
          {/if}
          <button type="button" class="tour-nav-next" onclick={nextStep}>
            {step < steps.length - 1 ? "Next" : "Done"}
            {#if step < steps.length - 1}
              <svg
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="3"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="m9 18 6-6-6-6"></path>
              </svg>
            {/if}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .tour-root {
    position: fixed;
    inset: 0;
    z-index: 10000;
    pointer-events: none;
    animation: tour-in 250ms ease forwards;
  }

  .tour-scrim {
    position: absolute;
    pointer-events: auto;
    /* Fixed dark scrim — never use a surface variable here or it will bleed into
       opaque surfaces like the filter popup when someone tweaks surface tokens. */
    background: rgba(0, 0, 0, 0.5);
  }

  .tour-scrim--full {
    inset: 0;
  }

  .tour-scrim--top {
    top: 0;
    left: 0;
    right: 0;
  }

  .tour-scrim--left {
    left: 0;
  }

  .tour-scrim--right {
    right: 0;
  }

  .tour-scrim--bottom {
    left: 0;
    right: 0;
    bottom: 0;
  }

  @keyframes tour-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .tour-card {
    position: fixed;
    z-index: 1;
    pointer-events: auto;
    width: min(360px, calc(100vw - 24px));
    background: var(--surface);
    border-radius: 16px;
    box-shadow:
      0 20px 60px var(--shadow-strong),
      0 0 0 1px var(--border-soft);
    overflow: hidden;
    transition:
      top 180ms cubic-bezier(0.16, 1, 0.3, 1),
      left 180ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .tour-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 14px 0;
  }

  .tour-card-counter {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .tour-pip {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    border: none;
    background: var(--border);
    cursor: pointer;
    padding: 0;
    transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .tour-pip--active {
    width: 18px;
    background: var(--accent);
  }

  .tour-pip--done {
    background: var(--accent);
    opacity: 0.3;
  }

  .tour-step-label {
    font-size: 10px;
    font-weight: 700;
    color: var(--soft-foreground);
    opacity: 0.35;
    margin-left: 6px;
    font-variant-numeric: tabular-nums;
  }

  .tour-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 999px;
    border: none;
    background: none;
    color: var(--soft-foreground);
    opacity: 0.35;
    cursor: pointer;
    transition: all 150ms;
  }

  .tour-close:hover {
    opacity: 1;
    background: var(--muted);
  }

  .tour-card-body {
    padding: 14px 18px 8px;
    animation: tour-step-in 280ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  @keyframes tour-step-in {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }

  .tour-title {
    font-family: "Fraunces", ui-serif, Georgia, serif;
    font-size: 17px;
    font-weight: 700;
    color: var(--foreground);
    margin: 0 0 6px;
    letter-spacing: -0.01em;
    line-height: 1.3;
  }

  .tour-body {
    font-size: 13px;
    line-height: 1.6;
    color: var(--soft-foreground);
    margin: 0;
  }

  .tour-nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px 14px;
  }

  .tour-nav-back,
  .tour-nav-next {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    border: none;
    cursor: pointer;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border-radius: 999px;
    padding: 8px 16px;
    transition: all 180ms;
  }

  .tour-nav-back {
    background: var(--surface-strong);
    color: var(--soft-foreground);
    opacity: 0.5;
  }

  .tour-nav-back:hover {
    opacity: 1;
    background: var(--muted);
  }

  .tour-nav-next {
    background: var(--foreground);
    color: var(--surface);
  }

  .tour-nav-github {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--soft-foreground);
    text-decoration: none;
    opacity: 0.6;
    transition:
      opacity 180ms ease,
      color 180ms ease;
  }

  .tour-nav-github:hover {
    opacity: 1;
    color: var(--foreground);
  }

  .tour-nav-next:hover {
    background: var(--accent-strong);
  }

  @media (max-width: 480px) {
    .tour-card {
      width: calc(100vw - 16px);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .tour-root,
    .tour-card-body,
    .tour-card {
      animation: none !important;
      transition: none !important;
    }
  }

  /* Ring the step target; scrim hole keeps content at full brightness. */
  :global(.tour-step-target) {
    position: relative;
    z-index: 1;
    border-radius: var(--radius-sm);
    box-shadow:
      0 0 0 2px var(--accent),
      0 0 0 4px color-mix(in srgb, var(--accent) 22%, transparent),
      0 12px 40px color-mix(in srgb, var(--foreground) 12%, transparent);
  }
</style>
