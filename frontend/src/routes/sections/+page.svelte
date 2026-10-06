<script lang="ts">
  import { onMount } from "svelte";
  import Masthead from "$lib/components/Masthead.svelte";
  import {
    listChannels,
    subscribeToChannel,
    unsubscribeFromChannel,
    type Channel,
  } from "$lib/api";

  let channels = $state<Channel[]>([]);
  let loading = $state(true);
  let loadError = $state<string | null>(null);

  let input = $state("");
  let adding = $state(false);
  let addMessage = $state<{ text: string; failed: boolean } | null>(null);

  let confirmingId = $state<string | null>(null);
  let removingId = $state<string | null>(null);
  let removeError = $state<string | null>(null);

  const sortedChannels = $derived(
    [...channels].sort((a, b) => a.name.localeCompare(b.name)),
  );

  function messageOf(cause: unknown): string {
    return cause instanceof Error ? cause.message : String(cause);
  }

  async function loadChannels() {
    loading = true;
    loadError = null;
    try {
      channels = await listChannels();
    } catch (cause) {
      loadError = messageOf(cause);
    } finally {
      loading = false;
    }
  }

  async function addChannel(event: SubmitEvent) {
    event.preventDefault();
    const value = input.trim();
    if (!value || adding) return;
    adding = true;
    addMessage = null;
    try {
      const channel = await subscribeToChannel(value);
      channels = [...channels.filter((c) => c.id !== channel.id), channel];
      input = "";
      addMessage = {
        text: `Added ${channel.name}. Its stories appear once they are summarized.`,
        failed: false,
      };
    } catch (cause) {
      addMessage = { text: messageOf(cause), failed: true };
    } finally {
      adding = false;
    }
  }

  async function removeChannel(channel: Channel) {
    removingId = channel.id;
    removeError = null;
    try {
      await unsubscribeFromChannel(channel.id);
      channels = channels.filter((c) => c.id !== channel.id);
      confirmingId = null;
    } catch (cause) {
      removeError = `Could not remove ${channel.name}: ${messageOf(cause)}`;
    } finally {
      removingId = null;
    }
  }

  onMount(() => {
    void loadChannels();
  });
</script>

<svelte:head>
  <title>Sections · dAstIll</title>
</svelte:head>

<Masthead>
  {#snippet actions()}
    <a class="text-button" href="/">Back to the paper</a>
  {/snippet}
</Masthead>

<section class="sections-page" aria-labelledby="sections-heading">
  <h1 id="sections-heading">Sections</h1>
  <p class="intro">Every channel you follow is a section of your paper.</p>

  <form class="add" onsubmit={addChannel}>
    <label class="label" for="channel-input">Add a channel</label>
    <div class="add-row">
      <input
        id="channel-input"
        type="text"
        bind:value={input}
        placeholder="@handle, channel link, or podcast feed"
        autocomplete="off"
        spellcheck="false"
      />
      <button type="submit" class="press" disabled={adding || !input.trim()}>
        {adding ? "Adding…" : "Add"}
      </button>
    </div>
    {#if addMessage}
      <p
        class="message"
        class:failed={addMessage.failed}
        role={addMessage.failed ? "alert" : "status"}
      >
        {addMessage.text}
      </p>
    {/if}
  </form>

  <h2 class="label list-heading">Following</h2>
  {#if loading}
    <p class="quiet">Loading sections…</p>
  {:else if loadError}
    <p class="message failed" role="alert">{loadError}</p>
    <button type="button" class="text-button" onclick={loadChannels}>
      Try again
    </button>
  {:else if sortedChannels.length === 0}
    <p class="quiet">You do not follow any channels yet.</p>
  {:else}
    {#if removeError}
      <p class="message failed" role="alert">{removeError}</p>
    {/if}
    <ul class="channel-list">
      {#each sortedChannels as channel (channel.id)}
        <li>
          <div class="channel">
            <span class="name">{channel.name}</span>
            {#if channel.handle}
              <span class="handle">{channel.handle}</span>
            {/if}
          </div>
          {#if confirmingId === channel.id}
            <div class="confirm">
              <span>Remove this section?</span>
              <button
                type="button"
                class="text-button danger"
                disabled={removingId === channel.id}
                onclick={() => removeChannel(channel)}
              >
                {removingId === channel.id ? "Removing…" : "Remove"}
              </button>
              <button
                type="button"
                class="text-button"
                onclick={() => (confirmingId = null)}
              >
                Keep
              </button>
            </div>
          {:else}
            <button
              type="button"
              class="text-button"
              aria-label={`Remove ${channel.name}`}
              onclick={() => (confirmingId = channel.id)}
            >
              Remove
            </button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .sections-page {
    max-width: 720px;
    margin: 0 auto;
    padding: var(--space-6) 0 var(--space-8);
  }

  h1 {
    margin: 0;
    font-size: 40px;
    line-height: 1.1;
  }

  .intro {
    margin: var(--space-2) 0 var(--space-6);
    font-style: italic;
    color: var(--ink-soft);
  }

  .add {
    padding: var(--space-4) 0;
    border-top: 2px solid var(--rule);
    border-bottom: 1px solid var(--hairline);
  }

  .add-row {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }

  input {
    flex: 1;
    min-width: 0;
    min-height: 52px;
    padding: 0 var(--space-3);
    border: 1px solid var(--rule);
    border-radius: 0;
    background: var(--paper-raised);
    font-family: var(--sans);
    font-size: 16px;
  }

  input::placeholder {
    color: var(--ink-faint);
  }

  .message {
    margin: var(--space-3) 0 0;
    font-family: var(--sans);
    font-size: 13px;
    color: var(--ink-soft);
  }

  .message.failed {
    color: var(--danger);
  }

  .list-heading {
    margin: var(--space-6) 0 0;
    padding-bottom: var(--space-2);
    border-bottom: 2px solid var(--rule);
  }

  .quiet {
    font-style: italic;
    color: var(--ink-soft);
  }

  .channel-list {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .channel-list li {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 0 var(--space-4);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--hairline);
  }

  .channel {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .name {
    font-size: 19px;
    font-weight: 700;
  }

  .handle {
    font-family: var(--sans);
    font-size: 12px;
    color: var(--ink-faint);
  }

  .confirm {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-family: var(--sans);
    font-size: 12px;
  }

  .danger {
    color: var(--danger);
  }
</style>
