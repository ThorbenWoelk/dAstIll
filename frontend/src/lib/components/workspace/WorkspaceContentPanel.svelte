<script lang="ts">
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import TranscriptView from "$lib/components/TranscriptView.svelte";
  import type {
    Channel,
    CreateHighlightRequest,
    Highlight,
    HighlightSource,
    TranscriptRenderMode,
    Video,
    VideoInfo,
  } from "$lib/types";
  import type { WorkspaceContentMode } from "$lib/workspace/types";
  import WorkspaceHighlightsPanel from "$lib/components/workspace/WorkspaceHighlightsPanel.svelte";
  import WorkspaceSummaryMeta from "$lib/components/workspace/WorkspaceSummaryMeta.svelte";
  import WorkspaceVideoInfoPanel from "$lib/components/workspace/WorkspaceVideoInfoPanel.svelte";

  const CONTENT_MODE_ORDER: WorkspaceContentMode[] = [
    "transcript",
    "summary",
    "highlights",
    "info",
  ];
  const CONTENT_MODE_LABELS: Record<WorkspaceContentMode, string> = {
    transcript: "Transcript",
    summary: "Summary",
    highlights: "Highlights",
    info: "Info",
  };
  const SWIPE_BACK_THRESHOLD_PX = 72;
  const SWIPE_BACK_EDGE_PX = 32;

  let {
    mobileVisible = false,
    selectedChannel = null,
    selectedVideo = null,
    selectedVideoId = null,
    contentMode = "transcript",
    loadingContent = false,
    editing = false,
    aiAvailable = false,
    summaryQualityScore = null,
    summaryQualityNote = null,
    summaryModelUsed = null,
    summaryQualityModelUsed = null,
    videoInfo = null,
    contentHtml = "",
    contentText = "",
    transcriptRenderMode = "plain_text",
    contentHighlights = [],
    selectedVideoHighlights = [],
    selectedVideoYoutubeUrl = null,
    draft = "",
    formattingContent = false,
    formattingVideoId = null,
    regeneratingSummary = false,
    regeneratingVideoId = null,
    revertingContent = false,
    revertingVideoId = null,
    creatingHighlight = false,
    creatingHighlightVideoId = null,
    deletingHighlightId = null,
    canRevertTranscript = false,
    formattingNotice = null,
    formattingNoticeVideoId = null,
    formattingNoticeTone = "info",
    onBack = () => {},
    onSetMode = async () => {},
    onStartEdit = () => {},
    onCancelEdit = () => {},
    onSaveEdit = async () => {},
    onCleanFormatting = async () => {},
    onRegenerateSummary = async () => {},
    onRevertTranscript = async () => {},
    onDraftChange = () => {},
    onToggleAcknowledge = async () => {},
    onCreateHighlight = async () => {},
    onDeleteHighlight = async () => {},
    onShowChannels = () => {},
    onShowVideos = () => {},
  }: {
    mobileVisible?: boolean;
    selectedChannel?: Channel | null;
    selectedVideo?: Video | null;
    selectedVideoId?: string | null;
    contentMode?: WorkspaceContentMode;
    loadingContent?: boolean;
    editing?: boolean;
    aiAvailable?: boolean;
    summaryQualityScore?: number | null;
    summaryQualityNote?: string | null;
    summaryModelUsed?: string | null;
    summaryQualityModelUsed?: string | null;
    videoInfo?: VideoInfo | null;
    contentHtml?: string;
    contentText?: string;
    transcriptRenderMode?: TranscriptRenderMode;
    contentHighlights?: Highlight[];
    selectedVideoHighlights?: Highlight[];
    selectedVideoYoutubeUrl?: string | null;
    draft?: string;
    formattingContent?: boolean;
    formattingVideoId?: string | null;
    regeneratingSummary?: boolean;
    regeneratingVideoId?: string | null;
    revertingContent?: boolean;
    revertingVideoId?: string | null;
    creatingHighlight?: boolean;
    creatingHighlightVideoId?: string | null;
    deletingHighlightId?: number | null;
    canRevertTranscript?: boolean;
    formattingNotice?: string | null;
    formattingNoticeVideoId?: string | null;
    formattingNoticeTone?: "info" | "success" | "warning";
    onBack?: () => void;
    onSetMode?: (mode: WorkspaceContentMode) => Promise<void> | void;
    onStartEdit?: () => void;
    onCancelEdit?: () => void;
    onSaveEdit?: () => Promise<void> | void;
    onCleanFormatting?: () => Promise<void> | void;
    onRegenerateSummary?: () => Promise<void> | void;
    onRevertTranscript?: () => Promise<void> | void;
    onDraftChange?: (value: string) => void;
    onToggleAcknowledge?: () => Promise<void> | void;
    onCreateHighlight?: (
      payload: CreateHighlightRequest,
    ) => Promise<void> | void;
    onDeleteHighlight?: (highlightId: number) => Promise<void> | void;
    onShowChannels?: () => void;
    onShowVideos?: () => void;
  } = $props();

  let touchGesture: {
    startX: number;
    startY: number;
    edgeStart: boolean;
    interactive: boolean;
  } | null = null;

  function isInteractiveSwipeTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) {
      return false;
    }

    return Boolean(
      target.closest(
        "button, a, input, textarea, select, label, [role='button'], [role='tab']",
      ),
    );
  }

  function handleSwipeStart(event: TouchEvent) {
    if (!mobileVisible || event.touches.length !== 1) {
      touchGesture = null;
      return;
    }

    const touch = event.touches[0];
    const edgeStart = touch.clientX <= SWIPE_BACK_EDGE_PX;
    touchGesture = {
      startX: touch.clientX,
      startY: touch.clientY,
      edgeStart,
      interactive: edgeStart ? false : isInteractiveSwipeTarget(event.target),
    };
  }

  function handleSwipeEnd(event: TouchEvent) {
    if (
      !touchGesture ||
      !touchGesture.edgeStart ||
      touchGesture.interactive ||
      !mobileVisible ||
      editing ||
      event.changedTouches.length !== 1
    ) {
      touchGesture = null;
      return;
    }

    const touch = event.changedTouches[0];
    const deltaX = touch.clientX - touchGesture.startX;
    const deltaY = touch.clientY - touchGesture.startY;

    touchGesture = null;

    if (
      deltaX < SWIPE_BACK_THRESHOLD_PX ||
      Math.abs(deltaX) <= Math.abs(deltaY) * 1.25
    ) {
      return;
    }

    onBack();
  }
