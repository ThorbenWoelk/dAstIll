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
  import { afterNavigate } from "$app/navigation";
  import Masthead from "$lib/components/Masthead.svelte";
  import SignIn from "$lib/components/SignIn.svelte";
  import { watchScroll, type ScrollWatch } from "$lib/navigation/tuck";
  import { session } from "$lib/session.svelte";

  let { children }: { children: Snippet } = $props();

  /** True while the reader scrolls down: the tab and section bars tuck away. */
  let tucked = $state(false);
  let scroll: ScrollWatch | null = null;

  onMount(() => {
    void session.start();
    if (!dev && "serviceWorker" in navigator) {
      navigator.serviceWorker.register("/sw.js").catch(() => undefined);
    }
    scroll = watchScroll((value) => (tucked = value));
    return () => scroll?.stop();
  });

  afterNavigate(() => scroll?.reset());
</script>

<div
  class="page"
  class:has-tab-bar={session.status === "signed-in"}
  style:--tucked={tucked ? 1 : 0}
>
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
    /* Space the phone tab bar takes at the bottom of the screen right now. */
    --tab-bar-offset: 0px;
    max-width: var(--column);
    margin: 0 auto;
    padding: 0 var(--space-4);
    /* A story swiped sideways must not widen the page. */
    overflow-x: clip;
  }

  .page.has-tab-bar {
    --tab-bar-offset: calc(
      (1 - var(--tucked)) * (var(--tab-bar) + env(safe-area-inset-bottom))
    );
    padding-bottom: calc(var(--tab-bar) + env(safe-area-inset-bottom));
  }

  @media (min-width: 640px) {
    .page,
    .page.has-tab-bar {
      --tab-bar-offset: 0px;
      padding: 0 var(--space-7);
    }
  }
</style>
