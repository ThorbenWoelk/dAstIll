<script lang="ts">
  import type { Snippet } from "svelte";
  import type { CreateHighlightRequest, Highlight } from "$lib/api";
  import HighlightIcon from "$lib/components/icons/HighlightIcon.svelte";
  import {
    buildHighlightDraft,
    placeHighlights,
  } from "$lib/highlights/anchoring";
  import {
    clearHighlightMarks,
    highlightIdAt,
    markHighlights,
    readPassage,
    spanOfRange,
  } from "$lib/highlights/passage";

  let {
    highlights = [],
    onHighlight,
    onRemoveHighlight,
    children,
  }: {
    highlights?: Highlight[];
    /** Omit to turn highlighting off. */
    onHighlight?: (draft: CreateHighlightRequest) => void;
    onRemoveHighlight?: (highlightId: string) => void;
    children: Snippet;
  } = $props();

  const TOOLBAR_HALF_WIDTH = 90;

  let root = $state<HTMLElement | null>(null);
  /** The selection the reader may turn into a highlight. */
  let selection = $state<{
    draft: CreateHighlightRequest;
    top: number;
    left: number;
  } | null>(null);
  /** The highlight the reader tapped, offered for removal. */
  let tapped = $state<{ id: string; top: number; left: number } | null>(null);

  $effect(() => {
    const element = root;
    const current = highlights;
    if (!element) return;
    clearHighlightMarks(element);
    const passage = readPassage(element);
    markHighlights(passage, placeHighlights(passage.text, current));
  });

  $effect(() => {
    if (tapped && !highlights.some((h) => h.id === tapped?.id)) tapped = null;
  });

  /** Position for a toolbar just below `rect`, inside the passage. */
  function below(rect: DOMRect) {
    const box = root!.getBoundingClientRect();
    const half = Math.min(TOOLBAR_HALF_WIDTH, box.width / 2);
    const center = rect.left + rect.width / 2 - box.left;
    return {
      top: rect.bottom - box.top + 8,
      left: Math.min(Math.max(center, half), box.width - half),
    };
  }

  /**
   * On phones a tap clears the text selection before the click arrives.
   * While the Highlight button is pressed, keep it and its draft on screen.
   */
  let pressingToolbar = false;
  let releaseTimer: ReturnType<typeof setTimeout> | undefined;

  function pressToolbar(event: PointerEvent) {
    event.preventDefault();
    clearTimeout(releaseTimer);
    pressingToolbar = true;
  }

  function releaseToolbar() {
    clearTimeout(releaseTimer);
    // The click follows pointerup; resync only if it never comes.
    releaseTimer = setTimeout(() => {
      pressingToolbar = false;
      readSelection();
    }, 400);
  }

  function readSelection() {
    if (!onHighlight || !root || pressingToolbar) return;
    const current = document.getSelection();
    if (!current || current.isCollapsed || current.rangeCount === 0) {
      selection = null;
      return;
    }
    const range = current.getRangeAt(0);
    const passage = readPassage(root);
    const span = spanOfRange(passage, root, range);
    const draft = span
      ? buildHighlightDraft(passage.text, span.start, span.end)
      : null;
    if (!draft) {
      selection = null;
      return;
    }
    tapped = null;
    selection = { draft, ...below(range.getBoundingClientRect()) };
  }

  function highlightSelection() {
    clearTimeout(releaseTimer);
    pressingToolbar = false;
    if (!selection || !onHighlight) return;
    onHighlight(selection.draft);
    document.getSelection()?.removeAllRanges();
    selection = null;
  }

  function handleClick(event: MouseEvent) {
    if (!onRemoveHighlight) return;
    const id = highlightIdAt(event.target);
    if (!id || !document.getSelection()?.isCollapsed) {
      tapped = null;
      return;
    }
    const mark = (event.target as Element).closest("mark")!;
    tapped = { id, ...below(mark.getBoundingClientRect()) };
  }

  function removeTapped() {
    if (!tapped || !onRemoveHighlight) return;
    onRemoveHighlight(tapped.id);
    tapped = null;
  }

  function handleKey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      selection = null;
      tapped = null;
    }
  }
</script>

<svelte:document onselectionchange={readSelection} onkeydown={handleKey} />

<!--
  Text the reader can highlight. Saved highlights show as marks; selecting
  text offers Highlight, and tapping a mark offers to remove it. Children
  marked `data-passage-skip` are left out.
-->
<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="passage" bind:this={root} onclick={handleClick}>
  {@render children()}

  {#if selection}
    <div
      class="toolbar"
      style:top="{selection.top}px"
      style:left="{selection.left}px"
      data-passage-skip
    >
      <button
        type="button"
        class="press"
        onpointerdown={pressToolbar}
        onpointerup={releaseToolbar}
        onpointercancel={releaseToolbar}
        onclick={highlightSelection}
      >
        <HighlightIcon />
        Highlight
      </button>
    </div>
  {:else if tapped}
    <div
      class="toolbar"
      style:top="{tapped.top}px"
      style:left="{tapped.left}px"
      data-passage-skip
    >
      <button type="button" class="press" onclick={removeTapped}>
        Remove highlight
      </button>
    </div>
  {/if}
</div>

<style>
  .passage {
    position: relative;
  }

  .passage :global(mark.reader-highlight) {
    padding: 0.05em 0;
    background: var(--marker);
    color: inherit;
    cursor: pointer;
    box-decoration-break: clone;
    -webkit-box-decoration-break: clone;
  }

  .toolbar {
    position: absolute;
    z-index: 2;
    transform: translateX(-50%);
  }

  .toolbar .press {
    min-height: var(--touch);
    padding: 0 var(--space-4);
    font-size: 12px;
  }
</style>
