<script lang="ts">
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import { formatShortDate } from "$lib/utils/date";

  interface Props {
    channelName: string;
    publishedAt: string | null | undefined;
    title: string;
    watchUrl: string | null | undefined;
    read: boolean;
    markingRead: boolean;
    onMarkRead: () => void;
  }

  let {
    channelName,
    publishedAt,
    title,
    watchUrl,
    read,
    markingRead,
    onMarkRead,
  }: Props = $props();
</script>

<header class="header">
  <div class="meta">
    <span class="channel">{channelName}</span>
    {#if publishedAt}
      <span class="dot" aria-hidden="true"></span>
      <span class="date">{formatShortDate(publishedAt)}</span>
    {/if}
  </div>

  <h1 class="title">{title}</h1>

  <div class="actions">
    {#if read}
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
        {markingRead ? "Saving" : "Mark as read"}
      </button>
    {/if}

    {#if watchUrl}
      <a
        class="source-link"
        href={watchUrl}
        rel="noopener noreferrer"
        target="_blank"
      >
        Watch source
        <ExternalLinkIcon size={11} />
      </a>
    {/if}
  </div>
</header>

<style>
  .header {
    margin-bottom: var(--space-xl);
  }
  .meta {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }
  .channel {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--accent);
  }
  .dot {
    width: 3px;
    height: 3px;
    border-radius: var(--radius-full);
    background: var(--border);
    flex-shrink: 0;
  }
  .date {
    font-size: 12px;
    color: var(--soft-foreground);
  }
  .title {
    font-family: "Fraunces", serif;
    font-size: 28px;
    font-weight: 600;
    letter-spacing: -0.03em;
    line-height: 1.12;
    font-variation-settings: "opsz" 72;
    margin: 0 0 var(--space-md);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-md);
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
    background: var(--accent-soft);
    color: var(--accent-strong);
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
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--soft-foreground);
    text-decoration: none;
    transition: color 120ms;
  }
  .source-link:hover {
    color: var(--accent);
  }

  @media (min-width: 640px) {
    .title {
      font-size: 44px;
    }
  }
</style>
