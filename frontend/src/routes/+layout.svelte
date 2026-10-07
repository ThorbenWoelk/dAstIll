<script lang="ts">
  import "@fontsource/libre-caslon-text/400.css";
  import "@fontsource/libre-caslon-text/400-italic.css";
  import "@fontsource/libre-caslon-text/700.css";
  import "@fontsource/libre-franklin/400.css";
  import "@fontsource/libre-franklin/600.css";
  import "@fontsource/libre-franklin/700.css";
  import "../app.css";

  import { onMount, type Snippet } from "svelte";
  import { dev } from "$app/environment";
  import Masthead from "$lib/components/Masthead.svelte";
  import SignIn from "$lib/components/SignIn.svelte";
  import { session } from "$lib/session.svelte";

  let { children }: { children: Snippet } = $props();

  onMount(() => {
    void session.start();
    if (!dev && "serviceWorker" in navigator) {
      navigator.serviceWorker.register("/sw.js").catch(() => undefined);
    }
  });
</script>

<div class="page">
  {#if session.status === "signed-in"}
    <!-- A new reader must not keep the previous paper or write into that account. -->
    {#key session.reader?.uid ?? ""}
      {@render children()}
    {/key}
  {:else}
    <Masthead signedIn={false} />
    {#if session.status === "signed-out"}
      <SignIn
        busy={session.busy}
        error={session.error}
        onSignIn={() => session.signIn()}
      />
    {/if}
  {/if}
</div>

<style>
  .page {
    max-width: var(--column);
    margin: 0 auto;
    padding: 0 var(--space-4);
  }

  @media (min-width: 640px) {
    .page {
      padding: 0 var(--space-7);
    }
  }
</style>
