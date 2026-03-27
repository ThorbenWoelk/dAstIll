<script lang="ts">
  import { formatShortDate } from "$lib/utils/date";
  import type { Video } from "$lib/types";
  import {
    queueStateAccentClass,
    queueVideoPipelineSteps,
    queueVideoPrimaryState,
  } from "$lib/queue/presentation";

  let {
    video,
    readOnly = false,
    retryingTranscriptVideoId = null as string | null,
    onRetryTranscript = undefined as
      | ((videoId: string) => void | Promise<void>)
      | undefined,
    onClearSelectedVideo = undefined as
      | (() => void | Promise<void>)
      | undefined,
    onOpenVideoInWorkspace = undefined as
      | ((video: Video) => void | Promise<void>)
      | undefined,
  }: {
    video: Video;
    readOnly?: boolean;
    retryingTranscriptVideoId?: string | null;
    onRetryTranscript?: (videoId: string) => void | Promise<void>;
    onClearSelectedVideo?: () => void | Promise<void>;
    onOpenVideoInWorkspace?: (video: Video) => void | Promise<void>;
  } = $props();

  const pipelineSteps = $derived(queueVideoPipelineSteps(video));
</script>

<article
  class="rounded-[var(--radius-md)] bg-[var(--panel-surface)] px-4 py-5 sm:px-5"
  aria-labelledby="queue-video-title"
>
  <header
    class="flex items-start justify-between gap-3 border-b border-[var(--border-soft)]/40 pb-4"
  >
    <p
      class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-60"
    >
      Selected video
    </p>
    {#if onClearSelectedVideo}
      <button
        type="button"
        class="shrink-0 rounded-full px-2 py-1 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
        onclick={() => void onClearSelectedVideo?.()}
      >
        Clear
      </button>
    {/if}
  </header>

  <h3
    id="queue-video-title"
    class="mt-4 font-serif text-[1.125rem] leading-snug text-[var(--foreground)] sm:text-[1.25rem]"
  >
    {video.title}
  </h3>
  <p class="mt-2 text-[13px] leading-relaxed text-[var(--soft-foreground)]">
    Published {formatShortDate(video.published_at)}
  </p>

  <div class="mt-6" aria-label="Processing pipeline">
    <p
      class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
    >
      Pipeline
    </p>
    <ol class="mt-3 grid grid-cols-3 gap-3 sm:gap-4" role="list">
      {#each pipelineSteps as step (step.key)}
        <li class="flex flex-col items-center gap-2 text-center">
          <span
            class="relative flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-[10px] font-bold uppercase tracking-[0.06em] transition-colors duration-200 {step.status ===
            'complete'
              ? 'bg-[var(--accent-soft)] text-[var(--accent-strong)] ring-1 ring-[var(--accent)]/20'
              : step.status === 'active'
                ? 'bg-[var(--accent-wash)] text-[var(--accent-strong)] ring-2 ring-[var(--accent)]/35'
                : step.status === 'failed'
                  ? 'bg-[var(--danger-soft)] text-[var(--danger)] ring-1 ring-[var(--danger)]/25'
                  : 'bg-[var(--muted)]/50 text-[var(--soft-foreground)]'}"
            aria-current={step.status === "active" ? "step" : undefined}
          >
            <span class="sr-only">{step.label}</span>
            {#if step.status === "complete"}
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
            {:else if step.status === "failed"}
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                aria-hidden="true"
              >
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            {:else if step.status === "active"}
              <span
                class="h-2 w-2 rounded-full bg-[var(--accent)] motion-safe:animate-pulse"
                aria-hidden="true"
              ></span>
            {:else}
              <span
                class="h-1.5 w-1.5 rounded-full bg-[var(--border)]"
                aria-hidden="true"
              ></span>
            {/if}
          </span>
          <span
            class="text-[9px] font-bold uppercase leading-tight tracking-[0.08em] text-[var(--soft-foreground)] opacity-80 {step.status ===
            'active'
              ? 'text-[var(--accent-strong)] opacity-100'
              : step.status === 'complete'
                ? 'text-[var(--foreground)] opacity-90'
                : ''}"
          >
            {step.label}
          </span>
        </li>
      {/each}
    </ol>
  </div>

  <div class="mt-8 border-t border-[var(--border-soft)]/40 pt-6">
    <p
      class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
    >
      Current status
    </p>
    <div class="mt-3 flex items-start gap-3">
      <span
        class="mt-1.5 h-2 w-2 shrink-0 rounded-full {queueStateAccentClass(
          video,
        )}"
        aria-hidden="true"
      ></span>
      <p
        class="font-serif text-[1.125rem] leading-snug text-[var(--foreground)]"
      >
        {queueVideoPrimaryState(video)}
      </p>
    </div>
  </div>

  <dl class="mt-6 space-y-1">
    <div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
      <dt
        class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50"
      >
        Quality
      </dt>
      <dd
        class="text-[13px] font-medium tabular-nums text-[var(--soft-foreground)]"
      >
        {video.quality_score != null && video.quality_score !== undefined
          ? String(video.quality_score)
          : "Not scored yet"}
      </dd>
    </div>
  </dl>

  <div
    class="mt-6 flex flex-wrap gap-2 border-t border-[var(--border-soft)]/40 pt-5"
  >
    {#if video.transcript_status === "failed" && !readOnly}
      <button
        type="button"
        class="inline-flex h-9 min-h-8 items-center justify-center rounded-full bg-[var(--foreground)] px-5 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--background)] transition-all duration-200 hover:bg-[var(--accent-strong)] disabled:opacity-40"
        onclick={() => void onRetryTranscript?.(video.id)}
        disabled={retryingTranscriptVideoId === video.id}
      >
        {retryingTranscriptVideoId === video.id ? "Retrying" : "Retry download"}
      </button>
    {/if}
    {#if onOpenVideoInWorkspace}
      <button
        type="button"
        class="inline-flex h-9 min-h-8 items-center justify-center rounded-full px-5 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--accent-strong)] transition-all duration-200 hover:bg-[var(--accent-wash)]"
        onclick={() => void onOpenVideoInWorkspace?.(video)}
      >
        Open in workspace
      </button>
    {/if}
  </div>
</article>
