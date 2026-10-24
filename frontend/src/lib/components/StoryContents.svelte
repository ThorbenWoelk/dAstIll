<script lang="ts">
  import type { ContentsEntry } from "$lib/edition/contents";

  let {
    entries,
    onJump,
  }: {
    entries: ContentsEntry[];
    onJump: (entry: ContentsEntry) => void;
  } = $props();
</script>

<nav class="contents" aria-label="In this story" data-passage-skip>
  <span class="label heading">In this story</span>
  <ul>
    {#each entries as entry, index (index)}
      <li>
        <button type="button" class="label" onclick={() => onJump(entry)}>
          {entry.title}
        </button>
      </li>
    {/each}
  </ul>
</nav>

<style>
  /*
   * One line right under the byline (pulled up over its bottom margin)
   * that scrolls sideways when the sections do not fit.
   */
  .contents {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    margin: calc(var(--space-5) * -1) 0 var(--space-5);
    border-bottom: 1px solid var(--hairline);
    overflow-x: auto;
    scrollbar-width: none;
  }

  .contents::-webkit-scrollbar {
    display: none;
  }

  .heading {
    flex: none;
    color: var(--ink-faint);
  }

  ul {
    display: flex;
    gap: var(--space-4);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  button {
    min-height: var(--touch);
    padding: 0;
    border: 0;
    background: none;
    color: var(--ink-soft);
    white-space: nowrap;
  }

  button:hover {
    color: var(--kicker);
  }
</style>
