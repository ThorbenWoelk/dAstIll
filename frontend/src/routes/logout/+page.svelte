<script lang="ts">
  import { onMount } from "svelte";
  import { signOutAndReloadHome } from "$lib/auth/logout";

  onMount(() => {
    void (async () => {
      await signOutAndReloadHome();
    })();
  });
</script>

<svelte:head>
  <title>Signing out — dAstIll</title>
</svelte:head>

<div
  class="flex min-h-screen items-center justify-center bg-[var(--background)] px-6 text-center"
>
  <p class="text-[14px] text-[var(--soft-foreground)]">Signing out…</p>
</div>
