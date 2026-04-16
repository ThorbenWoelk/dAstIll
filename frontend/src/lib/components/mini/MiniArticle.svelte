<script lang="ts">
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import TranscriptView from "$lib/components/TranscriptView.svelte";
  import type { CreateHighlightRequest, Highlight } from "$lib/types";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import type { MiniSummaryItem } from "$lib/transport-types";

  interface Props {
    summary: MiniSummaryItem;
    summaryHtml: string;
    markingRead: boolean;
    contentKey: number;
    highlights: Highlight[];
    creatingHighlight: boolean;
    deletingHighlightId: number | null;
    onMarkRead: () => void;
    onCreateHighlight: (
      payload: CreateHighlightRequest,
    ) => void | Promise<void>;
    onDeleteHighlight: (highlightId: number) => void | Promise<void>;
  }

  let {
    summary,
    summaryHtml,
    markingRead,
    contentKey,
    highlights,
    creatingHighlight,
    deletingHighlightId,
    onMarkRead,
    onCreateHighlight,
    onDeleteHighlight,
  }: Props = $props();

  function formatDate(dateStr: string | null | undefined): string {
    if (!dateStr) return "";
    const d = new Date(dateStr);
    return d.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }
</script>

{#key contentKey}
  <article class="reader-article">
    {#if summary.thumbnail_url}
      <div class="hero-thumb-wrap">
        <img
          class="hero-thumb"
          src={summary.thumbnail_url}
          alt={summary.title}
        />
      </div>
    {/if}

    <div class="reader-header">
      <div class="reader-meta">
        <span class="reader-channel">{summary.channel_name}</span>
        {#if summary.published_at}
          <span class="reader-date">{formatDate(summary.published_at)}</span>
        {/if}
      </div>

      <h1 class="reader-title">{summary.title}</h1>

      <div class="reader-actions">
        {#if summary.read}
          <span class="read-pill read-pill--done">
            <CheckIcon size={12} className="read-pill-icon" />
            Read
          </span>
        {:else}
          <button
            type="button"
            class="read-pill read-pill--action"
            disabled={markingRead}
            onclick={onMarkRead}
          >
            {#if markingRead}
              Saving
            {:else}
              Mark as read
            {/if}
          </button>
        {/if}

        <a
          class="source-link"
          href={summary.watch_url}
          rel="noopener noreferrer"
          target="_blank"
        >
          Watch source
          <ExternalLinkIcon size={12} />
        </a>
      </div>
    </div>

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
    animation: article-in 400ms ease-out;
  }
  @keyframes article-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .hero-thumb-wrap {
    margin-bottom: var(--space-lg);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .hero-thumb {
    display: block;
    width: 100%;
    aspect-ratio: 16 / 9;
    object-fit: cover;
  }

  .reader-header {
    margin-bottom: var(--space-xl);
  }
  .reader-meta {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }
  .reader-channel {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--accent);
  }
  .reader-date {
    font-size: 12px;
    color: var(--soft-foreground);
  }
  .reader-title {
    font-family: "Fraunces", serif;
    font-size: 26px;
    font-weight: 600;
    letter-spacing: -0.03em;
    line-height: 1.15;
    font-variation-settings: "opsz" 72;
    margin-bottom: var(--space-md);
  }

  .reader-actions {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }
  .read-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 32px;
    padding: 0 14px;
    border-radius: var(--radius-full);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    border: none;
    cursor: default;
    transition:
      background 120ms,
      color 120ms;
  }
  .read-pill--done {
    background: var(--surface);
    color: var(--soft-foreground);
  }
  :global(.read-pill-icon) {
    flex-shrink: 0;
  }
  .read-pill--action {
    background: var(--foreground);
    color: var(--background);
    cursor: pointer;
  }
  .read-pill--action:hover:not(:disabled) {
    background: var(--accent);
  }
  .read-pill--action:disabled {
    opacity: 0.6;
    cursor: wait;
  }
  .source-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 32px;
    padding: 0 14px;
    border-radius: var(--radius-full);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--foreground);
    background: var(--surface);
    text-decoration: none;
    transition: background 120ms;
  }
  .source-link:hover {
    background: var(--accent-wash);
  }

  /* Markdown body */
  .reader-body {
    color: var(--foreground);
    font-size: 1rem;
    line-height: 1.9;
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
  }
  .reader-body :global(h2) {
    font-size: 1.4rem;
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
    text-underline-offset: 0.22em;
  }
  .reader-body :global(a:hover) {
    color: var(--accent);
  }
  .reader-body :global(strong) {
    color: var(--foreground);
    font-weight: 700;
  }
  .reader-body :global(blockquote) {
    border-left: 2px solid var(--border);
    padding-left: var(--space-md);
    color: var(--soft-foreground);
    margin: 1.5rem 0;
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
    .reader-title {
      font-size: 40px;
    }
  }
</style>
