<script lang="ts">
  let {
    passage,
    onSave,
    onCancel,
  }: {
    /** The flagged text, quoted back so the reader knows what they flag. */
    passage: string;
    onSave: (comment: string) => void;
    onCancel: () => void;
  } = $props();

  const QUOTE_CHARS = 120;

  let comment = $state("");
  const quote = $derived(
    passage.length > QUOTE_CHARS
      ? `${passage.slice(0, QUOTE_CHARS).trimEnd()}…`
      : passage,
  );

  function save(event: SubmitEvent) {
    event.preventDefault();
    const note = comment.trim();
    if (note) onSave(note);
  }

  function focusOnOpen(element: HTMLTextAreaElement) {
    element.focus();
  }
</script>

<!-- Asks why a passage reads badly. The summary writer learns from the note. -->
<form class="flag-form" onsubmit={save} data-passage-skip>
  <p class="quote">“{quote}”</p>
  <label class="label" for="writing-flag-comment">What is wrong with it?</label>
  <textarea
    id="writing-flag-comment"
    rows="3"
    maxlength="1000"
    placeholder="For example: invents a ranking the video never makes"
    bind:value={comment}
    {@attach focusOnOpen}
  ></textarea>
  <p class="hint">Future summaries avoid writing like this.</p>
  <div class="actions">
    <button type="submit" class="press" disabled={!comment.trim()}>
      Flag writing
    </button>
    <button type="button" class="text-button" onclick={onCancel}>
      Cancel
    </button>
  </div>
</form>

<style>
  .flag-form {
    display: grid;
    gap: var(--space-2);
    width: min(320px, calc(100vw - 2 * var(--space-4)));
    padding: var(--space-3);
    border: 1px solid var(--rule);
    background: var(--paper-raised);
    font-family: var(--sans);
    font-size: 13px;
  }

  .quote {
    margin: 0;
    font-family: var(--serif);
    font-style: italic;
    color: var(--ink-soft);
  }

  textarea {
    width: 100%;
    padding: var(--space-2);
    border: 1px solid var(--hairline);
    background: var(--paper);
    resize: vertical;
  }

  .hint {
    margin: 0;
    font-size: 12px;
    color: var(--ink-soft);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-4);
  }

  .actions .press {
    min-height: var(--touch);
    padding: 0 var(--space-4);
    font-size: 12px;
  }
</style>
