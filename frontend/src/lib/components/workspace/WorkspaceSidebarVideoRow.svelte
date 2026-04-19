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
  class={`group flex w-full items-start gap-2 rounded-md px-3 py-2 text-left transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${selected ? "bg-[var(--surface-strong)] text-[var(--foreground)]" : "text-[var(--soft-foreground)] hover:bg-[var(--surface)] hover:text-[var(--foreground)]"} ${className}`}
  {onclick}
  {onmouseenter}
  {onmouseleave}
>
  <div class="min-w-0 flex-1">
    <p
      class={`line-clamp-2 text-[13px] leading-[1.35] tracking-tight ${selected ? "font-semibold" : "font-medium"}`}
    >
      {video.title}
    </p>
    <div
      class="mt-1 flex items-center gap-1.5 text-[11px] text-[var(--soft-foreground)]"
    >
      <span>{formatShortDate(video.published_at)}</span>
      {#if video.transcript_status === "loading" || video.summary_status === "loading"}
        <span class="h-1 w-1 rounded-full bg-[var(--border)]" aria-hidden="true"
        ></span>
        <span class="flex items-center gap-1">
          <span class="relative flex h-1.5 w-1.5">
            <span
              class="absolute inline-flex h-full w-full animate-ping rounded-full bg-[var(--accent)] opacity-75"
            ></span>
            <span
              class="relative inline-flex h-1.5 w-1.5 rounded-full bg-[var(--accent)]"
            ></span>
          </span>
          <span>Processing</span>
        </span>
      {:else if video.transcript_status === "failed" || video.summary_status === "failed"}
        <span class="h-1 w-1 rounded-full bg-[var(--border)]" aria-hidden="true"
        ></span>
        <span class="text-[var(--danger)]">Failed</span>
      {/if}
    </div>
  </div>
</button>
