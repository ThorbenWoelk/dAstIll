<script lang="ts">
  import WorkspaceSummaryAudioPlayer from "$lib/components/workspace/WorkspaceSummaryAudioPlayer.svelte";
  import WorkspaceSummaryMeta from "$lib/components/workspace/WorkspaceSummaryMeta.svelte";
  import type { Channel, Video } from "$lib/types";
  import type { WorkspaceContentMode } from "$lib/workspace/types";

  let {
    selectedChannel = null as Channel | null,
    selectedVideo = null as Video | null,
    selectedVideoId,
    contentMode,
    loadingContent,
    formattingContent,
    formattingVideoId,
    formattingNotice = null as string | null,
    formattingNoticeVideoId = null as string | null,
    formattingNoticeTone = "info" as "info" | "success" | "warning",
    summaryQualityScore = null as number | null,
    summaryQualityNote = null as string | null,
    summaryModelUsed = null as string | null,
    summaryQualityModelUsed = null as string | null,
    onShowChannels,
    onShowVideos,
  }: {
    selectedChannel?: Channel | null;
    selectedVideo?: Video | null;
    selectedVideoId: string | null;
    contentMode: WorkspaceContentMode;
    loadingContent: boolean;
    formattingContent: boolean;
    formattingVideoId: string | null;
    formattingNotice?: string | null;
    formattingNoticeVideoId?: string | null;
    formattingNoticeTone?: "info" | "success" | "warning";
    summaryQualityScore?: number | null;
    summaryQualityNote?: string | null;
    summaryModelUsed?: string | null;
    summaryQualityModelUsed?: string | null;
    onShowChannels: () => void;
    onShowVideos: () => void;
  } = $props();
</script>

{#if selectedVideoId && !loadingContent && selectedVideo}
  <nav
    class="mb-3 flex flex-wrap items-center gap-x-1.5 gap-y-0.5 text-[12px] text-[var(--soft-foreground)] opacity-60 sm:mb-4"
    aria-label="Breadcrumb"
  >
    {#if selectedChannel}
      <button
        type="button"
        class="shrink-0 transition-colors hover:text-[var(--foreground)]"
        onclick={onShowChannels}
      >
        {selectedChannel.name}
      </button>
      <svg
        class="shrink-0"
        width="10"
        height="10"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polyline points="9 18 15 12 9 6" />
      </svg>
    {/if}
    <button
      type="button"
      class="text-left font-medium text-[var(--foreground)] opacity-80 transition-opacity hover:opacity-100"
      onclick={onShowVideos}
    >
      {selectedVideo.title}
    </button>
  </nav>
{/if}

{#if contentMode === "transcript" && selectedVideoId && ((formattingContent && formattingVideoId === selectedVideoId) || (formattingNotice && formattingNoticeVideoId === selectedVideoId))}
  <div
    class={`mb-4 flex flex-wrap items-center gap-3 rounded-[var(--radius-md)] border p-4 transition-all duration-500 sm:mb-8 ${
      formattingNoticeTone === "warning"
        ? "border-[var(--accent)]/20 bg-[var(--accent-soft)]/50 text-[var(--accent-strong)]"
        : "border-[var(--accent-border-soft)] bg-[var(--accent-wash)] text-[var(--soft-foreground)]"
    }`}
    role="status"
    aria-live="polite"
  >
    {#if formattingContent && formattingVideoId === selectedVideoId}
      <span class="relative flex h-2 w-2">
        <span
          class="absolute inline-flex h-full w-full animate-ping rounded-full bg-current opacity-75"
        ></span>
        <span class="relative inline-flex h-2 w-2 rounded-full bg-current"
        ></span>
      </span>
    {:else}
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="3"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="12" cy="12" r="10" />
        <polyline points="12 6 12 12 16 14" />
      </svg>
    {/if}
    <p class="text-[12px] font-bold uppercase tracking-wide">
      {formattingContent && formattingVideoId === selectedVideoId
        ? formattingNotice || "Refining transcript with Ollama..."
        : formattingNotice}
    </p>
  </div>
{/if}

{#if contentMode === "summary" && selectedVideoId && !loadingContent}
  <WorkspaceSummaryMeta
    score={summaryQualityScore}
    note={summaryQualityNote}
    modelUsed={summaryModelUsed}
    qualityModelUsed={summaryQualityModelUsed}
  />
  <WorkspaceSummaryAudioPlayer
    videoId={selectedVideoId}
    summaryReady={selectedVideo?.summary_status === "ready"}
  />
{/if}
