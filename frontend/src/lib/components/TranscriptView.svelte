<script lang="ts">
  import { tick } from "svelte";
  import type {
    CreateHighlightRequest,
    Highlight,
    HighlightSource,
  } from "$lib/types";
  import HighlighterIcon from "$lib/components/icons/HighlighterIcon.svelte";
  import TrashIcon from "$lib/components/icons/TrashIcon.svelte";
  import {
    buildHighlightDraft,
    resolveRangeTextOffsets,
    resolveHighlightRanges,
    resolveTooltipPosition,
  } from "$lib/utils/highlights";
  import { scrollToCitationInArticle } from "$lib/utils/citation-scroll";

  type Props = {
    html?: string;
    text?: string;
    mode?: "markdown" | "plain_text";
    formatting?: boolean;
    highlights?: Highlight[];
    highlightSource?: HighlightSource | null;
    highlightEnabled?: boolean;
    creatingHighlight?: boolean;
    creatingVocabularyReplacement?: boolean;
    deletingHighlightId?: number | null;
    onCreateHighlight?:
      | ((payload: CreateHighlightRequest) => Promise<void> | void)
      | undefined;
    onCreateVocabularyReplacement?:
      | ((selectedText: string) => Promise<void> | void)
      | undefined;
    onDeleteHighlight?:
      | ((highlightId: number) => Promise<void> | void)
      | undefined;
    citationScrollText?: string | null;
    onCitationScrollConsumed?: () => void;
  };

  let {
    html = "",
    text = "",
    mode = "markdown",
    formatting = false,
    highlights = [],
    highlightSource = null,
    highlightEnabled = false,
    creatingHighlight = false,
    creatingVocabularyReplacement = false,
    deletingHighlightId = null,
    onCreateHighlight = undefined,
    onCreateVocabularyReplacement = undefined,
    onDeleteHighlight = undefined,
    citationScrollText = null,
    onCitationScrollConsumed = undefined,
  }: Props = $props();

  let containerElement = $state<HTMLDivElement | null>(null);
  let articleElement = $state<HTMLElement | null>(null);
  let isMobile = $state(false);

  $effect(() => {
    if (typeof window === "undefined") return;
    const mediaQuery = window.matchMedia("(max-width: 1023px)");
    isMobile = mediaQuery.matches;

    const handler = (event: MediaQueryListEvent) => {
      isMobile = event.matches;
    };

    mediaQuery.addEventListener("change", handler);
    return () => mediaQuery.removeEventListener("change", handler);
  });

  let tooltip = $state<
    | {
        kind: "create";
        top: number;
        left: number;
        draft: CreateHighlightRequest;
      }
    | {
        kind: "delete";
        top: number;
        left: number;
        highlightId: number;
      }
    | null
  >(null);

  function unwrapAppliedHighlights() {
    if (!articleElement) return;
    const marks = articleElement.querySelectorAll<HTMLElement>(
      "mark[data-dastill-highlight='true']",
    );

    for (const mark of marks) {
      const parent = mark.parentNode;
      if (!parent) continue;
      while (mark.firstChild) {
        parent.insertBefore(mark.firstChild, mark);
      }
      parent.removeChild(mark);
      parent.normalize();
    }
  }

  function wrapTextNodeSegment(
    node: Text,
    startOffset: number,
    endOffset: number,
    highlightId: number,
  ) {
    if (startOffset >= endOffset) {
      return;
    }

    let targetNode = node;
    let relativeEnd = endOffset;

    if (startOffset > 0) {
      targetNode = targetNode.splitText(startOffset);
      relativeEnd -= startOffset;
    }

    if (relativeEnd < targetNode.data.length) {
      targetNode.splitText(relativeEnd);
    }

    const parent = targetNode.parentNode;
    if (!parent) {
      return;
    }

    const wrapper = document.createElement("mark");
    wrapper.dataset.dastillHighlight = "true";
    wrapper.dataset.highlightId = `${highlightId}`;
    wrapper.className = "reader-highlight";
    parent.replaceChild(wrapper, targetNode);
    wrapper.appendChild(targetNode);
  }

  function wrapHighlightRange(
    root: HTMLElement,
    start: number,
    end: number,
    highlightId: number,
  ) {
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
    const segments: Array<{
      node: Text;
      startOffset: number;
      endOffset: number;
    }> = [];
    let offset = 0;

    while (walker.nextNode()) {
      const node = walker.currentNode as Text;
      const length = node.data.length;
      const nodeStart = offset;
      const nodeEnd = nodeStart + length;

      offset = nodeEnd;

      if (length === 0 || end <= nodeStart) {
        continue;
      }

      if (start >= nodeEnd || end <= nodeStart) {
        continue;
      }

      segments.push({
        node,
        startOffset: Math.max(0, start - nodeStart),
        endOffset: Math.min(length, end - nodeStart),
      });
    }

    for (let index = segments.length - 1; index >= 0; index -= 1) {
      const segment = segments[index];
      wrapTextNodeSegment(
        segment.node,
        segment.startOffset,
        segment.endOffset,
        highlightId,
      );
    }
  }

  function applyHighlights() {
    if (!articleElement) return;
    unwrapAppliedHighlights();

    if (highlights.length === 0) {
      return;
    }

    const fullText = articleElement.textContent ?? "";
    const ranges = resolveHighlightRanges(fullText, highlights)
      .slice()
      .sort((a, b) => b.start - a.start);

    for (const range of ranges) {
      wrapHighlightRange(
        articleElement,
        range.start,
        range.end,
        range.highlightId,
      );
    }
  }

  function clearTooltip() {
    tooltip = null;
  }

  function updateTooltipFromSelection() {
    if (
      !articleElement ||
      !highlightEnabled ||
      !highlightSource ||
      !onCreateHighlight
    ) {
      if (tooltip?.kind === "create") {
        clearTooltip();
      }
      return;
    }

    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0 || selection.isCollapsed) {
      if (tooltip?.kind === "create") {
        clearTooltip();
      }
      return;
    }

    const range = selection.getRangeAt(0);
    // Ensure the selection is entirely within the article
    if (!articleElement.contains(range.commonAncestorContainer)) {
      if (tooltip?.kind === "create") {
        clearTooltip();
      }
      return;
    }

    const fullText = articleElement.textContent ?? "";
    const offsets = resolveRangeTextOffsets(articleElement, range);
    const draft = offsets
      ? buildHighlightDraft(
          fullText,
          highlightSource,
          offsets.start,
          offsets.end,
        )
      : null;

    if (!draft) {
      if (tooltip?.kind === "create") {
        clearTooltip();
      }
      return;
    }

    const rects = range.getClientRects();
    if (rects.length === 0) return;

    // Use the last rect for positioning to stay close to the end of selection
    const lastRect = rects[rects.length - 1];
    const containerRect = containerElement?.getBoundingClientRect();
    if (!containerRect) {
      if (tooltip?.kind === "create") {
        clearTooltip();
      }
      return;
    }

    tooltip = {
      kind: "create",
      ...resolveTooltipPosition(lastRect, containerRect),
      draft,
    };
  }

  async function handleCreateHighlight() {
    if (
      !tooltip ||
      tooltip.kind !== "create" ||
      !onCreateHighlight ||
      creatingHighlight
    ) {
      return;
    }

    const draft = tooltip.draft;
    const result = onCreateHighlight(draft);
    const selection = window.getSelection();
    if (selection) selection.removeAllRanges();
    clearTooltip();
    await result;
  }

  async function handleCreateVocabularyReplacement() {
    if (
      !tooltip ||
      tooltip.kind !== "create" ||
      !onCreateVocabularyReplacement ||
      creatingVocabularyReplacement
    ) {
      return;
    }

    const text = tooltip.draft.text;
    const result = onCreateVocabularyReplacement(text);
    const selection = window.getSelection();
    if (selection) selection.removeAllRanges();
    clearTooltip();
    await result;
  }

  async function handleDeleteHighlight() {
    if (
      !tooltip ||
      tooltip.kind !== "delete" ||
      !onDeleteHighlight ||
      deletingHighlightId === tooltip.highlightId
    ) {
      return;
    }

    await onDeleteHighlight(tooltip.highlightId);
    clearTooltip();
  }

  function handleArticleClick(event: MouseEvent) {
    if (!containerElement || !onDeleteHighlight) {
      return;
    }

    const target = event.target;
    if (!(target instanceof HTMLElement)) {
      return;
    }

    const mark = target.closest<HTMLElement>("mark[data-highlight-id]");
    if (!mark) {
      return;
    }

    const highlightId = Number(mark.dataset.highlightId);
    if (!Number.isFinite(highlightId)) {
      return;
    }

    event.preventDefault();
    const selection = window.getSelection();
    if (selection) selection.removeAllRanges();

    const containerRect = containerElement.getBoundingClientRect();
    tooltip = {
      kind: "delete",
      highlightId,
      ...resolveTooltipPosition(mark.getBoundingClientRect(), containerRect, {
        topOffset: 40,
      }),
    };
  }

  $effect(() => {
    html;
    text;
    mode;
    highlights;
    if (!articleElement) {
      return;
    }
    applyHighlights();
  });

  $effect(() => {
    html;
    text;
    mode;
    highlights;
    citationScrollText;
    formatting;
    if (
      !citationScrollText?.trim() ||
      !articleElement ||
      formatting ||
      typeof document === "undefined"
    ) {
      return;
    }
    const query = citationScrollText.trim();
    void tick().then(() => {
      requestAnimationFrame(() => {
        if (!articleElement) return;
        const ok = scrollToCitationInArticle(articleElement, query);
        if (ok) {
          onCitationScrollConsumed?.();
        }
      });
    });
  });

  $effect(() => {
    highlightEnabled;
    highlightSource;
    onCreateHighlight;
    onCreateVocabularyReplacement;
    onDeleteHighlight;
    if (!highlightEnabled || !highlightSource || !onCreateHighlight) {
      if (tooltip?.kind === "create") {
        clearTooltip();
      }
    }
    if (!onDeleteHighlight && tooltip?.kind === "delete") {
      clearTooltip();
    }
  });

  $effect(() => {
    if (typeof document === "undefined") {
      return;
    }

    // Timer to defer tooltip clear so a pending tap on the toolbar fires first.
    // On Android, tapping a toolbar button fires selectionchange (collapsed) before
    // the synthesised click arrives; without the delay the tooltip disappears and
    // the click handler finds tooltip === null.
    let clearTooltipTimer: ReturnType<typeof setTimeout> | null = null;
    // Debounce timer for selectionchange-based show. On Android, pointerup fires
    // before the selection is committed, so we also watch selectionchange and show
    // the toolbar once the selection has been stable for a short period.
    let selectionDebounceTimer: ReturnType<typeof setTimeout> | null = null;

    const handleSelectionChange = () => {
      const selection = window.getSelection();
      if (!selection || selection.isCollapsed) {
        // Cancel any pending debounced show.
        if (selectionDebounceTimer) {
          clearTimeout(selectionDebounceTimer);
          selectionDebounceTimer = null;
        }
        if (tooltip?.kind === "create") {
          // Defer the clear so a button tap (whose click arrives ~100 ms later)
          // still finds the tooltip intact.
          if (!clearTooltipTimer) {
            clearTooltipTimer = setTimeout(() => {
              clearTooltipTimer = null;
              const sel = window.getSelection();
              if (!sel || sel.isCollapsed) {
                clearTooltip();
              }
            }, 350);
          }
        }
      } else {
        // Selection is non-empty — cancel any pending deferred clear.
        if (clearTooltipTimer) {
          clearTimeout(clearTooltipTimer);
          clearTooltipTimer = null;
        }
        // Show toolbar once selection stabilises (debounced to avoid firing on
        // every cursor move while the user is still dragging a selection handle).
        if (selectionDebounceTimer) clearTimeout(selectionDebounceTimer);
        selectionDebounceTimer = setTimeout(() => {
          selectionDebounceTimer = null;
          updateTooltipFromSelection();
        }, 200);
      }
    };

    const handleStableSelection = () => {
      // On mobile, native selection menu appears after pointerup.
      // We wait a tick to ensure selection is stable.
      if (isMobile) {
        void tick().then(updateTooltipFromSelection);
      } else {
        updateTooltipFromSelection();
      }
    };

    const handleScroll = () => {
      if (tooltip) {
        if (tooltip.kind === "create") {
          updateTooltipFromSelection();
        } else {
          clearTooltip();
        }
      }
    };

    const handlePointerDown = (event: PointerEvent) => {
      if (!containerElement) {
        return;
      }
      const target = event.target;
      if (target instanceof Element && target.closest(".text-action-toolbar")) {
        return;
      }
      if (!containerElement.contains(event.target as Node)) {
        clearTooltip();
      }
    };

    document.addEventListener("selectionchange", handleSelectionChange);
    document.addEventListener("pointerup", handleStableSelection);
    document.addEventListener("keyup", handleStableSelection);
    document.addEventListener("pointerdown", handlePointerDown);
    window.addEventListener("resize", handleStableSelection);
    window.addEventListener("scroll", handleScroll, true);

    return () => {
      if (clearTooltipTimer) clearTimeout(clearTooltipTimer);
      if (selectionDebounceTimer) clearTimeout(selectionDebounceTimer);
      document.removeEventListener("selectionchange", handleSelectionChange);
      document.removeEventListener("pointerup", handleStableSelection);
      document.removeEventListener("keyup", handleStableSelection);
      document.removeEventListener("pointerdown", handlePointerDown);
      window.removeEventListener("resize", handleStableSelection);
      window.removeEventListener("scroll", handleScroll, true);
    };
  });

  $effect(() => {
    if (!articleElement) {
      return;
    }

    const handleClick = (event: Event) => {
      handleArticleClick(event as MouseEvent);
    };

    articleElement.addEventListener("click", handleClick);
    return () => {
      articleElement?.removeEventListener("click", handleClick);
    };
  });
