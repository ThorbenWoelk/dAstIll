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

  const CONTENT_MODE_EYEBROW: Record<WorkspaceContentMode, string> = {
    transcript: "Source transcript",
    summary: "Distilled summary",
    highlights: "Saved highlights",
    info: "Video context",
  };
</script>

{#if selectedVideoId && !loadingContent && selectedVideo}
  <nav
    class="mb-3 flex flex-wrap items-center gap-x-1.5 gap-y-0.5 text-[11px] font-medium uppercase tracking-[0.08em] text-[var(--soft-foreground)] opacity-60 sm:mb-4"
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
      class="text-left font-medium tracking-normal text-[var(--foreground)] opacity-80 transition-opacity hover:opacity-100"
      onclick={onShowVideos}
    >
      {selectedVideo.title}
    </button>
  </nav>

  <div class="content-hero">
    <div class="content-hero-copy">
      <p class="content-hero-eyebrow">{CONTENT_MODE_EYEBROW[contentMode]}</p>
      <h1 class="content-hero-title">{selectedVideo.title}</h1>
    </div>

    {#if contentMode === "summary"}
      <div class="content-hero-meta">
        <WorkspaceSummaryMeta
          score={summaryQualityScore}
          note={summaryQualityNote}
          modelUsed={summaryModelUsed}
          qualityModelUsed={summaryQualityModelUsed}
        />
      </div>
    {/if}
  </div>
{/if}

{#if contentMode === "transcript" && selectedVideoId && ((formattingContent && formattingVideoId === selectedVideoId) || (formattingNotice && formattingNoticeVideoId === selectedVideoId))}
  <div
    class={`mb-5 flex flex-wrap items-center gap-3 rounded-[var(--radius-full)] border px-4 py-3 transition-all duration-500 sm:mb-8 sm:px-5 ${
      formattingNoticeTone === "warning"
        ? "border-[var(--accent)]/20 bg-[var(--accent-soft)] text-[var(--accent-strong)]"
        : "border-[var(--accent-border-soft)] bg-[var(--surface)] text-[var(--soft-foreground)]"
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
    <p class="text-[11px] font-bold uppercase tracking-[0.12em]">
      {formattingContent && formattingVideoId === selectedVideoId
        ? formattingNotice || "Refining transcript with Ollama..."
        : formattingNotice}
    </p>
  </div>
{/if}

{#if contentMode === "summary" && selectedVideoId && !loadingContent}
  <div class="summary-embed-strip">
    <WorkspaceSummaryAudioPlayer
      videoId={selectedVideoId}
      summaryReady={selectedVideo?.summary_status === "ready"}
    />
  </div>
{/if}

<style>
  .content-hero {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 1.5rem;
    align-items: start;
    margin-bottom: 1.5rem;
  }

  .content-hero-copy {
    min-width: 0;
    max-width: 58rem;
  }

  .content-hero-eyebrow {
    margin: 0 0 0.55rem;
    font-size: 0.68rem;
    font-weight: 800;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--soft-foreground);
    opacity: 0.7;
  }

  .content-hero-title {
    margin: 0;
    font-family: "Fraunces", serif;
    font-size: clamp(2.2rem, 5vw, 4.3rem);
    line-height: 0.98;
    letter-spacing: -0.04em;
    text-wrap: balance;
    font-variation-settings:
      "opsz" 72,
      "wght" 650;
  }

  .content-hero-meta {
    padding-top: 0.4rem;
  }

  .summary-embed-strip {
    display: block;
    max-width: 42rem;
    margin-bottom: 0.5rem;
  }

  .summary-embed-strip :global(.waveform-player) {
    width: 100%;
    max-width: 42rem;
    align-items: start;
  }

  @media (max-width: 1023px) {
    .content-hero {
      grid-template-columns: 1fr;
      gap: 0.85rem;
      margin-bottom: 1.1rem;
    }

    .content-hero-title {
      font-size: clamp(1.8rem, 8.5vw, 3rem);
      line-height: 1.02;
    }

    .content-hero-meta {
      padding-top: 0;
    }

    .summary-embed-strip {
      max-width: none;
      margin-bottom: 0.25rem;
    }
  }
</style>
