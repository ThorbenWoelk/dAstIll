<script lang="ts">
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import SignInRequiredModal from "$lib/components/SignInRequiredModal.svelte";
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import ErrorToast from "$lib/components/ErrorToast.svelte";
  import WorkspaceContentContextStrip from "$lib/components/workspace/WorkspaceContentContextStrip.svelte";
  import type { HighlightSource } from "$lib/types";
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
  import WorkspaceContentMobileHeader from "$lib/components/workspace/WorkspaceContentMobileHeader.svelte";
  import WorkspaceContentSurface from "$lib/components/workspace/WorkspaceContentSurface.svelte";
  import { shouldRetryReadySummaryLoad } from "$lib/workspace/content";
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
      showAddSourceFeedback: false,
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
      creatingVocabularyReplacement: false,
      deletingHighlightId: null,
      canRevertTranscript: false,
      showRevertTranscriptAction: false,
      formattingNotice: null,
      formattingNoticeVideoId: null,
      formattingNoticeTone: "info",
      citationScrollText: null,
      canPersistHighlights: true,
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
      onCreateHighlight: undefined,
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
  let creatingVocabularyReplacement = $derived(
    content.creatingVocabularyReplacement,
  );
  let deletingHighlightId = $derived(content.deletingHighlightId);
  let canRevertTranscript = $derived(content.canRevertTranscript);
  let showRevertTranscriptAction = $derived(content.showRevertTranscriptAction);
  let formattingNotice = $derived(content.formattingNotice);
  let formattingNoticeVideoId = $derived(content.formattingNoticeVideoId);
  let formattingNoticeTone = $derived(content.formattingNoticeTone);
  let citationScrollText = $derived(content.citationScrollText ?? null);
  let canPersistHighlights = $derived(content.canPersistHighlights);
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
  let onCreateVocabularyReplacement = $derived(
    actions.onCreateVocabularyReplacement,
  );
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
  <WorkspaceContentMobileHeader
    {mobileBackInTopBar}
    {contentMode}
    {selectedVideoId}
    {selectedVideo}
    {loadingContent}
    {editing}
    {aiAvailable}
    {formattingContent}
    {formattingVideoId}
    {summaryRegeneratingForSelection}
    {revertingContent}
    {revertingVideoId}
    {resettingVideo}
    {resettingVideoId}
    {showRevertTranscriptAction}
    {canRevertTranscript}
    {selectedVideoYoutubeUrl}
    {draft}
    {onBack}
    {onSetMode}
    {onStartEdit}
    {onCancelEdit}
    {onSaveEdit}
    {onCleanFormatting}
    {onRegenerateSummary}
    {onRevertTranscript}
    onRequestResetVideo={() => {
      showResetConfirm = true;
    }}
    {onDraftChange}
    {onToggleAcknowledge}
  />

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
    <WorkspaceContentContextStrip
      {selectedChannel}
      {selectedVideo}
      {selectedVideoId}
      {contentMode}
      {loadingContent}
      {formattingContent}
      {formattingVideoId}
      {formattingNotice}
      {formattingNoticeVideoId}
      {formattingNoticeTone}
      {summaryQualityScore}
      {summaryQualityNote}
      {summaryModelUsed}
      {summaryQualityModelUsed}
      {onShowChannels}
      {onShowVideos}
    />

    <WorkspaceContentSurface
      {selectedVideoId}
      {selectedVideo}
      {contentMode}
      {loadingContent}
      {editing}
      {aiAvailable}
      {summaryRegeneratingForSelection}
      {contentText}
      {contentHtml}
      {transcriptRenderMode}
      {contentHighlights}
      {contentHighlightSource}
      {creatingHighlight}
      {creatingHighlightVideoId}
      {creatingVocabularyReplacement}
      {deletingHighlightId}
      {selectedVideoHighlights}
      {videoInfo}
      {draft}
      {formattingContent}
      {formattingVideoId}
      {revertingContent}
      {revertingVideoId}
      {resettingVideo}
      {resettingVideoId}
      {showRevertTranscriptAction}
      {canRevertTranscript}
      {selectedVideoYoutubeUrl}
      {citationScrollText}
      {summaryBodyRetrying}
      {canPersistHighlights}
      {onStartEdit}
      {onCancelEdit}
      {onSaveEdit}
      {onCleanFormatting}
      {onRegenerateSummary}
      {onRevertTranscript}
      onRequestResetVideo={() => {
        showResetConfirm = true;
      }}
      {onDraftChange}
      {onToggleAcknowledge}
      {onCreateHighlight}
      {onCreateVocabularyReplacement}
      {onDeleteHighlight}
      {onCitationScrollConsumed}
    />
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

<SignInRequiredModal
  show={overlays.showDeleteAccessPrompt}
  message="Sign in to remove channels and manage your library."
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