</script>

<section
  class={`fade-in stagger-3 relative z-10 flex min-h-0 min-w-0 flex-col overflow-visible border-0 lg:sticky lg:top-4 lg:h-[calc(100vh-4rem)] lg:gap-4 lg:py-6 lg:pl-5 ${mobileVisible ? "h-full" : "hidden lg:flex"}`}
  id="content-view"
>
  <div class="flex flex-col gap-3 px-4 max-lg:pb-1 max-lg:pt-3 sm:px-6 lg:px-0">
    <h2 class="sr-only">Display Content</h2>
    <div
      class="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between"
    >
      <div class="min-w-0 flex-1" id="content-mode-tabs">
        <div class="-mx-4 min-w-0 flex-1 overflow-x-auto px-4 sm:mx-0 sm:px-0">
          <div
            class="flex min-w-max items-center gap-5 border-b border-[var(--accent-border-soft)] pr-4 sm:pr-0"
          >
            {#each CONTENT_MODE_ORDER as mode}
              <button
                type="button"
                class={`-mb-px border-b-2 pb-3 text-[11px] font-bold uppercase tracking-[0.12em] transition-colors ${
                  contentMode === mode
                    ? "border-[var(--accent)] text-[var(--accent-strong)]"
                    : "border-transparent text-[var(--soft-foreground)] opacity-75 hover:text-[var(--foreground)] hover:opacity-100"
                }`}
                aria-pressed={contentMode === mode}
                onclick={() => void onSetMode(mode)}
              >
                {CONTENT_MODE_LABELS[mode]}
              </button>
            {/each}
          </div>
        </div>
      </div>

      {#if selectedVideoId && !loadingContent && !editing && contentMode !== "info" && contentMode !== "highlights"}
        <div
          id="content-actions"
          class="relative z-20 flex h-10 items-center justify-end self-end lg:shrink-0 lg:self-auto"
        >
          <ContentEditor
            editing={false}
            busy={loadingContent}
            {aiAvailable}
            formatting={formattingContent &&
              formattingVideoId === selectedVideoId}
            regenerating={regeneratingSummary &&
              regeneratingVideoId === selectedVideoId}
            reverting={revertingContent && revertingVideoId === selectedVideoId}
            showFormatAction={contentMode === "transcript"}
            showRegenerateAction={contentMode === "summary"}
            showRevertAction={contentMode === "transcript"}
            canRevert={canRevertTranscript}
            youtubeUrl={selectedVideoYoutubeUrl}
            value={draft}
            acknowledged={selectedVideo?.acknowledged ?? false}
            onEdit={onStartEdit}
            onCancel={onCancelEdit}
            onSave={onSaveEdit}
            onFormat={onCleanFormatting}
            onRegenerate={onRegenerateSummary}
            onRevert={onRevertTranscript}
            onChange={(value) => onDraftChange(value)}
            onAcknowledgeToggle={onToggleAcknowledge}
          />
        </div>
      {/if}
    </div>
  </div>

  <div
    class="custom-scrollbar mobile-bottom-stack-padding w-full min-h-0 flex-1 overflow-y-auto px-4 max-lg:pt-4 sm:px-6 lg:px-0 lg:pr-4 lg:pb-0"
    role="region"
    aria-label="Content panel"
    ontouchstart={handleSwipeStart}
    ontouchend={handleSwipeEnd}
    ontouchcancel={() => {
      touchGesture = null;
    }}
  >
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
    {/if}

    {#if !selectedVideoId}
      <div
        class="flex h-full flex-col items-center justify-center py-20 text-center"
      >
        <div
          class="max-w-[24rem] rounded-[var(--radius-lg)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] px-6 py-8 shadow-sm"
        >
          <div
            class="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-[var(--accent-soft)]/60 text-[var(--accent-strong)]"
          >
            <svg
              width="22"
              height="22"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <path
                d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"
              />
              <polyline points="14 2 14 8 20 8" />
              <line x1="8" y1="13" x2="16" y2="13" />
              <line x1="8" y1="17" x2="13" y2="17" />
            </svg>
          </div>
          <p class="mt-4 text-[17px] font-semibold text-[var(--foreground)]">
            Select a video
          </p>
          <p class="mt-2 text-[14px] leading-6 text-[var(--soft-foreground)]">
            Open any video from the library to read its transcript, inspect the
            summary, and capture highlights.
          </p>
        </div>
      </div>
    {:else if loadingContent}
      {@const contentStatus =
        contentMode === "summary"
          ? selectedVideo?.summary_status
          : contentMode === "transcript"
            ? selectedVideo?.transcript_status
            : null}
      {@const isProcessing = contentStatus === "loading"}
      {@const isUnavailable =
        contentStatus === "pending" || contentStatus === "failed"}
      {#if isUnavailable}
        <div
          class="flex h-full flex-col items-center justify-center py-20 text-center"
          role="status"
          aria-live="polite"
        >
          <p class="text-[13px] text-[var(--soft-foreground)] opacity-40">
            {contentStatus === "failed"
              ? `${contentMode === "summary" ? "Summary" : "Transcript"} generation failed.`
              : `${contentMode === "summary" ? "Summary" : "Transcript"} not yet available.`}
          </p>
          {#if contentStatus === "failed" && contentMode === "summary"}
            <button
              type="button"
              class="mt-4 rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] px-3 py-1.5 text-[12px] font-medium text-[var(--soft-foreground)] transition-colors hover:border-[var(--accent)]/40 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] disabled:opacity-40 disabled:pointer-events-none"
              disabled={!aiAvailable || regeneratingSummary}
              onclick={onRegenerateSummary}
            >
              {regeneratingSummary ? "Retrying…" : "Retry"}
            </button>
          {/if}
        </div>
      {:else}
        <div
          class="mt-4 space-y-8 animate-pulse"
          role="status"
          aria-live="polite"
        >
          <div
            class="h-10 w-3/5 rounded-[var(--radius-sm)] bg-[var(--muted)]/60"
          ></div>
          <div class="space-y-4 pt-4">
            <div class="h-4 w-full rounded-full bg-[var(--muted)]/50"></div>
            <div class="h-4 w-11/12 rounded-full bg-[var(--muted)]/50"></div>
            <div class="h-4 w-10/12 rounded-full bg-[var(--muted)]/50"></div>
            <div class="h-4 w-full rounded-full bg-[var(--muted)]/50"></div>
            <div class="h-4 w-3/4 rounded-full bg-[var(--muted)]/50"></div>
          </div>
          <p
            class="pt-10 text-center text-[10px] font-bold uppercase tracking-[0.4em] text-[var(--accent)]"
          >
            {isProcessing
              ? `Processing ${contentMode}...`
              : `Loading ${contentMode}...`}
          </p>
        </div>
      {/if}
    {:else if contentMode === "highlights"}
      <WorkspaceHighlightsPanel
        {selectedVideo}
        highlights={selectedVideoHighlights}
        {deletingHighlightId}
        {onDeleteHighlight}
      />
    {:else if contentMode === "info"}
      <WorkspaceVideoInfoPanel {videoInfo} />
    {:else if editing}
      <div class="pb-20">
        <ContentEditor
          editing
          busy={loadingContent}
          {aiAvailable}
          formatting={formattingContent &&
            formattingVideoId === selectedVideoId}
          regenerating={regeneratingSummary &&
            regeneratingVideoId === selectedVideoId}
          reverting={revertingContent && revertingVideoId === selectedVideoId}
          showFormatAction={contentMode === "transcript"}
          showRegenerateAction={contentMode === "summary"}
          showRevertAction={contentMode === "transcript"}
          canRevert={canRevertTranscript}
          youtubeUrl={selectedVideoYoutubeUrl}
          value={draft}
          acknowledged={selectedVideo?.acknowledged ?? false}
          onEdit={onStartEdit}
          onCancel={onCancelEdit}
          onSave={onSaveEdit}
          onFormat={onCleanFormatting}
          onRegenerate={onRegenerateSummary}
          onRevert={onRevertTranscript}
          onChange={(value) => onDraftChange(value)}
          onAcknowledgeToggle={onToggleAcknowledge}
        />
      </div>
    {:else if contentMode === "summary" && selectedVideo && selectedVideo.summary_status !== "ready" && selectedVideo.summary_status !== "loading" && !contentText.trim()}
      <div
        class="flex h-full flex-col items-center justify-center py-20 text-center"
      >
        <p class="text-[13px] text-[var(--soft-foreground)] opacity-40">
          {selectedVideo.summary_status === "failed"
            ? "Summary generation failed."
            : "Summary not yet available."}
        </p>
        {#if selectedVideo.summary_status === "failed"}
          <button
            type="button"
            class="mt-4 rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] px-3 py-1.5 text-[12px] font-medium text-[var(--soft-foreground)] transition-colors hover:border-[var(--accent)]/40 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] disabled:opacity-40 disabled:pointer-events-none"
            disabled={!aiAvailable || regeneratingSummary}
            onclick={onRegenerateSummary}
          >
            {regeneratingSummary ? "Retrying…" : "Retry"}
          </button>
        {/if}
      </div>
    {:else}
      <div class="max-lg:pb-32">
        <TranscriptView
          html={contentHtml}
          text={contentText}
          mode={contentMode === "transcript"
            ? transcriptRenderMode
            : "markdown"}
          formatting={contentMode === "transcript" &&
            formattingContent &&
            formattingVideoId === selectedVideoId}
          highlights={contentHighlights}
          highlightSource={contentMode === "transcript" ||
          contentMode === "summary"
            ? (contentMode as HighlightSource)
            : null}
          highlightEnabled={Boolean(
            selectedVideoId &&
            !loadingContent &&
            !editing &&
            (contentMode === "transcript" || contentMode === "summary"),
          )}
          creatingHighlight={creatingHighlight &&
            creatingHighlightVideoId === selectedVideoId}
          {deletingHighlightId}
          {onCreateHighlight}
          {onDeleteHighlight}
        />
      </div>
    {/if}
  </div>
</section>
