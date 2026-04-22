<script lang="ts">
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import CopyIcon from "$lib/components/icons/CopyIcon.svelte";
  import WorkspaceSummaryAudioPlayer from "$lib/components/workspace/WorkspaceSummaryAudioPlayer.svelte";
  import WorkspaceSummaryMeta from "$lib/components/workspace/WorkspaceSummaryMeta.svelte";
  import type { Channel, Video } from "$lib/types";
  import type { WorkspaceContentMode } from "$lib/workspace/types";
  import { formatShortDate } from "$lib/utils/date";

  let {
    selectedChannel = null as Channel | null,
    selectedVideo = null as Video | null,
    selectedVideoId,
    contentMode,
    contentText = "",
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
    summaryTags = [] as string[],
    summaryTagsEvaluated = false,
  }: {
    selectedChannel?: Channel | null;
    selectedVideo?: Video | null;
    selectedVideoId: string | null;
    contentMode: WorkspaceContentMode;
    contentText?: string;
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
    summaryTags?: string[];
    summaryTagsEvaluated?: boolean;
  } = $props();

  let publishedLabel = $derived(
    selectedVideo ? formatShortDate(selectedVideo.published_at) : null,
  );
  let copyState = $state<"idle" | "copied" | "error">("idle");
  let copyResetTimer: ReturnType<typeof setTimeout> | null = null;
  let copyKind = $derived(
    contentMode === "summary"
      ? "summary"
      : contentMode === "transcript"
        ? "transcript"
        : "content",
  );
  let canCopyContent = $derived(
    (contentMode === "summary" || contentMode === "transcript") &&
      contentText.trim().length > 0,
  );
  let copyText = $derived.by(() => {
    const title = selectedVideo?.title.trim() ?? "";
    const body = contentText.trim();
    return title ? `${title}\n\n${body}` : body;
  });
  let copyLabel = $derived(
    copyState === "copied"
      ? `Copied ${copyKind}`
      : copyState === "error"
        ? `Copy ${copyKind} failed`
        : `Copy ${copyKind} text`,
  );
  let hasMetadataBeforeCopy = $derived(
    Boolean(
      selectedChannel ||
      publishedLabel ||
      selectedVideo?.is_short ||
      contentMode === "summary",
    ),
  );

  function resetCopyStateLater() {
    if (copyResetTimer) clearTimeout(copyResetTimer);
    copyResetTimer = setTimeout(() => {
      copyState = "idle";
      copyResetTimer = null;
    }, 2000);
  }

  async function copyCurrentContent() {
    if (!canCopyContent) return;

    try {
      await navigator.clipboard.writeText(copyText);
      copyState = "copied";
    } catch {
      copyState = "error";
    }

    resetCopyStateLater();
  }

  $effect(() => {
    contentMode;
    copyText;
    if (copyResetTimer) {
      clearTimeout(copyResetTimer);
      copyResetTimer = null;
    }
    copyState = "idle";
  });

  $effect(() => {
    return () => {
      if (copyResetTimer) {
        clearTimeout(copyResetTimer);
        copyResetTimer = null;
      }
    };
  });
</script>

{#if selectedVideoId && !loadingContent && selectedVideo}
  <div
    class="mb-6 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs font-medium tracking-wide text-[var(--soft-foreground)]"
  >
    {#if selectedChannel}
      <a
        href={`/channels/${encodeURIComponent(selectedChannel.id)}`}
        class="transition-colors hover:text-[var(--foreground)]"
      >
        {selectedChannel.name}
      </a>
    {/if}
    {#if selectedChannel && publishedLabel}
      <span class="h-1 w-1 rounded-full bg-[var(--border)]" aria-hidden="true"
      ></span>
    {/if}
    {#if publishedLabel}
      <time datetime={selectedVideo.published_at}>{publishedLabel}</time>
    {/if}
    {#if selectedVideo.is_short}
      <span class="h-1 w-1 rounded-full bg-[var(--border)]" aria-hidden="true"
      ></span>
      <span>Short</span>
    {/if}
    {#if contentMode === "summary"}
      <span class="h-1 w-1 rounded-full bg-[var(--border)]" aria-hidden="true"
      ></span>
      <WorkspaceSummaryMeta
        compact
        score={summaryQualityScore}
        note={summaryQualityNote}
        modelUsed={summaryModelUsed}
        qualityModelUsed={summaryQualityModelUsed}
        tags={summaryTags}
        tagsEvaluated={summaryTagsEvaluated}
      />
    {/if}
    {#if canCopyContent}
      {#if hasMetadataBeforeCopy}
        <span class="h-1 w-1 rounded-full bg-[var(--border)]" aria-hidden="true"
        ></span>
      {/if}
      <button
        type="button"
        class="-my-2 inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 sm:-my-1 sm:h-8 sm:w-8"
        aria-label={copyLabel}
        onclick={copyCurrentContent}
      >
        {#if copyState === "copied"}
          <CheckIcon
            size={16}
            strokeWidth={2}
            className="text-[var(--accent)]"
          />
        {:else}
          <CopyIcon size={14} strokeWidth={2} />
        {/if}
      </button>
    {/if}
  </div>

  <h1 class="content-hero-title mb-6 text-[var(--foreground)] sm:mb-8">
    {selectedVideo.title}
  </h1>
{/if}

{#if contentMode === "transcript" && selectedVideoId && ((formattingContent && formattingVideoId === selectedVideoId) || (formattingNotice && formattingNoticeVideoId === selectedVideoId))}
  <div
    class={`mb-5 flex flex-wrap items-center gap-3 rounded-md border px-4 py-2.5 transition-all duration-500 sm:mb-6 ${
      formattingNoticeTone === "warning"
        ? "border-[var(--accent)]/20 bg-[var(--accent-soft)] text-[var(--accent-strong)]"
        : "border-[var(--border-soft)] bg-[var(--surface)] text-[var(--soft-foreground)]"
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
        stroke-width="1.75"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="12" cy="12" r="10" />
        <polyline points="12 6 12 12 16 14" />
      </svg>
    {/if}
    <p class="text-[12px] font-medium">
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
  .content-hero-title {
    margin: 0;
    font-family: "Fraunces", serif;
    font-size: clamp(1.75rem, 3vw, 2.5rem);
    line-height: 1.15;
    letter-spacing: -0.02em;
    text-wrap: balance;
    font-variation-settings:
      "opsz" 36,
      "wght" 580;
  }

  .summary-embed-strip {
    max-width: 52rem;
    margin-bottom: 0.5rem;
    min-width: 0;
  }

  .summary-embed-strip :global(.waveform-player) {
    width: 100%;
    max-width: 42rem;
    align-items: start;
  }

  @media (max-width: 1023px) {
    .content-hero-title {
      font-size: clamp(1.5rem, 6vw, 2rem);
      line-height: 1.2;
    }

    .summary-embed-strip {
      max-width: none;
      margin-bottom: 0.25rem;
    }
  }
</style>
