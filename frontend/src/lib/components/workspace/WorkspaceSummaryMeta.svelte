<script lang="ts">
  import { clickOutside } from "$lib/actions/click-outside";
  import { renderMarkdown } from "../../utils/markdown";
  import { fly } from "svelte/transition";

  let {
    score = null,
    note = null,
    modelUsed = null,
    qualityModelUsed = null,
    tags = [],
    tagsEvaluated = false,
    compact = false,
  }: {
    score?: number | null;
    note?: string | null;
    modelUsed?: string | null;
    qualityModelUsed?: string | null;
    tags?: string[];
    tagsEvaluated?: boolean;
    compact?: boolean;
  } = $props();

  let drawerOpen = $state(false);
  let triggerEl = $state<HTMLButtonElement | null>(null);

  const displayScore = $derived(
    score !== null
      ? Number.isInteger(score)
        ? String(score)
        : score.toFixed(1)
      : null,
  );
  const trimmedNote = $derived(note?.trim() || null);
  const displayTags = $derived(tags.filter((tag) => tag.trim().length > 0));
  const showLoadingState = $derived(
    displayScore !== null &&
      !trimmedNote &&
      displayTags.length === 0 &&
      !tagsEvaluated,
  );
  const showEmptyState = $derived(
    displayScore !== null &&
      !trimmedNote &&
      displayTags.length === 0 &&
      tagsEvaluated,
  );

  function toggleDrawer() {
    if (displayScore !== null) drawerOpen = !drawerOpen;
  }

  function closeDrawer() {
    drawerOpen = false;
  }

  function handleDrawerClickOutside(event: PointerEvent) {
    if (
      triggerEl &&
      event.target instanceof Node &&
      triggerEl.contains(event.target)
    ) {
      return;
    }
    closeDrawer();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (drawerOpen && e.key === "Escape") {
      e.preventDefault();
      closeDrawer();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if compact}
  {#if displayScore !== null}
    <button
      bind:this={triggerEl}
      type="button"
      class="meta-score-pill"
      onclick={toggleDrawer}
      aria-expanded={drawerOpen}
      aria-controls="summary-quality-note"
      title={drawerOpen ? "Hide evaluation" : "Show evaluation"}
    >
      <span class="meta-score-pill-value">{displayScore}</span>
      <span class="meta-score-pill-label">Quality</span>
    </button>
  {:else}
    <span
      class="meta-score-pill meta-score-pill--pending"
      role="status"
      aria-live="polite"
    >
      <span class="meta-score-dot"></span>
      <span class="meta-score-pill-label">Evaluating</span>
    </span>
  {/if}
{:else}
  <div class="summary-meta-gutter" role="status" aria-live="polite">
    {#if displayScore !== null}
      <button
        bind:this={triggerEl}
        type="button"
        class="meta-score-block"
        onclick={toggleDrawer}
        aria-expanded={drawerOpen}
        aria-controls="summary-quality-note"
        title={drawerOpen ? "Hide evaluation" : "Show evaluation"}
      >
        <span class="meta-score-value">{displayScore}</span>
        <span class="meta-score-label">Quality</span>
      </button>
    {:else}
      <div class="meta-score-block">
        <span class="meta-score-value meta-score-pending">
          <span class="meta-score-dot"></span>
        </span>
        <span class="meta-score-label">Evaluating</span>
      </div>
    {/if}
  </div>
{/if}

{#if drawerOpen && displayScore !== null}
  <div
    class="eval-drawer"
    id="summary-quality-note"
    role="complementary"
    aria-label="Quality evaluation"
    use:clickOutside={{
      enabled: drawerOpen,
      onClickOutside: handleDrawerClickOutside,
    }}
    transition:fly={{ x: 320, duration: 200 }}
  >
    <header class="eval-drawer-header">
      <div>
        <p class="eval-drawer-eyebrow">Evaluation</p>
        <p class="eval-drawer-score">
          {displayScore}<span class="eval-drawer-score-max">/10</span>
        </p>
      </div>
      <button
        type="button"
        class="eval-drawer-close"
        onclick={closeDrawer}
        aria-label="Close evaluation"
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
        >
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </header>

    <div class="eval-drawer-body">
      {#if trimmedNote}
        <div class="eval-note-markdown">
          {@html renderMarkdown(trimmedNote)}
        </div>
      {/if}

      {#if displayTags.length > 0}
        <div class="eval-tag-list" aria-label="Evaluation tags">
          {#each displayTags as tag (tag)}
            <span class="eval-tag-chip">{tag}</span>
          {/each}
        </div>
      {/if}

      {#if showLoadingState}
        <p class="eval-drawer-status">
          Detailed evaluation is still loading. The score is available, but the
          note has not landed yet.
        </p>
      {:else if showEmptyState}
        <p class="eval-drawer-status">
          This evaluation only returned a score and metadata. No detailed note
          was generated for this summary.
        </p>
      {/if}
    </div>

    <footer class="eval-drawer-footer">
      {#if qualityModelUsed}
        <span class="eval-drawer-meta">Eval by {qualityModelUsed}</span>
      {/if}
      {#if modelUsed}
        <span class="eval-drawer-meta">Distilled by {modelUsed}</span>
      {/if}
    </footer>
  </div>
{/if}

<style>
  .meta-score-pill {
    display: inline-flex;
    align-items: baseline;
    gap: 0.35rem;
    border: none;
    background: none;
    padding: 0;
    margin: 0;
    color: inherit;
    font-family: inherit;
    cursor: pointer;
    line-height: 1;
    transition: opacity 0.15s ease;
  }

  .meta-score-pill:hover {
    opacity: 0.7;
  }

  .meta-score-pill--pending {
    cursor: default;
  }

  .meta-score-pill--pending:hover {
    opacity: 1;
  }

  .meta-score-pill-value {
    font-weight: 600;
    color: var(--foreground);
    font-size: 0.78rem;
    letter-spacing: -0.01em;
  }

  .meta-score-pill-label {
    color: inherit;
  }

  .summary-meta-gutter {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.85rem;
  }

  .meta-score-block {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.15rem;
    min-width: 5rem;
    border: none;
    border-left: 2px solid var(--accent-border-soft);
    padding: 0.2rem 0 0.2rem 1rem;
    background: none;
    cursor: default;
    text-align: right;
    font-family: inherit;
    color: inherit;
    transition: opacity 0.15s ease;
  }

  button.meta-score-block {
    cursor: pointer;
  }

  button.meta-score-block:hover {
    opacity: 0.7;
  }

  .meta-score-value {
    font-family: "Fraunces", serif;
    font-size: 3.05rem;
    font-weight: 300;
    line-height: 1;
    letter-spacing: -0.03em;
    color: var(--foreground);
    font-variation-settings: "opsz" 72;
  }

  .meta-score-pending {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    height: 2.5rem;
  }

  .meta-score-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--soft-foreground);
    opacity: 0.3;
    animation: pulse-dot 1.5s ease-in-out infinite;
  }

  @keyframes pulse-dot {
    0%,
    100% {
      opacity: 0.2;
    }
    50% {
      opacity: 0.5;
    }
  }

  .meta-score-label {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--soft-foreground);
    opacity: 0.5;
    margin-top: 2px;
  }

  .eval-drawer {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 340px;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-left: 1px solid var(--border-soft);
    box-shadow:
      -4px 0 16px rgba(0, 0, 0, 0.06),
      -1px 0 4px rgba(0, 0, 0, 0.03);
    z-index: var(--z-mobile-sheet);
    overflow-y: auto;
  }

  .eval-drawer-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
    padding: 1.25rem 1.25rem 0.75rem;
  }

  .eval-drawer-eyebrow {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--soft-foreground);
    opacity: 0.6;
  }

  .eval-drawer-score {
    font-family: "Fraunces", serif;
    font-size: 2rem;
    font-weight: 300;
    line-height: 1.1;
    letter-spacing: -0.03em;
    color: var(--foreground);
    font-variation-settings: "opsz" 72;
    margin-top: 0.25rem;
  }

  .eval-drawer-score-max {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--soft-foreground);
    opacity: 0.5;
    letter-spacing: 0;
    font-family: "Manrope", system-ui, sans-serif;
    margin-left: 2px;
  }

  .eval-drawer-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: none;
    background: none;
    color: var(--soft-foreground);
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }

  .eval-drawer-close:hover {
    background: var(--accent-wash);
    color: var(--foreground);
  }

  .eval-drawer-body {
    flex: 1;
    padding: 0 1.25rem 1.25rem;
  }

  .eval-tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    margin-top: 1rem;
  }

  .eval-tag-chip {
    display: inline-flex;
    align-items: center;
    min-height: 1.85rem;
    padding: 0.28rem 0.75rem;
    border-radius: 9999px;
    background: color-mix(in srgb, var(--surface) 88%, var(--accent-soft));
    border: 1px solid var(--accent-border-soft);
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    color: var(--foreground);
  }

  .eval-note-markdown {
    font-size: 13px;
    line-height: 1.65;
    color: var(--foreground);
    opacity: 0.8;
  }

  .eval-drawer-status {
    margin: 1rem 0 0;
    font-size: 13px;
    line-height: 1.65;
    color: var(--soft-foreground);
  }

  .eval-note-markdown :global(ul) {
    list-style-type: disc;
    margin-left: 1.25rem;
    margin-top: 0.25rem;
    margin-bottom: 0.5rem;
  }

  .eval-note-markdown :global(li) {
    margin-bottom: 0.125rem;
  }

  .eval-note-markdown :global(strong) {
    display: block;
    margin-top: 0.5rem;
    font-weight: 700;
    text-transform: uppercase;
    font-size: 9px;
    letter-spacing: 0.06em;
    color: var(--soft-foreground);
  }

  .eval-note-markdown :global(p) {
    margin-bottom: 0.25rem;
  }

  .eval-drawer-footer {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    padding: 0.75rem 1.25rem;
    border-top: 1px solid var(--border-soft);
  }

  .eval-drawer-meta {
    font-size: 10px;
    color: var(--soft-foreground);
    opacity: 0.45;
  }

  @media (max-width: 1023px) {
    .summary-meta-gutter {
      align-items: flex-start;
      margin-bottom: 0;
    }

    .meta-score-block {
      flex-direction: row;
      align-items: center;
      gap: 0.5rem;
      min-width: 0;
      border: 1px solid var(--accent-border-soft);
      border-left-width: 1px;
      border-radius: 9999px;
      background: var(--surface);
      padding: 0.55rem 0.9rem;
      text-align: left;
    }

    .meta-score-value {
      font-size: 1.6rem;
    }

    .meta-score-pending {
      height: auto;
    }

    .meta-score-label {
      margin-top: 0;
    }

    .eval-drawer {
      top: auto;
      left: 0;
      width: 100%;
      max-height: 60vh;
      border-left: none;
      border-top: 1px solid var(--border-soft);
      border-radius: var(--radius-lg) var(--radius-lg) 0 0;
      box-shadow:
        0 -4px 16px rgba(0, 0, 0, 0.06),
        0 -1px 4px rgba(0, 0, 0, 0.03);
    }
  }
</style>
