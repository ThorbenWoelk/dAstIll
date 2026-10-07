<script lang="ts">
  import type { StoryPosition, TurnDirection } from "$lib/edition/stories";

  let {
    position,
    onTurn,
  }: {
    position: StoryPosition;
    onTurn: (direction: TurnDirection) => void;
  } = $props();
</script>

<nav class="pager" aria-label="Stories in this section">
  <button
    type="button"
    class="turn"
    disabled={!position.previous}
    aria-label="Previous story"
    title="Previous story (←)"
    onclick={() => onTurn("previous")}
  >
    <span class="arrow" aria-hidden="true">‹</span> Previous
  </button>
  <p class="label place">{position.number} of {position.total}</p>
  <button
    type="button"
    class="turn"
    disabled={!position.next}
    aria-label="Next story"
    title="Next story (→)"
    onclick={() => onTurn("next")}
  >
    Next <span class="arrow" aria-hidden="true">›</span>
  </button>
</nav>

<style>
  .pager {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    border-bottom: 1px solid var(--hairline);
  }

  .turn {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    min-width: var(--touch);
    min-height: var(--touch);
    padding: 0;
    border: 0;
    background: none;
    font-family: var(--sans);
    font-size: 12px;
    font-weight: 600;
    color: var(--ink-soft);
  }

  .turn:hover:not(:disabled) {
    color: var(--ink);
  }

  /* Keeps its place so the count stays centered. */
  .turn:disabled {
    visibility: hidden;
  }

  .arrow {
    font-size: 18px;
    line-height: 1;
  }

  .place {
    margin: 0;
    color: var(--ink-faint);
  }
</style>
