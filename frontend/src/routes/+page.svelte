<script lang="ts">
  import { onMount } from "svelte";
  import { pushState } from "$app/navigation";
  import { page } from "$app/state";
  import AlsoInEdition from "$lib/components/AlsoInEdition.svelte";
  import EditionNotice from "$lib/components/EditionNotice.svelte";
  import FrontPage from "$lib/components/FrontPage.svelte";
  import Masthead from "$lib/components/Masthead.svelte";
  import RefreshIcon from "$lib/components/icons/RefreshIcon.svelte";
  import ReadBar from "$lib/components/ReadBar.svelte";
  import SectionNav from "$lib/components/SectionNav.svelte";
  import StoryArticle from "$lib/components/StoryArticle.svelte";
  import StoryMargin from "$lib/components/StoryMargin.svelte";
  import StoryPager from "$lib/components/StoryPager.svelte";
  import { EditionReader } from "$lib/edition/reader.svelte";
  import { StoryHighlights } from "$lib/highlights/story-highlights.svelte";
  import {
    describeStoriesLeft,
    FRONT_PAGE,
    type SectionId,
    type TurnDirection,
  } from "$lib/edition/stories";
  import { storySwipe } from "$lib/navigation/story-swipe";
  import { session } from "$lib/session.svelte";

  // The layout renders this page only for a signed-in reader.
  const reader = new EditionReader(session.reader?.uid ?? "unknown");
  const highlights = new StoryHighlights();
  /** How the next lead story arrives on the page. */
  let entrance = $state<"print" | TurnDirection>("print");
  /**
   * Wide screens open on the front page grid and read a story picked from
   * it. Narrower screens always show the reader; CSS decides which shows.
   */
  const storyOpen = $derived(page.state.storyOpen === true);

  onMount(() => {
    void reader.refresh();
  });

  $effect(() => {
    const storyId = reader.lead?.id;
    if (storyId) void highlights.show(storyId);
  });

  function backToTop() {
    window.scrollTo({ top: 0 });
  }

  function markAsRead() {
    entrance = "print";
    void reader.markLeadRead();
    backToTop();
  }

  function undoLastRead() {
    entrance = "print";
    void reader.undoLastRead();
    backToTop();
  }

  function pickStory(storyId: string) {
    entrance = "print";
    reader.pick(storyId);
    backToTop();
  }

  /** Read a story picked on the front page grid. Back returns to the grid. */
  function openStory(storyId: string) {
    entrance = "print";
    reader.pick(storyId);
    pushState("", { storyOpen: true });
    backToTop();
  }

  function backToFrontPage() {
    history.back();
  }

  /** Swipe, pager, and arrow keys: browse the section without reading. */
  function turn(direction: TurnDirection) {
    if (!reader.turn(direction)) return;
    entrance = direction;
    backToTop();
  }

  function chooseSection(section: SectionId) {
    entrance = "print";
    reader.showSection(section);
    backToTop();
  }

  function isTyping(target: EventTarget | null): boolean {
    return (
      target instanceof HTMLElement &&
      target.closest("input, textarea, select, [contenteditable='true']") !==
        null
    );
  }

  const NEXT_KEYS = ["ArrowRight", "j"];
  const PREVIOUS_KEYS = ["ArrowLeft", "k"];

  function handleShortcut(event: KeyboardEvent) {
    if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) {
      return;
    }
    if (event.defaultPrevented || isTyping(event.target)) return;
    if (event.key === "r" && reader.lead) {
      event.preventDefault();
      markAsRead();
    } else if (event.key === "u" && reader.lastRead) {
      event.preventDefault();
      undoLastRead();
    } else if (NEXT_KEYS.includes(event.key) && reader.canTurn("next")) {
      event.preventDefault();
      turn("next");
    } else if (
      PREVIOUS_KEYS.includes(event.key) &&
      reader.canTurn("previous")
    ) {
      event.preventDefault();
      turn("previous");
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
  current="front-page"
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
  {/snippet}
  {#snippet nav()}
    {#if reader.channels.length > 0}
      <div class="top-sections" class:story-open={storyOpen}>
        <SectionNav
          sections={reader.sections}
          active={reader.section}
          onSelect={chooseSection}
        />
      </div>
    {/if}
  {/snippet}
</Masthead>

{#if reader.notice}
  <EditionNotice
    message={reader.notice}
    onDismiss={() => reader.dismissNotice()}
  />
{/if}
{#if highlights.error}
  <EditionNotice
    message={highlights.error}
    onDismiss={() => highlights.dismissError()}
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
    <a class="press" href="/profile">Add a channel</a>
  </section>
{:else if reader.lead}
  {@const lead = reader.lead}
  <div class="edition" class:story-open={storyOpen}>
    <div class="front-view">
      <FrontPage
        stories={reader.visible}
        {lead}
        undoTitle={reader.lastRead?.title ?? null}
        onOpen={openStory}
        onRead={markAsRead}
        onUndo={undoLastRead}
      />
    </div>
    <div class="spread">
      <div class="side-sections">
        <SectionNav
          sections={reader.sections}
          active={reader.section}
          onSelect={chooseSection}
          orientation="column"
        />
        <dl class="keys" aria-label="Keyboard shortcuts">
          <dt>R</dt>
          <dd>Mark as read</dd>
          <dt>U</dt>
          <dd>Undo</dd>
          <dt>J / K</dt>
          <dd>Next / previous</dd>
        </dl>
      </div>
      <div class="lead">
        {#if storyOpen}
          <button type="button" class="back" onclick={backToFrontPage}>
            <span aria-hidden="true">‹</span> Front page
          </button>
        {/if}
        <StoryPager position={reader.position} onTurn={turn} />
        <div
          {@attach storySwipe({
            canGo: (direction) => reader.canTurn(direction),
            go: turn,
          })}
        >
          <StoryArticle
            story={reader.lead}
            {entrance}
            highlights={highlights.items}
            onHighlight={(draft) => highlights.add(lead.id, draft)}
            onRemoveHighlight={(id) => highlights.remove(lead.id, id)}
          >
            {#snippet footer()}
              <ReadBar
                undoTitle={reader.lastRead?.title ?? null}
                onRead={markAsRead}
                onUndo={undoLastRead}
              />
            {/snippet}
          </StoryArticle>
        </div>
      </div>
      <div class="rail">
        <div class="also">
          <AlsoInEdition
            stories={reader.alsoInEdition}
            remaining={reader.visible.length - 1}
            onPick={pickStory}
          />
        </div>
        <div class="margin-notes">
          <StoryMargin
            story={lead}
            highlights={highlights.items}
            onRemoveHighlight={(id) => highlights.remove(lead.id, id)}
          />
        </div>
      </div>
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
          onclick={() => chooseSection(FRONT_PAGE)}
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
    /* Story section headings stick just below the phone section bar. */
    --story-heading-top: calc((1 - var(--tucked, 0)) * (var(--touch) + 1px));
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

  /* No boxes of their own, so the rail's lists can stick beside the story. */
  .top-sections,
  .also {
    display: contents;
  }

  /* Wide screens only: the front page grid, side sections, and margin. */
  .front-view,
  .side-sections,
  .margin-notes {
    display: none;
  }

  .back {
    display: none;
    align-items: center;
    gap: var(--space-1);
    min-height: var(--touch);
    padding: 0 var(--space-2) 0 0;
    border: 0;
    background: none;
    font-family: var(--sans);
    font-size: 12px;
    font-weight: 600;
    color: var(--ink-soft);
  }

  .back:hover {
    color: var(--ink);
  }

  .keys {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--space-2) var(--space-3);
    margin: var(--space-6) 0 0;
    padding-top: var(--space-3);
    border-top: 1px solid var(--rule);
    font-family: var(--sans);
    font-size: 12px;
    color: var(--ink-soft);
  }

  .keys dt {
    font-weight: 700;
    color: var(--ink);
  }

  .keys dd {
    margin: 0;
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
    .spread {
      --story-heading-top: 0px;
    }

    .state h1 {
      font-size: 40px;
    }
  }

  /*
   * Wide screens: the front page opens on the grid. A story picked there
   * reads between the sections and the margin, and Back returns to the grid.
   */
  @media (min-width: 1360px) {
    .edition:not(.story-open) .spread,
    .top-sections.story-open,
    .also {
      display: none;
    }

    .edition:not(.story-open) .front-view {
      display: block;
    }

    .spread {
      display: grid;
      grid-template-columns: 200px minmax(0, 1fr) 300px;
      column-gap: var(--space-7);
      align-items: start;
    }

    .back {
      display: inline-flex;
    }

    .side-sections {
      position: sticky;
      top: var(--space-4);
      display: block;
      padding-top: var(--space-6);
    }

    /* Full height, so the margin can stick while the story scrolls. */
    .rail {
      align-self: stretch;
    }

    .margin-notes {
      display: contents;
    }
  }
</style>
