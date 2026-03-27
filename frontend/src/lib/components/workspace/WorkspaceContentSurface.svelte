<script lang="ts">
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import LoadingSkeleton from "$lib/components/LoadingSkeleton.svelte";
  import TranscriptView from "$lib/components/TranscriptView.svelte";
  import type {
    CreateHighlightRequest,
    Highlight,
    HighlightSource,
    TranscriptRenderMode,
    Video,
    VideoInfo,
  } from "$lib/types";
  import WorkspaceHighlightsPanel from "$lib/components/workspace/WorkspaceHighlightsPanel.svelte";
  import WorkspaceVideoInfoPanel from "$lib/components/workspace/WorkspaceVideoInfoPanel.svelte";
  import type { WorkspaceContentMode } from "$lib/workspace/types";

  let {
    selectedVideoId,
    selectedVideo = null as Video | null,
    contentMode,
    loadingContent,
    editing,
    aiAvailable,
    summaryRegeneratingForSelection,
    contentText,
    contentHtml,
    transcriptRenderMode,
    contentHighlights,
    contentHighlightSource = null as HighlightSource | null,
    creatingHighlight,
    creatingHighlightVideoId,
    creatingVocabularyReplacement,
    deletingHighlightId,
    selectedVideoHighlights,
    videoInfo = null as VideoInfo | null,
    draft,
    formattingContent,
    formattingVideoId,
    revertingContent,
    revertingVideoId,
    resettingVideo,
    resettingVideoId,
    showRevertTranscriptAction,
    canRevertTranscript,
    selectedVideoYoutubeUrl,
    citationScrollText = null as string | null,
    summaryBodyRetrying,
    onStartEdit,
    onCancelEdit,
    onSaveEdit,
    onCleanFormatting,
    onRegenerateSummary,
    onRevertTranscript,
    onRequestResetVideo,
    onDraftChange,
    onToggleAcknowledge,
    onCreateHighlight,
    onCreateVocabularyReplacement = undefined as
      | ((selectedText: string) => void | Promise<void>)
      | undefined,
    onDeleteHighlight = undefined as
      | ((highlightId: number) => void | Promise<void>)
      | undefined,
    onCitationScrollConsumed = undefined as
      | (() => void | Promise<void>)
      | undefined,
  }: {
    selectedVideoId: string | null;
    selectedVideo?: Video | null;
    contentMode: WorkspaceContentMode;
    loadingContent: boolean;
    editing: boolean;
    aiAvailable: boolean;
    summaryRegeneratingForSelection: boolean;
    contentText: string;
    contentHtml: string;
    transcriptRenderMode: TranscriptRenderMode;
    contentHighlights: Highlight[];
    contentHighlightSource?: HighlightSource | null;
    creatingHighlight: boolean;
    creatingHighlightVideoId: string | null;
    creatingVocabularyReplacement: boolean;
    deletingHighlightId: number | null;
    selectedVideoHighlights: Highlight[];
    videoInfo?: VideoInfo | null;
    draft: string;
    formattingContent: boolean;
    formattingVideoId: string | null;
    revertingContent: boolean;
    revertingVideoId: string | null;
    resettingVideo: boolean;
    resettingVideoId: string | null;
    showRevertTranscriptAction: boolean;
    canRevertTranscript: boolean;
    selectedVideoYoutubeUrl: string | null;
    citationScrollText?: string | null;
    summaryBodyRetrying: boolean;
    onStartEdit: () => void;
    onCancelEdit: () => void;
    onSaveEdit: () => void | Promise<void>;
    onCleanFormatting: () => void | Promise<void>;
    onRegenerateSummary: () => void | Promise<void>;
    onRevertTranscript: () => void | Promise<void>;
    onRequestResetVideo: () => void;
    onDraftChange: (value: string) => void;
    onToggleAcknowledge: () => void | Promise<void>;
    onCreateHighlight: (
      payload: CreateHighlightRequest,
    ) => void | Promise<void>;
    onCreateVocabularyReplacement?: (
      selectedText: string,
    ) => void | Promise<void>;
    onDeleteHighlight?: (highlightId: number) => void | Promise<void>;
    onCitationScrollConsumed?: () => void | Promise<void>;
  } = $props();
