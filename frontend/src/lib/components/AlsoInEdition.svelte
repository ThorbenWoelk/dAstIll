<script lang="ts">
  import { describeReleaseDay, type Story } from "$lib/edition/stories";
  import { splitSummary } from "$lib/edition/summary";

  let {
    stories,
    remaining,
    onPick,
  }: {
    stories: Story[];
    /** Unread stories in this section beyond the lead. */
    remaining: number;
    onPick: (storyId: string) => void;
  } = $props();

  const more = $derived(remaining - stories.length);

  function describeMeta(story: Story): string {
    const day = describeReleaseDay(story.publishedAt);
    return day ? `${story.channelName} · ${day}` : story.channelName;
  }
</script>

<aside class="rail" aria-labelledby="rail-heading">
  <h2 id="rail-heading" class="label">Also in this edition</h2>
  {#if stories.length === 0}
    <p class="last">This is the last story in this section.</p>
  {:else}
    <ol>
      {#each stories as story (story.id)}
        <li>
          <button type="button" onclick={() => onPick(story.id)}>
            <span class="label meta">{describeMeta(story)}</span>
            <span class="title">{story.title}</span>
            <span class="standfirst"
              >{splitSummary(story.summary).standfirst}</span
            >
          </button>
        </li>
      {/each}
    </ol>
    {#if more > 0}
      <p class="more">and {more} more after these</p>
    {/if}
  {/if}
</aside>

<style>
  .rail {
    padding-top: var(--space-6);
  }

  h2 {
    margin: 0;
    padding-bottom: var(--space-2);
    border-bottom: 2px solid var(--rule);
  }

  ol {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    border-bottom: 1px solid var(--hairline);
  }

  button {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
    padding: var(--space-4) 0;
    border: 0;
    background: none;
    text-align: left;
  }

  button:hover .title {
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 3px;
  }

  .meta {
    font-size: 10px;
    color: var(--ink-faint);
  }

  .title {
    font-size: 19px;
    font-weight: 700;
    line-height: 1.25;
  }

  .standfirst {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    overflow: hidden;
    font-size: 14px;
    line-height: 1.45;
    color: var(--ink-soft);
  }

  .standfirst:empty {
    display: none;
  }

  .last,
  .more {
    margin: var(--space-4) 0 0;
    font-style: italic;
    color: var(--ink-soft);
  }

  @media (min-width: 960px) {
    .rail {
      position: sticky;
      top: var(--space-4);
      padding-top: var(--space-5);
    }
  }
</style>
