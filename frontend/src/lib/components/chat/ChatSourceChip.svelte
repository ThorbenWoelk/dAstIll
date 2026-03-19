<script lang="ts">
  import { buildWorkspaceViewHref } from "$lib/view-url";
  import type { ChatSource } from "$lib/types";

  let { source }: { source: ChatSource } = $props();

  let href = $derived(
    buildWorkspaceViewHref({
      selectedChannelId: source.channel_id,
      selectedVideoId: source.video_id,
      contentMode: source.source_kind,
      videoTypeFilter: "all",
      acknowledgedFilter: "all",
    }),
  );
</script>

<a
  {href}
  class="group flex min-w-0 flex-col rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-3 py-2 transition-colors hover:border-[var(--accent)]/35 hover:bg-[var(--accent-wash)]"
>
  <div
    class="flex items-center gap-2 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)]"
  >
    <span>{source.source_kind}</span>
    <span class="opacity-35">•</span>
    <span class="truncate">{source.channel_name}</span>
  </div>
  <p class="mt-1 truncate text-[12px] font-semibold text-[var(--foreground)]">
    {source.video_title}
  </p>
  {#if source.section_title}
    <p class="mt-1 truncate text-[11px] text-[var(--soft-foreground)]">
      {source.section_title}
    </p>
  {/if}
  <p
    class="mt-2 line-clamp-3 text-[11px] leading-relaxed text-[var(--soft-foreground)]"
  >
    {source.snippet}
  </p>
</a>
