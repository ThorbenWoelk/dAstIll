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
  {#if videoInfo?.thumbnail_url}
    <div
      class="w-full max-w-[50%] overflow-hidden rounded-[var(--radius-lg)] bg-[var(--muted)]/40 max-sm:max-w-full"
    >
      <img
        src={videoInfo.thumbnail_url}
        alt={videoInfo?.title || "Video thumbnail"}
        class="aspect-video w-full object-cover"
        loading="lazy"
        referrerpolicy="no-referrer"
      />
    </div>
  {/if}

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
      <p class="font-semibold text-[14px]">
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
        <p class="font-semibold text-[14px]">
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
      <p class="truncate font-semibold text-[14px]">
        {videoInfo?.channel_name || "Unknown"}
      </p>
    </div>
  </div>

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
