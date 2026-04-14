<script lang="ts">
  import { CONTACT_EMAIL, DOCS_URL } from "$lib/app-config";
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
      The app is paused for the rest of this billing cycle. We’ll bring
      dAstIll back soon.
    </p>

    <div
      class="mt-10 flex flex-col gap-4 text-sm text-[var(--soft-foreground)] sm:flex-row sm:items-center sm:gap-6"
    >
      <a
        class="w-fit text-[var(--foreground)] underline decoration-[var(--accent)] underline-offset-4 transition-colors hover:text-[var(--accent)]"
        href={DOCS_URL}
      >
        Browse the docs if you'd like to learn more.
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
