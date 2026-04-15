<script lang="ts">
  import { CONTACT_EMAIL, DOCS_URL, SUPPORT_URL } from "$lib/app-config";
  import { authState } from "$lib/auth-state.svelte";

  let readerHref = $derived(
    authState.current.authState === "authenticated"
      ? "/mini"
      : "/login?redirectTo=%2Fmini",
  );
  let readerLabel = $derived(
    authState.current.authState === "authenticated"
      ? "Continue to dastill-mini"
      : "Sign in and continue to dastill-mini",
  );
</script>

<svelte:head>
  <title>dAstIll | Budget cap</title>
  <meta
    name="description"
    content="dAstIll is temporarily paused after hitting the current Turso budget cap. The docs site remains available."
  />
</svelte:head>

<div class="min-h-screen bg-[var(--background)] text-[var(--foreground)]">
  <main
    class="mx-auto flex min-h-screen max-w-3xl flex-col justify-center px-6 py-16 sm:px-10 lg:px-12"
  >
    <p
      class="text-[11px] font-bold uppercase tracking-[0.18em] text-[var(--soft-foreground)]"
    >
      dAstIll app
    </p>
    <h1
      class="mt-4 max-w-2xl font-['Fraunces',serif] text-4xl leading-tight tracking-[-0.03em] sm:text-5xl"
    >
      Sorry, we hit the budget cap :(
    </h1>
    <p
      class="mt-6 max-w-xl text-base leading-7 text-[var(--soft-foreground)] sm:text-lg"
    >
      The app is paused for the rest of this billing cycle. We’ll bring dAstIll
      back soon.
    </p>

    <div class="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center">
      <a
        class="inline-flex h-12 items-center justify-center rounded-full bg-[var(--foreground)] px-6 text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--background)] transition hover:bg-[var(--accent-strong)]"
        href={readerHref}
      >
        {readerLabel}
      </a>
    </div>

    <div
      class="mt-10 flex flex-col gap-4 text-sm text-[var(--soft-foreground)] sm:flex-row sm:items-center sm:gap-6"
    >
      {#if SUPPORT_URL}
        <a
          class="w-fit text-[var(--foreground)] underline decoration-[var(--accent)] underline-offset-4 transition-colors hover:text-[var(--accent)]"
          href={SUPPORT_URL}
          rel="noopener noreferrer"
          target="_blank"
        >
          Support dAstIll <span>💖</span>
        </a>
      {/if}

      <a
        class="w-fit text-[var(--foreground)] underline decoration-[var(--accent)] underline-offset-4 transition-colors hover:text-[var(--accent)]"
        href={DOCS_URL}
      >
        Browse the docs to find out more
      </a>

      {#if CONTACT_EMAIL}
        <a
          class="w-fit text-[var(--soft-foreground)] transition-colors hover:text-[var(--foreground)]"
          href={`mailto:${CONTACT_EMAIL}`}
        >
          {CONTACT_EMAIL}
        </a>
      {/if}
    </div>
  </main>
</div>
