<script lang="ts">
  import { onMount } from "svelte";
  import { deleteHighlight, listHighlights } from "$lib/api";
  import Masthead from "$lib/components/Masthead.svelte";
  import { describeReleaseDay } from "$lib/edition/stories";
  import {
    countHighlights,
    filterHighlightedStories,
    listHighlightedStories,
    withoutHighlight,
    type HighlightedStory,
  } from "$lib/highlights/collection";

  let stories = $state.raw<HighlightedStory[]>([]);
  let loading = $state(true);
  let failure = $state<string | null>(null);
  let removeError = $state<string | null>(null);
  let query = $state("");

  const shown = $derived(filterHighlightedStories(stories, query));
  const total = $derived(countHighlights(stories));
  const shownCount = $derived(countHighlights(shown));

  function messageOf(cause: unknown): string {
    return cause instanceof Error ? cause.message : String(cause);
  }

  function plural(count: number, word: string): string {
    return `${count} ${word}${count === 1 ? "" : "s"}`;
  }

  async function load() {
    loading = true;
    failure = null;
    try {
      stories = listHighlightedStories(await listHighlights());
    } catch (cause) {
      failure = messageOf(cause);
    } finally {
      loading = false;
    }
  }

  async function remove(highlightId: string) {
    const before = stories;
    stories = withoutHighlight(stories, highlightId);
    removeError = null;
    try {
      await deleteHighlight(highlightId);
    } catch (cause) {
      stories = before;
      removeError = `Could not remove the highlight: ${messageOf(cause)}`;
    }
  }

  onMount(() => {
    void load();
  });
</script>

<svelte:head>
  <title>Highlights · dAstIll</title>
</svelte:head>

<Masthead current="highlights" />

<section class="highlights-page" aria-labelledby="highlights-heading">
  <h1 id="highlights-heading">Highlights</h1>
  <p class="intro">
    Passages you marked while reading, newest stories first. Select text in any
    story to add one.
  </p>

  {#if loading}
    <p class="quiet" aria-live="polite">Gathering your highlights…</p>
  {:else if failure}
    <p class="message" role="alert">{failure}</p>
    <button type="button" class="text-button" onclick={load}>Try again</button>
  {:else if total === 0}
    <p class="quiet">
      No highlights yet. Select a passage in a story and choose Highlight.
    </p>
  {:else}
    <div class="search">
      <input
        type="search"
        bind:value={query}
        placeholder="Search highlights"
        aria-label="Search highlights"
        autocomplete="off"
      />
      <p class="count" aria-live="polite">
        {query.trim()
          ? `${shownCount} of ${plural(total, "highlight")}`
          : plural(total, "highlight")}
      </p>
    </div>

    {#if removeError}
      <p class="message" role="alert">{removeError}</p>
    {/if}

    {#each shown as story (story.videoId)}
      <article class="video">
        <p class="label kicker">
          {story.channelName}
          {#if describeReleaseDay(story.publishedAt)}
            <span class="day">· {describeReleaseDay(story.publishedAt)}</span>
          {/if}
        </p>
        <h2>
          <a href={`/stories/${encodeURIComponent(story.videoId)}`}>
            {story.title}
          </a>
        </h2>
        <ul>
          {#each story.highlights as highlight (highlight.id)}
            <li>
              <blockquote>{highlight.text}</blockquote>
              <button
                type="button"
                class="text-button"
                aria-label={`Remove highlight: ${highlight.text.slice(0, 60)}`}
                onclick={() => remove(highlight.id)}
              >
                Remove
              </button>
            </li>
          {/each}
        </ul>
      </article>
    {:else}
      <p class="quiet">No highlight matches "{query.trim()}".</p>
    {/each}
  {/if}
</section>

<style>
  .highlights-page {
    max-width: 720px;
    margin: 0 auto;
    padding: var(--space-6) 0 var(--space-8);
  }

  h1 {
    margin: 0;
    font-size: 40px;
    line-height: 1.1;
  }

  .intro {
    margin: var(--space-2) 0 var(--space-5);
    font-style: italic;
    color: var(--ink-soft);
  }

  .search {
    padding: var(--space-3) 0;
    border-top: 2px solid var(--rule);
    border-bottom: 1px solid var(--hairline);
  }

  input {
    width: 100%;
    min-height: 48px;
    padding: 0 var(--space-3);
    border: 1px solid var(--rule);
    border-radius: 0;
    background: var(--paper-raised);
    font-family: var(--sans);
    font-size: 16px;
  }

  input::placeholder {
    color: var(--ink-faint);
  }

  .count {
    margin: var(--space-2) 0 0;
    font-family: var(--sans);
    font-size: 12px;
    color: var(--ink-soft);
  }

  .kicker {
    margin: 0 0 var(--space-1);
    color: var(--kicker);
  }

  .day {
    color: var(--ink-soft);
  }

  .video {
    padding: var(--space-4) 0;
    border-bottom: 1px solid var(--hairline);
  }

  h2 {
    margin: 0 0 var(--space-3);
    font-size: 21px;
    line-height: 1.25;
    text-wrap: balance;
  }

  h2 a {
    text-decoration: none;
  }

  h2 a:hover {
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li + li {
    margin-top: var(--space-3);
  }

  blockquote {
    margin: 0;
    padding-left: var(--space-4);
    border-left: 3px solid var(--marker);
    font-size: 17px;
    line-height: 1.6;
    white-space: pre-line;
  }

  li .text-button {
    min-height: 32px;
    margin-left: var(--space-4);
  }

  .quiet {
    font-style: italic;
    color: var(--ink-soft);
  }

  .message {
    margin: var(--space-3) 0 0;
    font-family: var(--sans);
    font-size: 13px;
    color: var(--danger);
  }
</style>
