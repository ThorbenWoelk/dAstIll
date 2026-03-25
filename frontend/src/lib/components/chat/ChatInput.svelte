<script lang="ts">
  import { tick } from "svelte";

  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";

  const MAX_TEXTAREA_HEIGHT = 160;

  let {
    value = $bindable(""),
    deepResearch = $bindable(false),
    selectedModelId = $bindable(""),
    modelOptions = [],
    disabled = false,
    busy = false,
    canCancel = false,
    /** Increment (e.g. after "New conversation") to move focus into this input. */
    focusSignal = 0,
    onSubmit = (_value: string) => {},
    onCancel = () => {},
  }: {
    value?: string;
    deepResearch?: boolean;
    selectedModelId?: string;
    modelOptions?: { id: string; label: string }[];
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
  let modelSelectDisabled = $derived(
    disabled || busy || modelOptions.length === 0,
  );

  $effect(() => {
    if (modelOptions.length === 0) {
      return;
    }
    if (
      !selectedModelId ||
      !modelOptions.some((opt) => opt.id === selectedModelId)
    ) {
      selectedModelId = modelOptions[0].id;
    }
  });
</script>

<form
  class="rounded-[var(--radius-lg)] bg-[var(--panel-surface)] px-3 py-3 shadow-sm"
  aria-busy={busy}
  onsubmit={(event) => {
    event.preventDefault();
    submit();
  }}
>
  <textarea
    bind:value
    bind:this={textareaElement}
    rows="1"
    class="mb-3 min-h-10 max-h-40 w-full resize-none overflow-y-hidden break-words bg-transparent px-1 py-2 text-[14px] leading-5 text-[var(--foreground)] placeholder:text-[var(--soft-foreground)] focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-60"
    placeholder="Ask about your indexed transcripts and summaries…"
    wrap="soft"
    {disabled}
    oninput={syncTextareaHeight}
    onkeydown={handleKeydown}
  ></textarea>

  <div
    class="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between sm:gap-2"
  >
    <div class="flex min-w-0 flex-wrap items-center gap-2">
      <div
        class="relative min-w-0 max-w-full sm:max-w-[min(100%,22rem)]"
        title="Ollama cloud model for this message"
      >
        <select
          bind:value={selectedModelId}
          class="w-full min-w-[10rem] cursor-pointer appearance-none rounded-full bg-[var(--accent-wash)]/60 py-1.5 pl-2.5 pr-8 text-[11px] font-bold uppercase tracking-[0.06em] text-[var(--foreground)] transition-colors duration-200 ease-[cubic-bezier(0.16,1,0.3,1)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] disabled:cursor-not-allowed disabled:opacity-50"
          aria-label="Ollama cloud model"
          disabled={modelSelectDisabled}
        >
          {#if modelOptions.length === 0}
            <option value="">Loading…</option>
          {:else}
            {#each modelOptions as option (option.id)}
              <option value={option.id}>{option.label}</option>
            {/each}
          {/if}
        </select>
        <span
          class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-[var(--soft-foreground)]"
          aria-hidden="true"
        >
          <ChevronIcon direction="down" size={12} />
        </span>
      </div>
      <button
        type="button"
        class="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-[11px] font-bold uppercase tracking-[0.06em] transition-colors duration-200 ease-[cubic-bezier(0.16,1,0.3,1)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] disabled:pointer-events-none disabled:opacity-50 {deepResearch
          ? 'border-[var(--accent)]/25 bg-[var(--accent-soft)] text-[var(--accent-strong)] shadow-sm'
          : 'border-transparent bg-transparent text-[var(--soft-foreground)] hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]'}"
        aria-pressed={deepResearch}
        aria-label={deepResearch ? "Deep research on" : "Deep research off"}
        data-tooltip={deepResearch
          ? "Maximum library retrieval for this message"
          : "Search more of your library (slower, richer context)"}
        disabled={disabled || busy}
        onclick={() => {
          deepResearch = !deepResearch;
        }}
      >
        <svg
          viewBox="0 0 24 24"
          class="h-3.5 w-3.5 shrink-0"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M4 19h4" />
          <path d="M6 19v-2" />
          <path d="M8 17h8" />
          <path d="M10 17V9l4-2 2 6-4 2" />
          <path d="m14 7 3-3" />
          <circle cx="17.5" cy="4.5" r="1.5" />
        </svg>
        Deep research
      </button>
    </div>

    <div class="flex items-end justify-end gap-2">
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
  </div>
</form>