</script>

<div class="relative" bind:this={containerElement}>
  {#if mode === "markdown"}
    <article
      bind:this={articleElement}
      class={`prose max-w-none break-words leading-relaxed transition-opacity duration-500 prose-headings:font-serif prose-headings:font-bold prose-headings:tracking-tight prose-h1:text-xl prose-h2:text-lg prose-h3:text-base prose-p:text-[17px] prose-p:leading-[1.75] prose-p:tracking-[-0.01em] prose-strong:font-bold prose-a:text-[var(--accent)] prose-a:underline-offset-4 prose-blockquote:border-l-[var(--accent)] prose-blockquote:bg-[var(--accent-soft)]/30 prose-blockquote:py-1 prose-blockquote:px-6 prose-blockquote:rounded-r-lg ${
        formatting ? "opacity-40 grayscale blur-[1px]" : "opacity-100"
      }`}
    >
      {@html html}
    </article>
  {:else}
    <article
      bind:this={articleElement}
      class={`max-w-none whitespace-pre-wrap break-words text-[17px] leading-[1.75] tracking-[-0.01em] text-[var(--foreground)] transition-opacity duration-500 ${
        formatting ? "opacity-40 grayscale blur-[1px]" : "opacity-100"
      }`}
    >
      {text}
    </article>
  {/if}

  {#if tooltip}
    {#if isMobile}
      <div
        role="group"
        aria-label="Text selection actions"
        class="text-action-toolbar fixed bottom-[calc(var(--mobile-bottom-stack-height)+1.5rem)] left-1/2 z-50 flex -translate-x-1/2 items-center gap-2 rounded-2xl px-2 py-2 shadow-2xl"
        onpointerdown={(event) => {
          event.preventDefault();
          event.stopPropagation();
        }}
      >
        {#if tooltip.kind === "create"}
          <button
            type="button"
            class="text-action-btn inline-flex h-9 w-9 items-center justify-center rounded-full text-[var(--soft-foreground)] hover:bg-[var(--accent-wash)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-50"
            onclick={handleCreateHighlight}
            disabled={creatingHighlight}
            aria-label="Save selected text as a highlight"
            title="Save highlight"
          >
            <HighlighterIcon
              class={`h-4 w-4 ${creatingHighlight ? "animate-pulse" : ""}`}
            />
          </button>
          <button
            type="button"
            class="text-action-btn inline-flex items-center justify-center rounded-full px-4 py-2 text-[11px] font-bold uppercase tracking-[0.08em] text-[var(--accent-strong)] hover:bg-[var(--accent-soft)] disabled:cursor-not-allowed disabled:opacity-50"
            style="background: var(--accent-wash);"
            onclick={handleCreateVocabularyReplacement}
            disabled={!onCreateVocabularyReplacement ||
              creatingVocabularyReplacement}
          >
            {creatingVocabularyReplacement ? "Saving" : "Correct"}
          </button>
        {:else}
          <button
            type="button"
            class="text-action-btn inline-flex h-9 w-9 items-center justify-center rounded-full text-[var(--soft-foreground)] hover:bg-[var(--danger-soft)] hover:text-[var(--danger)] disabled:cursor-not-allowed disabled:opacity-50"
            onclick={handleDeleteHighlight}
            disabled={deletingHighlightId === tooltip.highlightId}
            aria-label="Delete highlight"
            title="Delete highlight"
          >
            <TrashIcon
              size={16}
              strokeWidth={2}
              class={deletingHighlightId === tooltip.highlightId
                ? "animate-pulse"
                : ""}
            />
          </button>
        {/if}
      </div>
    {:else}
      <div
        role="group"
        aria-label="Text selection actions"
        class="text-action-toolbar absolute z-40 flex items-center gap-1 rounded-full px-1.5 py-1.5"
        style={`top: ${tooltip.top}px; left: ${tooltip.left}px; transform: translateX(-50%);`}
        onpointerdown={(event) => event.preventDefault()}
      >
        {#if tooltip.kind === "create"}
          <button
            type="button"
            class="text-action-btn inline-flex h-8 w-8 items-center justify-center rounded-full text-[var(--soft-foreground)] hover:bg-[var(--accent-wash)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-50"
            onclick={handleCreateHighlight}
            disabled={creatingHighlight}
            aria-label="Save selected text as a highlight"
            title="Save highlight"
          >
            <HighlighterIcon
              class={`h-3.5 w-3.5 ${creatingHighlight ? "animate-pulse" : ""}`}
            />
          </button>
          <button
            type="button"
            class="text-action-btn inline-flex items-center justify-center rounded-full px-3.5 py-1.5 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--accent-strong)] hover:bg-[var(--accent-soft)] disabled:cursor-not-allowed disabled:opacity-50"
            style="background: var(--accent-wash);"
            onclick={handleCreateVocabularyReplacement}
            disabled={!onCreateVocabularyReplacement ||
              creatingVocabularyReplacement}
          >
            {creatingVocabularyReplacement ? "Saving" : "Correct"}
          </button>
        {:else}
          <button
            type="button"
            class="text-action-btn inline-flex h-8 w-8 items-center justify-center rounded-full text-[var(--soft-foreground)] hover:bg-[var(--danger-soft)] hover:text-[var(--danger)] disabled:cursor-not-allowed disabled:opacity-50"
            onclick={handleDeleteHighlight}
            disabled={deletingHighlightId === tooltip.highlightId}
            aria-label="Delete highlight"
            title="Delete highlight"
          >
            <TrashIcon
              size={14}
              strokeWidth={2}
              class={deletingHighlightId === tooltip.highlightId
                ? "animate-pulse"
                : ""}
            />
          </button>
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .text-action-toolbar {
    background: var(--surface-frost);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    box-shadow:
      0 2px 12px var(--shadow-soft),
      0 1px 3px var(--shadow-strong);
    touch-action: none;
  }

  .text-action-btn {
    transition: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  :global(.prose h1, .prose h2, .prose h3) {
    font-variation-settings:
      "opsz" 72,
      "wght" 700;
  }

  :global(.prose > :first-child) {
    margin-top: 0;
  }

  :global(.prose mark.reader-highlight),
  :global(mark.reader-highlight) {
    background: var(--reader-highlight-bg);
    border-radius: 0.18em;
    box-decoration-break: clone;
    -webkit-box-decoration-break: clone;
    color: inherit;
    cursor: pointer;
    padding: 0;
    transition: background-color 160ms ease;
  }

  :global(mark.reader-highlight:hover) {
    background: var(--reader-highlight-bg-hover);
  }
</style>
