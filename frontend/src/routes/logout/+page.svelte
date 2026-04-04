<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import { normalizeRedirectTarget } from "$lib/auth";
  import { authState } from "$lib/auth-state.svelte";

  const redirectTarget = $derived(
    (() => {
      const redirectTo = normalizeRedirectTarget(
        page.url.searchParams.get("redirectTo"),
      );
      return redirectTo === "/logout" ? "/" : redirectTo;
    })(),
  );

  onMount(() => {
    void (async () => {
      await authState.signOut();
      await goto(redirectTarget, { replaceState: true });
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
