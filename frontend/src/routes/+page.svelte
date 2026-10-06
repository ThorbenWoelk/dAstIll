<script lang="ts">
  import { onMount } from "svelte";
  import AlsoInEdition from "$lib/components/AlsoInEdition.svelte";
  import EditionNotice from "$lib/components/EditionNotice.svelte";
  import Masthead from "$lib/components/Masthead.svelte";
  import RefreshIcon from "$lib/components/icons/RefreshIcon.svelte";
  import ReadBar from "$lib/components/ReadBar.svelte";
  import SectionNav from "$lib/components/SectionNav.svelte";
  import StoryArticle from "$lib/components/StoryArticle.svelte";
  import { EditionReader } from "$lib/edition/reader.svelte";
  import { describeStoriesLeft, FRONT_PAGE } from "$lib/edition/stories";
  import { session } from "$lib/session.svelte";

  // The layout renders this page only for a signed-in reader.
  const reader = new EditionReader(session.reader?.uid ?? "unknown");

  onMount(() => {
    void reader.refresh();
  });

  function backToTop() {
    window.scrollTo({ top: 0 });
  }

  function markAsRead() {
    void reader.markLeadRead();
    backToTop();
  }

  function undoLastRead() {
    void reader.undoLastRead();
    backToTop();
  }

  function pickStory(storyId: string) {
    reader.pick(storyId);
    backToTop();
  }

  async function signOut() {
    reader.forget();
    await session.signOut();
  }

  function isTyping(target: EventTarget | null): boolean {
    return (
      target instanceof HTMLElement &&
      target.closest("input, textarea, select, [contenteditable='true']") !==
        null
    );
  }

  function handleShortcut(event: KeyboardEvent) {
    if (event.metaKey || event.ctrlKey || event.altKey) return;
    if (event.defaultPrevented || isTyping(event.target)) return;
    if (event.key === "r" && reader.lead) {
      event.preventDefault();
      markAsRead();
    } else if (event.key === "u" && reader.lastRead) {
      event.preventDefault();
      undoLastRead();
    }
  }

  function handleVisibility() {
    if (document.visibilityState === "visible") reader.refreshIfStale();
  }
</script>

<svelte:window onkeydown={handleShortcut} />
<svelte:document onvisibilitychange={handleVisibility} />
<svelte:head>
  <title>{reader.lead ? `${reader.lead.title} · dAstIll` : "dAstIll"}</title>
</svelte:head>

<Masthead
  status={reader.status === "ready"
    ? describeStoriesLeft(reader.visible.length)
    : ""}
>
  {#snippet actions()}
    <button
      type="button"
      class="icon-button"
      aria-label="Refresh edition"
      title="Refresh edition"
      onclick={() => reader.refresh()}
    >
      <RefreshIcon spinning={reader.refreshing} />
    </button>
    <a class="text-button" href="/sections">Sections</a>
    <button type="button" class="text-button" onclick={signOut}>
      Sign out
    </button>
  {/snippet}
  {#snippet nav()}
    {#if reader.channels.length > 0}
      <SectionNav
        sections={reader.sections}
        active={reader.section}
        onSelect={(section) => reader.showSection(section)}
      />
    {/if}
  {/snippet}
</Masthead>

{#if reader.notice}
  <EditionNotice
    message={reader.notice}
    onDismiss={() => reader.dismissNotice()}
  />
{/if}

{#if reader.status === "loading"}
  <p class="state" aria-live="polite">Printing today's edition…</p>
{:else if reader.status === "failed"}
  <section class="state" role="alert">
    <h1>The presses stopped.</h1>
    <p>{reader.failure}</p>
    <button type="button" class="press" onclick={() => reader.refresh()}>
      Try again
    </button>
  </section>
{:else if reader.channels.length === 0}
  <section class="state">
    <h1>Your paper has no sections yet.</h1>
    <p>Follow a YouTube channel or podcast and its videos arrive here.</p>
    <a class="press" href="/sections">Add a channel</a>
  </section>
{:else if reader.lead}
  <div class="spread">
    <div class="lead">
      <StoryArticle story={reader.lead}>
        {#snippet footer()}
          <ReadBar
            undoTitle={reader.lastRead?.title ?? null}
            onRead={markAsRead}
            onUndo={undoLastRead}
          />
        {/snippet}
      </StoryArticle>
    </div>
    <div class="rail">
      <AlsoInEdition
        stories={reader.alsoInEdition}
        remaining={reader.visible.length - 1}
        onPick={pickStory}
      />
    </div>
  </div>
{:else}
  <section class="state print-in">
    <h1>That is the whole edition.</h1>
    <p>New stories appear as soon as new videos are summarized.</p>
    <div class="state-actions">
      {#if reader.lastRead}
        <button type="button" class="text-button" onclick={undoLastRead}>
          Undo "{reader.lastRead.title}"
        </button>
      {/if}
      {#if reader.section !== FRONT_PAGE && reader.stories.length > 0}
        <button
          type="button"
          class="press"
          onclick={() => reader.showSection(FRONT_PAGE)}
        >
          Front page ({reader.stories.length})
        </button>
      {/if}
    </div>
  </section>
{/if}

<style>
  .icon-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--touch);
    height: var(--touch);
    margin-right: calc(var(--space-2) * -1);
    border: 0;
    background: none;
    color: var(--ink-soft);
  }

  .icon-button:hover {
    color: var(--ink);
  }

  .spread {
    display: flex;
    flex-wrap: wrap;
    column-gap: var(--space-7);
    padding-bottom: var(--space-8);
  }

  .lead {
    flex: 999 1 600px;
    min-width: 0;
  }

  .rail {
    flex: 1 1 260px;
    min-width: 0;
  }

  .state {
    max-width: 560px;
    margin: 0 auto;
    padding: var(--space-8) 0;
    font-style: italic;
    text-align: center;
    color: var(--ink-soft);
  }

  .state h1 {
    margin: 0 0 var(--space-3);
    font-size: 32px;
    font-style: normal;
    line-height: 1.15;
    color: var(--ink);
  }

  .state p {
    margin: 0 0 var(--space-5);
  }

  .state a.press {
    text-decoration: none;
  }

  .state-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    align-items: center;
    gap: var(--space-4);
    font-style: normal;
  }

  @media (min-width: 640px) {
    .state h1 {
      font-size: 40px;
    }
  }
</style>
