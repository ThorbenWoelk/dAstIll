<script lang="ts">
  import TranscriptView from "$lib/components/TranscriptView.svelte";
  import type { CreateHighlightRequest, Highlight } from "$lib/types";
  import type { MiniSummaryItem } from "$lib/transport-types";
  import MiniArticleAside from "./MiniArticleAside.svelte";
  import MiniArticleHeader from "./MiniArticleHeader.svelte";

  interface Props {
    summary: MiniSummaryItem;
    summaryHtml: string;
    contentKey: number;
    highlights: Highlight[];
    creatingHighlight: boolean;
    deletingHighlightId: number | null;
    onCreateHighlight: (
      payload: CreateHighlightRequest,
    ) => void | Promise<void>;
    onDeleteHighlight: (highlightId: number) => void | Promise<void>;
  }

  let {
    summary,
    summaryHtml,
    contentKey,
    highlights,
    creatingHighlight,
    deletingHighlightId,
    onCreateHighlight,
    onDeleteHighlight,
  }: Props = $props();
</script>

{#key contentKey}
  <article class="reader-article">
    <MiniArticleHeader
      channelName={summary.channel_name}
      title={summary.title}
    />

    <MiniArticleAside
      publishedAt={summary.published_at}
      watchUrl={summary.watch_url}
    />

    <div class="reader-body" aria-live="polite">
      <TranscriptView
        html={summaryHtml}
        text={summary.summary_content}
        mode="markdown"
        {highlights}
        highlightSource="summary"
        highlightEnabled={true}
        {creatingHighlight}
        {deletingHighlightId}
        {onCreateHighlight}
        {onDeleteHighlight}
      />
    </div>
  </article>
{/key}

<style>
  .reader-article {
    max-width: 680px;
    margin: 0 auto;
    padding: var(--space-lg) var(--space-md)
      calc(80px + env(safe-area-inset-bottom, 0px));
    animation: article-in 500ms ease-out;
  }
  @keyframes article-in {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .reader-body {
    color: var(--foreground);
    font-size: 17px;
    line-height: 1.75;
  }
  .reader-body :global(.workspace-article) {
    max-width: 100%;
    margin: 0;
  }
  .reader-body :global(h1),
  .reader-body :global(h2),
  .reader-body :global(h3) {
    font-family: "Fraunces", serif;
    letter-spacing: -0.02em;
    line-height: 1.2;
    margin-top: 2rem;
    margin-bottom: 0.75rem;
    font-variation-settings: "opsz" 72;
  }
  .reader-body :global(h2) {
    font-size: 1.45rem;
    border-top: 1px solid var(--border-soft);
    padding-top: 1.25rem;
    margin-top: 2.5rem;
  }
  .reader-body :global(h2:first-child) {
    border-top: none;
    padding-top: 0;
  }
  .reader-body :global(h3) {
    font-size: 1.15rem;
  }
  .reader-body :global(p) {
    margin: 0;
  }
  .reader-body :global(p + p) {
    margin-top: 1rem;
  }
  .reader-body :global(ul),
  .reader-body :global(ol) {
    margin: 1rem 0;
    padding-left: 1.25rem;
  }
  .reader-body :global(li + li) {
    margin-top: 0.35rem;
  }
  .reader-body :global(a) {
    color: var(--foreground);
    text-decoration: underline;
    text-decoration-color: var(--accent);
    text-decoration-thickness: 1px;
    text-underline-offset: 0.24em;
    transition: color 120ms;
  }
  .reader-body :global(a:hover) {
    color: var(--accent);
  }
  .reader-body :global(strong) {
    color: var(--foreground);
    font-weight: 700;
  }
  .reader-body :global(blockquote) {
    border-left: 2px solid var(--accent-soft);
    padding-left: var(--space-md);
    color: var(--soft-foreground);
    margin: 1.5rem 0;
    font-style: italic;
  }
  .reader-body :global(code) {
    font-size: 0.9em;
    background: var(--muted);
    padding: 2px 6px;
    border-radius: 4px;
  }
  .reader-body :global(pre) {
    background: var(--surface);
    padding: var(--space-md);
    border-radius: var(--radius-sm);
    overflow-x: auto;
    margin: 1.5rem 0;
  }
  .reader-body :global(pre code) {
    background: none;
    padding: 0;
  }

  @media (min-width: 640px) {
    .reader-article {
      padding: var(--space-xl) var(--space-lg) 80px;
    }
    .reader-body {
      font-size: 18px;
    }
  }

  @media (min-width: 960px) {
    .reader-article {
      max-width: 1040px;
      padding: var(--space-xl) var(--space-xl) var(--space-xl);
      display: grid;
      grid-template-columns: minmax(0, 1fr) 220px;
      grid-template-areas:
        "head aside"
        "body aside";
      column-gap: var(--space-xl);
      align-items: start;
    }
    .reader-article > :global(header) {
      grid-area: head;
      min-width: 0;
    }
    .reader-article > :global(aside) {
      grid-area: aside;
    }
    .reader-body {
      grid-area: body;
      min-width: 0;
      font-size: 17px;
      line-height: 1.65;
    }
    .reader-body :global(h1),
    .reader-body :global(h2),
    .reader-body :global(h3) {
      margin-top: 1.5rem;
      margin-bottom: 0.5rem;
    }
    .reader-body :global(p + p) {
      margin-top: 0.75rem;
    }
    .reader-body :global(ul),
    .reader-body :global(ol) {
      margin: 0.75rem 0;
    }
    .reader-body :global(blockquote) {
      margin: 1rem 0;
    }
  }
</style>
