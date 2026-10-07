<script lang="ts">
  import { onMount } from "svelte";
  import { fetchFinishedStories, type FinishedSummaryItem } from "$lib/api";
  import Masthead from "$lib/components/Masthead.svelte";
  import { describeReleaseDay, readingMinutes } from "$lib/edition/stories";

  const PAGE_SIZE = 30;

  let stories = $state.raw<FinishedSummaryItem[]>([]);
  let hasMore = $state(false);
  let loading = $state(true);
  let failure = $state<string | null>(null);

  /** Stories grouped by the day they were finished, newest day first. */
  const days = $derived.by(() => {
    const groups: { day: string; stories: FinishedSummaryItem[] }[] = [];
    for (const story of stories) {
      const day = describeReleaseDay(story.finished_at) || "Earlier";
      const last = groups.at(-1);
      if (last?.day === day) last.stories.push(story);
      else groups.push({ day, stories: [story] });
    }
    return groups;
  });

  /** "Published today", "Published on Monday", "Published on 6 Oct". */
  function describePublished(publishedAt: string): string {
    const day = describeReleaseDay(publishedAt);
    if (day === "Today" || day === "Yesterday") {
      return `Published ${day.toLowerCase()}`;
    }
    return `Published on ${day}`;
  }

  function messageOf(cause: unknown): string {
    return cause instanceof Error ? cause.message : String(cause);
  }

  async function loadMore() {
    loading = true;
    failure = null;
    try {
      const page = await fetchFinishedStories(stories.length, PAGE_SIZE);
      const known = new Set(stories.map((story) => story.video_id));
      stories = [
        ...stories,
        ...page.stories.filter((story) => !known.has(story.video_id)),
      ];
      hasMore = page.has_more;
    } catch (cause) {
      failure = messageOf(cause);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadMore();
  });
</script>

<svelte:head>
  <title>Finished · dAstIll</title>
</svelte:head>

<Masthead current="finished" />

<section class="finished-page" aria-labelledby="finished-heading">
  <h1 id="finished-heading">Finished</h1>
  <p class="intro">Stories you marked as read, most recent first.</p>

  {#each days as group (group.day)}
    <h2 class="label day">{group.day}</h2>
    <ul class="story-list">
      {#each group.stories as story (story.video_id)}
        <li>
          <p class="label kicker">{story.channel_name}</p>
          <a
            class="title"
            href={`/stories/${encodeURIComponent(story.video_id)}`}
          >
            {story.title}
          </a>
          <p class="meta">
            {#if story.published_at}
              {describePublished(story.published_at)} ·
            {/if}
            {readingMinutes(story.summary_content)} min read
          </p>
        </li>
      {/each}
    </ul>
  {/each}

  {#if failure}
    <p class="message" role="alert">{failure}</p>
    <button type="button" class="text-button" onclick={loadMore}>
      Try again
    </button>
  {:else if loading}
    <p class="quiet" aria-live="polite">Fetching finished stories…</p>
  {:else if stories.length === 0}
    <p class="quiet">
      Nothing finished yet. Stories you mark as read land here.
    </p>
  {:else if hasMore}
    <button type="button" class="press more" onclick={loadMore}>
      Show older stories
    </button>
  {/if}
</section>

<style>
  .finished-page {
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
    margin: var(--space-2) 0 var(--space-6);
    font-style: italic;
    color: var(--ink-soft);
  }

  .day {
    margin: var(--space-5) 0 0;
    padding-bottom: var(--space-2);
    border-bottom: 2px solid var(--rule);
  }

  .story-list {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .story-list li {
    padding: var(--space-3) 0;
    border-bottom: 1px solid var(--hairline);
  }

  .kicker {
    margin: 0;
    color: var(--kicker);
  }

  .title {
    display: block;
    margin: var(--space-1) 0;
    font-size: 21px;
    font-weight: 700;
    line-height: 1.25;
    text-decoration: none;
    text-wrap: balance;
  }

  .title:hover {
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  .meta {
    margin: 0;
    font-family: var(--sans);
    font-size: 12px;
    color: var(--ink-soft);
  }

  .quiet {
    font-style: italic;
    color: var(--ink-soft);
  }

  .message {
    margin: var(--space-4) 0 0;
    font-family: var(--sans);
    font-size: 13px;
    color: var(--danger);
  }

  .more {
    margin-top: var(--space-5);
  }
</style>
