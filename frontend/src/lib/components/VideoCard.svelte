<script lang="ts">
  import type { Video } from "$lib/types";

  export let video: Video;
  export let active = false;
  export let onSelect: () => void = () => {};
  const dateFormatter = new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });

  const formatDate = (value: string) => {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
      return "Date Unavailable";
    }
    return dateFormatter.format(date);
  };
</script>

<button
  type="button"
  class={`group flex w-full min-w-0 gap-3 rounded-[var(--radius-sm)] p-2 text-left transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${
    active ? "bg-[var(--surface)] shadow-sm" : "hover:bg-[var(--surface)]/60"
  }`}
  onclick={onSelect}
>
  <div
    class="w-[88px] shrink-0 aspect-video overflow-hidden rounded-[6px] bg-[var(--muted)] relative"
  >
    {#if video.thumbnail_url}
      <img
        src={video.thumbnail_url}
        alt={video.title}
        width="176"
        height="99"
        loading="lazy"
        class="h-full w-full object-cover"
      />
    {:else}
      <div
        class="flex h-full w-full items-center justify-center text-[8px] font-bold uppercase tracking-[0.2em] text-[var(--soft-foreground)] opacity-40"
      >
        --
      </div>
    {/if}
  </div>
  <div class="flex min-w-0 flex-1 flex-col justify-center gap-1">
    <p
      class="line-clamp-2 text-[13px] font-semibold leading-[1.35] tracking-tight text-[var(--foreground)]"
    >
      {video.title}
    </p>
    <div class="flex items-center gap-2">
      <p
        class="text-[11px] font-medium text-[var(--soft-foreground)] opacity-50"
      >
        {formatDate(video.published_at)}
      </p>
      <div class="text-[9px] font-bold tracking-[0.04em]">
        {#if video.transcript_status === "loading" || video.summary_status === "loading"}
          <span class="text-[var(--accent)] flex items-center gap-1">
            <span class="relative flex h-1.5 w-1.5">
              <span
                class="animate-ping absolute inline-flex h-full w-full rounded-full bg-[var(--accent)] opacity-75"
              ></span>
              <span
                class="relative inline-flex rounded-full h-1.5 w-1.5 bg-[var(--accent)]"
              ></span>
            </span>
          </span>
        {:else if video.transcript_status === "failed" || video.summary_status === "failed"}
          <span class="text-rose-500 flex items-center gap-1">
            <svg
              width="9"
              height="9"
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
          </span>
        {:else if video.transcript_status === "ready" && video.summary_status === "ready"}
          <span class="text-[var(--soft-foreground)] opacity-25">
            <svg
              width="9"
              height="9"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="3"
              stroke-linecap="round"
              stroke-linejoin="round"><polyline points="20 6 9 17 4 12" /></svg
            >
          </span>
        {/if}
      </div>
    </div>
  </div>
</button>
