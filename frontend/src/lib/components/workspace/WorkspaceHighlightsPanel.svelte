<script lang="ts">
  import TrashIcon from "$lib/components/icons/TrashIcon.svelte";
  import type { Highlight, Video } from "$lib/types";
  import { formatPublishedAt } from "$lib/workspace/content";

  let {
    selectedVideo = null,
    highlights = [],
    deletingHighlightId = null,
    onDeleteHighlight = undefined,
  }: {
    selectedVideo?: Video | null;
    highlights?: Highlight[];
    deletingHighlightId?: number | null;
    onDeleteHighlight?:
      | ((highlightId: number) => Promise<void> | void)
      | undefined;
  } = $props();
</script>

<div class="space-y-5 pb-20">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <div>
      <p
        class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50"
      >
        Saved highlights
      </p>
      <h3
        class="mt-1 text-[20px] font-bold font-serif leading-tight text-[var(--foreground)]"
      >
        {selectedVideo?.title || "Highlights"}
      </h3>
    </div>
    <p
      class="text-[12px] font-semibold text-[var(--soft-foreground)] opacity-60"
    >
      {highlights.length} saved
    </p>
  </div>

  {#if highlights.length === 0}
    <div
      class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--muted)]/20 px-4 py-5 text-[14px] text-[var(--soft-foreground)] opacity-70"
    >
      Select text in the transcript or summary to save your first highlight for
      this video.
    </div>
  {:else}
    <div class="space-y-3">
      {#each highlights as highlight (highlight.id)}
        <article
          class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-frost-strong)] px-4 py-4 shadow-sm"
        >
          <div class="flex flex-wrap items-center justify-between gap-2">
            <span
              class="inline-flex rounded-full bg-[var(--accent-soft)]/60 px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--accent-strong)]"
            >
              {highlight.source}
            </span>
            <div class="flex items-center gap-3">
              <span
                class="text-[11px] text-[var(--soft-foreground)] opacity-50"
              >
                {formatPublishedAt(highlight.created_at)}
              </span>
              {#if onDeleteHighlight}
                <button
                  type="button"
                  class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--danger)] disabled:cursor-not-allowed disabled:opacity-50"
                  onclick={() => void onDeleteHighlight(highlight.id)}
                  disabled={deletingHighlightId === highlight.id}
                  aria-label="Delete highlight"
                >
                  <TrashIcon
                    size={14}
                    strokeWidth={2.2}
                    class={deletingHighlightId === highlight.id
                      ? "animate-pulse"
                      : ""}
                  />
                </button>
              {/if}
            </div>
          </div>
          <p
            class="mt-3 whitespace-pre-wrap text-[15px] leading-relaxed text-[var(--foreground)]"
          >
            {highlight.text}
          </p>
        </article>
      {/each}
    </div>
  {/if}
</div>
