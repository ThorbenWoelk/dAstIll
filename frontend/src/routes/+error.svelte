<script lang="ts">
  import { page } from "$app/state";
</script>

<section class="error-page">
  <p class="label">Error {page.status}</p>
  <h1>This page is not in today's paper.</h1>
  <a class="text-button" href="/">Back to the front page</a>
</section>

<style>
  .error-page {
    padding: var(--space-8) 0;
    text-align: center;
  }

  .label {
    color: var(--kicker);
  }

  h1 {
    margin: var(--space-2) 0 var(--space-5);
    font-size: 32px;
  }
</style>
