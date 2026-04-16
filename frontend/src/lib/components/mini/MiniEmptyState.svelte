<script lang="ts">
  interface Props {
    variant:
      | "loading"
      | "error"
      | "no-subscriptions"
      | "all-read"
      | "no-summaries";
    errorMessage?: string;
    onClearFilter?: () => void;
    onRetry?: () => void;
  }

  let { variant, errorMessage, onClearFilter, onRetry }: Props = $props();
</script>

{#if variant === "loading"}
  <div class="skeleton-wrap">
    <div class="skel skel-hero"></div>
    <div class="skel-row">
      <div class="skel skel-meta-1"></div>
      <div class="skel skel-meta-2"></div>
    </div>
    <div class="skel skel-title"></div>
    <div class="skel-body">
      <div class="skel skel-line"></div>
      <div class="skel skel-line skel-line--short"></div>
      <div class="skel skel-line"></div>
    </div>
  </div>
{:else}
  <div class="empty-state">
    <div class="empty-inner">
      {#if variant === "error"}
        <p class="empty-label">Reader unavailable</p>
        <p class="empty-body">{errorMessage ?? "Something went wrong."}</p>
        {#if onRetry}
          <button type="button" class="empty-action" onclick={onRetry}>
            Try again
          </button>
        {/if}
      {:else if variant === "no-subscriptions"}
        <p class="empty-label">No subscriptions</p>
        <h2 class="empty-title">Nothing to read yet</h2>
        <p class="empty-body">
          dastill-mini shows summaries from channels you subscribed to. Add
          subscriptions in the full app when the main workspace is back.
        </p>
      {:else if variant === "all-read"}
        <p class="empty-label">No unread summaries</p>
        <h2 class="empty-title">You're all caught up</h2>
        <p class="empty-body">
          Every long-form summary in this channel is already marked as read.
        </p>
        {#if onClearFilter}
          <button type="button" class="empty-action" onclick={onClearFilter}>
            Clear filters
          </button>
        {/if}
      {:else if variant === "no-summaries"}
        <p class="empty-label">No summaries</p>
        <h2 class="empty-title">This channel is quiet for now</h2>
        <p class="empty-body">
          The selected channel has no long-form readable videos available yet.
        </p>
      {/if}
    </div>
  </div>
{/if}

<style>
  /* Skeleton loading */
  .skeleton-wrap {
    max-width: 680px;
    margin: 0 auto;
    padding: var(--space-lg) var(--space-md);
  }
  .skel {
    background: var(--muted);
    border-radius: 4px;
    animation: pulse-subtle 1.4s ease-in-out infinite;
  }
  .skel-hero {
    width: 100%;
    aspect-ratio: 16 / 9;
    border-radius: var(--radius-md);
    margin-bottom: var(--space-lg);
  }
  .skel-row {
    display: flex;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }
  .skel-meta-1 {
    width: 80px;
    height: 10px;
  }
  .skel-meta-2 {
    width: 60px;
    height: 10px;
  }
  .skel-title {
    width: 70%;
    height: 24px;
    margin-bottom: var(--space-xl);
  }
  .skel-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }
  .skel-line {
    width: 100%;
    height: 12px;
  }
  .skel-line--short {
    width: 65%;
  }

  @keyframes pulse-subtle {
    0%,
    100% {
      opacity: 0.4;
    }
    50% {
      opacity: 0.8;
    }
  }

  /* Empty states */
  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 60dvh;
    padding: var(--space-xl);
  }
  .empty-inner {
    max-width: 400px;
    text-align: center;
  }
  .empty-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--soft-foreground);
  }
  .empty-title {
    font-family: "Fraunces", serif;
    font-size: 28px;
    letter-spacing: -0.03em;
    margin-top: var(--space-sm);
  }
  .empty-body {
    font-size: 14px;
    line-height: 1.7;
    color: var(--soft-foreground);
    margin-top: var(--space-md);
  }
  .empty-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 40px;
    margin-top: var(--space-lg);
    border-radius: var(--radius-full);
    border: none;
    background: var(--foreground);
    color: var(--background);
    padding: 0 18px;
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    cursor: pointer;
    transition:
      background 120ms,
      color 120ms;
  }
  .empty-action:hover {
    background: var(--accent);
    color: white;
  }
</style>
