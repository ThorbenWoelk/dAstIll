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

  const CONTENT_MODE_LABELS: Record<WorkspaceContentMode, string> = {
    transcript: "Transcript",
    summary: "Summary",
    highlights: "Highlights",
    info: "Info",
  };
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
      summaryTags: [],
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
  /** Desktop video action strip — rendered in WorkspaceDesktopTopBar. Mobile uses WorkspaceContentMobileHeader. */
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
  let summaryTags = $derived(content.summaryTags);
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

  let showMobileContentTabs = $derived(
    mobileVisible && Boolean(selectedVideoId) && !editing,
  );

  $effect(() => {
    if (typeof document === "undefined") return;
    const height = showMobileContentTabs
      ? "calc(48px + max(0.25rem, env(safe-area-inset-bottom)))"
      : "0px";
    document.documentElement.style.setProperty(
      "--mobile-tab-bar-height",
      height,
    );
    return () => {
      document.documentElement.style.setProperty(
        "--mobile-tab-bar-height",
        "0px",
      );
    };
  });

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
        "button, a, input, textarea, select, label, [role='button'], [role='tab'], mark",
      ),
    );
  }

  function handleSwipeStart(event: TouchEvent) {
    if (!mobileVisible || event.touches.length !== 1) {
      touchGesture = null;
      return;
    }

    // If the user already has text selected (e.g. they are about to drag a
    // selection handle), don't track a swipe gesture so we don't interfere
    // with the browser's text-selection drag behaviour.
    const selection = window.getSelection();
    if (selection && !selection.isCollapsed) {
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

    if (
      touchGesture?.axisLocked === "x" &&
      !touchGesture.interactive &&
      window.getSelection()?.isCollapsed
    ) {
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
  class={`fade-in stagger-3 workspace-content-shell relative z-10 flex min-h-0 min-w-0 flex-col overflow-visible border-0 lg:h-full lg:gap-4 ${mobileVisible ? "h-full" : "hidden lg:flex"}`}
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
    class="custom-scrollbar mobile-bottom-stack-padding w-full min-h-0 flex-1 overflow-y-auto px-4 max-lg:pt-4 sm:px-6 lg:px-10 lg:pt-6 lg:pb-6 xl:px-14"
    role="region"
    aria-label="Content panel"
    ontouchstart={handleSwipeStart}
    ontouchmove={handleSwipeMove}
    ontouchend={handleSwipeEnd}
    ontouchcancel={() => {
      touchGesture = null;
    }}
  >
    <div class="mx-auto flex w-full max-w-[84rem] flex-col">
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
        {summaryTags}
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
  </div>

  {#if showMobileContentTabs}
    <nav
      id="workspace-tabs-mobile"
      data-mobile-content-tabs
      class="fixed bottom-0 left-0 right-0 z-[60] border-t border-[var(--border-soft)]/50 bg-[var(--surface)]/100 px-4 pt-1.5 pb-[max(0.45rem,env(safe-area-inset-bottom))] lg:hidden"
      aria-label="Content tabs"
    >
      <div class="mx-auto grid max-w-[36rem] grid-cols-4 items-end gap-1">
        {#each WORKSPACE_CONTENT_MODE_ORDER as mode}
          <button
            type="button"
            data-workspace-content-tab={mode}
            class={`relative flex min-h-[46px] min-w-0 items-center justify-center px-1 pb-1.5 pt-3 text-[11px] font-bold uppercase tracking-[0.1em] transition-colors ${
              contentMode === mode
                ? "text-[var(--foreground)]"
                : "text-[var(--soft-foreground)] active:text-[var(--foreground)]"
            }`}
            aria-pressed={contentMode === mode}
            onclick={() => void onSetMode(mode)}
          >
            {#if contentMode === mode}
              <span
                class="absolute left-1/2 top-0.5 h-1 w-8 -translate-x-1/2 rounded-full bg-[var(--foreground)]"
                aria-hidden="true"
              ></span>
            {/if}
            <span class="truncate">{CONTENT_MODE_LABELS[mode]}</span>
          </button>
        {/each}
      </div>
    </nav>
  {/if}
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
