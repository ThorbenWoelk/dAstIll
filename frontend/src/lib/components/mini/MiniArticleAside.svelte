<script lang="ts">
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import { formatShortDate } from "$lib/utils/date";

  interface Props {
    publishedAt: string | null | undefined;
    watchUrl: string | null | undefined;
  }

  let { publishedAt, watchUrl }: Props = $props();
</script>

<aside class="aside">
  {#if publishedAt}
    <div class="field">
      <span class="label">Published</span>
      <time class="value value--serif">{formatShortDate(publishedAt)}</time>
    </div>
  {/if}

  {#if watchUrl}
    <div class="field">
      <span class="label">Source</span>
      <a
        class="value source"
        href={watchUrl}
        rel="noopener noreferrer"
        target="_blank"
      >
        Watch on YouTube
        <ExternalLinkIcon size={11} />
      </a>
    </div>
  {/if}
</aside>

<style>
  .aside {
    display: flex;
    flex-direction: row;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-sm) var(--space-md);
    margin-bottom: var(--space-lg);
  }
  .field {
    display: flex;
    align-items: baseline;
    gap: var(--space-xs);
  }
  .label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--soft-foreground);
  }
  .value {
    font-size: 12px;
    color: var(--foreground);
  }
  .value--serif {
    font-family: "Fraunces", serif;
    font-size: 13px;
    font-style: italic;
  }
  .source {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--foreground);
    text-decoration: none;
    transition: color 120ms;
  }
  .source:hover {
    color: var(--soft-foreground);
  }

  @media (min-width: 960px) {
    .aside {
      flex-direction: column;
      align-items: stretch;
      gap: var(--space-md);
      margin: 0;
      padding: 0;
      position: sticky;
      top: var(--space-lg);
      align-self: start;
    }
    .field {
      flex-direction: column;
      align-items: flex-start;
      gap: 2px;
    }
  }
</style>
