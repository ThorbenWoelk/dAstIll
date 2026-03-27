<script lang="ts">
  import { tick } from "svelte";

  import { getChannelSuggestions, getVideoSuggestions } from "$lib/chat-api";
  import {
    extractChatMentions,
    parseChatMentionSegments,
    resolveChatMention,
    type ChatMentionSegment,
    type ResolvedChatMention,
  } from "$lib/chat-mentions";
  import ChatMentionTag from "$lib/components/chat/ChatMentionTag.svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import type { ChatSuggestionItem } from "$lib/types";

  const MAX_TEXTAREA_HEIGHT = 160;
  const SUGGESTION_LIMIT = 8;
  const SUGGESTION_DEBOUNCE_MS = 120;
  const COMPOSER_PAD_CHAR = "\u2007";
  const COMPOSER_TAG_HORIZONTAL_PADDING_PX = 12;
  const COMPOSER_TAG_WIDTH_FUDGE_PX = 4;

  type ActiveTrigger = {
    trigger: "@" | "+";
    query: string;
    start: number;
    end: number;
  };

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

  let composerValue = $state("");
  let history: string[] = [];
  let historyIndex = -1;
  let savedDraft = "";
  let activeTrigger = $state<ActiveTrigger | null>(null);
  let suggestionItems = $state<ChatSuggestionItem[]>([]);
  let suggestionIndex = $state(0);
  let suggestionsLoading = $state(false);
  let suggestionError = $state<string | null>(null);
  let suggestionController: AbortController | null = null;
  let suggestionRequestId = 0;
  let suggestionDebounce: ReturnType<typeof setTimeout> | null = null;
  let resolvedDraftMentions = $state<Record<string, ResolvedChatMention>>({});
  let composerScrollTop = $state(0);
  let composerPadCounts = $state<Record<string, number>>({});

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
    composerValue;
    syncTextareaHeight();
  });

  function stripComposerPadding(text: string): string {
    return text.split(COMPOSER_PAD_CHAR).join("");
  }

  function countChars(text: string): number {
    return Array.from(text).length;
  }

  function measureTextWidthPx(text: string, font: string): number {
    if (typeof document === "undefined") {
      return text.length * 8;
    }
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) {
      return text.length * 8;
    }
    context.font = font;
    return context.measureText(text).width;
  }

  function composerMeasurementFont(): string | null {
    if (!textareaElement || typeof window === "undefined") {
      return null;
    }
    const styles = window.getComputedStyle(textareaElement);
    return `${styles.fontStyle} ${styles.fontVariant} ${styles.fontWeight} ${styles.fontSize} ${styles.fontFamily}`;
  }

  function composerTagLabelFont(): string | null {
    const measurementFont = composerMeasurementFont();
    if (!measurementFont || typeof window === "undefined") {
      return null;
    }
    const family = window.getComputedStyle(textareaElement!).fontFamily;
    return `400 12px ${family}`;
  }

  function composerPadCountForLabel(rawToken: string, label: string): number {
    const rawFont = composerMeasurementFont();
    const labelFont = composerTagLabelFont();
    if (!rawFont || !labelFont) {
      return Math.max(0, countChars(label) + 4 - countChars(rawToken));
    }

    const rawWidth = measureTextWidthPx(rawToken, rawFont);
    const labelWidth =
      measureTextWidthPx(label, labelFont) +
      COMPOSER_TAG_HORIZONTAL_PADDING_PX +
      COMPOSER_TAG_WIDTH_FUDGE_PX;
    const padWidth = measureTextWidthPx(COMPOSER_PAD_CHAR, rawFont);
    if (padWidth <= 0) {
      return Math.max(0, countChars(label) + 4 - countChars(rawToken));
    }
    return Math.max(0, Math.ceil((labelWidth - rawWidth) / padWidth));
  }

  function buildComposerDisplayValue(
    rawValue: string,
    mentions: Record<string, ResolvedChatMention>,
  ): string {
    return parseChatMentionSegments(rawValue)
      .map((segment) => {
        if (segment.type === "text") {
          return segment.value;
        }
        const label =
          mentions[segment.mention.raw]?.label ?? segment.mention.query;
        const padCount =
          composerPadCounts[segment.mention.raw] ??
          composerPadCountForLabel(segment.mention.raw, label);
        const pad = COMPOSER_PAD_CHAR.repeat(padCount);
        return `${segment.mention.raw}${pad}`;
      })
      .join("");
  }

  function composerDisplayIndexToRawIndex(
    displayValue: string,
    displayIndex: number,
  ): number {
    let rawIndex = 0;
    for (const char of Array.from(displayValue.slice(0, displayIndex))) {
      if (char !== COMPOSER_PAD_CHAR) {
        rawIndex += 1;
      }
    }
    return rawIndex;
  }

  function rawIndexToComposerDisplayIndex(
    displayValue: string,
    rawIndex: number,
  ): number {
    if (rawIndex <= 0) {
      return 0;
    }
    let visibleCount = 0;
    let displayCount = 0;
    for (const char of Array.from(displayValue)) {
      displayCount += 1;
      if (char !== COMPOSER_PAD_CHAR) {
        visibleCount += 1;
        if (visibleCount >= rawIndex) {
          return displayCount;
        }
      }
    }
    return displayCount;
  }

  function syncComposerValueFromRaw() {
    const nextComposerValue = buildComposerDisplayValue(
      value,
      resolvedDraftMentions,
    );
    if (nextComposerValue === composerValue) {
      return;
    }

    const currentDisplaySelection = textareaElement?.selectionStart ?? 0;
    const rawCaret = composerDisplayIndexToRawIndex(
      composerValue,
      currentDisplaySelection,
    );

    composerValue = nextComposerValue;

    void tick().then(() => {
      if (!textareaElement) {
        return;
      }
      const nextDisplayCaret = rawIndexToComposerDisplayIndex(
        composerValue,
        rawCaret,
      );
      textareaElement.setSelectionRange(nextDisplayCaret, nextDisplayCaret);
    });
  }

  $effect(() => {
    value;
    resolvedDraftMentions;
    syncComposerValueFromRaw();
  });

  $effect(() => {
    textareaElement;
    value;
    resolvedDraftMentions;
    const nextCounts = Object.fromEntries(
      extractChatMentions(value).map((mention) => {
        const label =
          resolvedDraftMentions[mention.raw]?.label ?? mention.query;
        return [mention.raw, composerPadCountForLabel(mention.raw, label)];
      }),
    );
    composerPadCounts = nextCounts;
  });

  $effect(() => {
    const mentions = extractChatMentions(value);
    if (mentions.length === 0) {
      resolvedDraftMentions = {};
      return;
    }

    let cancelled = false;
    void Promise.all(
      mentions.map((mention) =>
        resolveChatMention(mention)
          .then((resolved) => [mention.raw, resolved] as const)
          .catch(
            () =>
              [
                mention.raw,
                {
                  kind: mention.kind,
                  raw: mention.raw,
                  query: mention.query,
                  label: mention.query,
                  resolved: false,
                },
              ] as const,
          ),
      ),
    ).then((nextMentions) => {
      if (!cancelled) {
        resolvedDraftMentions = Object.fromEntries(nextMentions);
      }
    });

    return () => {
      cancelled = true;
    };
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
    closeSuggestions();
    if (history[0] !== trimmed) {
      history = [trimmed, ...history];
    }
    historyIndex = -1;
    savedDraft = "";
    onSubmit(trimmed);
  }

  async function setCursorAfterValueChange(pos: number) {
    await tick();
    const displayPos = rawIndexToComposerDisplayIndex(composerValue, pos);
    textareaElement?.setSelectionRange(displayPos, displayPos);
  }

  function isOnFirstLine(): boolean {
    if (!textareaElement) return true;
    const rawSelectionStart = composerDisplayIndexToRawIndex(
      composerValue,
      textareaElement.selectionStart,
    );
    const before = value.substring(0, rawSelectionStart);
    return !before.includes("\n");
  }

  function isOnLastLine(): boolean {
    if (!textareaElement) return true;
    const rawSelectionStart = composerDisplayIndexToRawIndex(
      composerValue,
      textareaElement.selectionStart,
    );
    const after = value.substring(rawSelectionStart);
    return !after.includes("\n");
  }

  function closeSuggestions() {
    activeTrigger = null;
    suggestionItems = [];
    suggestionIndex = 0;
    suggestionError = null;
    suggestionsLoading = false;
    suggestionController?.abort();
    suggestionController = null;
    if (suggestionDebounce) {
      clearTimeout(suggestionDebounce);
      suggestionDebounce = null;
    }
  }

  function detectActiveTrigger(): ActiveTrigger | null {
    const ta = textareaElement;
    if (!ta || ta.selectionStart !== ta.selectionEnd) {
      return null;
    }

    const caret = composerDisplayIndexToRawIndex(
      composerValue,
      ta.selectionStart,
    );
    const beforeCaret = value.slice(0, caret);
    const match = /(?:^|\s)([@+])([^\s@+{}"]*)$/.exec(beforeCaret);
    if (!match || match.index == null) {
      return null;
    }

    const token = `${match[1]}${match[2] ?? ""}`;
    const start = beforeCaret.lastIndexOf(token);
    if (start < 0) {
      return null;
    }

    return {
      trigger: match[1] as "@" | "+",
      query: match[2] ?? "",
      start,
      end: caret,
    };
  }

  function scheduleSuggestions() {
    const nextTrigger = detectActiveTrigger();
    if (!nextTrigger || disabled) {
      closeSuggestions();
      return;
    }

    activeTrigger = nextTrigger;
    suggestionError = null;
    if (suggestionDebounce) {
      clearTimeout(suggestionDebounce);
    }
    suggestionDebounce = setTimeout(() => {
      void loadSuggestions(nextTrigger);
    }, SUGGESTION_DEBOUNCE_MS);
  }

  async function loadSuggestions(trigger: ActiveTrigger) {
    suggestionDebounce = null;
    suggestionController?.abort();
    const controller = new AbortController();
    suggestionController = controller;
    const requestId = ++suggestionRequestId;
    suggestionsLoading = true;

    try {
      const items =
        trigger.trigger === "@"
          ? await getChannelSuggestions(trigger.query, {
              limit: SUGGESTION_LIMIT,
              signal: controller.signal,
            })
          : await getVideoSuggestions(trigger.query, {
              limit: SUGGESTION_LIMIT,
              signal: controller.signal,
            });
      if (requestId !== suggestionRequestId) {
        return;
      }
      suggestionItems = items;
      suggestionIndex =
        items.length === 0 ? 0 : Math.min(suggestionIndex, items.length - 1);
      suggestionError = null;
    } catch (error) {
      if ((error as Error).name === "AbortError") {
        return;
      }
      if (requestId !== suggestionRequestId) {
        return;
      }
      suggestionItems = [];
      suggestionError = (error as Error).message;
    } finally {
      if (requestId === suggestionRequestId) {
        suggestionsLoading = false;
      }
    }
  }

  function suggestionToken(item: ChatSuggestionItem): string {
    return item.kind === "channel" ? `@{${item.label}}` : `+{${item.label}}`;
  }

  async function acceptSuggestion(item: ChatSuggestionItem) {
    const trigger = activeTrigger;
    if (!trigger) {
      return;
    }

    const replacement = `${suggestionToken(item)} `;
    value =
      value.slice(0, trigger.start) + replacement + value.slice(trigger.end);
    closeSuggestions();
    const nextCaret = trigger.start + replacement.length;
    await setCursorAfterValueChange(nextCaret);
    textareaElement?.focus();
  }

  function handleKeydown(event: KeyboardEvent) {
    const ta = textareaElement;
    if (!ta) return;

    if (activeTrigger) {
      if (event.key === "ArrowDown") {
        event.preventDefault();
        if (suggestionItems.length > 0) {
          suggestionIndex = (suggestionIndex + 1) % suggestionItems.length;
        }
        return;
      }

      if (event.key === "ArrowUp") {
        event.preventDefault();
        if (suggestionItems.length > 0) {
          suggestionIndex =
            (suggestionIndex - 1 + suggestionItems.length) %
            suggestionItems.length;
        }
        return;
      }

      if (
        (event.key === "Enter" || event.key === "Tab") &&
        suggestionItems.length > 0
      ) {
        event.preventDefault();
        void acceptSuggestion(suggestionItems[suggestionIndex]);
        return;
      }

      if (event.key === "Escape") {
        event.preventDefault();
        closeSuggestions();
        return;
      }
    }

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
      const start = composerDisplayIndexToRawIndex(
        composerValue,
        ta.selectionStart,
      );
      const lineStart = value.lastIndexOf("\n", start - 1) + 1;
      value = value.substring(0, lineStart) + value.substring(start);
      void setCursorAfterValueChange(lineStart);
      return;
    }

    if (event.ctrlKey && event.key === "k") {
      event.preventDefault();
      const start = composerDisplayIndexToRawIndex(
        composerValue,
        ta.selectionStart,
      );
      const lineEnd = value.indexOf("\n", start);
      const end = lineEnd === -1 ? value.length : lineEnd;
      value = value.substring(0, start) + value.substring(end);
      void setCursorAfterValueChange(start);
      return;
    }

    if (event.ctrlKey && event.key === "w") {
      event.preventDefault();
      const start = composerDisplayIndexToRawIndex(
        composerValue,
        ta.selectionStart,
      );
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
  let composerSegments = $derived(parseChatMentionSegments(value));
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

  $effect(() => {
    return () => {
      closeSuggestions();
    };
  });

  function composerMentionLabel(
    segment: Extract<ChatMentionSegment, { type: "mention" }>,
  ) {
    return (
      resolvedDraftMentions[segment.mention.raw]?.label ?? segment.mention.query
    );
  }

  function handleComposerScroll(event: Event) {
    composerScrollTop = (event.currentTarget as HTMLTextAreaElement).scrollTop;
  }
</script>

<form
  class="rounded-[var(--radius-lg)] bg-[var(--panel-surface)] px-3 py-3 shadow-sm"
  aria-busy={busy}
  onsubmit={(event) => {
    event.preventDefault();
    submit();
  }}
>
  <div class="relative mb-3">
    <div
      aria-hidden="true"
      class="pointer-events-none absolute inset-0 overflow-hidden px-1 py-2"
    >
      <div
        class="min-h-10 break-words text-[14px] leading-5 text-[var(--foreground)]"
        style={`transform: translateY(-${composerScrollTop}px);`}
      >
        {#if value}
          <div class="whitespace-pre-wrap">
            {#each composerSegments as segment, index (`${segment.type}:${index}`)}
              {#if segment.type === "text"}
                <span>{segment.value}</span>
              {:else}
                <ChatMentionTag
                  label={composerMentionLabel(segment)}
                  compact={true}
                />
              {/if}
            {/each}
          </div>
        {:else}
          <span class="text-[var(--soft-foreground)]">
            Ask something about your content… Use @channel or +video to scope.
          </span>
        {/if}
      </div>
    </div>

    <textarea
      value={composerValue}
      bind:this={textareaElement}
      rows="1"
      class="min-h-10 max-h-40 w-full resize-none overflow-y-hidden break-words bg-transparent px-1 py-2 text-[14px] leading-5 text-transparent caret-[var(--foreground)] focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-60"
      style="-webkit-text-fill-color: transparent;"
      placeholder=""
      wrap="soft"
      {disabled}
      oninput={(event) => {
        if (!(event.currentTarget instanceof HTMLTextAreaElement)) return;
        const nextComposerValue = event.currentTarget.value;

        const currentCaret =
          event.currentTarget.selectionStart ?? nextComposerValue.length;
        const rawCaret = composerDisplayIndexToRawIndex(
          nextComposerValue,
          currentCaret,
        );

        // Update value (raw text). Svelte 5 will sync this to the parent.
        const nextValue = stripComposerPadding(nextComposerValue);
        if (value !== nextValue) {
          value = nextValue;
        }

        // Re-calculate composerValue (with padding) immediately to keep the UI in sync.
        const syncComposerValue = buildComposerDisplayValue(
          value,
          resolvedDraftMentions,
        );

        // Even if the value didn't change (e.g. padding only), we MUST update the textarea value
        // to match what the user typed so the browser doesn't fight us.
        // But since we use one-way value={composerValue}, we must ensure composerValue is correct.
        composerValue = syncComposerValue;

        syncTextareaHeight();
        scheduleSuggestions();

        void tick().then(() => {
          if (!textareaElement) return;
          const displayCaret = rawIndexToComposerDisplayIndex(
            composerValue,
            rawCaret,
          );
          textareaElement.setSelectionRange(displayCaret, displayCaret);
        });
      }}
      onclick={scheduleSuggestions}
      onscroll={handleComposerScroll}
      onkeyup={(event) => {
        if (
          [
            "ArrowLeft",
            "ArrowRight",
            "ArrowUp",
            "ArrowDown",
            "Home",
            "End",
            "Backspace",
            "Delete",
          ].includes(event.key)
        ) {
          scheduleSuggestions();
        }
      }}
      onkeydown={handleKeydown}
    ></textarea>
  </div>

  {#if activeTrigger}
    <div
      class="mb-3 overflow-hidden rounded-[var(--radius-md)] bg-[var(--surface)] shadow-sm"
    >
      {#if suggestionItems.length > 0}
        <div
          class="border-b border-[var(--accent-border-soft)] px-3 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
        >
          {activeTrigger.trigger === "@"
            ? "Channel suggestions"
            : "Video suggestions"}
        </div>
        <div class="max-h-56 overflow-y-auto py-1">
          {#each suggestionItems as item, index (item.kind + ":" + item.id)}
            <button
              type="button"
              class={`flex w-full items-start justify-between gap-3 px-3 py-2 text-left transition-colors ${
                index === suggestionIndex
                  ? "bg-[var(--accent-wash)] text-[var(--foreground)]"
                  : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"
              }`}
              onmousedown={(event) => {
                event.preventDefault();
                void acceptSuggestion(item);
              }}
            >
              <span class="min-w-0">
                <span class="block truncate text-[13px] leading-5">
                  {item.label}
                </span>
                {#if item.subtitle}
                  <span
                    class="block truncate text-[11px] uppercase tracking-[0.05em] text-[var(--soft-foreground)]"
                  >
                    {item.subtitle}
                  </span>
                {/if}
              </span>
              <span
                class="shrink-0 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
              >
                {item.kind}
              </span>
            </button>
          {/each}
        </div>
      {:else if suggestionsLoading}
        <div class="px-3 py-3 text-[12px] text-[var(--soft-foreground)]">
          Loading suggestions…
        </div>
      {:else}
        <div class="px-3 py-3 text-[12px] text-[var(--soft-foreground)]">
          {suggestionError ?? "No suggestions found."}
        </div>
      {/if}
    </div>
  {/if}

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
