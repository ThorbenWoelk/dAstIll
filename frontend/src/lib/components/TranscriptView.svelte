<script lang="ts">
  import type {
    CreateHighlightRequest,
    Highlight,
    HighlightSource,
  } from "$lib/types";
  import HighlighterIcon from "$lib/components/icons/HighlighterIcon.svelte";
  import TrashIcon from "$lib/components/icons/TrashIcon.svelte";
  import {
    buildHighlightDraft,
    resolveHighlightRanges,
    resolveTooltipPosition,
  } from "$lib/utils/highlights";

  type Props = {
    html?: string;
    text?: string;
    mode?: "markdown" | "plain_text";
    formatting?: boolean;
    highlights?: Highlight[];
    highlightSource?: HighlightSource | null;
    highlightEnabled?: boolean;
    creatingHighlight?: boolean;
    deletingHighlightId?: number | null;
    onCreateHighlight?:
      | ((payload: CreateHighlightRequest) => Promise<void> | void)
      | undefined;
    onDeleteHighlight?:
      | ((highlightId: number) => Promise<void> | void)
      | undefined;
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
    deletingHighlightId = null,
    onCreateHighlight = undefined,
    onDeleteHighlight = undefined,
  }: Props = $props();

  let containerElement = $state<HTMLDivElement | null>(null);
  let articleElement = $state<HTMLElement | null>(null);
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
    if (!articleElement.contains(range.commonAncestorContainer)) {
      if (tooltip?.kind === "create") {
        clearTooltip();
      }
      return;
    }

    const preRange = document.createRange();
    preRange.selectNodeContents(articleElement);
    preRange.setEnd(range.startContainer, range.startOffset);

    const start = preRange.toString().length;
    const end = start + selection.toString().length;
    const fullText = articleElement.textContent ?? "";
    const draft = buildHighlightDraft(fullText, highlightSource, start, end);

    if (!draft) {
      if (tooltip?.kind === "create") {
        clearTooltip();
      }
      return;
    }

    const rect = range.getBoundingClientRect();
    const containerRect = containerElement?.getBoundingClientRect();
    if (!containerRect) {
      if (tooltip?.kind === "create") {
        clearTooltip();
      }
      return;
    }

    tooltip = {
      kind: "create",
      ...resolveTooltipPosition(rect, containerRect),
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

    const result = onCreateHighlight(tooltip.draft);
    window.getSelection()?.removeAllRanges();
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
    window.getSelection()?.removeAllRanges();

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
    highlightEnabled;
    highlightSource;
    onCreateHighlight;
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

    const handleSelectionChange = () => {
      updateTooltipFromSelection();
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
      if (!containerElement.contains(event.target as Node)) {
        clearTooltip();
      }
    };

    document.addEventListener("selectionchange", handleSelectionChange);
    document.addEventListener("pointerdown", handlePointerDown);
    window.addEventListener("resize", handleSelectionChange);
    window.addEventListener("scroll", handleScroll, true);

    return () => {
      document.removeEventListener("selectionchange", handleSelectionChange);
      document.removeEventListener("pointerdown", handlePointerDown);
      window.removeEventListener("resize", handleSelectionChange);
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
      class={`prose max-w-none break-words leading-relaxed transition-all duration-500 prose-headings:font-serif prose-headings:font-bold prose-headings:tracking-tight prose-h1:text-xl prose-h2:text-lg prose-h3:text-base prose-p:text-[17px] prose-p:leading-[1.75] prose-p:tracking-[-0.01em] prose-strong:font-bold prose-a:text-[var(--accent)] prose-a:underline-offset-4 prose-blockquote:border-l-[var(--accent)] prose-blockquote:bg-[var(--accent-soft)]/30 prose-blockquote:py-1 prose-blockquote:px-6 prose-blockquote:rounded-r-lg ${
        formatting ? "opacity-40 grayscale blur-[1px]" : "opacity-100"
      }`}
    >
      {@html html}
    </article>
  {:else}
    <article
      bind:this={articleElement}
      class={`max-w-none whitespace-pre-wrap break-words text-[17px] leading-[1.75] tracking-[-0.01em] text-[var(--foreground)] transition-all duration-500 ${
        formatting ? "opacity-40 grayscale blur-[1px]" : "opacity-100"
      }`}
    >
      {text}
    </article>
  {/if}

  {#if tooltip}
    <button
      type="button"
      class={`absolute z-40 shadow-lg transition-all disabled:cursor-not-allowed disabled:opacity-60 ${
        tooltip.kind === "create"
          ? "inline-flex h-10 w-10 items-center justify-center rounded-full border border-[var(--accent)]/30 bg-[var(--foreground)] text-white hover:bg-[var(--accent-strong)]"
          : "inline-flex h-10 w-10 items-center justify-center rounded-full bg-[var(--surface)] text-[var(--soft-foreground)] hover:bg-[var(--danger-soft)] hover:text-[var(--danger)]"
      }`}
      style={`top: ${tooltip.top}px; left: ${tooltip.left}px; transform: translateX(-50%);`}
      onmousedown={(event) => event.preventDefault()}
      onclick={tooltip.kind === "create"
        ? handleCreateHighlight
        : handleDeleteHighlight}
      disabled={tooltip.kind === "create"
        ? creatingHighlight
        : deletingHighlightId === tooltip.highlightId}
      aria-label={tooltip.kind === "create"
        ? "Save selected text as a highlight"
        : "Delete highlight"}
      title={tooltip.kind === "create" ? "Save highlight" : "Delete highlight"}
    >
      {#if tooltip.kind === "create"}
        <HighlighterIcon
          class={`h-4 w-4 ${creatingHighlight ? "animate-pulse" : ""}`}
        />
      {:else}
        <TrashIcon
          size={16}
          strokeWidth={2.2}
          class={deletingHighlightId === tooltip.highlightId
            ? "animate-pulse"
            : ""}
        />
      {/if}
    </button>
  {/if}
</div>

<style>
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
    background: color-mix(in srgb, rgb(248 231 162) 84%, white);
    border-radius: 0.18em;
    box-decoration-break: clone;
    -webkit-box-decoration-break: clone;
    color: inherit;
    cursor: pointer;
    padding: 0;
    transition: background-color 160ms ease;
  }

  :global(mark.reader-highlight:hover) {
    background: color-mix(in srgb, rgb(244 221 136) 88%, white);
  }
</style>
