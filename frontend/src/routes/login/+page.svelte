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
    try {
      await authState.signInWithGoogle();
      await continueToWorkspace();
    } catch {
      // Error is handled in authState.error which is displayed in UI
    }
  }

  async function handleSignOut() {
    await authState.signOut();
  }
</script>

<svelte:head>
  <title>Sign in — dAstIll</title>
</svelte:head>

<div class="flex min-h-screen w-full bg-[var(--background)]">
  <!-- Desktop Visual Side -->
  <aside
    class="relative hidden w-[55%] flex-col items-center justify-center overflow-hidden border-r border-[var(--border-soft)] bg-[var(--surface-strong)] lg:flex"
  >
    <img
      src="/auth_background.png"
      alt=""
      class="absolute inset-0 h-full w-full object-cover opacity-50 contrast-[1.05] saturate-[0.85]"
      aria-hidden="true"
    />
    <div
      class="relative z-10 flex flex-col items-center space-y-6 px-12 text-center"
    >
      <div class="space-y-2">
        <h1
          class="font-serif text-[84px] leading-none tracking-[-0.04em] text-[var(--foreground)] fade-in"
        >
          dAstIll
        </h1>
        <p
          class="text-[11px] font-bold uppercase tracking-[0.2em] text-[var(--accent)] fade-in stagger-1"
        >
          Distillation Engine
        </p>
      </div>
      <p
        class="max-w-[360px] text-[18px] font-medium leading-relaxed text-[var(--soft-foreground)] fade-in stagger-2"
      >
        Deep listening. Precise distillation. Your personal library for video
        insights.
      </p>
    </div>
  </aside>

  <!-- Content Side -->
  <main
    class="relative flex flex-1 items-center justify-center px-6 py-12 lg:px-12"
  >
    <!-- Background for mobile -->
    <div class="fixed inset-0 z-0 lg:hidden">
      <img
        src="/auth_background.png"
        alt=""
        class="h-full w-full object-cover opacity-20"
        aria-hidden="true"
      />
    </div>

    <div class="relative z-10 w-full max-w-[400px] space-y-12">
      <!-- Header -->
      <header class="space-y-4 fade-in stagger-1">
        <div class="lg:hidden">
          <h1
            class="font-serif text-[42px] leading-none tracking-[-0.03em] text-[var(--foreground)]"
          >
            dAstIll
          </h1>
        </div>
        <div class="space-y-4">
          <h2
            class="font-serif text-[32px] font-semibold tracking-[-0.02em] text-[var(--foreground)]"
          >
            Welcome back
          </h2>
          <p class="text-[15px] leading-relaxed text-[var(--soft-foreground)]">
            Organize your wisdom. dAstIll allows you to build a permanent
            library of distilled knowledge from any video source.
          </p>
        </div>
      </header>

      <!-- Session Status -->
      <section
        class="space-y-4 rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--surface)] p-6 shadow-sm fade-in stagger-2"
      >
        <div class="space-y-1">
          <p
            class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-60"
          >
            Session Identity
          </p>
          <p
            class="text-[16px] font-semibold text-[var(--foreground)] {authState.syncing
              ? 'animate-pulse-subtle'
              : ''}"
          >
            {#if authState.current.authState === "authenticated"}
              {authState.current.email ?? "Signed in"}
            {:else if authState.current.userId}
              Anonymous Participant
            {:else if authState.syncing}
              Establishing connection...
            {:else}
              Connection offline
            {/if}
          </p>
          {#if authState.current.userId}
            <p
              class="font-mono text-[11px] text-[var(--soft-foreground)] transition-opacity hover:opacity-100 opacity-40"
            >
              {authState.current.userId}
            </p>
          {/if}
        </div>

        {#if authState.error}
          <div
            class="rounded-md bg-[var(--danger-soft)] p-3 text-[13px] text-[var(--danger-foreground)]"
          >
            {authState.error}
          </div>
        {/if}
      </section>

      <!-- Actions -->
      <nav class="space-y-3 fade-in stagger-3">
        {#if authState.current.authState === "authenticated"}
          <button
            type="button"
            class="inline-flex h-12 w-full items-center justify-center rounded-full bg-[var(--foreground)] px-6 text-[12px] font-bold uppercase tracking-[0.1em] text-[var(--background)] transition hover:scale-[1.01] hover:bg-[var(--accent-strong)] active:scale-[0.99] focus-visible:outline-none"
            onclick={continueToWorkspace}
          >
            Enter Workspace
          </button>
          <button
            type="button"
            class="inline-flex h-12 w-full items-center justify-center rounded-full border border-[var(--border)] bg-[var(--surface)] px-6 text-[12px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)] transition hover:bg-[var(--accent-wash)] focus-visible:outline-none"
            onclick={handleSignOut}
          >
            Sign out
          </button>
        {:else}
          <button
            type="button"
            class="inline-flex h-12 w-full items-center justify-center rounded-full bg-[var(--foreground)] px-6 text-[12px] font-bold uppercase tracking-[0.1em] text-[var(--background)] transition hover:scale-[1.01] hover:bg-[var(--accent-strong)] active:scale-[0.99] disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline-none"
            disabled={authState.syncing}
            onclick={handleGoogleSignIn}
          >
            <span class={authState.syncing ? "animate-pulse-subtle" : ""}>
              {authState.syncing ? "Connecting…" : "Continue with Google"}
            </span>
          </button>
          <button
            type="button"
            class="inline-flex h-12 w-full items-center justify-center rounded-full border border-[var(--border)] bg-[var(--surface)] px-6 text-[12px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)] transition hover:bg-[var(--accent-wash)] focus-visible:outline-none"
            onclick={continueToWorkspace}
          >
            Continue as Guest
          </button>
        {/if}
      </nav>

      <!-- Footer -->
      <footer class="pt-4 text-center fade-in stagger-3">
        <p class="text-[12px] text-[var(--soft-foreground)] opacity-60">
          By continuing, you agree to our zen approach to data privacy.
        </p>
      </footer>
    </div>
  </main>
</div>
