<script lang="ts">
  import type { Highlight } from "$lib/api";
  import { outlineStory } from "$lib/edition/sections";
  import type { Story } from "$lib/edition/stories";

  let {
    story,
    highlights = [],
    onRemoveHighlight,
  }: {
    story: Story;
    highlights?: Highlight[];
    onRemoveHighlight: (highlightId: string) => void;
  } = $props();

  const outline = $derived(outlineStory(story.summary));

  /** Buttons, not hash links: a hash link would leave the opened story. */
  function showPart(partId: string) {
    document.getElementById(partId)?.scrollIntoView({ block: "start" });
  }

  function showHighlight(highlightId: string) {
    // The front page grid marks the same highlight; look in the story.
    document
      .getElementById("story-headline")
      ?.closest("article")
      ?.querySelector(`mark[data-highlight-id="${CSS.escape(highlightId)}"]`)
      ?.scrollIntoView({ block: "center" });
  }
</script>

<!-- Beside the story on wide screens: its outline and the reader's highlights. -->
<aside class="story-margin" aria-label="In the margin">
  {#if outline.length > 0}
    <nav aria-labelledby="outline-heading">
      <h2 id="outline-heading" class="label">In this story</h2>
      <ol>
        {#each outline as part (part.id)}
          <li>
            <button
              type="button"
              class="part"
              onclick={() => showPart(part.id)}
            >
              {part.title}
            </button>
          </li>
        {/each}
      </ol>
    </nav>
  {/if}

  <section aria-labelledby="margin-highlights-heading">
    <h2 id="margin-highlights-heading" class="label">Your highlights</h2>
    {#if highlights.length === 0}
      <p class="hint">Select text in the story to highlight it.</p>
    {:else}
      <ul>
        {#each highlights as highlight (highlight.id)}
          <li>
            <button
              type="button"
              class="quote"
              onclick={() => showHighlight(highlight.id)}
            >
              <mark>{highlight.text}</mark>
            </button>
            <button
              type="button"
              class="text-button"
              aria-label={`Remove highlight: ${highlight.text}`}
              onclick={() => onRemoveHighlight(highlight.id)}
            >
              Remove
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</aside>

<style>
  .story-margin {
    position: sticky;
    top: var(--space-4);
    max-height: calc(100vh - var(--space-4));
    overflow-y: auto;
    padding-top: var(--space-5);
  }

  h2 {
    display: flex;
    align-items: center;
    min-height: var(--touch);
    margin: 0;
    border-bottom: 1px solid var(--rule);
  }

  nav + section {
    margin-top: var(--space-6);
  }

  ol,
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .part {
    display: flex;
    align-items: center;
    width: 100%;
    min-height: 36px;
    padding: 0;
    border: 0;
    background: none;
    font-family: var(--sans);
    font-size: 14px;
    color: var(--ink-soft);
    text-align: left;
  }

  .part:hover {
    color: var(--kicker);
  }

  ul li {
    padding: var(--space-3) 0 var(--space-1);
    border-bottom: 1px solid var(--hairline);
  }

  .quote {
    display: block;
    width: 100%;
    padding: 0;
    border: 0;
    background: none;
    font-size: 15px;
    line-height: 1.5;
    color: inherit;
    text-align: left;
  }

  mark {
    padding: 0.05em 0;
    background: var(--marker);
    color: inherit;
    box-decoration-break: clone;
    -webkit-box-decoration-break: clone;
  }

  .hint {
    margin: var(--space-3) 0 0;
    font-size: 14px;
    font-style: italic;
    line-height: 1.5;
    color: var(--ink-soft);
  }
</style>
