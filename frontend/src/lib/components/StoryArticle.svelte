<script lang="ts">
  import type { Snippet } from "svelte";
  import type {
    CreateHighlightRequest,
    CreateWritingFlagRequest,
    Highlight,
    WritingFlag,
  } from "$lib/api";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import HighlightPassage from "$lib/components/HighlightPassage.svelte";
  import StorySection from "$lib/components/StorySection.svelte";
  import { renderMarkdown } from "$lib/edition/markdown";
  import {
    GLANCE_PART_ID,
    storyPartId,
    storySections,
  } from "$lib/edition/sections";
  import {
    describeAuthor,
    describeReleaseDay,
    readingMinutes,
    type Story,
    type TurnDirection,
  } from "$lib/edition/stories";
  import { splitSummary } from "$lib/edition/summary";

  let {
    story,
    entrance = "print",
    footer,
    highlights = [],
    flags = [],
    onHighlight,
    onRemoveHighlight,
    onFlag,
    onRemoveFlag,
  }: {
    story: Story;
    /** How a new story arrives: printed in place, or turned in from a side. */
    entrance?: "print" | TurnDirection;
    footer?: Snippet;
    highlights?: Highlight[];
    /** Omit to turn highlighting off. */
    onHighlight?: (draft: CreateHighlightRequest) => void;
    onRemoveHighlight?: (highlightId: string) => void;
    /** Passages the reader flagged as bad writing. */
    flags?: WritingFlag[];
    /** Omit to turn flagging off. */
    onFlag?: (draft: CreateWritingFlagRequest) => void;
    onRemoveFlag?: (flagId: string) => void;
  } = $props();

  const parts = $derived(splitSummary(story.summary));
  const glanceHtml = $derived(renderMarkdown(parts.glance));
  const sections = $derived(
    storySections(parts.body).map((section) => ({
      ...section,
      html: renderMarkdown(section.markdown),
    })),
  );
  const author = $derived(describeAuthor(story.summaryModel));
  const releaseDay = $derived(describeReleaseDay(story.publishedAt));
  const minutes = $derived(readingMinutes(story.summary));
</script>

{#key story.id}
  <article
    class={["story", entrance === "print" ? "print-in" : `turn-in-${entrance}`]}
    aria-labelledby="story-headline"
  >
    <HighlightPassage
      {highlights}
      {flags}
      {onHighlight}
      {onRemoveHighlight}
      {onFlag}
      {onRemoveFlag}
    >
      <p class="label kicker" data-passage-skip>{story.channelName}</p>
      <h1 id="story-headline" data-passage-skip>{story.title}</h1>
      {#if parts.standfirst}
        <p class="standfirst">{parts.standfirst}</p>
      {/if}
      <p class="byline" data-passage-skip>
        {#if author}<span class="author">By {author}</span>{/if}
        {#if releaseDay}<span>{releaseDay}</span>{/if}
        <span>{minutes} min read</span>
        <a href={story.watchUrl} target="_blank" rel="noopener noreferrer">
          Watch the video <ExternalLinkIcon />
        </a>
      </p>

      {#if glanceHtml}
        <aside class="glance" aria-labelledby="glance-heading">
          <StorySection
            title="At a glance"
            headingId="glance-heading"
            anchorId={GLANCE_PART_ID}
            passageSkip
          >
            <div class="glance-list">{@html glanceHtml}</div>
          </StorySection>
        </aside>
      {/if}

      <div class="body">
        {#each sections as section, index (index)}
          {#if section.title === null}
            <div class={["text", index === 0 && "lead-text"]}>
              {@html section.html}
            </div>
          {:else}
            <StorySection
              title={section.title}
              level={section.level}
              anchorId={storyPartId(index)}
            >
              <div class={["text", index === 0 && "lead-text"]}>
                {@html section.html}
              </div>
            </StorySection>
          {/if}
        {/each}
      </div>
    </HighlightPassage>

    <div class="footer" data-passage-skip>
      {@render footer?.()}
    </div>
  </article>
{/key}

<style>
  .story {
    padding-top: var(--space-5);
  }

  /* No box of its own, so a sticky read bar still sticks to the article. */
  .footer {
    display: contents;
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

  .author {
    font-weight: 600;
    color: var(--ink);
  }

  .byline a {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .glance {
    margin: 0 0 var(--space-5);
    padding: 0 0 var(--space-1);
    border-top: 2px solid var(--rule);
    border-bottom: 1px solid var(--hairline);
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

  /* Subheadings inside a section. Section headings are StorySection's. */
  .text :global(h1),
  .text :global(h2),
  .text :global(h3) {
    margin: var(--space-5) 0 var(--space-2);
    font-family: var(--sans);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    break-after: avoid;
  }

  .text :global(h3) {
    letter-spacing: 0.08em;
    color: var(--ink-soft);
  }

  .text > :global(:first-child) {
    margin-top: 0;
  }

  .body :global(p) {
    margin: 0 0 var(--space-4);
  }

  .lead-text > :global(p:first-of-type::first-letter) {
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
    .text {
      columns: 2;
      column-gap: 40px;
      column-rule: 1px solid var(--hairline);
    }
  }

  @media (min-width: 1360px) {
    h1 {
      font-size: 56px;
    }
  }
</style>
