<script lang="ts">
  import { tick } from "svelte";

  const MAX_TEXTAREA_HEIGHT = 160;

  let {
    value = $bindable(""),
    disabled = false,
    busy = false,
    canCancel = false,
    onSubmit = (_value: string) => {},
    onCancel = () => {},
  }: {
    value?: string;
    disabled?: boolean;
    busy?: boolean;
    canCancel?: boolean;
    onSubmit?: (value: string) => void;
    onCancel?: () => void;
  } = $props();

  let textareaElement: HTMLTextAreaElement | null = null;

  // Input history for terminal-style Up/Down navigation.
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

    // History: previous (Up / Ctrl+P) — guard against Cmd+Up (jump to doc start)
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

    // History: next (Down / Ctrl+N) — guard against Cmd+Down (jump to doc end)
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

    // Ctrl+U: delete from cursor to start of current line
    if (event.ctrlKey && event.key === "u") {
      event.preventDefault();
      const start = ta.selectionStart;
      const lineStart = value.lastIndexOf("\n", start - 1) + 1;
      value = value.substring(0, lineStart) + value.substring(start);
      void setCursorAfterValueChange(lineStart);
      return;
    }

    // Ctrl+K: delete from cursor to end of current line
    if (event.ctrlKey && event.key === "k") {
      event.preventDefault();
      const start = ta.selectionStart;
      const lineEnd = value.indexOf("\n", start);
      const end = lineEnd === -1 ? value.length : lineEnd;
      value = value.substring(0, start) + value.substring(end);
      void setCursorAfterValueChange(start);
      return;
    }

    // Ctrl+W: delete word before cursor
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
</script>

<form
  class="rounded-[var(--radius-lg)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-3 py-2.5 shadow-sm"
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
      class="min-h-10 max-h-40 flex-1 resize-none overflow-y-hidden break-words bg-transparent px-1 py-2 text-[13px] leading-5 text-[var(--foreground)] placeholder:text-[var(--soft-foreground)] focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-60"
      placeholder="Ask about your indexed transcripts and summaries…"
      wrap="soft"
      {disabled}
      oninput={syncTextareaHeight}
      onkeydown={handleKeydown}
    ></textarea>

    {#if canCancel}
      <button
        type="button"
        class="inline-flex h-9 shrink-0 items-center justify-center rounded-full border border-[var(--accent)]/20 bg-[var(--accent)] px-4 text-[12px] font-bold uppercase tracking-[0.1em] text-white transition-colors hover:bg-[var(--accent-strong)]"
        onclick={onCancel}
      >
        Cancel
      </button>
    {:else}
      <button
        type="submit"
        class="inline-flex h-9 shrink-0 items-center justify-center rounded-full border border-[var(--accent)]/15 bg-[var(--accent-wash-strong)] px-4 text-[12px] font-bold uppercase tracking-[0.1em] text-[var(--accent-strong)] transition-colors hover:bg-[var(--accent)]/15 disabled:cursor-not-allowed disabled:opacity-50"
        disabled={disabled || !value.trim()}
      >
        {busy ? "Sending…" : "Send"}
      </button>
    {/if}
  </div>
</form>
