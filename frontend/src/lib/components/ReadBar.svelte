<script lang="ts">
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";

  let {
    undoTitle = null,
    onRead,
    onUndo,
  }: {
    undoTitle?: string | null;
    onRead: () => void;
    onUndo: () => void;
  } = $props();
</script>

<div class="read-bar">
  {#if undoTitle}
    <button
      type="button"
      class="undo"
      onclick={onUndo}
      title="Undo (U)"
      aria-label={`Undo: mark "${undoTitle}" unread`}
    >
      <span class="undo-verb">Undo</span>
      <span class="undo-title">{undoTitle}</span>
    </button>
  {/if}
  <button type="button" class="press" onclick={onRead} title="Mark as read (R)">
    <CheckIcon />
    Mark as read
  </button>
</div>

<style>
  .read-bar {
    position: sticky;
    bottom: 0;
    z-index: 1;
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-6);
    padding: var(--space-3) 0 max(var(--space-3), env(safe-area-inset-bottom));
    border-top: 1px solid var(--rule);
    background: var(--paper);
  }

  .press {
    flex: 1;
    padding: 0 var(--space-3);
  }

  .undo {
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-width: 0;
    max-width: 40%;
    min-height: 52px;
    padding: 0 var(--space-3);
    border: 1px solid var(--rule);
    background: none;
    font-family: var(--sans);
    text-align: left;
  }

  .undo-verb {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .undo-title {
    overflow: hidden;
    font-size: 11px;
    color: var(--ink-soft);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @media (min-width: 960px) {
    .read-bar {
      position: static;
      flex-direction: row-reverse;
      justify-content: flex-end;
      margin-top: var(--space-6);
      padding: 0;
      border-top: 0;
    }

    .press {
      flex: none;
      min-height: 48px;
      padding: 0 var(--space-5);
    }

    .undo {
      min-height: 48px;
      max-width: 320px;
    }
  }
</style>
