<script lang="ts">
  import { formatShortDate } from "$lib/utils/date";
  import type { Video } from "$lib/types";

  let {
    video,
    selected = false,
    className = "",
    onclick,
    onmouseenter,
    onmouseleave,
  }: {
    video: Video;
    selected?: boolean;
    className?: string;
    onclick: () => void;
    onmouseenter?: () => void;
    onmouseleave?: () => void;
  } = $props();
</script>

<button
  type="button"
  class={`group flex w-full items-center gap-2 rounded-[var(--radius-sm)] px-2 py-1.5 text-left transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${selected ? "bg-[var(--accent-wash)]" : "hover:bg-[var(--accent-wash)]"} ${className}`}
  {onclick}
  {onmouseenter}
  {onmouseleave}
>
  <div class="min-w-0 flex-1">
    <p
      class="line-clamp-2 text-[12px] font-medium leading-tight tracking-tight text-[var(--foreground)]"
    >
      {video.title}
    </p>
    <div class="mt-1 flex items-center gap-2">
      <span class="text-[10px] text-[var(--soft-foreground)] opacity-50"
        >{formatShortDate(video.published_at)}</span
      >
      {#if video.transcript_status === "loading" || video.summary_status === "loading"}
        <span class="relative flex h-2 w-2"
          ><span
            class="absolute inline-flex h-full w-full animate-ping rounded-full bg-[var(--accent)] opacity-75"
          ></span><span
            class="relative inline-flex h-2 w-2 rounded-full bg-[var(--accent)]"
          ></span></span
        >
      {:else if video.transcript_status === "failed" || video.summary_status === "failed"}
        <svg
          class="text-[var(--danger)]"
          width="8"
          height="8"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="3"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><circle cx="12" cy="12" r="10" /><line
            x1="12"
            y1="8"
            x2="12"
            y2="12"
          /><line x1="12" y1="16" x2="12.01" y2="16" /></svg
        >
      {/if}
    </div>
  </div>
</button>
