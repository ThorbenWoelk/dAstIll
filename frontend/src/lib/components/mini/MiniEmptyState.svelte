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
  <div class="skeleton-wrap" aria-busy="true" aria-label="Loading article">
    <div class="skel skel-kicker"></div>
    <div class="skel-title-block">
      <div class="skel skel-title-1"></div>
      <div class="skel skel-title-2"></div>
    </div>
    <div class="skel-aside-row">
      <div class="skel skel-aside-1"></div>
      <div class="skel skel-aside-2"></div>
    </div>
    <div class="skel-body">
      <div class="skel skel-line"></div>
      <div class="skel skel-line"></div>
      <div class="skel skel-line skel-line--85"></div>
      <div class="skel skel-line skel-line--70"></div>
    </div>
    <div class="skel-body skel-body--gap">
      <div class="skel skel-line"></div>
      <div class="skel skel-line"></div>
      <div class="skel skel-line skel-line--90"></div>
      <div class="skel skel-line skel-line--55"></div>
    </div>
    <p class="skel-stage-label">Loading…</p>
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
    padding: var(--space-lg) var(--space-md)
      calc(80px + env(safe-area-inset-bottom, 0px));
  }
  .skel {
    background: var(--muted);
    border-radius: 4px;
    animation: pulse-subtle 1.4s ease-in-out infinite;
  }
  .skel-kicker {
    width: 96px;
    height: 10px;
    margin-bottom: var(--space-sm);
  }
  .skel-title-block {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-bottom: var(--space-md);
  }
  .skel-title-1 {
    width: 88%;
    height: 28px;
    border-radius: 6px;
  }
  .skel-title-2 {
    width: 62%;
    height: 28px;
    border-radius: 6px;
  }
  .skel-aside-row {
    display: flex;
    gap: var(--space-md);
    margin-bottom: var(--space-xl);
  }
  .skel-aside-1 {
    width: 72px;
    height: 10px;
  }
  .skel-aside-2 {
    width: 100px;
    height: 10px;
  }
  .skel-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }
  .skel-body--gap {
    margin-top: var(--space-lg);
  }
  .skel-line {
    width: 100%;
    height: 13px;
  }
  .skel-line--85 {
    width: 85%;
  }
  .skel-line--90 {
    width: 90%;
  }
  .skel-line--70 {
    width: 70%;
  }
  .skel-line--55 {
    width: 55%;
  }
  .skel-stage-label {
    margin-top: var(--space-xl);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--soft-foreground);
    opacity: 0.5;
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
    opacity: 0.8;
  }
</style>
