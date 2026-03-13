<script lang="ts">
  import type { VideoInfo } from "$lib/types";
  import {
    formatDuration,
    formatPublishedAt,
    hasKnownDuration,
  } from "$lib/workspace/content";

  let {
    videoInfo = null,
  }: {
    videoInfo?: VideoInfo | null;
  } = $props();
</script>

<div class="space-y-8 pb-20 text-[15px] leading-relaxed">
  <h3
    class="text-[20px] font-bold font-serif leading-tight text-[var(--foreground)]"
  >
    {videoInfo?.title || "Untitled"}
  </h3>

  <div class="grid grid-cols-2 gap-x-6 gap-y-4 lg:grid-cols-4">
    <div>
      <p
        class="mb-1 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50"
      >
        Published
      </p>
      <p class="font-semibold text-[13px]">
        {formatPublishedAt(videoInfo?.published_at)}
      </p>
    </div>
    {#if hasKnownDuration(videoInfo?.duration_seconds, videoInfo?.duration_iso8601)}
      <div>
        <p
          class="mb-1 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50"
        >
          Duration
        </p>
        <p class="font-semibold text-[13px]">
          {formatDuration(
            videoInfo?.duration_seconds,
            videoInfo?.duration_iso8601,
          )}
        </p>
      </div>
    {/if}
    <div>
      <p
        class="mb-1 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50"
      >
        Channel
      </p>
      <p class="truncate font-semibold text-[13px]">
        {videoInfo?.channel_name || "Unknown"}
      </p>
    </div>
  </div>

  {#if videoInfo?.watch_url}
    <a
      href={videoInfo.watch_url}
      target="_blank"
      rel="noopener noreferrer"
      class="group inline-flex items-center gap-2 text-[13px] font-semibold text-[var(--accent)] hover:text-[var(--accent-strong)]"
    >
      <span>Open on YouTube</span>
      <svg
        width="12"
        height="12"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="transition-transform group-hover:-translate-y-0.5 group-hover:translate-x-0.5"
      >
        <line x1="7" y1="17" x2="17" y2="7" />
        <polyline points="7 7 17 7 17 17" />
      </svg>
    </a>
  {/if}

  {#if videoInfo?.description}
    <div>
      <p
        class="mb-3 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50"
      >
        Description
      </p>
      <p
        class="whitespace-pre-wrap text-[14px] font-medium leading-relaxed text-[var(--foreground)] opacity-70"
      >
        {videoInfo.description}
      </p>
    </div>
  {/if}
</div>