</script>

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
  {@const isUnavailable = contentStatus === "failed"}
  {#if isUnavailable}
    <div
      class="flex h-full flex-col items-center justify-center py-20 text-center"
      role="status"
      aria-live="polite"
    >
      <p class="text-[14px] text-[var(--soft-foreground)] opacity-40">
        {contentMode === "summary"
          ? "Summary not available."
          : contentStatus === "failed"
            ? "Transcript generation failed."
            : "Transcript not yet available."}
      </p>
      {#if contentStatus === "failed" && contentMode === "summary"}
        <button
          type="button"
          class="mt-4 rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] px-3 py-1.5 text-[12px] font-medium text-[var(--soft-foreground)] transition-colors hover:border-[var(--accent)]/40 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] disabled:pointer-events-none disabled:opacity-40"
          disabled={!aiAvailable || summaryRegeneratingForSelection}
          onclick={onRegenerateSummary}
        >
          {summaryRegeneratingForSelection ? "Retrying…" : "Retry"}
        </button>
      {/if}
    </div>
  {:else}
    <LoadingSkeleton
      message={contentMode === "summary"
        ? contentStatus === "pending"
          ? "Summary queued and being generated..."
          : "Summary is loading..."
        : contentStatus === "pending"
          ? `Queued for ${contentMode}...`
          : `Loading ${contentMode}...`}
    />
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
      formatting={formattingContent && formattingVideoId === selectedVideoId}
      regenerating={summaryRegeneratingForSelection}
      reverting={revertingContent && revertingVideoId === selectedVideoId}
      resetting={resettingVideo && resettingVideoId === selectedVideoId}
      showFormatAction={contentMode === "transcript"}
      showRegenerateAction={contentMode === "summary"}
      showRevertAction={showRevertTranscriptAction}
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
      onReset={onRequestResetVideo}
      onChange={(value) => onDraftChange(value)}
      onAcknowledgeToggle={onToggleAcknowledge}
    />
  </div>
{:else if contentMode === "summary" && selectedVideo && (selectedVideo.summary_status !== "ready" || summaryBodyRetrying) && !contentText.trim()}
  {#if selectedVideo.summary_status === "pending" || selectedVideo.summary_status === "loading" || summaryBodyRetrying}
    <LoadingSkeleton
      message={summaryBodyRetrying || selectedVideo.summary_status === "loading"
        ? "Summary is loading..."
        : "Summary queued and being generated..."}
    />
  {:else}
    <div
      class="flex h-full flex-col items-center justify-center py-20 text-center"
    >
      <p class="text-[14px] text-[var(--soft-foreground)] opacity-40">
        Summary not available.
      </p>
      {#if selectedVideo.summary_status === "failed"}
        <button
          type="button"
          class="mt-4 rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] px-3 py-1.5 text-[12px] font-medium text-[var(--soft-foreground)] transition-colors hover:border-[var(--accent)]/40 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] disabled:pointer-events-none disabled:opacity-40"
          disabled={!aiAvailable || summaryRegeneratingForSelection}
          onclick={onRegenerateSummary}
        >
          {summaryRegeneratingForSelection ? "Retrying…" : "Retry"}
        </button>
      {/if}
    </div>
  {/if}
{:else}
  <div class="max-lg:pb-32">
    <TranscriptView
      html={contentHtml}
      text={contentText}
      mode={contentMode === "transcript" ? transcriptRenderMode : "markdown"}
      formatting={contentMode === "transcript" &&
        formattingContent &&
        formattingVideoId === selectedVideoId}
      highlights={contentHighlights}
      highlightSource={contentHighlightSource}
      highlightEnabled={Boolean(
        selectedVideoId &&
        !loadingContent &&
        !editing &&
        (contentMode === "transcript" || contentMode === "summary"),
      )}
      creatingHighlight={creatingHighlight &&
        creatingHighlightVideoId === selectedVideoId}
      {creatingVocabularyReplacement}
      {deletingHighlightId}
      {onCreateHighlight}
      {onCreateVocabularyReplacement}
      {onDeleteHighlight}
      {citationScrollText}
      {onCitationScrollConsumed}
    />
  </div>
{/if}
