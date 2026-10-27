<script lang="ts">
  import { onMount } from "svelte";
  import {
    deleteWritingFlag,
    listWritingFlags,
    type WritingFlag,
  } from "$lib/api";

  let flags = $state<WritingFlag[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let removeError = $state<string | null>(null);

  function messageOf(cause: unknown): string {
    return cause instanceof Error ? cause.message : String(cause);
  }

  async function loadFlags() {
    loading = true;
    loadError = null;
    try {
      flags = await listWritingFlags();
    } catch (cause) {
      loadError = messageOf(cause);
    } finally {
      loading = false;
    }
  }

  async function removeFlag(flag: WritingFlag) {
    removeError = null;
    const before = flags;
    flags = flags.filter((f) => f.id !== flag.id);
    try {
      await deleteWritingFlag(flag.id);
    } catch (cause) {
      flags = before;
      removeError = `Could not remove the flag: ${messageOf(cause)}`;
    }
  }

  onMount(() => {
    void loadFlags();
  });
</script>

<!-- Passages the reader flagged as bad writing, and why. -->
<section class="part" aria-labelledby="writing-feedback-heading">
  <h2 id="writing-feedback-heading" class="label part-heading">
    Writing feedback
  </h2>
  <p class="intro">
    Passages you flagged as bad writing. New summaries are written to avoid
    them. Select text in a story and choose Flag writing to add one.
  </p>

  {#if loading}
    <p class="quiet">Loading flagged passages…</p>
  {:else if loadError}
    <p class="message" role="alert">{loadError}</p>
    <button type="button" class="text-button" onclick={loadFlags}>
      Try again
    </button>
  {:else if flags.length === 0}
    <p class="quiet">You have not flagged any passages yet.</p>
  {:else}
    {#if removeError}
      <p class="message" role="alert">{removeError}</p>
    {/if}
    <ul class="flag-list">
      {#each flags as flag (flag.id)}
        <li>
          <blockquote>{flag.text}</blockquote>
          <p class="note">{flag.comment}</p>
          <div class="actions">
            <a
              class="text-button"
              href={`/stories/${encodeURIComponent(flag.video_id)}`}
            >
              Open story
            </a>
            <button
              type="button"
              class="text-button"
              aria-label={`Remove flag: ${flag.comment}`}
              onclick={() => removeFlag(flag)}
            >
              Remove
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .part {
    margin-top: var(--space-7);
  }

  .part-heading {
    margin: 0;
    padding-bottom: var(--space-2);
    border-bottom: 2px solid var(--rule);
  }

  .intro {
    margin: var(--space-3) 0 var(--space-4);
    font-style: italic;
    color: var(--ink-soft);
  }

  .quiet {
    font-style: italic;
    color: var(--ink-soft);
  }

  .message {
    margin: var(--space-3) 0 0;
    font-family: var(--sans);
    font-size: 13px;
    color: var(--danger);
  }

  .flag-list {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .flag-list li {
    padding: var(--space-3) 0;
    border-bottom: 1px solid var(--hairline);
  }

  blockquote {
    margin: 0;
    font-style: italic;
    text-decoration: underline wavy var(--danger);
    text-decoration-thickness: 1px;
    text-underline-offset: 4px;
    overflow-wrap: anywhere;
  }

  .note {
    margin: var(--space-2) 0 0;
    font-family: var(--sans);
    font-size: 14px;
  }

  .actions {
    display: flex;
    gap: var(--space-4);
  }
</style>
