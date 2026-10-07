<script lang="ts">
  import type { Snippet } from "svelte";
  import {
    describeEditionDate,
    describeShortEditionDate,
  } from "$lib/edition/stories";
  import { PAPER_PAGES, type PaperPage } from "$lib/navigation/pages";

  let {
    status = "",
    current = null,
    signedIn = true,
    actions,
    nav,
  }: {
    status?: string;
    /** The page on screen; null for a single story. */
    current?: PaperPage | null;
    /** Signed-out readers see no page links. */
    signedIn?: boolean;
    actions?: Snippet;
    /** Rendered under the masthead, outside it, so it can stick on phones. */
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
    {#if signedIn}
      <div class="dateline-end">
        {@render actions?.()}
        <!-- A tab bar along the bottom on phones, links in the dateline from 640px. -->
        <nav class="pages" aria-label="Pages">
          {#each PAPER_PAGES as page (page.id)}
            <a
              href={page.href}
              class:own-front-page={page.id === "front-page" &&
                current === "front-page"}
              aria-current={page.id === current ? "page" : undefined}
            >
              {page.name}
            </a>
          {/each}
        </nav>
      </div>
    {/if}
  </div>
  <a class="title" href="/">dAstIll</a>
  <div class="double-rule" aria-hidden="true"></div>
</header>
{@render nav?.()}

<style>
  .masthead {
    padding-top: max(var(--space-2), env(safe-area-inset-top));
  }

  .dateline {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 0 var(--space-4);
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
    margin-left: auto;
  }

  /* Phones: a tab bar in thumb reach. It tucks away while reading down. */
  .pages {
    position: fixed;
    right: 0;
    bottom: 0;
    left: 0;
    z-index: 10;
    display: grid;
    grid-auto-columns: 1fr;
    grid-auto-flow: column;
    padding: 0 var(--space-2) env(safe-area-inset-bottom);
    border-top: 1px solid var(--rule);
    background: var(--paper);
    transform: translateY(calc(var(--tucked, 0) * 100%));
  }

  .pages a {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: var(--tab-bar);
    margin-top: -1px;
    border-top: 2px solid transparent;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-align: center;
    text-decoration: none;
    text-transform: uppercase;
    color: var(--ink-soft);
  }

  .pages a[aria-current="page"] {
    border-top-color: var(--kicker);
    color: var(--kicker);
  }

  @media (prefers-reduced-motion: no-preference) {
    .pages {
      transition: transform var(--tuck-ms) ease;
    }
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

    /* Wider screens: quiet links in the dateline, set like .text-button. */
    .pages {
      position: static;
      display: flex;
      gap: var(--space-3);
      padding: 0;
      border-top: 0;
      background: none;
      transform: none;
      transition: none;
    }

    .pages a {
      min-height: var(--touch);
      margin-top: 0;
      border-top: 0;
      font-size: 12px;
      font-weight: 400;
      letter-spacing: 0;
      text-decoration: underline;
      text-transform: none;
      text-underline-offset: 3px;
    }

    .pages a:hover {
      color: var(--ink);
    }

    .pages a[aria-current="page"] {
      color: var(--ink);
      font-weight: 700;
      text-decoration: none;
    }

    /* The front page does not link to itself. */
    .pages .own-front-page {
      display: none;
    }
  }
</style>
