<script lang="ts">
  import type { Snippet } from "svelte";
  import type { CreateHighlightRequest, Highlight } from "$lib/api";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import HighlightIcon from "$lib/components/icons/HighlightIcon.svelte";
  import { renderMarkdown } from "$lib/edition/markdown";
  import {
    describeReleaseDay,
    readingMinutes,
    type Story,
  } from "$lib/edition/stories";
  import { splitSummary } from "$lib/edition/summary";
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
    story,
    footer,
    highlights = [],
    onHighlight,
    onRemoveHighlight,
  }: {
    story: Story;
    footer?: Snippet;
    highlights?: Highlight[];
    /** Omit to turn highlighting off. */
    onHighlight?: (draft: CreateHighlightRequest) => void;
    onRemoveHighlight?: (highlightId: string) => void;
  } = $props();

  const TOOLBAR_HALF_WIDTH = 90;

  const parts = $derived(splitSummary(story.summary));
  const glanceHtml = $derived(renderMarkdown(parts.glance));
  const bodyHtml = $derived(renderMarkdown(parts.body));
  const releaseDay = $derived(describeReleaseDay(story.publishedAt));
  const minutes = $derived(readingMinutes(story.summary));

  let article = $state<HTMLElement | null>(null);
  /** The selection the reader may turn into a highlight. */
  let selection = $state<{
    draft: CreateHighlightRequest;
    top: number;
    left: number;
  } | null>(null);
  /** The highlight the reader tapped, offered for removal. */
  let tapped = $state<{ id: string; top: number; left: number } | null>(null);

  $effect(() => {
    const root = article;
    const current = highlights;
    if (!root) return;
    clearHighlightMarks(root);
    const passage = readPassage(root);
    markHighlights(passage, placeHighlights(passage.text, current));
  });

  $effect(() => {
    if (tapped && !highlights.some((h) => h.id === tapped?.id)) tapped = null;
  });

  /** Position for a toolbar just below `rect`, inside the article. */
  function below(rect: DOMRect) {
    const box = article!.getBoundingClientRect();
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
    if (!onHighlight || !article || pressingToolbar) return;
    const current = document.getSelection();
    if (!current || current.isCollapsed || current.rangeCount === 0) {
      selection = null;
      return;
    }
    const range = current.getRangeAt(0);
    const passage = readPassage(article);
    const span = spanOfRange(passage, article, range);
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

{#key story.id}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
  <article
    class="story print-in"
    aria-labelledby="story-headline"
    bind:this={article}
    onclick={handleClick}
  >
    <p class="label kicker" data-passage-skip>{story.channelName}</p>
    <h1 id="story-headline" data-passage-skip>{story.title}</h1>
    {#if parts.standfirst}
      <p class="standfirst">{parts.standfirst}</p>
    {/if}
    <p class="byline" data-passage-skip>
      {#if releaseDay}<span>{releaseDay}</span>{/if}
      <span>{minutes} min read</span>
      <a href={story.watchUrl} target="_blank" rel="noopener noreferrer">
        Watch the video <ExternalLinkIcon />
      </a>
    </p>

    {#if glanceHtml}
      <aside class="glance" aria-labelledby="glance-heading">
        <h2 id="glance-heading" class="label" data-passage-skip>At a glance</h2>
        <div class="glance-list">{@html glanceHtml}</div>
      </aside>
    {/if}

    <div class="body">{@html bodyHtml}</div>

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

    <div class="footer" data-passage-skip>
      {@render footer?.()}
    </div>
  </article>
{/key}

<style>
  .story {
    position: relative;
    padding-top: var(--space-5);
  }

  .story :global(mark.reader-highlight) {
    padding: 0.05em 0;
    background: var(--marker);
    color: inherit;
    cursor: pointer;
    box-decoration-break: clone;
    -webkit-box-decoration-break: clone;
  }

  /* No box of its own, so a sticky read bar still sticks to the article. */
  .footer {
    display: contents;
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

  .kicker {
    margin: 0;
    color: var(--kicker);
  }

  h1 {
    margin: var(--space-2) 0 var(--space-3);
    font-size: 32px;
    font-weight: 700;
    line-height: 1.1;
    letter-spacing: -0.01em;
    text-wrap: balance;
  }

  .standfirst {
    margin: 0 0 var(--space-4);
    font-size: 19px;
    font-style: italic;
    line-height: 1.45;
    color: var(--ink-soft);
    text-wrap: pretty;
  }

  .byline {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1) var(--space-4);
    margin: 0 0 var(--space-5);
    padding: var(--space-2) 0;
    border-top: 1px solid var(--hairline);
    border-bottom: 1px solid var(--hairline);
    font-family: var(--sans);
    font-size: 12px;
    color: var(--ink-soft);
  }

  .byline a {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .glance {
    margin: 0 0 var(--space-5);
    padding: var(--space-3) 0 var(--space-1);
    border-top: 2px solid var(--rule);
    border-bottom: 1px solid var(--hairline);
  }

  .glance h2 {
    margin: 0 0 var(--space-2);
  }

  .glance-list :global(ul) {
    margin: 0;
    padding-left: 1.1em;
    font-family: var(--sans);
    font-size: 15px;
    line-height: 1.5;
  }

  .glance-list :global(li) {
    margin-bottom: var(--space-2);
    break-inside: avoid;
  }

  .body {
    font-size: 17px;
    line-height: 1.65;
    hyphens: auto;
    overflow-wrap: break-word;
  }

  .body :global(h1),
  .body :global(h2),
  .body :global(h3) {
    margin: var(--space-5) 0 var(--space-2);
    font-family: var(--sans);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    break-after: avoid;
  }

  .body :global(h3) {
    letter-spacing: 0.08em;
    color: var(--ink-soft);
  }

  .body > :global(:first-child) {
    margin-top: 0;
  }

  .body :global(p) {
    margin: 0 0 var(--space-4);
  }

  .body > :global(p:first-of-type::first-letter) {
    float: left;
    padding: 6px var(--space-2) 0 0;
    font-size: 56px;
    font-weight: 700;
    line-height: 0.85;
  }

  .body :global(ul),
  .body :global(ol) {
    margin: 0 0 var(--space-4);
    padding-left: 1.2em;
  }

  .body :global(li) {
    margin-bottom: var(--space-2);
    break-inside: avoid;
  }

  .body :global(strong) {
    font-weight: 700;
  }

  .body :global(blockquote) {
    margin: 0 0 var(--space-4);
    padding-left: var(--space-4);
    border-left: 2px solid var(--rule);
    font-style: italic;
  }

  .body :global(code) {
    font-size: 0.9em;
  }

  @media (min-width: 640px) {
    h1 {
      font-size: 48px;
      line-height: 1.05;
      letter-spacing: -0.015em;
    }

    .standfirst {
      font-size: 22px;
    }

    .glance-list :global(ul) {
      columns: 2;
      column-gap: var(--space-6);
    }
  }

  @media (min-width: 960px) {
    .body {
      columns: 2;
      column-gap: 40px;
      column-rule: 1px solid var(--hairline);
    }
  }
</style>
