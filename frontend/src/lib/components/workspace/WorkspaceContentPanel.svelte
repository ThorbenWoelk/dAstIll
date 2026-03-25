<script lang="ts">
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import ErrorToast from "$lib/components/ErrorToast.svelte";
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
  import type {
    WorkspaceContentActions,
    WorkspaceContentSelection,
    WorkspaceContentState,
    WorkspaceOverlaysActions,
    WorkspaceOverlaysState,
  } from "$lib/workspace/component-props";
  import type { WorkspaceContentMode } from "$lib/workspace/types";
  import {
    resolveSwipedContentMode,
    WORKSPACE_CONTENT_MODE_ORDER,
  } from "$lib/workspace/navigation";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import WorkspaceHighlightsPanel from "$lib/components/workspace/WorkspaceHighlightsPanel.svelte";
  import WorkspaceSummaryMeta from "$lib/components/workspace/WorkspaceSummaryMeta.svelte";
  import WorkspaceVideoInfoPanel from "$lib/components/workspace/WorkspaceVideoInfoPanel.svelte";
  import { shouldRetryReadySummaryLoad } from "$lib/workspace/content";

  const CONTENT_MODE_LABELS: Record<WorkspaceContentMode, string> = {
    transcript: "Transcript",
    summary: "Summary",
    highlights: "Highlights",
    info: "Info",
  };
  const SWIPE_BACK_THRESHOLD_PX = 72;
  const SWIPE_TAB_THRESHOLD_PX = 56;
  const SWIPE_BACK_EDGE_PX = 32;
  const SWIPE_LOCK_THRESHOLD_PX = 12;

  let {
    selection = {
      mobileVisible: false,
      mobileBackInTopBar: false,
      selectedChannel: null,
      selectedVideo: null,
      selectedVideoId: null,
      contentMode: "info",
    },
    overlays = {
      errorMessage: null,
      showDeleteConfirmation: false,
      showDeleteAccessPrompt: false,
      showResetVideoConfirmation: false,
    },
    overlayActions = {
      onDismissError: () => {},
      onConfirmDelete: () => {},
      onCancelDelete: () => {},
      onConfirmAccessPrompt: async () => {},
      onCancelAccessPrompt: () => {},
      onConfirmResetVideo: async () => {},
      onCancelResetVideo: () => {},
    },
    content = {
      loadingContent: false,
      editing: false,
      aiAvailable: false,
      summaryQualityScore: null,
      summaryQualityNote: null,
      summaryModelUsed: null,
      summaryQualityModelUsed: null,
      videoInfo: null,
      contentHtml: "",
      contentText: "",
      transcriptRenderMode: "plain_text",
      contentHighlights: [],
      selectedVideoHighlights: [],
      selectedVideoYoutubeUrl: null,
      draft: "",
      formattingContent: false,
      formattingVideoId: null,
      regeneratingSummaryVideoIds: [],
      revertingContent: false,
      revertingVideoId: null,
      resettingVideo: false,
      resettingVideoId: null,
      creatingHighlight: false,
      creatingHighlightVideoId: null,
      deletingHighlightId: null,
      canRevertTranscript: false,
      showRevertTranscriptAction: false,
      formattingNotice: null,
      formattingNoticeVideoId: null,
      formattingNoticeTone: "info",
      citationScrollText: null,
    },
    actions = {
      onBack: () => {},
      onSetMode: async () => {},
      onStartEdit: () => {},
      onCancelEdit: () => {},
      onSaveEdit: async () => {},
      onCleanFormatting: async () => {},
      onRegenerateSummary: async () => {},
      onRevertTranscript: async () => {},
      onResetVideo: async () => {},
      onDraftChange: () => {},
      onToggleAcknowledge: async () => {},
      onCreateHighlight: async (_payload: CreateHighlightRequest) => {},
      onDeleteHighlight: undefined,
      onShowChannels: () => {},
      onShowVideos: () => {},
      onCitationScrollConsumed: undefined,
    },
  }: {
    selection?: WorkspaceContentSelection;
    overlays?: WorkspaceOverlaysState;
    overlayActions?: WorkspaceOverlaysActions;
    content?: WorkspaceContentState;
    actions?: WorkspaceContentActions;
  } = $props();

  let mobileVisible = $derived(selection.mobileVisible);
  let mobileBackInTopBar = $derived(selection.mobileBackInTopBar ?? false);
  /** Mobile uses `AppBottomNav` for video actions; this strip is desktop-only (see `max-lg:hidden`). */
  let selectedChannel = $derived(selection.selectedChannel);
  let selectedVideo = $derived(selection.selectedVideo);
  let selectedVideoId = $derived(selection.selectedVideoId);
  let contentMode = $derived(selection.contentMode);

  let loadingContent = $derived(content.loadingContent);
  let editing = $derived(content.editing);
  let aiAvailable = $derived(content.aiAvailable);
  let summaryQualityScore = $derived(content.summaryQualityScore);
  let summaryQualityNote = $derived(content.summaryQualityNote);
  let summaryModelUsed = $derived(content.summaryModelUsed);
  let summaryQualityModelUsed = $derived(content.summaryQualityModelUsed);
  let videoInfo = $derived(content.videoInfo);
  let contentHtml = $derived(content.contentHtml);
  let contentText = $derived(content.contentText);
  let transcriptRenderMode = $derived(content.transcriptRenderMode);
  let contentHighlights = $derived(content.contentHighlights);
  let selectedVideoHighlights = $derived(content.selectedVideoHighlights);
  let selectedVideoYoutubeUrl = $derived(content.selectedVideoYoutubeUrl);
  let draft = $derived(content.draft);
  let formattingContent = $derived(content.formattingContent);
  let formattingVideoId = $derived(content.formattingVideoId);
  let regeneratingSummaryVideoIds = $derived(
    content.regeneratingSummaryVideoIds,
  );
  let summaryRegeneratingForSelection = $derived(
    Boolean(
      selectedVideoId && regeneratingSummaryVideoIds.includes(selectedVideoId),
    ),
  );
  let revertingContent = $derived(content.revertingContent);
  let revertingVideoId = $derived(content.revertingVideoId);
  let resettingVideo = $derived(content.resettingVideo);
  let resettingVideoId = $derived(content.resettingVideoId);
  let creatingHighlight = $derived(content.creatingHighlight);
  let creatingHighlightVideoId = $derived(content.creatingHighlightVideoId);
  let deletingHighlightId = $derived(content.deletingHighlightId);
  let canRevertTranscript = $derived(content.canRevertTranscript);
  let showRevertTranscriptAction = $derived(content.showRevertTranscriptAction);
  let formattingNotice = $derived(content.formattingNotice);
  let formattingNoticeVideoId = $derived(content.formattingNoticeVideoId);
  let formattingNoticeTone = $derived(content.formattingNoticeTone);
  let citationScrollText = $derived(content.citationScrollText ?? null);
  let contentHighlightSource = $derived.by((): HighlightSource | null =>
    contentMode === "transcript" || contentMode === "summary"
      ? contentMode
      : null,
  );

  let summaryBodyRetrying = $derived.by((): boolean =>
    shouldRetryReadySummaryLoad({
      contentMode,
      selectedVideo,
      contentText,
      loadingContent,
      editing,
    }),
  );

  let onBack = $derived(actions.onBack);
  let onSetMode = $derived(actions.onSetMode);
  let onStartEdit = $derived(actions.onStartEdit);
  let onCancelEdit = $derived(actions.onCancelEdit);
  let onSaveEdit = $derived(actions.onSaveEdit);
  let onCleanFormatting = $derived(actions.onCleanFormatting);
  let onRegenerateSummary = $derived(actions.onRegenerateSummary);
  let onRevertTranscript = $derived(actions.onRevertTranscript);
  let onResetVideo = $derived(actions.onResetVideo);
  let onDraftChange = $derived(actions.onDraftChange);
  let onToggleAcknowledge = $derived(actions.onToggleAcknowledge);
  let onCreateHighlight = $derived(actions.onCreateHighlight);
  let onDeleteHighlight = $derived(actions.onDeleteHighlight);
  let onShowChannels = $derived(actions.onShowChannels);
  let onShowVideos = $derived(actions.onShowVideos);
  let onCitationScrollConsumed = $derived(actions.onCitationScrollConsumed);

  let showResetConfirm = $state(false);

  async function confirmResetVideo() {
    showResetConfirm = false;
    await overlayActions.onConfirmResetVideo();
  }

  function cancelResetVideo() {
    showResetConfirm = false;
    overlayActions.onCancelResetVideo();
  }

  let touchGesture: {
    startX: number;
    startY: number;
    edgeStart: boolean;
    interactive: boolean;
    axisLocked: "x" | "y" | null;
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
      axisLocked: null,
    };
  }

  function handleSwipeMove(event: TouchEvent) {
    if (
      !touchGesture ||
      !mobileVisible ||
      editing ||
      event.touches.length !== 1
    ) {
      return;
    }

    const touch = event.touches[0];
    const deltaX = touch.clientX - touchGesture.startX;
    const deltaY = touch.clientY - touchGesture.startY;

    if (!touchGesture.axisLocked) {
      if (
        Math.abs(deltaX) < SWIPE_LOCK_THRESHOLD_PX &&
        Math.abs(deltaY) < SWIPE_LOCK_THRESHOLD_PX
      ) {
        return;
      }

      touchGesture = {
        ...touchGesture,
        axisLocked: Math.abs(deltaX) > Math.abs(deltaY) * 1.1 ? "x" : "y",
      };
    }

    if (touchGesture?.axisLocked === "x" && !touchGesture.interactive) {
      event.preventDefault();
    }
  }

  function handleSwipeEnd(event: TouchEvent) {
    if (
      !touchGesture ||
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
    const gesture = touchGesture;

    touchGesture = null;

    if (gesture.axisLocked !== "x") {
      return;
    }

    if (gesture.edgeStart) {
      if (
        deltaX >= SWIPE_BACK_THRESHOLD_PX &&
        Math.abs(deltaX) > Math.abs(deltaY) * 1.25
      ) {
        onBack();
      }
      return;
    }

    if (!selectedVideoId) {
      return;
    }

    const nextMode = resolveSwipedContentMode(
      contentMode,
      deltaX,
      deltaY,
      SWIPE_TAB_THRESHOLD_PX,
    );

    if (nextMode && nextMode !== contentMode) {
      void onSetMode(nextMode);
    }
  }
</script>

<section
  class={`fade-in stagger-3 relative z-10 flex min-h-0 min-w-0 flex-col overflow-visible border-0 lg:h-full lg:gap-4 ${mobileVisible ? "h-full" : "hidden lg:flex"}`}
  id="content-view"
>
  <div
    class="flex flex-col gap-3 px-4 max-lg:pb-1 max-lg:pt-3 sm:px-6 lg:hidden"
  >
    <h2 class="sr-only">Display Content</h2>
    {#if !mobileBackInTopBar}
      <div class="flex items-center gap-3">
        <button
          type="button"
          class="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
          onclick={onBack}
          aria-label="Back"
        >
          <ChevronIcon direction="left" size={18} strokeWidth={2.2} />
        </button>
      </div>
    {/if}
    <div
      class="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between"
    >
      <div class="min-w-0 flex-1" id="workspace-tabs-mobile">
        <div class="-mx-4 min-w-0 flex-1 overflow-x-auto px-4 sm:mx-0 sm:px-0">
          <div
            class="flex min-w-max items-center gap-5 border-b border-[var(--accent-border-soft)] pr-4 sm:pr-0"
          >
            {#each WORKSPACE_CONTENT_MODE_ORDER as mode}
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

      {#if selectedVideoId && !loadingContent && !editing}
        <div
          id="content-actions"
          class="relative z-20 flex h-10 max-lg:hidden items-center justify-end self-end lg:shrink-0 lg:self-auto"
        >
          <ContentEditor
            editing={false}
            busy={loadingContent}
            {aiAvailable}
            formatting={formattingContent &&
              formattingVideoId === selectedVideoId}
            regenerating={summaryRegeneratingForSelection}
            reverting={revertingContent && revertingVideoId === selectedVideoId}
            resetting={resettingVideo && resettingVideoId === selectedVideoId}
            showFormatAction={contentMode === "transcript"}
            showRegenerateAction={contentMode === "summary"}
            showRevertAction={showRevertTranscriptAction}
            showEditAction={contentMode === "transcript" ||
              contentMode === "summary"}
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
            onReset={() => {
              showResetConfirm = true;
            }}
            onChange={(value) => onDraftChange(value)}
            onAcknowledgeToggle={onToggleAcknowledge}
          />
        </div>
      {/if}
    </div>
  </div>

  <div
    class="custom-scrollbar mobile-bottom-stack-padding w-full min-h-0 flex-1 overflow-y-auto px-4 max-lg:pt-4 sm:px-6 lg:px-8 lg:pt-4 lg:pb-4"
    role="region"
    aria-label="Content panel"
    ontouchstart={handleSwipeStart}
    ontouchmove={handleSwipeMove}
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
              class="mt-4 rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] px-3 py-1.5 text-[12px] font-medium text-[var(--soft-foreground)] transition-colors hover:border-[var(--accent)]/40 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] disabled:opacity-40 disabled:pointer-events-none"
              disabled={!aiAvailable || summaryRegeneratingForSelection}
              onclick={onRegenerateSummary}
            >
              {summaryRegeneratingForSelection ? "Retrying…" : "Retry"}
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
            {contentMode === "summary"
              ? contentStatus === "pending"
                ? "Summary queued and being generated..."
                : "Summary is loading..."
              : contentStatus === "pending"
                ? `Queued for ${contentMode}...`
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
          onReset={() => {
            showResetConfirm = true;
          }}
          onChange={(value) => onDraftChange(value)}
          onAcknowledgeToggle={onToggleAcknowledge}
        />
      </div>
    {:else if contentMode === "summary" && selectedVideo && (selectedVideo.summary_status !== "ready" || summaryBodyRetrying) && !contentText.trim()}
      {#if selectedVideo.summary_status === "pending" || selectedVideo.summary_status === "loading" || summaryBodyRetrying}
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
            {summaryBodyRetrying
              ? "Summary is loading..."
              : selectedVideo.summary_status === "pending"
                ? "Summary queued and being generated..."
                : "Summary is loading..."}
          </p>
        </div>
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
              class="mt-4 rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] px-3 py-1.5 text-[12px] font-medium text-[var(--soft-foreground)] transition-colors hover:border-[var(--accent)]/40 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] disabled:opacity-40 disabled:pointer-events-none"
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
          mode={contentMode === "transcript"
            ? transcriptRenderMode
            : "markdown"}
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
          {deletingHighlightId}
          {onCreateHighlight}
          {onDeleteHighlight}
          {citationScrollText}
          {onCitationScrollConsumed}
        />
      </div>
    {/if}
  </div>
</section>

{#if overlays.errorMessage}
  <ErrorToast
    message={overlays.errorMessage}
    onDismiss={overlayActions.onDismissError}
  />
{/if}

<ConfirmationModal
  show={overlays.showDeleteConfirmation}
  title="Remove Channel?"
  message="Are you sure you want to remove this channel? All its downloaded transcripts and summaries will be permanently deleted."
  confirmLabel="Delete"
  cancelLabel="Keep"
  tone="danger"
  onConfirm={overlayActions.onConfirmDelete}
  onCancel={overlayActions.onCancelDelete}
/>

<ConfirmationModal
  show={overlays.showDeleteAccessPrompt}
  title="Admin sign-in required"
  message="Deleting channels is restricted to admins. Sign in to unlock channel management."
  confirmLabel="Sign in"
  cancelLabel="Not now"
  tone="info"
  onConfirm={overlayActions.onConfirmAccessPrompt}
  onCancel={overlayActions.onCancelAccessPrompt}
/>

<ConfirmationModal
  show={showResetConfirm || overlays.showResetVideoConfirmation}
  title="Regenerate from scratch?"
  message="This will permanently delete the transcript and summary for this video. They will be re-generated automatically."
  confirmLabel="Reset"
  cancelLabel="Cancel"
  tone="danger"
  onConfirm={confirmResetVideo}
  onCancel={cancelResetVideo}
/>
