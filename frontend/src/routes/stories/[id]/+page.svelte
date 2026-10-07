<script lang="ts">
  import { page } from "$app/state";
  import { ApiError, fetchStory, setStoryRead } from "$lib/api";
  import EditionNotice from "$lib/components/EditionNotice.svelte";
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import Masthead from "$lib/components/Masthead.svelte";
  import StoryArticle from "$lib/components/StoryArticle.svelte";
  import { toStory, type Story } from "$lib/edition/stories";
  import { StoryHighlights } from "$lib/highlights/story-highlights.svelte";

  type Status = "loading" | "ready" | "missing" | "failed";

  const storyId = $derived(page.params.id ?? "");
  const highlights = new StoryHighlights();

  let story = $state.raw<Story | null>(null);
  let read = $state(false);
  let status = $state<Status>("loading");
  let failure = $state<string | null>(null);
  let saving = $state(false);
  let notice = $state<string | null>(null);

  function messageOf(cause: unknown): string {
    return cause instanceof Error ? cause.message : String(cause);
  }

  async function load(id: string) {
    status = "loading";
    story = null;
    try {
      const item = await fetchStory(id);
      if (storyId !== id) return;
      story = toStory(item);
      read = item.read;
      status = "ready";
    } catch (cause) {
      if (storyId !== id) return;
      if (cause instanceof ApiError && cause.status === 404) {
        status = "missing";
      } else {
        failure = messageOf(cause);
        status = "failed";
      }
    }
  }

  $effect(() => {
    const id = storyId;
    void load(id);
    void highlights.show(id);
  });

  async function setRead(next: boolean) {
    if (!story || saving) return;
    saving = true;
    read = next;
    try {
      await setStoryRead(story.id, next);
    } catch (cause) {
      read = !next;
      notice = `Could not save: ${messageOf(cause)}`;
    } finally {
      saving = false;
    }
  }
</script>

<svelte:head>
  <title>{story ? `${story.title} · dAstIll` : "dAstIll"}</title>
</svelte:head>

<Masthead />

{#if notice}
  <EditionNotice message={notice} onDismiss={() => (notice = null)} />
{/if}
{#if highlights.error}
  <EditionNotice
    message={highlights.error}
    onDismiss={() => highlights.dismissError()}
  />
{/if}

{#if status === "loading"}
  <p class="state" aria-live="polite">Finding the story…</p>
{:else if status === "missing"}
  <section class="state">
    <h1>This story is not in your paper.</h1>
    <p>Its channel may no longer be one of your sections.</p>
    <a class="press" href="/">Front page</a>
  </section>
{:else if status === "failed"}
  <section class="state" role="alert">
    <h1>The story did not load.</h1>
    <p>{failure}</p>
    <button type="button" class="press" onclick={() => load(storyId)}>
      Try again
    </button>
  </section>
{:else if story}
  <div class="single">
    <StoryArticle
      {story}
      highlights={highlights.items}
      onHighlight={(draft) => highlights.add(storyId, draft)}
      onRemoveHighlight={(id) => highlights.remove(storyId, id)}
    >
      {#snippet footer()}
        <div class="read-state">
          {#if read}
            <p>You finished this story.</p>
            <button
              type="button"
              class="text-button"
              disabled={saving}
              onclick={() => setRead(false)}
            >
              Mark as unread
            </button>
          {:else}
            <button
              type="button"
              class="press"
              disabled={saving}
              onclick={() => setRead(true)}
            >
              <CheckIcon />
              Mark as read
            </button>
          {/if}
        </div>
      {/snippet}
    </StoryArticle>
  </div>
{/if}

<style>
  .single {
    max-width: 860px;
    margin: 0 auto;
    padding-bottom: var(--space-8);
  }

  .read-state {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0 var(--space-4);
    margin-top: var(--space-6);
    padding-top: var(--space-3);
    border-top: 1px solid var(--rule);
    font-family: var(--sans);
    font-size: 13px;
    color: var(--ink-soft);
  }

  .read-state p {
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
</style>
