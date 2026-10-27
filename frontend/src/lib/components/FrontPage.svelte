<script lang="ts">
  import type { CreateHighlightRequest, Highlight } from "$lib/api";
  import HighlightPassage from "$lib/components/HighlightPassage.svelte";
  import { renderMarkdown } from "$lib/edition/markdown";
  import {
    describeReleaseDay,
    layOutFrontPage,
    type Story,
  } from "$lib/edition/stories";
  import { splitSummary } from "$lib/edition/summary";

  let {
    stories,
    lead,
    undoTitle = null,
    highlights = [],
    onOpen,
    onRead,
    onUndo,
    onHighlight,
    onRemoveHighlight,
  }: {
    /** Unread stories in this section, newest first. */
    stories: Story[];
    lead: Story;
    undoTitle?: string | null;
    /** The lead's highlights; its standfirst and glance can be highlighted. */
    highlights?: Highlight[];
    onOpen: (storyId: string) => void;
    onRead: () => void;
    onUndo: () => void;
    onHighlight?: (draft: CreateHighlightRequest) => void;
    onRemoveHighlight?: (highlightId: string) => void;
  } = $props();

  const layout = $derived(layOutFrontPage(stories, lead));
  const leadParts = $derived(splitSummary(lead.summary));
  const glanceHtml = $derived(renderMarkdown(leadParts.glance));

  function standfirstOf(story: Story): string {
    return splitSummary(story.summary).standfirst;
  }

  function describeMeta(story: Story): string {
    const day = describeReleaseDay(story.publishedAt);
    return day ? `${story.channelName} · ${day}` : story.channelName;
  }
</script>

<!--
  The edition as a newspaper front page: the lead, the next stories beside
  it, and the rest in a grid below. Every story opens in the reader.
