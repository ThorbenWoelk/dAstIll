<script lang="ts">
  import { formatShortDate } from "$lib/utils/date";
  import type { Video } from "$lib/types";

  let {
    videos,
    readOnly = false,
    retryingTranscriptVideoId = null as string | null,
    onRetryTranscript = undefined as
      | ((videoId: string) => void | Promise<void>)
      | undefined,
  }: {
    videos: Video[];
    readOnly?: boolean;
    retryingTranscriptVideoId?: string | null;
    onRetryTranscript?: (videoId: string) => void | Promise<void>;
  } = $props();
</script>

{#if videos.length > 0}
  <div class="border-t border-[var(--border-soft)] pt-6">
    <div class="flex items-center justify-between gap-3">
      <p
        class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
      >
        Failed downloads
      </p>
      <span
        class="rounded-full bg-[var(--danger-soft)] px-2 py-1 text-[11px] font-semibold text-[var(--danger-foreground)]"
      >
        {videos.length} failed
      </span>
    </div>
    <p class="mt-2 text-[14px] font-semibold text-[var(--foreground)]">
      Retry transcript extraction
    </p>

    <div class="mt-4 space-y-0">
      {#each videos as video (video.id)}
        <div
          class="flex flex-col gap-3 border-t border-[var(--border-soft)] py-4 first:border-t-0 sm:flex-row sm:items-center sm:justify-between"
        >
          <div class="min-w-0">
            <p
              class="line-clamp-2 text-[14px] font-semibold text-[var(--foreground)]"
            >
              {video.title}
            </p>
            <p class="mt-1 text-[12px] text-[var(--soft-foreground)]">
              Published {formatShortDate(video.published_at)}
            </p>
          </div>

          {#if !readOnly}
            <button
              type="button"
              class="inline-flex shrink-0 items-center justify-center rounded-full bg-[var(--foreground)] px-4 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--background)] transition-all hover:bg-[var(--accent-strong)] disabled:opacity-40"
              onclick={() => void onRetryTranscript?.(video.id)}
              disabled={retryingTranscriptVideoId === video.id}
            >
              {retryingTranscriptVideoId === video.id ? "Retrying" : "Retry"}
            </button>
          {/if}
        </div>
      {/each}
    </div>
  </div>
{/if}
