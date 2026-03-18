<script lang="ts">
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import Toggle from "$lib/components/Toggle.svelte";
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
</script>

<section
  class={`fade-in stagger-3 relative z-10 flex min-h-0 min-w-0 flex-col overflow-visible border-0 lg:sticky lg:top-4 lg:h-[calc(100vh-4rem)] lg:gap-4 lg:py-6 lg:pl-6 ${mobileVisible ? "h-full" : "hidden lg:flex"}`}
  id="content-view"
>
  <div
    class="flex flex-wrap items-center justify-between gap-3 px-4 max-lg:pb-1 max-lg:pt-3 sm:px-6 lg:px-0"
  >
    <h2 class="sr-only">Display Content</h2>
    <div class="flex items-center gap-3 sm:gap-4" id="content-mode-tabs">
      <Toggle
        options={["transcript", "summary", "highlights", "info"]}
        value={contentMode}
        onChange={(value) => void onSetMode(value as WorkspaceContentMode)}
      />
    </div>

    {#if selectedVideoId && !loadingContent && !editing && contentMode !== "info" && contentMode !== "highlights"}
      <div
        id="content-actions"
        class="relative z-20 flex h-10 items-center justify-end"
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

  <div
    class="custom-scrollbar mobile-bottom-stack-padding w-full min-h-0 flex-1 overflow-y-auto px-4 max-lg:pt-4 sm:px-6 lg:px-0 lg:pr-4 lg:pb-0"
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
            : "border-[var(--border-soft)] bg-[var(--muted)]/30 text-[var(--soft-foreground)]"
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
        <p class="text-[15px] text-[var(--soft-foreground)] opacity-30">
          Select a video to view its content.
        </p>
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
