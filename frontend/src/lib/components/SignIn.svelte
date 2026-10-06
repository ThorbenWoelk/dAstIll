<script lang="ts">
  let {
    busy,
    error,
    onSignIn,
  }: {
    busy: boolean;
    error: string | null;
    onSignIn: () => void;
  } = $props();
</script>

<section class="sign-in print-in" aria-labelledby="sign-in-heading">
  <h1 id="sign-in-heading">Your channels, printed as a quiet morning paper.</h1>
  <p>
    Every new video from the channels you follow arrives as a short article.
    Read it, mark it as read, and the next one takes its place.
  </p>
  <button type="button" class="press" onclick={onSignIn} disabled={busy}>
    {busy ? "Signing in…" : "Sign in with Google"}
  </button>
  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</section>

<style>
  .sign-in {
    max-width: 560px;
    margin: 0 auto;
    padding: var(--space-8) 0;
    text-align: center;
  }

  h1 {
    margin: 0 0 var(--space-4);
    font-size: 32px;
    line-height: 1.15;
    text-wrap: balance;
  }

  p {
    margin: 0 0 var(--space-6);
    font-style: italic;
    color: var(--ink-soft);
  }

  .error {
    margin: var(--space-4) 0 0;
    font-style: normal;
    color: var(--danger);
  }

  @media (min-width: 640px) {
    h1 {
      font-size: 44px;
    }
  }
</style>
