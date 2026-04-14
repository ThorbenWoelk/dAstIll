import { goto } from "$app/navigation";
import { page } from "$app/state";
import { onMount, tick } from "svelte";

import { authState } from "$lib/auth-state.svelte";
import { getAuthStorageScopeKey, getScopedStorageKey } from "$lib/auth-storage";
import { savePreferences } from "$lib/api";
import { resolveAiIndicatorPresentation } from "$lib/ai-status";
import { DOCS_URL } from "$lib/app-config";
import type {
  AddVideoResult,
  Channel,
  HighlightSource,
  VideoTypeFilter,
} from "$lib/types";
import { renderMarkdown } from "$lib/utils/markdown";
import { createAddSourceFeedbackController } from "$lib/workspace/add-source-feedback.svelte";
import { createAiStatusPoller } from "$lib/utils/ai-poller";
import { buildWorkspaceViewHref } from "$lib/view-url";
import {
  cloneSyncDepthState,
  cloneVideos,
  createChannelViewCache,
} from "$lib/channel-view-cache";
import { loadWorkspaceState } from "$lib/channel-workspace";
import { resolveNextChannelSelection } from "$lib/workspace/route-helpers";
import {
  hasCompleteSummaryEvaluation,
  shouldRetryReadySummaryLoad,
} from "$lib/workspace/content";
import { createSidebarState } from "$lib/workspace/sidebar-state.svelte";
import {
  type AcknowledgedFilter,
  type WorkspaceContentMode,
  isAcknowledgedFilter,
  isWorkspaceContentMode,
  isWorkspaceVideoTypeFilter,
} from "$lib/workspace/types";
import { createGuideState } from "$lib/workspace/guide-state.svelte";
import { createHomeTourSteps } from "$lib/workspace/home-tour";
import { createContentState } from "$lib/workspace/content-state.svelte";
import { DASTILL_SET_WORKSPACE_CONTENT_MODE_EVENT } from "$lib/utils/keyboard-shortcuts";
import { createVocabularyController } from "$lib/workspace/vocabulary-controller.svelte";
import { createHomeWorkspaceHighlightController } from "$lib/workspace/home-workspace-highlight-controller.svelte";
import {
  createHomeWorkspaceDataController,
  type CachedChannelVideoState,
} from "$lib/workspace/home-workspace-data-controller.svelte";
import { clearWorkspaceForScopeChange } from "$lib/workspace/home-workspace-auth-scope";
import { createHomeWorkspaceAcknowledgeController } from "$lib/workspace/home-workspace-acknowledge-controller.svelte";
import { createHomeWorkspacePageState } from "$lib/workspace/home-workspace-page-state.svelte";
import { createHomeWorkspacePersistenceController } from "$lib/workspace/home-workspace-persistence-controller.svelte";
import { createHomeWorkspaceViewModel } from "$lib/workspace/home-workspace-view-model.svelte";

