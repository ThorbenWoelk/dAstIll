<script lang="ts">
  import type { Snippet } from "svelte";
  import type {
    CreateHighlightRequest,
    CreateWritingFlagRequest,
    Highlight,
    WritingFlag,
  } from "$lib/api";
  import HighlightIcon from "$lib/components/icons/HighlightIcon.svelte";
  import WritingFlagForm from "$lib/components/WritingFlagForm.svelte";
  import {
    buildHighlightDraft,
    combineMarks,
    placeHighlights,
    type MarkKind,
  } from "$lib/highlights/anchoring";
  import {
    clearHighlightMarks,
    markAt,
    markHighlights,
    readPassage,
    spanOfRange,
  } from "$lib/highlights/passage";

  let {
    highlights = [],
    flags = [],
    onHighlight,
    onRemoveHighlight,
    onFlag,
    onRemoveFlag,
    children,
  }: {
    highlights?: Highlight[];
    /** Passages the reader flagged as bad writing. */
    flags?: WritingFlag[];
    /** Omit to turn highlighting off. */
    onHighlight?: (draft: CreateHighlightRequest) => void;
    onRemoveHighlight?: (highlightId: string) => void;
    /** Omit to turn flagging off. */
    onFlag?: (draft: CreateWritingFlagRequest) => void;
    onRemoveFlag?: (flagId: string) => void;
    children: Snippet;
  } = $props();

  /** Half the widest toolbar: one button, or two and the flag form. */
  const toolbarHalfWidth = $derived(onFlag ? 160 : 90);

  let root = $state<HTMLElement | null>(null);
  /** The selection the reader may turn into a highlight. */
  let selection = $state<{
    draft: CreateHighlightRequest;
    top: number;
    left: number;
  } | null>(null);
  /** The highlight or flag the reader tapped, offered for removal. */
  let tapped = $state<{
    kind: MarkKind;
    id: string;
    top: number;
    left: number;
  } | null>(null);
  /** The selection the reader is writing a flag note for. */
  let flagging = $state<{
    draft: CreateHighlightRequest;
    top: number;
    left: number;
  } | null>(null);
  const tappedFlag = $derived(
    tapped?.kind === "flag" ? flags.find((f) => f.id === tapped?.id) : null,
  );

  $effect(() => {
    const element = root;
    const currentHighlights = highlights;
    const currentFlags = flags;
    if (!element) return;
    clearHighlightMarks(element);
    const passage = readPassage(element);
    markHighlights(
      passage,
      combineMarks(
        placeHighlights(passage.text, currentHighlights),
        placeHighlights(passage.text, currentFlags),
      ),
    );
  });

  $effect(() => {
    if (!tapped) return;
    const list: { id: string }[] = tapped.kind === "flag" ? flags : highlights;
    if (!list.some((item) => item.id === tapped?.id)) tapped = null;
  });

  /** Position for a toolbar just below `rect`, inside the passage. */
  function below(rect: DOMRect) {
    const box = root!.getBoundingClientRect();
    const half = Math.min(toolbarHalfWidth, box.width / 2);
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
    if ((!onHighlight && !onFlag) || !root || pressingToolbar || flagging) {
      return;
    }
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

  /** Keeps the selection, and asks why the passage reads badly. */
  function startFlag() {
    clearTimeout(releaseTimer);
    pressingToolbar = false;
    if (!selection || !onFlag) return;
    flagging = selection;
    selection = null;
  }

  function saveFlag(comment: string) {
    if (!flagging || !onFlag) return;
    const { text, prefix_context, suffix_context } = flagging.draft;
    onFlag({ text, comment, prefix_context, suffix_context });
    document.getSelection()?.removeAllRanges();
    flagging = null;
  }

  function cancelFlag() {
    flagging = null;
    readSelection();
  }

  function handleClick(event: MouseEvent) {
    const mark = markAt(event.target);
    const removable = mark?.kind === "flag" ? onRemoveFlag : onRemoveHighlight;
    if (!mark || !removable || !document.getSelection()?.isCollapsed) {
      tapped = null;
      return;
    }
    tapped = {
      kind: mark.kind,
      id: mark.id,
      ...below(mark.element.getBoundingClientRect()),
    };
  }

  function removeTapped() {
    if (!tapped) return;
    if (tapped.kind === "flag") onRemoveFlag?.(tapped.id);
    else onRemoveHighlight?.(tapped.id);
    tapped = null;
  }

  function handleKey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      selection = null;
      tapped = null;
      flagging = null;
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

  {#if flagging}
    <div
      class="toolbar"
      style:top="{flagging.top}px"
      style:left="{flagging.left}px"
      data-passage-skip
    >
      <WritingFlagForm
        passage={flagging.draft.text}
        onSave={saveFlag}
        onCancel={cancelFlag}
      />
    </div>
  {:else if selection}
    <div
      class="toolbar actions"
      style:top="{selection.top}px"
      style:left="{selection.left}px"
      data-passage-skip
    >
      {#if onHighlight}
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
      {/if}
      {#if onFlag}
        <button
          type="button"
          class="press flag"
          onpointerdown={pressToolbar}
          onpointerup={releaseToolbar}
          onpointercancel={releaseToolbar}
          onclick={startFlag}
        >
          Flag writing
        </button>
      {/if}
    </div>
  {:else if tapped}
    <div
      class="toolbar"
      style:top="{tapped.top}px"
      style:left="{tapped.left}px"
      data-passage-skip
    >
      {#if tappedFlag}
        <p class="flag-note">{tappedFlag.comment}</p>
      {/if}
      <button type="button" class="press" onclick={removeTapped}>
        {tapped.kind === "flag" ? "Remove flag" : "Remove highlight"}
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

  .passage :global(mark.writing-flag) {
    padding: 0.05em 0;
    background: none;
    color: inherit;
    text-decoration: underline wavy var(--danger);
    text-decoration-thickness: 1.5px;
    text-underline-offset: 4px;
    cursor: pointer;
  }

  .toolbar {
    position: absolute;
    z-index: 2;
    transform: translateX(-50%);
  }

  .toolbar.actions {
    display: flex;
    gap: 1px;
  }

  .toolbar .press.flag {
    background: var(--danger);
  }

  .flag-note {
    max-width: min(320px, calc(100vw - 2 * var(--space-4)));
    margin: 0 0 1px;
    padding: var(--space-2) var(--space-3);
    background: var(--paper-raised);
    border: 1px solid var(--rule);
    font-family: var(--sans);
    font-size: 13px;
    line-height: 1.4;
  }

  .toolbar .press {
    min-height: var(--touch);
    padding: 0 var(--space-4);
    font-size: 12px;
  }
</style>
