import type { page as pageState } from "$app/state";

import type { createHomeWorkspaceHighlightController } from "$lib/workspace/home-workspace-highlight-controller.svelte";
import type { createHomeWorkspacePageState } from "$lib/workspace/home-workspace-page-state.svelte";
import type { createVocabularyController } from "$lib/workspace/vocabulary-controller.svelte";
import type { createContentState } from "$lib/workspace/content-state.svelte";
import type { createHomeWorkspaceDataController } from "$lib/workspace/home-workspace-data-controller.svelte";
import type { Channel, Highlight, Video, VideoInfo } from "$lib/types";
import type { TranscriptRenderMode } from "$lib/types";
import type { WorkspaceContentMode } from "$lib/workspace/types";

export function createHomeWorkspaceViewModel(options: {
  page: typeof pageState;
  replaceWorkspaceUrl: (href: string) => void;
  pageState: ReturnType<typeof createHomeWorkspacePageState>;
  content: ReturnType<typeof createContentState>;
  highlightController: ReturnType<
    typeof createHomeWorkspaceHighlightController
  >;
  vocabulary: ReturnType<typeof createVocabularyController>;
  dataController: ReturnType<typeof createHomeWorkspaceDataController>;
  getLoadingContent: () => boolean;
  getEditing: () => boolean;
  getContentText: () => string;
  getContentMode: () => WorkspaceContentMode;
  getSelectedChannel: () => Channel | null;
  getSelectedVideo: () => Video | null;
  getSelectedVideoId: () => string | null;
  getSelectedVideoYoutubeUrl: () => string | null;
  getSelectedVideoHighlights: () => Highlight[];
  getContentHighlights: () => Highlight[];
  getVideoInfo: () => VideoInfo | null;
  getContentHtml: () => string;
  getTranscriptRenderMode: () => TranscriptRenderMode;
  getCanRevertTranscript: () => boolean;
  getHasUpdatedTranscript: () => boolean;
  onToggleAcknowledge: () => Promise<void>;
}) {
  const workspaceContentSelection = $derived({
    mobileVisible: true,
    mobileBackInTopBar:
      !options.pageState.mobileBrowseOpen &&
      Boolean(options.getSelectedVideoId()),
    selectedChannel: options.getSelectedChannel(),
    selectedVideo: options.getSelectedVideo(),
    selectedVideoId: options.getSelectedVideoId(),
    contentMode: options.getContentMode(),
  });

  const citationScrollText = $derived.by(() => {
    const url = options.page.url;
    const cite = url.searchParams.get("cite")?.trim();
    if (!cite || options.getLoadingContent()) {
      return null;
    }
    const videoParam =
      url.searchParams.get("item")?.trim() ??
      url.searchParams.get("video")?.trim();
    const selectedVideoId = options.getSelectedVideoId();
    if (videoParam && selectedVideoId && videoParam !== selectedVideoId) {
      return null;
    }
    return cite;
  });

  function onCitationScrollConsumed() {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- transient URL for one-shot citation navigation cleanup
    const url = new URL(options.page.url.href);
    if (!url.searchParams.has("cite") && !url.searchParams.has("chunk")) {
      return;
    }
    url.searchParams.delete("cite");
    url.searchParams.delete("chunk");
    options.replaceWorkspaceUrl(`${url.pathname}${url.search}${url.hash}`);
  }

  const workspaceContentState = $derived({
    loadingContent: options.getLoadingContent(),
    editing: options.getEditing(),
    aiAvailable: options.pageState.aiAvailable ?? false,
    summaryQualityScore: options.content.summaryQualityScore,
    summaryQualityNote: options.content.summaryQualityNote,
    summaryModelUsed: options.content.summaryModelUsed,
    summaryQualityModelUsed: options.content.summaryQualityModelUsed,
    videoInfo: options.getVideoInfo(),
    contentHtml: options.getContentHtml(),
    contentText: options.getContentText(),
    transcriptRenderMode: options.getTranscriptRenderMode(),
    contentHighlights: options.getContentHighlights(),
    selectedVideoHighlights: options.getSelectedVideoHighlights(),
    selectedVideoYoutubeUrl: options.getSelectedVideoYoutubeUrl(),
    draft: options.content.draft,
    formattingContent: options.content.formattingContent,
    formattingVideoId: options.content.formattingVideoId,
    regeneratingSummaryVideoIds: options.content.regeneratingSummaryVideoIds,
    revertingContent: options.content.revertingContent,
    revertingVideoId: options.content.revertingVideoId,
    resettingVideo: options.content.resettingVideo,
    resettingVideoId: options.content.resettingVideoId,
    creatingHighlight: options.highlightController.creatingHighlight,
    creatingHighlightVideoId:
      options.highlightController.creatingHighlightVideoId,
    creatingVocabularyReplacement: options.vocabulary.creating,
    deletingHighlightId: options.highlightController.deletingHighlightId,
    canRevertTranscript: options.getCanRevertTranscript(),
    showRevertTranscriptAction: options.getHasUpdatedTranscript(),
    formattingNotice: options.content.formattingNotice,
    formattingNoticeVideoId: options.content.formattingNoticeVideoId,
    formattingNoticeTone: options.content.formattingNoticeTone,
    citationScrollText,
    canPersistHighlights: true,
  });

  const workspaceContentActions = $derived.by(() => ({
    onBack: options.pageState.openMobileBrowse,
    onSetMode: options.dataController.setMode,
    onStartEdit: options.content.startEdit,
    onCancelEdit: options.content.cancelEdit,
    onSaveEdit: options.content.saveEdit,
    onCleanFormatting: options.content.cleanFormatting,
    onRegenerateSummary: options.content.regenerateSummaryContent,
    onRevertTranscript: options.content.revertToOriginalTranscript,
    onResetVideo: options.content.resetVideoContent,
    onDraftChange: options.content.setDraft,
    onToggleAcknowledge: options.onToggleAcknowledge,
    onCreateHighlight: options.highlightController.saveSelectionHighlight,
    onCreateVocabularyReplacement: options.vocabulary.open,
    onDeleteHighlight: options.highlightController.deleteExistingHighlight,
    onShowChannels: options.pageState.openMobileBrowse,
    onShowVideos: options.pageState.openMobileBrowse,
    onCitationScrollConsumed,
  }));

  return {
    get workspaceContentSelection() {
      return workspaceContentSelection;
    },
    get workspaceContentState() {
      return workspaceContentState;
    },
    get workspaceContentActions() {
      return workspaceContentActions;
    },
  };
}
