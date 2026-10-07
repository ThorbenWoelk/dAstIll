<script lang="ts">
  import type { Snippet } from "svelte";
  import {
    describeEditionDate,
    describeShortEditionDate,
  } from "$lib/edition/stories";

  type PaperPage = "front-page" | "highlights" | "finished" | "sections";

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
    nav?: Snippet;
  } = $props();

  const PAGES: { id: PaperPage; name: string; href: string }[] = [
    { id: "front-page", name: "Front page", href: "/" },
    { id: "highlights", name: "Highlights", href: "/highlights" },
    { id: "finished", name: "Finished", href: "/finished" },
    { id: "sections", name: "Sections", href: "/sections" },
  ];

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
      <nav class="dateline-end" aria-label="Pages">
        {@render actions?.()}
        {#each PAGES as page (page.id)}
          {#if page.id !== "front-page" || current !== "front-page"}
            <a
              class="text-button"
              href={page.href}
              aria-current={page.id === current ? "page" : undefined}
            >
              {page.name}
            </a>
          {/if}
        {/each}
      </nav>
    {/if}
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

  .dateline-end [aria-current="page"] {
    color: var(--ink);
    font-weight: 700;
    text-decoration: none;
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
