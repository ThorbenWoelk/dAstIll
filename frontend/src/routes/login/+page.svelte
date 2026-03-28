<script lang="ts">
  import { goto } from "$app/navigation";
  import { authState } from "$lib/auth-state.svelte";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  const workspaceHref = $derived(
    data.redirectTo === "/login" ? "/" : data.redirectTo,
  );

  async function continueToWorkspace() {
    await goto(workspaceHref);
  }

  async function handleGoogleSignIn() {
    await authState.signInWithGoogle();
  }

  async function handleSignOut() {
    await authState.signOut();
  }
</script>

<svelte:head>
  <title>Sign in — dAstIll</title>
</svelte:head>

<div class="flex min-h-full items-center justify-center px-6 py-16">
  <section
    class="w-full max-w-md space-y-6 rounded-[var(--radius-lg)] bg-[var(--surface)] px-6 py-7 shadow-sm"
  >
    <div class="space-y-2">
      <p
        class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-60"
      >
        Firebase auth
      </p>
      <h1
        class="font-serif text-[28px] font-semibold tracking-[-0.02em] text-[var(--foreground)]"
      >
        Sign in
      </h1>
      <p class="text-[14px] leading-6 text-[var(--soft-foreground)]">
        Anonymous browsing starts automatically. Use Google when you want an
        authenticated session attached to the app.
      </p>
    </div>

    <div
      class="space-y-3 rounded-[var(--radius-md)] bg-[var(--background)] px-4 py-4"
    >
      <div class="space-y-1">
        <p
          class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-60"
        >
          Current session
        </p>
        <p class="text-[15px] font-semibold text-[var(--foreground)]">
          {#if authState.current.authState === "authenticated"}
            Signed in as {authState.current.email ?? "user"}
          {:else if authState.current.userId}
            Anonymous session ready
          {:else if authState.syncing}
            Bootstrapping anonymous session
          {:else}
            No session cookie yet
          {/if}
        </p>
        <p class="text-[12px] leading-5 text-[var(--soft-foreground)]">
          {#if authState.current.userId}
            User ID: {authState.current.userId}
          {:else}
            The app will silently create an anonymous Firebase identity when
            needed.
          {/if}
        </p>
      </div>
    </div>

    {#if authState.error}
      <p class="text-[12px] text-[var(--danger)]">{authState.error}</p>
    {/if}

    <div class="space-y-3">
      {#if authState.current.authState === "authenticated"}
        <button
          type="button"
          class="inline-flex h-11 w-full items-center justify-center rounded-full bg-[var(--foreground)] px-5 text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--background)] transition hover:bg-[var(--accent-strong)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
          onclick={continueToWorkspace}
        >
          Continue to workspace
        </button>
        <button
          type="button"
          class="inline-flex h-11 w-full items-center justify-center rounded-full bg-[var(--surface)] px-5 text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--foreground)] transition hover:bg-[var(--background)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
          onclick={handleSignOut}
        >
          Return to anonymous session
        </button>
      {:else}
        <button
          type="button"
          class="inline-flex h-11 w-full items-center justify-center rounded-full bg-[var(--foreground)] px-5 text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--background)] transition hover:bg-[var(--accent-strong)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 disabled:cursor-not-allowed disabled:opacity-60"
          disabled={authState.syncing}
          onclick={handleGoogleSignIn}
        >
          {authState.syncing ? "Connecting…" : "Continue with Google"}
        </button>
        <button
          type="button"
          class="inline-flex h-11 w-full items-center justify-center rounded-full bg-[var(--surface)] px-5 text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--foreground)] transition hover:bg-[var(--background)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
          onclick={continueToWorkspace}
        >
          Continue without signing in
        </button>
      {/if}
    </div>
  </section>
</div>
