<script lang="ts">
  import type { ChatSuggestionItem } from "$lib/types";

  let {
    trigger = null,
    suggestionItems = [],
    suggestionIndex = 0,
    suggestionsLoading = false,
    suggestionError = null,
    suggestionListScrollEl = $bindable(null),
    onAcceptSuggestion = () => {},
  }: {
    trigger?: "@" | "+" | null;
    suggestionItems?: ChatSuggestionItem[];
    suggestionIndex?: number;
    suggestionsLoading?: boolean;
    suggestionError?: string | null;
    suggestionListScrollEl?: HTMLDivElement | null;
    onAcceptSuggestion?: (item: ChatSuggestionItem) => void | Promise<void>;
  } = $props();
</script>

{#if trigger}
  <div
    class="mb-3 overflow-hidden rounded-[var(--radius-md)] bg-[var(--surface)] shadow-sm"
  >
    {#if suggestionItems.length > 0}
      <div
        class="border-b border-[var(--accent-border-soft)] px-3 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)]"
      >
        {trigger === "@" ? "Channel suggestions" : "Video suggestions"}
      </div>
      <div
        bind:this={suggestionListScrollEl}
        class="max-h-56 overflow-y-auto py-1"
      >
        {#each suggestionItems as item, index (item.kind + ":" + item.id)}
          <button
            type="button"
            data-suggestion-index={index}
            class={`flex w-full items-start justify-between gap-3 px-3 py-2 text-left transition-colors ${
              index === suggestionIndex
                ? "bg-[var(--accent-wash)] text-[var(--foreground)]"
                : "text-[var(--foreground)] hover:bg-[var(--accent-wash)]"
            }`}
            onmousedown={(event) => {
              event.preventDefault();
              void onAcceptSuggestion(item);
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
