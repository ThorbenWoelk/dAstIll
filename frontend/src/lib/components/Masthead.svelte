<script lang="ts">
  import type { Snippet } from "svelte";
  import {
    describeEditionDate,
    describeShortEditionDate,
  } from "$lib/edition/stories";

  let {
    status = "",
    actions,
    nav,
  }: {
    status?: string;
    actions?: Snippet;
    nav?: Snippet;
  } = $props();

  const today = new Date();
</script>

<header class="masthead">
  <div class="dateline">
    <p class="dateline-start">
      <time datetime={today.toISOString().slice(0, 10)}>
        <span class="long-date">{describeEditionDate(today)}</span>
        <span class="short-date">{describeShortEditionDate(today)}</span>
      </time>
      {#if status}<span aria-live="polite">· {status}</span>{/if}
    </p>
    <div class="dateline-end">
      {@render actions?.()}
    </div>
  </div>
  <a class="title" href="/">dAstIll</a>
  <div class="double-rule" aria-hidden="true"></div>
  {@render nav?.()}
</header>

<style>
  .masthead {
    padding-top: max(var(--space-2), env(safe-area-inset-top));
  }

  .dateline {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    min-height: var(--touch);
    font-family: var(--sans);
    font-size: 12px;
    color: var(--ink-soft);
  }

  .dateline-start {
    display: flex;
    gap: var(--space-1);
    margin: 0;
    white-space: nowrap;
  }

  .dateline-end {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .long-date {
    display: none;
  }

  .title {
    display: block;
    padding: 0 0 var(--space-3);
    font-size: 40px;
    font-weight: 700;
    line-height: 1;
    letter-spacing: -0.02em;
    text-align: center;
    text-decoration: none;
  }

  .double-rule {
    height: 2px;
    background: var(--rule);
  }

  @media (min-width: 640px) {
    .long-date {
      display: inline;
    }

    .short-date {
      display: none;
    }

    .title {
      padding: var(--space-2) 0 var(--space-4);
      font-size: 64px;
    }
  }
</style>