export function createHomeWorkspacePage() {
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  const channelLastRefreshedAt = new Map<string, number>();
  const channelVideoStateCache =
    createChannelViewCache<CachedChannelVideoState>((state) => ({
      ...state,
      videos: cloneVideos(state.videos),
      syncDepth: cloneSyncDepthState(state.syncDepth),
    }));
  const pageState = createHomeWorkspacePageState();

  const addSourceFeedbackCtrl = createAddSourceFeedbackController();
  const workspaceStorageKey = $derived(
    getScopedStorageKey(
      "dastill.workspace.state.v1",
      getAuthStorageScopeKey(authState.current),
    ),
  );
  const workspaceCacheScopeKey = $derived(
    getAuthStorageScopeKey(authState.current),
  );

  const sidebarState = createSidebarState({
    initialChannelId: page.data.selectedChannelId,
    initialVideoId: page.data.selectedVideoId,
    initialVideoTypeFilter: page.data.videoTypeFilter ?? "all",
    initialAcknowledgedFilter: page.data.acknowledgedFilter ?? "all",
    getViewCacheScopeKey: () => workspaceCacheScopeKey,
    onSelectVideo: (videoId: string, context?: { forceReload?: boolean }) =>
      dataController.selectVideo(videoId, true, context?.forceReload ?? false),
    onChannelSelected: (channelId: string) => {
      if (!sidebarState.selectedVideoId) {
        content.clearSelectionMetadata();
      }
      const href = buildWorkspaceViewHref({
        selectedChannelId: channelId,
        selectedVideoId: sidebarState.selectedVideoId,
        contentMode,
        videoTypeFilter: sidebarState.videoTypeFilter,
        acknowledgedFilter: sidebarState.acknowledgedFilter,
      });
      persistenceController.replaceWorkspaceUrl(href);
    },
    onChannelDeleted: (channelId: string) => {
      if (sidebarState.selectedChannelId !== channelId) {
        return;
      }
      const nextChannelId = resolveNextChannelSelection(
        sidebarState.channels,
        channelId,
      );
      if (nextChannelId) {
        void sidebarState.selectChannel(nextChannelId);
      } else {
        sidebarState.clearChannelSelectionState();
        dataController.clearSelectedVideoState();
      }
    },
    onVideoTypeFilterChange: (value: VideoTypeFilter) => {
      const href = buildWorkspaceViewHref({
        selectedChannelId: sidebarState.selectedChannelId,
        selectedVideoId,
        contentMode,
        videoTypeFilter: value,
        acknowledgedFilter: sidebarState.acknowledgedFilter,
      });
      persistenceController.replaceWorkspaceUrl(href);
    },
    onAcknowledgedFilterChange: (value: AcknowledgedFilter) => {
      const href = buildWorkspaceViewHref({
        selectedChannelId: sidebarState.selectedChannelId,
        selectedVideoId,
        contentMode,
        videoTypeFilter: sidebarState.videoTypeFilter,
        acknowledgedFilter: value,
      });
      persistenceController.replaceWorkspaceUrl(href);
    },
    onOpenChannelOverview: async (channelId: string) => {
      await sidebarState.selectChannel(channelId);
    },
    onChannelAdded: (channel: Channel) => {
      void addSourceFeedbackCtrl.trackAddedChannel(channel);
    },
    onVideoAdded: (result: AddVideoResult) => {
      void addSourceFeedbackCtrl.trackAddedVideo(result);
    },
    onVideoListReset: () => {
      // Managed inside sidebar state.
    },
  });

  const content = createContentState({
    getSelectedVideoId: () => sidebarState.selectedVideoId,
    getSelectedChannelId: () => sidebarState.selectedChannelId,
    setVideoStatus: (videoId, transcriptStatus, summaryStatus) => {
      sidebarState.setVideoStatus(videoId, transcriptStatus, summaryStatus);
    },
    initialContentMode: page.data.contentMode ?? undefined,
  });

  const vocabulary = createVocabularyController({
    getReplacements: () => pageState.vocabularyReplacements,
    setReplacements: pageState.setVocabularyReplacements,
    onError: pageState.setErrorMessage,
    onSave: async (replacements) => {
      await savePreferences({
        channel_order: sidebarState.channelOrder,
        channel_sort_mode: sidebarState.channelSortMode,
        vocabulary_replacements: replacements,
      });
    },
  });

  const highlightController = createHomeWorkspaceHighlightController({
    getSelectedVideoId: () => sidebarState.selectedVideoId,
    getSelectedChannelId: () => sidebarState.selectedChannelId,
    getContentMode: () => content.contentMode,
    getCanManageLibrary: () => authState.current.authState === "authenticated",
    onError: pageState.setErrorMessage,
  });

  const dataController = createHomeWorkspaceDataController({
    sidebarState,
    content,
    channelLastRefreshedAt,
    channelVideoStateCache,
    getAllowLoadedVideoSyncDepthOverride: () =>
      pageState.allowLoadedVideoSyncDepthOverride,
    setAllowLoadedVideoSyncDepthOverride:
      pageState.setAllowLoadedVideoSyncDepthOverride,
    getPendingSelectedVideo: () => pageState.pendingSelectedVideo,
    setPendingSelectedVideo: pageState.setPendingSelectedVideo,
    getErrorMessage: () => pageState.errorMessage,
    setErrorMessage: pageState.setErrorMessage,
    getMobileBrowseOpen: () => pageState.mobileBrowseOpen,
    setMobileBrowseOpen: pageState.setMobileBrowseOpen,
    getMobileViewportMq: () => pageState.mobileViewportMq,
    getWorkspaceCacheScopeKey: () => workspaceCacheScopeKey,
    getVideoHighlightsByVideoId: () =>
      highlightController.videoHighlightsByVideoId,
    hydrateVideoHighlights: highlightController.hydrateVideoHighlights,
  });

  const persistenceController = createHomeWorkspacePersistenceController({
    sidebarState,
    content,
    getWorkspaceStorageKey: () => workspaceStorageKey,
    getWorkspaceCacheScopeKey: () => workspaceCacheScopeKey,
    getMobileViewportMq: () => pageState.mobileViewportMq,
    setMobileViewportMq: pageState.setMobileViewportMq,
    getMobileBrowseOpen: () => pageState.mobileBrowseOpen,
    setMobileBrowseOpen: pageState.setMobileBrowseOpen,
    getAiAvailable: () => pageState.aiAvailable,
    setAiAvailable: pageState.setAiAvailable,
    getAiStatus: () => pageState.aiStatus,
    setAiStatus: pageState.setAiStatus,
    getSearchStatus: () => pageState.searchStatus,
    setSearchStatus: pageState.setSearchStatus,
    getVocabularyReplacements: () => pageState.vocabularyReplacements,
    setVocabularyReplacements: pageState.setVocabularyReplacements,
    buildWorkspaceSnapshotCacheKey:
      dataController.buildWorkspaceSnapshotCacheKey,
    restoreGuideFromUrl: () => {
      tour.restoreFromUrl();
    },
    applyChannelSnapshot: dataController.applyChannelSnapshot,
    loadBootstrapRefresh: dataController.loadBootstrapRefresh,
  });

  const acknowledgeController = createHomeWorkspaceAcknowledgeController({
    sidebarState,
    content,
    getPendingSelectedVideo: () => pageState.pendingSelectedVideo,
    setPendingSelectedVideo: pageState.setPendingSelectedVideo,
    setErrorMessage: pageState.setErrorMessage,
    getSelectedChannelId: () => selectedChannelId,
    selectVideo: (videoId) => dataController.selectVideo(videoId),
    setVideoAcknowledgeSync: pageState.setVideoAcknowledgeSync,
  });

  const contentMode = $derived(content.contentMode);
  const loadingContent = $derived(content.loadingContent);
  const editing = $derived(content.editing);
  const contentText = $derived(content.contentText);
  const transcriptRenderMode = $derived(content.transcriptRenderMode);
  const videoInfo = $derived(content.videoInfo);
  const formattingContent = $derived(content.formattingContent);
  const formattingVideoId = $derived(content.formattingVideoId);
  const regeneratingSummaryVideoIds = $derived(
    content.regeneratingSummaryVideoIds,
  );
  const revertingContent = $derived(content.revertingContent);
  const revertingVideoId = $derived(content.revertingVideoId);
  const resettingVideo = $derived(content.resettingVideo);
  const resettingVideoId = $derived(content.resettingVideoId);
  const draft = $derived(content.draft);

  const selectedChannelId = $derived(sidebarState.selectedChannelId);
  const selectedChannel = $derived(sidebarState.selectedChannel);
  const videos = $derived(sidebarState.videos);
  const selectedVideoId = $derived(sidebarState.selectedVideoId);
  const selectedVideo = $derived(
    videos.find((video) => video.id === selectedVideoId) ??
      (pageState.pendingSelectedVideo?.id === selectedVideoId
        ? pageState.pendingSelectedVideo
        : null),
  );
  const selectedVideoHighlights = $derived(
    selectedVideoId
      ? (highlightController.videoHighlightsByVideoId[selectedVideoId] ?? [])
      : [],
  );
  const contentHighlights = $derived(
    contentMode === "transcript" || contentMode === "summary"
      ? selectedVideoHighlights.filter(
          (highlight) => highlight.source === (contentMode as HighlightSource),
        )
      : [],
  );
  const selectedVideoYoutubeUrl = $derived(
    selectedVideoId
      ? `https://www.youtube.com/watch?v=${selectedVideoId}`
      : null,
  );
  const selectedOriginalTranscript = $derived(
    selectedVideoId
      ? (content.originalTranscriptByVideoId[selectedVideoId] ?? null)
      : null,
  );
  const hasUpdatedTranscript = $derived(
    contentMode === "transcript" &&
      selectedOriginalTranscript !== null &&
      content.contentText !== selectedOriginalTranscript,
  );
  const canRevertTranscript = $derived(
    contentMode === "transcript" &&
      selectedOriginalTranscript !== null &&
      (editing
        ? content.draft !== selectedOriginalTranscript
        : content.contentText !== selectedOriginalTranscript),
  );

  const canManageLibrary = $derived(
    authState.current.authState === "authenticated",
  );
  const aiIndicator = $derived(
    pageState.aiStatus
      ? resolveAiIndicatorPresentation(pageState.aiStatus)
      : null,
  );
  const contentHtml = $derived(renderMarkdown(content.contentText));

  const tour = createGuideState(10);
  const tourSteps = createHomeTourSteps({
    get mobileBrowseOpen() {
      return pageState.mobileBrowseOpen;
    },
    set mobileBrowseOpen(value) {
      pageState.setMobileBrowseOpen(value);
    },
    get selectedVideoId() {
      return sidebarState.selectedVideoId;
    },
    get selectedChannelId() {
      return sidebarState.selectedChannelId;
    },
    get videos() {
      return sidebarState.videos;
    },
    get contentMode() {
      return content.contentMode;
    },
    isAuthenticated: () => authState.current.authState === "authenticated",
    selectVideo: (id, fromUserInteraction, forceReload) =>
      dataController.selectVideo(id, fromUserInteraction, forceReload),
    setMode: (mode) => {
      void dataController.setMode(mode as WorkspaceContentMode);
    },
    tick,
  });
  const guideOpen = $derived(tour.isOpen);
  const guideStep = $derived(tour.step);

  function openGuide() {
    tour.open();
  }

  function closeGuide() {
    tour.close();
  }

  function setGuideStep(step: number) {
    tour.setStep(step);
  }

  async function handleDeleteChannel(channelId: string) {
    const result = await dataController.handleDeleteChannel(channelId);
    if (result === "auth_required") {
      pageState.openDeleteAccessPrompt();
    }
  }

  onMount(() => {
    const handler = (event: Event) => {
      const detail = (event as CustomEvent<{ mode?: unknown }>).detail;
      if (!isWorkspaceContentMode(detail?.mode)) {
        return;
      }
      void dataController.setMode(detail.mode);
    };
    window.addEventListener(DASTILL_SET_WORKSPACE_CONTENT_MODE_EVENT, handler);
    return () =>
      window.removeEventListener(
        DASTILL_SET_WORKSPACE_CONTENT_MODE_EVENT,
        handler,
      );
  });

  async function openAddSourceFeedbackTarget() {
    await addSourceFeedbackCtrl.openTarget({
      onOpenVideo: async (videoId, channelId) => {
        await sidebarState.selectChannel(channelId, videoId, true);
        await dataController.selectVideo(videoId, true, true);
      },
      onOpenChannel: async (channelId) => {
        pageState.openMobileBrowse();
        await sidebarState.selectChannel(channelId, null, true);
      },
    });
  }

  onMount(() => {
    return () => {
      addSourceFeedbackCtrl.cancelPolling();
    };
  });

  $effect(() => {
    if (typeof window === "undefined") {
      return;
    }

    if (pageState.hydratedWorkspaceScopeKey === null) {
      pageState.setHydratedWorkspaceScopeKey(workspaceCacheScopeKey);
      return;
    }

    if (pageState.hydratedWorkspaceScopeKey === workspaceCacheScopeKey) {
      return;
    }

    pageState.setHydratedWorkspaceScopeKey(workspaceCacheScopeKey);
    clearWorkspaceForScopeChange(sidebarState);
    // Restore filter preferences saved under the incoming auth scope so that
    // the subsequent loadBootstrapRefresh uses the correct filter values.
    if (typeof localStorage !== "undefined") {
      const saved = loadWorkspaceState(localStorage, workspaceStorageKey);
      if (
        saved?.acknowledgedFilter &&
        isAcknowledgedFilter(saved.acknowledgedFilter)
      ) {
        sidebarState.setAcknowledgedFilter(saved.acknowledgedFilter);
      }
      if (
        saved?.videoTypeFilter &&
        isWorkspaceVideoTypeFilter(saved.videoTypeFilter)
      ) {
        sidebarState.setVideoTypeFilter(saved.videoTypeFilter);
      }
    }
    void dataController.loadBootstrapRefresh();
  });

  $effect(() => {
    if (
      !pageState.mobileViewportMq ||
      !pageState.mobileBrowseOpen ||
      !sidebarState.selectedChannelId
    ) {
      return;
    }

    let cancelled = false;
    void (async () => {
      await tick();
      await dataController.loadAllVideosForMobileBrowse(() => cancelled);
    })();

    return () => {
      cancelled = true;
    };
  });

  async function refreshSummaryQuality() {
    if (
      !selectedVideoId ||
      contentMode !== "summary" ||
      editing ||
      loadingContent
    ) {
      return;
    }
    const targetVideoId = selectedVideoId;
    try {
      const { getSummary } = await import("$lib/api");
      const summary = await getSummary(targetVideoId);
      if (
        selectedVideoId !== targetVideoId ||
        contentMode !== "summary" ||
        editing
      ) {
        return;
      }
      const hadEmptyContent = !content.contentText.trim();
      content.applyBackgroundSummaryRefresh(summary, targetVideoId);
      if (
        hadEmptyContent &&
        highlightController.videoHighlightsByVideoId[targetVideoId] ===
          undefined
      ) {
        void highlightController.hydrateVideoHighlights(targetVideoId);
      }
    } catch {
      // Keep previous quality state if background refresh fails.
    }
  }

  $effect(() =>
    createAiStatusPoller({
      onStatus: (status) => {
        pageState.setAiAvailable(status.available);
        pageState.setAiStatus(status.status);
      },
    }),
  );

  $effect(() => {
    if (
      contentMode !== "summary" ||
      !selectedVideoId ||
      editing ||
      loadingContent ||
      hasCompleteSummaryEvaluation({
        score: content.summaryQualityScore,
        note: content.summaryQualityNote,
        tagsEvaluated: content.summaryTagsEvaluated,
      })
    ) {
      return;
    }

    const needsReadySummaryRetry = shouldRetryReadySummaryLoad({
      contentMode,
      selectedVideo,
      contentText: content.contentText,
      loadingContent,
      editing,
    });
    const intervalMs = needsReadySummaryRetry ? 2000 : 7000;
    if (needsReadySummaryRetry) {
      void refreshSummaryQuality();
    }
    const timer = setInterval(() => {
      void refreshSummaryQuality();
    }, intervalMs);
    return () => clearInterval(timer);
  });

  const browseFilterDisabled = $derived(
    !sidebarState.selectedChannelId || sidebarState.videoState.loadingVideos,
  );

  const viewModel = createHomeWorkspaceViewModel({
    page,
    replaceWorkspaceUrl: persistenceController.replaceWorkspaceUrl,
    pageState,
    content,
    highlightController,
    vocabulary,
    dataController,
    getLoadingContent: () => loadingContent,
    getEditing: () => editing,
    getContentText: () => contentText,
    getContentMode: () => contentMode,
    getSelectedChannel: () => selectedChannel,
    getSelectedVideo: () => selectedVideo,
    getSelectedVideoId: () => selectedVideoId,
    getSelectedVideoYoutubeUrl: () => selectedVideoYoutubeUrl,
    getSelectedVideoHighlights: () => selectedVideoHighlights,
    getContentHighlights: () => contentHighlights,
    getVideoInfo: () => videoInfo,
    getContentHtml: () => contentHtml,
    getTranscriptRenderMode: () => transcriptRenderMode,
    getCanRevertTranscript: () => canRevertTranscript,
    getHasUpdatedTranscript: () => hasUpdatedTranscript,
    onToggleAcknowledge: acknowledgeController.toggleAcknowledge,
  });
  const workspaceOverlaysState = $derived({
    errorMessage: pageState.errorMessage,
    showDeleteConfirmation: sidebarState.showDeleteConfirmation,
    showDeleteAccessPrompt: pageState.showDeleteAccessPrompt,
    showAddSourceFeedback:
      !!addSourceFeedbackCtrl.feedback && !addSourceFeedbackCtrl.dismissed,
    showResetVideoConfirmation: pageState.showResetVideoConfirmation,
  });
  const workspaceOverlaysActions = {
    onDismissError: pageState.clearErrorMessage,
    onConfirmDelete: dataController.confirmDeleteChannel,
    onCancelDelete: () => sidebarState.setShowDeleteConfirmation(false),
    onConfirmAccessPrompt: async () => {
      pageState.closeDeleteAccessPrompt();
      const redirectTo = `${page.url.pathname}${page.url.search}`;
      await goto(`/login?redirectTo=${encodeURIComponent(redirectTo)}`);
    },
    onCancelAccessPrompt: pageState.closeDeleteAccessPrompt,
    onConfirmResetVideo: async () => {
      pageState.closeResetVideoConfirmation();
      await content.resetVideoContent();
    },
    onCancelResetVideo: pageState.closeResetVideoConfirmation,
  };

  return {
    page,
    DOCS_URL,
    get aiIndicator() {
      return aiIndicator;
    },
    openGuide,
    closeGuide,
    setGuideStep,
    sidebarState,
    get errorMessage() {
      return pageState.errorMessage;
    },
    get videoAcknowledgeSync() {
      return pageState.videoAcknowledgeSync;
    },
    handleChannelSyncDateSaved: dataController.handleChannelSyncDateSaved,
    handleDeleteChannel,
    get showDeleteAccessPrompt() {
      return pageState.showDeleteAccessPrompt;
    },
    openDeleteAccessPrompt: pageState.openDeleteAccessPrompt,
    closeDeleteAccessPrompt: pageState.closeDeleteAccessPrompt,
    get mobileBrowseOpen() {
      return pageState.mobileBrowseOpen;
    },
    openMobileBrowse: pageState.openMobileBrowse,
    closeMobileBrowse: pageState.closeMobileBrowse,
    get selectedVideoId() {
      return selectedVideoId;
    },
    get browseFilterDisabled() {
      return browseFilterDisabled;
    },
    onBrowseVideoTypeFilterChange: dataController.onBrowseVideoTypeFilterChange,
    onBrowseAcknowledgedFilterChange:
      dataController.onBrowseAcknowledgedFilterChange,
    clearBrowseVideoFilters: dataController.clearBrowseVideoFilters,
    get contentMode() {
      return contentMode;
    },
    setMode: dataController.setMode,
    get loadingContent() {
      return loadingContent;
    },
    get editing() {
      return editing;
    },
    get hasUpdatedTranscript() {
      return hasUpdatedTranscript;
    },
    get formattingContent() {
      return formattingContent;
    },
    get formattingVideoId() {
      return formattingVideoId;
    },
    get regeneratingSummaryVideoIds() {
      return regeneratingSummaryVideoIds;
    },
    get revertingContent() {
      return revertingContent;
    },
    get revertingVideoId() {
      return revertingVideoId;
    },
    get resettingVideo() {
      return resettingVideo;
    },
    get resettingVideoId() {
      return resettingVideoId;
    },
    get aiAvailable() {
      return pageState.aiAvailable;
    },
    get canRevertTranscript() {
      return canRevertTranscript;
    },
    get selectedVideoYoutubeUrl() {
      return selectedVideoYoutubeUrl;
    },
    get draft() {
      return draft;
    },
    get selectedVideo() {
      return selectedVideo;
    },
    get showResetVideoConfirmation() {
      return pageState.showResetVideoConfirmation;
    },
    openResetVideoConfirmation: pageState.openResetVideoConfirmation,
    closeResetVideoConfirmation: pageState.closeResetVideoConfirmation,
    toggleAcknowledge: acknowledgeController.toggleAcknowledge,
    get WorkspaceSearchBarComponent() {
      return persistenceController.WorkspaceSearchBarComponent;
    },
    get searchStatus() {
      return pageState.searchStatus;
    },
    handleSearchResultSelection: dataController.handleSearchResultSelection,
    loadMoreVideos: dataController.loadMoreVideos,
    get canManageLibrary() {
      return canManageLibrary;
    },
    startEdit: content.startEdit,
    cancelEdit: content.cancelEdit,
    saveEdit: content.saveEdit,
    cleanFormatting: content.cleanFormatting,
    regenerateSummaryContent: content.regenerateSummaryContent,
    revertToOriginalTranscript: content.revertToOriginalTranscript,
    setDraft: content.setDraft,
    get workspaceContentSelection() {
      return viewModel.workspaceContentSelection;
    },
    get workspaceContentState() {
      return viewModel.workspaceContentState;
    },
    get workspaceContentActions() {
      return viewModel.workspaceContentActions;
    },
    get workspaceOverlaysState() {
      return workspaceOverlaysState;
    },
    workspaceOverlaysActions,
    get addSourceFeedback() {
      return addSourceFeedbackCtrl.feedback;
    },
    get addSourceFeedbackDismissed() {
      return addSourceFeedbackCtrl.dismissed;
    },
    dismissAddSourceFeedback: () => addSourceFeedbackCtrl.dismiss(),
    openAddSourceFeedbackTarget,
    get guideOpen() {
      return guideOpen;
    },
    get guideStep() {
      return guideStep;
    },
    tourSteps,
    get vocabularyModalSource() {
      return vocabulary.modalSource;
    },
    get vocabularyModalValue() {
      return vocabulary.modalValue;
    },
    setVocabularyModalValue: vocabulary.setModalValue,
    get creatingVocabularyReplacement() {
      return vocabulary.creating;
    },
    confirmVocabularyReplacement: () => vocabulary.confirm(),
    closeVocabularyModal: () => vocabulary.close(),
  };
}
