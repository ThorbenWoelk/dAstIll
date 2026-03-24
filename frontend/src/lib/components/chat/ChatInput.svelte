<script lang="ts">
  import { tick } from "svelte";

  const MAX_TEXTAREA_HEIGHT = 160;

  let {
    value = $bindable(""),
    disabled = false,
    busy = false,
    canCancel = false,
    /** Increment (e.g. after "New conversation") to move focus into this input. */
    focusSignal = 0,
    onSubmit = (_value: string) => {},
    onCancel = () => {},
  }: {
    value?: string;
    disabled?: boolean;
    busy?: boolean;
    canCancel?: boolean;
    focusSignal?: number;
    onSubmit?: (value: string) => void;
    onCancel?: () => void;
  } = $props();

  let textareaElement: HTMLTextAreaElement | null = null;

  let history: string[] = [];
  let historyIndex = -1;
  let savedDraft = "";

  function syncTextareaHeight() {
    if (!textareaElement) {
      return;
    }

    textareaElement.style.height = "0px";
    const nextHeight = Math.min(
      textareaElement.scrollHeight,
      MAX_TEXTAREA_HEIGHT,
    );
    textareaElement.style.height = `${nextHeight}px`;
    textareaElement.style.overflowY =
      textareaElement.scrollHeight > MAX_TEXTAREA_HEIGHT ? "auto" : "hidden";
  }

  $effect(() => {
    value;
    syncTextareaHeight();
  });

  $effect(() => {
    if (focusSignal <= 0) {
      return;
    }
    void tick().then(() => {
      textareaElement?.focus();
    });
  });

  function submit() {
    const trimmed = value.trim();
    if (!trimmed || disabled) {
      return;
    }
    if (history[0] !== trimmed) {
      history = [trimmed, ...history];
    }
    historyIndex = -1;
    savedDraft = "";
    onSubmit(trimmed);
  }

  async function setCursorAfterValueChange(pos: number) {
    await tick();
    textareaElement?.setSelectionRange(pos, pos);
  }

  function isOnFirstLine(): boolean {
    if (!textareaElement) return true;
    const before = value.substring(0, textareaElement.selectionStart);
    return !before.includes("\n");
  }

  function isOnLastLine(): boolean {
    if (!textareaElement) return true;
    const after = value.substring(textareaElement.selectionStart);
    return !after.includes("\n");
  }

  function handleKeydown(event: KeyboardEvent) {
    const ta = textareaElement;
    if (!ta) return;

    if (
      (event.key === "ArrowUp" || (event.ctrlKey && event.key === "p")) &&
      !event.shiftKey &&
      !event.metaKey &&
      !(event.ctrlKey && event.key === "ArrowUp") &&
      isOnFirstLine()
    ) {
      if (history.length === 0) return;
      event.preventDefault();
      if (historyIndex === -1) savedDraft = value;
      if (historyIndex < history.length - 1) {
        historyIndex++;
        value = history[historyIndex];
      }
      return;
    }

    if (
      (event.key === "ArrowDown" || (event.ctrlKey && event.key === "n")) &&
      !event.shiftKey &&
      !event.metaKey &&
      !(event.ctrlKey && event.key === "ArrowDown") &&
      historyIndex >= 0 &&
      isOnLastLine()
    ) {
      event.preventDefault();
      historyIndex--;
      value = historyIndex === -1 ? savedDraft : history[historyIndex];
      return;
    }

    if (event.ctrlKey && event.key === "u") {
      event.preventDefault();
      const start = ta.selectionStart;
      const lineStart = value.lastIndexOf("\n", start - 1) + 1;
      value = value.substring(0, lineStart) + value.substring(start);
      void setCursorAfterValueChange(lineStart);
      return;
    }

    if (event.ctrlKey && event.key === "k") {
      event.preventDefault();
      const start = ta.selectionStart;
      const lineEnd = value.indexOf("\n", start);
      const end = lineEnd === -1 ? value.length : lineEnd;
      value = value.substring(0, start) + value.substring(end);
      void setCursorAfterValueChange(start);
      return;
    }

    if (event.ctrlKey && event.key === "w") {
      event.preventDefault();
      const start = ta.selectionStart;
      let i = start;
      while (i > 0 && /\s/.test(value[i - 1])) i--;
      while (i > 0 && !/\s/.test(value[i - 1])) i--;
      value = value.substring(0, i) + value.substring(start);
      void setCursorAfterValueChange(i);
      return;
    }

    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      submit();
    }
  }

  let actionDisabled = $derived(
    canCancel ? false : busy || disabled || !value.trim(),
  );
  let ariaLabel = $derived(
    canCancel ? "Cancel generation" : busy ? "Sending" : "Send message",
  );
</script>

<form
  class="rounded-[var(--radius-lg)] bg-[var(--panel-surface)] px-3 py-2 shadow-sm"
  aria-busy={busy}
  onsubmit={(event) => {
    event.preventDefault();
    submit();
  }}
>
  <div class="flex items-end gap-2">
    <textarea
      bind:value
      bind:this={textareaElement}
      rows="1"
      class="min-h-10 max-h-40 flex-1 resize-none overflow-y-hidden break-words bg-transparent px-1 py-2 text-[14px] leading-5 text-[var(--foreground)] placeholder:text-[var(--soft-foreground)] focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-60"
      placeholder="Ask about your indexed transcripts and summaries…"
      wrap="soft"
      {disabled}
      oninput={syncTextareaHeight}
      onkeydown={handleKeydown}
    ></textarea>

    {#if canCancel}
      <button
        type="button"
        class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)]"
        onclick={onCancel}
        aria-label={ariaLabel}
      >
        <svg
          viewBox="0 0 24 24"
          class="h-4 w-4"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          aria-hidden="true"
        >
          <path d="M18 6 6 18M6 6l12 12" />
        </svg>
      </button>
    {:else}
      <button
        type="submit"
        class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] disabled:cursor-not-allowed disabled:opacity-50"
        disabled={actionDisabled}
        aria-label={ariaLabel}
      >
        {#if busy}
          <svg
            viewBox="0 0 24 24"
            class="h-4 w-4 animate-spin"
            aria-hidden="true"
          >
            <circle
              cx="12"
              cy="12"
              r="9"
              fill="none"
              stroke="currentColor"
              stroke-opacity="0.25"
              stroke-width="2"
            />
            <path
              d="M12 3a9 9 0 0 1 9 9"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
            />
          </svg>
        {:else}
          <svg
            viewBox="0 0 24 24"
            class="h-4 w-4"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path d="M22 2 11 13" />
            <path d="M22 2 15 22l-4-9-9-4Z" />
          </svg>
        {/if}
      </button>
    {/if}
  </div>
</form>
