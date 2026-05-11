<script lang="ts">
  import { page } from "$app/state";
  import { CONTACT_EMAIL, SUPPORT_URL } from "$lib/config/app";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import RefreshIcon from "$lib/components/icons/RefreshIcon.svelte";

  let supportHref = $derived(
    CONTACT_EMAIL ? `mailto:${CONTACT_EMAIL}` : SUPPORT_URL,
  );
  let supportExternal = $derived(!CONTACT_EMAIL && Boolean(SUPPORT_URL));
  let statusLabel = $derived(
    page.status === 404 ? "Page not found" : `Error ${page.status}`,
  );
  let heading = $derived(
    page.status === 404
      ? "This page is out of reach."
      : "Something interrupted dAstIll.",
  );
  let message = $derived(
    page.status === 404
      ? "The link may be old, or the page may have moved."
      : "The app hit an unexpected failure while loading this view.",
  );

  function reloadPage() {
    window.location.reload();
  }
</script>

<svelte:head>
  <title>{statusLabel} | dAstIll</title>
  <meta
    name="description"
    content="dAstIll could not load this page. Go home, reload the page, or contact support."
  />
</svelte:head>

<main class="min-h-screen bg-[var(--background)] text-[var(--foreground)]">
  <section
    class="mx-auto flex min-h-screen w-full max-w-3xl flex-col justify-center px-6 py-16 sm:px-10 lg:px-12"
  >
    <p
      class="text-[11px] font-bold uppercase tracking-[0.18em] text-[var(--danger-foreground)]"
    >
      {statusLabel}
    </p>

    <h1
      class="mt-4 max-w-2xl font-['Fraunces',serif] text-4xl leading-tight sm:text-5xl"
    >
      {heading}
    </h1>

    <p
      class="mt-6 max-w-xl text-base leading-7 text-[var(--soft-foreground)] sm:text-lg"
    >
      {message}
    </p>

    <div class="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center">
      <a
        class="inline-flex h-12 items-center justify-center gap-2 rounded-full bg-[var(--foreground)] px-6 text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--background)] transition hover:bg-[var(--accent-strong)]"
        href="/"
      >
        Go home
        <ChevronIcon direction="right" size={14} />
      </a>

      <button
        type="button"
        class="inline-flex h-12 items-center justify-center gap-2 rounded-full bg-[var(--surface-strong)] px-6 text-[11px] font-bold uppercase tracking-[0.12em] text-[var(--foreground)] transition hover:bg-[var(--accent-wash)]"
        onclick={reloadPage}
      >
        <RefreshIcon size={16} />
        Reload
      </button>
    </div>

    <p class="mt-10 max-w-xl text-sm leading-6 text-[var(--soft-foreground)]">
      If this keeps happening,
      <a
        class="inline-flex items-center gap-1 text-[var(--foreground)] underline decoration-[var(--accent)] underline-offset-4 transition-colors hover:text-[var(--accent)]"
        href={supportHref}
        target={supportExternal ? "_blank" : undefined}
        rel={supportExternal ? "noopener noreferrer" : undefined}
      >
        contact support
        {#if supportExternal}
          <ExternalLinkIcon size={14} />
        {/if}
      </a>
      and include the page you were trying to open.
    </p>
  </section>
</main>
