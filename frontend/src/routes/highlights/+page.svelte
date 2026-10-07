<script lang="ts">
  import { onMount } from "svelte";
  import {
    deleteHighlight,
    listHighlights,
    type HighlightChannelGroup,
  } from "$lib/api";
  import Masthead from "$lib/components/Masthead.svelte";
  import {
    countHighlights,
    filterHighlightGroups,
    withoutHighlight,
  } from "$lib/highlights/collection";

  let groups = $state.raw<HighlightChannelGroup[]>([]);
  let loading = $state(true);
  let failure = $state<string | null>(null);
  let removeError = $state<string | null>(null);
  let query = $state("");

  const shown = $derived(filterHighlightGroups(groups, query));
  const total = $derived(countHighlights(groups));
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
      groups = await listHighlights();
    } catch (cause) {
      failure = messageOf(cause);
    } finally {
      loading = false;
    }
  }

  async function remove(highlightId: string) {
    const before = groups;
    groups = withoutHighlight(groups, highlightId);
    removeError = null;
    try {
      await deleteHighlight(highlightId);
    } catch (cause) {
      groups = before;
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
    Passages you marked while reading. Select text in any story to add one.
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

    {#each shown as channel (channel.channel_id)}
      <section class="channel" aria-label={channel.channel_name}>
        <h2 class="label channel-name">{channel.channel_name}</h2>
        {#each channel.videos as video (video.video_id)}
          <article class="video">
            <h3>
              <a href={`/stories/${encodeURIComponent(video.video_id)}`}>
                {video.title}
              </a>
            </h3>
            <ul>
              {#each video.highlights as highlight (highlight.id)}
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
        {/each}
      </section>
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

  .channel-name {
    margin: var(--space-6) 0 0;
    padding-bottom: var(--space-2);
    border-bottom: 2px solid var(--rule);
    color: var(--kicker);
  }

  .video {
    padding: var(--space-4) 0;
    border-bottom: 1px solid var(--hairline);
  }

  h3 {
    margin: 0 0 var(--space-3);
    font-size: 21px;
    line-height: 1.25;
    text-wrap: balance;
  }

  h3 a {
    text-decoration: none;
  }

  h3 a:hover {
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
