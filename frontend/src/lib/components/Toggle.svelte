<script lang="ts">
  import HighlighterIcon from "$lib/components/icons/HighlighterIcon.svelte";

  let {
    options = [],
    value = "",
    onChange = () => {},
    labels = {},
    ariaLabel = "Options",
    showIcons = true,
  }: {
    options?: string[];
    value?: string;
    onChange?: (next: string) => void;
    labels?: Record<string, string>;
    ariaLabel?: string;
    showIcons?: boolean;
  } = $props();

  const resolveLabel = (option: string) => labels[option] ?? option;
</script>

<div
  class="inline-flex items-center gap-1"
  role="tablist"
  aria-label={ariaLabel}
>
  {#each options as option}
    <button
      type="button"
      role="tab"
      aria-selected={value === option}
      class={`flex h-7 flex-none items-center gap-1 rounded-full border px-3 text-[11px] font-bold uppercase tracking-[0.1em] transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/30 ${
        value === option
          ? "cursor-default border-transparent bg-[var(--accent-wash-strong)] text-[var(--accent-strong)] shadow-sm"
          : "border-transparent text-[var(--soft-foreground)] opacity-90 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] hover:opacity-100"
      }`}
      onclick={() => onChange(option)}
    >
      {#if showIcons}
        {#if option === "transcript" || option === "transcripts"}
          <svg
            class="size-3 shrink-0"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path
              d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"
            />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="8" y2="13" />
            <line x1="16" y1="17" x2="8" y2="17" />
          </svg>
        {:else if option === "summary" || option === "summaries"}
          <svg
            class="size-3 shrink-0"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <line x1="21" y1="4" x2="3" y2="4" />
            <line x1="14" y1="8" x2="3" y2="8" />
            <line x1="21" y1="12" x2="3" y2="12" />
            <line x1="14" y1="16" x2="3" y2="16" />
          </svg>
        {:else if option === "highlights"}
          <HighlighterIcon size={12} strokeWidth={2.5} class="shrink-0" />
        {:else if option === "info" || option === "evaluations"}
          <svg
            class="size-3 shrink-0"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="12" cy="12" r="9" />
            <path d="M12 17v-5" />
            <path d="M12 8h.01" />
          </svg>
        {:else if option === "channels"}
          <svg
            class="size-3 shrink-0"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <rect x="3" y="4" width="6" height="16" rx="1.5" />
            <rect x="10" y="4" width="5" height="16" rx="1.5" />
            <rect x="16" y="4" width="5" height="16" rx="1.5" />
          </svg>
        {:else if option === "videos"}
          <svg
            class="size-3 shrink-0"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polygon points="6,3 20,12 6,21" />
          </svg>
        {:else if option === "content" || option === "details"}
          <svg
            class="size-3 shrink-0"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
            <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
          </svg>
        {/if}
      {/if}
      <span class="leading-none">{resolveLabel(option)}</span>
    </button>
  {/each}
</div>