-->
<div class="front-page">
  <div class="top">
    {#key lead.id}
      <section class="lead print-in" aria-labelledby="front-lead-headline">
        <HighlightPassage {highlights} {onHighlight} {onRemoveHighlight}>
          <p class="label channel" data-passage-skip>{lead.channelName}</p>
          <h2 id="front-lead-headline" class="lead-headline" data-passage-skip>
            <button type="button" onclick={() => onOpen(lead.id)}>
              {lead.title}
            </button>
          </h2>
          {#if leadParts.standfirst}
            <p class="lead-standfirst">{leadParts.standfirst}</p>
          {/if}
          {#if glanceHtml}
            <section class="lead-glance" aria-labelledby="front-glance-heading">
              <h3 id="front-glance-heading" class="label" data-passage-skip>
                At a glance
              </h3>
              <div class="lead-glance-list">{@html glanceHtml}</div>
            </section>
          {/if}
          <div class="actions" data-passage-skip>
            <button type="button" class="press" onclick={() => onOpen(lead.id)}>
              Read the story
            </button>
            <button
              type="button"
              class="text-button"
              onclick={onRead}
              title="Mark as read (R)"
            >
              Mark as read
            </button>
            {#if undoTitle}
              <button
                type="button"
                class="text-button"
                onclick={onUndo}
                title="Undo (U)"
                aria-label={`Undo: mark "${undoTitle}" unread`}
              >
                Undo
              </button>
            {/if}
          </div>
        </HighlightPassage>
      </section>
    {/key}

    {#if layout.beside.length > 0}
      <ol class="beside" aria-label="Next in this edition">
        {#each layout.beside as story (story.id)}
          <li>
            <button type="button" onclick={() => onOpen(story.id)}>
              <span class="label channel">{story.channelName}</span>
              <span class="beside-title">{story.title}</span>
              <span class="beside-standfirst">{standfirstOf(story)}</span>
            </button>
          </li>
        {/each}
      </ol>
    {/if}
  </div>

  {#if layout.below.length > 0}
    <section class="more" aria-labelledby="more-heading">
      <h2 id="more-heading" class="label">More in this edition</h2>
      <ol>
        {#each layout.below as story (story.id)}
          <li>
            <button type="button" onclick={() => onOpen(story.id)}>
              <span class="label meta">{describeMeta(story)}</span>
              <span class="more-title">{story.title}</span>
              <span class="more-standfirst">{standfirstOf(story)}</span>
            </button>
          </li>
        {/each}
      </ol>
    </section>
  {/if}
</div>

<style>
  .front-page {
    padding: var(--space-6) 0 var(--space-8);
  }

  .top {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-6) var(--space-7);
  }

  .lead {
    flex: 999 1 560px;
    min-width: 0;
  }

  .channel {
    margin: 0;
    color: var(--kicker);
  }

  ol {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  button:not(.press, .text-button) {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    width: 100%;
    padding: 0;
    border: 0;
    background: none;
    color: inherit;
    text-align: left;
  }

  button:not(.press, .text-button):hover :is(.beside-title, .more-title),
  .lead-headline button:hover {
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 4px;
  }

  .lead-headline {
    margin: var(--space-3) 0 var(--space-4);
    font-size: 56px;
    font-weight: 700;
    line-height: 1.04;
    letter-spacing: -0.015em;
    text-wrap: balance;
  }

  .lead-headline button {
    display: inline;
    font: inherit;
    letter-spacing: inherit;
    text-wrap: balance;
  }

  .lead-standfirst {
    max-width: 52ch;
    margin: 0 0 var(--space-5);
    font-size: 22px;
    font-style: italic;
    line-height: 1.45;
    color: var(--ink-soft);
    text-wrap: pretty;
  }

  .lead-glance {
    padding-top: var(--space-3);
    border-top: 2px solid var(--rule);
  }

  .lead-glance h3 {
    margin: 0 0 var(--space-2);
  }

  .lead-glance-list :global(ul) {
    margin: 0;
    padding-left: 1.1em;
    columns: 2;
    column-gap: var(--space-6);
    font-family: var(--sans);
    font-size: 15px;
    line-height: 1.5;
  }

  .lead-glance-list :global(li) {
    margin-bottom: var(--space-2);
    break-inside: avoid;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-5);
    margin-top: var(--space-5);
  }

  .actions .press {
    min-height: 48px;
  }

  .beside {
    flex: 1 1 320px;
    min-width: 0;
    max-width: 480px;
    padding-left: var(--space-6);
    border-left: 1px solid var(--hairline);
  }

  .beside li + li {
    border-top: 1px solid var(--hairline);
  }

  .beside button:not(.press, .text-button) {
    padding: var(--space-5) 0;
  }

  .beside li:first-child button {
    padding-top: 0;
  }

  .beside-title {
    font-size: 28px;
    font-weight: 700;
    line-height: 1.15;
    text-wrap: balance;
  }

  .beside-standfirst {
    font-size: 16px;
    font-style: italic;
    line-height: 1.5;
    color: var(--ink-soft);
  }

  .more {
    margin-top: var(--space-7);
  }

  .more h2 {
    margin: 0;
    padding-bottom: var(--space-2);
    border-bottom: 2px solid var(--rule);
  }

  .more ol {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    column-gap: var(--space-6);
  }

  .more li {
    border-bottom: 1px solid var(--hairline);
  }

  .more button:not(.press, .text-button) {
    gap: 6px;
    padding: var(--space-5) 0;
  }

  .meta {
    font-size: 10px;
    color: var(--ink-faint);
  }

  .more-title {
    font-size: 19px;
    font-weight: 700;
    line-height: 1.25;
  }

  .more-standfirst {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    overflow: hidden;
    font-size: 14px;
    line-height: 1.45;
    color: var(--ink-soft);
  }

  :is(.beside-standfirst, .more-standfirst):empty {
    display: none;
  }

  @media (min-width: 1360px) {
    .more ol {
      grid-template-columns: repeat(4, minmax(0, 1fr));
    }
  }
</style>
