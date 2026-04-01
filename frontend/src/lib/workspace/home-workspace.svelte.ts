import { goto } from "$app/navigation";
import { page } from "$app/state";
import { onMount, tick } from "svelte";

import { authState } from "$lib/auth-state.svelte";
import { getAuthStorageScopeKey, getScopedStorageKey } from "$lib/auth-storage";
import { savePreferences, updateAcknowledged } from "$lib/api";
import { resolveAiIndicatorPresentation } from "$lib/ai-status";
import { DOCS_URL } from "$lib/app-config";
import type {
  AiStatus,
  AddVideoResult,
  Channel,
  HighlightSource,
  SearchStatus,
  Video,
  VideoTypeFilter,
  VocabularyReplacement,
} from "$lib/types";
import { renderMarkdown } from "$lib/utils/markdown";
import { createAddSourceFeedbackController } from "$lib/workspace/add-source-feedback.svelte";
import { createAiStatusPoller } from "$lib/utils/ai-poller";
import { buildWorkspaceViewHref } from "$lib/view-url";
import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
import { track } from "$lib/analytics/tracker";
import {
  cloneSyncDepthState,
  cloneVideos,
  createChannelViewCache,
} from "$lib/channel-view-cache";
import {
  buildOptimisticAcknowledgeSidebarList,
  isStillSelectedAfterAcknowledgeSuccess,
  matchesAcknowledgedFilterVideo,
  resolveRevertedVideoForAcknowledge,
  resolveVideoForAcknowledgeToggle,
  selectionDroppedAfterAcknowledgeOptimistic,
} from "$lib/workspace/acknowledge-toggle";
import { resolveNextChannelSelection } from "$lib/workspace/route-helpers";
import { shouldRetryReadySummaryLoad } from "$lib/workspace/content";
import { createSidebarState } from "$lib/workspace/sidebar-state.svelte";
import { mobileBottomBar } from "$lib/mobile-navigation/mobileBottomBar";
import {
  type AcknowledgedFilter,
  type WorkspaceContentMode,
  isWorkspaceContentMode,
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
import { createHomeWorkspacePersistenceController } from "$lib/workspace/home-workspace-persistence-controller.svelte";

export function createHomeWorkspacePage() {
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  const channelLastRefreshedAt = new Map<string, number>();
  const channelVideoStateCache =
    createChannelViewCache<CachedChannelVideoState>((state) => ({
      ...state,
      videos: cloneVideos(state.videos),
      syncDepth: cloneSyncDepthState(state.syncDepth),
    }));

  let aiAvailable = $state<boolean | null>(null);
  let aiStatus = $state<AiStatus | null>(null);
  let searchStatus = $state<SearchStatus | null>(null);
  let vocabularyReplacements = $state<VocabularyReplacement[]>([]);

  let errorMessage = $state<string | null>(null);
  let showDeleteAccessPrompt = $state(false);
  let showResetVideoConfirmation = $state(false);
  let allowLoadedVideoSyncDepthOverride = $state(false);
  let mobileViewportMq = $state(false);
  let mobileBrowseOpen = $state(true);
  let pendingSelectedVideo = $state<Video | null>(null);

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
    onSelectVideo: (videoId: string, context?: { forceReload?: boolean }) =>
      dataController.selectVideo(videoId, true, context?.forceReload ?? false),
    onChannelSelected: (channelId: string) => {
      if (!sidebarState.selectedVideoId) {
        content.resetSummaryQuality();
        content.videoInfo = null;
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
        sidebarState.setSelectedChannelId(null);
        sidebarState.setSelectedVideoId(null);
        sidebarState.setVideos([]);
        sidebarState.setSyncDepth(null);
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
      await goto(`/channels/${encodeURIComponent(channelId)}`);
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
    getReplacements: () => vocabularyReplacements,
    setReplacements: (replacements) => {
      vocabularyReplacements = replacements;
    },
    onError: (message) => {
      errorMessage = message;
    },
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
    onError: (message) => {
      errorMessage = message;
    },
  });

  const dataController = createHomeWorkspaceDataController({
    sidebarState,
    content,
    channelLastRefreshedAt,
    channelVideoStateCache,
    getAllowLoadedVideoSyncDepthOverride: () =>
      allowLoadedVideoSyncDepthOverride,
    setAllowLoadedVideoSyncDepthOverride: (value) => {
      allowLoadedVideoSyncDepthOverride = value;
    },
    getPendingSelectedVideo: () => pendingSelectedVideo,
    setPendingSelectedVideo: (value) => {
      pendingSelectedVideo = value;
    },
    getErrorMessage: () => errorMessage,
    setErrorMessage: (value) => {
      errorMessage = value;
    },
    getMobileBrowseOpen: () => mobileBrowseOpen,
    setMobileBrowseOpen: (value) => {
      mobileBrowseOpen = value;
    },
    getMobileViewportMq: () => mobileViewportMq,
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
    getMobileViewportMq: () => mobileViewportMq,
    setMobileViewportMq: (value) => {
      mobileViewportMq = value;
    },
    getMobileBrowseOpen: () => mobileBrowseOpen,
    setMobileBrowseOpen: (value) => {
      mobileBrowseOpen = value;
    },
    getAiAvailable: () => aiAvailable,
    setAiAvailable: (value) => {
      aiAvailable = value;
    },
    getAiStatus: () => aiStatus,
    setAiStatus: (value) => {
      aiStatus = value;
    },
    getSearchStatus: () => searchStatus,
    setSearchStatus: (value) => {
      searchStatus = value;
    },
    getVocabularyReplacements: () => vocabularyReplacements,
    setVocabularyReplacements: (value) => {
      vocabularyReplacements = value;
    },
    buildWorkspaceSnapshotCacheKey:
      dataController.buildWorkspaceSnapshotCacheKey,
    restoreGuideFromUrl: () => {
      tour.restoreFromUrl();
    },
    applyChannelSnapshot: dataController.applyChannelSnapshot,
    loadBootstrapRefresh: dataController.loadBootstrapRefresh,
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

  let videoAcknowledgeSeq = 0;
  let videoAcknowledgeSync = $state<{
    seq: number;
    video: Video;
    confirmed: boolean;
  } | null>(null);

  const selectedChannelId = $derived(sidebarState.selectedChannelId);
  const selectedChannel = $derived(sidebarState.selectedChannel);
  const videos = $derived(sidebarState.videos);
  const selectedVideoId = $derived(sidebarState.selectedVideoId);
  const selectedVideo = $derived(
    videos.find((video) => video.id === selectedVideoId) ??
      (pendingSelectedVideo?.id === selectedVideoId
        ? pendingSelectedVideo
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
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  const contentHtml = $derived(renderMarkdown(content.contentText));

  const tour = createGuideState(10);
  const tourSteps = createHomeTourSteps({
    get mobileBrowseOpen() {
      return mobileBrowseOpen;
    },
    set mobileBrowseOpen(value) {
      mobileBrowseOpen = value;
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
      showDeleteAccessPrompt = true;
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
        mobileBrowseOpen = true;
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
    if (mobileBrowseOpen) {
      mobileBottomBar.set({ kind: "hidden" });
      return () => {
        mobileBottomBar.set({ kind: "sections" });
      };
    }

    const inVideoDetail =
      !mobileBrowseOpen && Boolean(selectedVideoId) && !editing;
    if (!inVideoDetail) {
      mobileBottomBar.set({ kind: "sections" });
    } else {
      mobileBottomBar.set({
        kind: "videoActions",
        youtubeUrl: selectedVideoYoutubeUrl,
        showRegenerate: contentMode === "summary",
        regenerating: selectedVideoId
          ? content.regeneratingSummaryVideoIds.includes(selectedVideoId)
          : false,
        aiAvailable: aiAvailable ?? false,
        onRegenerate: content.regenerateSummaryContent,
        showFormatAction: contentMode === "transcript",
        formatting:
          content.formattingContent &&
          content.formattingVideoId === selectedVideoId,
        onFormat: content.cleanFormatting,
        showRevertAction: hasUpdatedTranscript,
        reverting:
          content.revertingContent &&
          content.revertingVideoId === selectedVideoId,
        canRevert: canRevertTranscript,
        onRevert: content.revertToOriginalTranscript,
        busy: loadingContent,
        onRequestResetVideo: () => {
          showResetVideoConfirmation = true;
        },
        resetting:
          content.resettingVideo &&
          content.resettingVideoId === selectedVideoId,
        showAcknowledgeToggle: true,
        acknowledged: selectedVideo?.acknowledged ?? false,
        onAcknowledgeToggle: toggleAcknowledge,
        showEditAction:
          contentMode === "transcript" || contentMode === "summary",
        onEdit: content.startEdit,
      });
    }
    return () => {
      mobileBottomBar.set({ kind: "sections" });
    };
  });

  $effect(() => {
    if (
      !mobileViewportMq ||
      !mobileBrowseOpen ||
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

  async function toggleAcknowledge() {
    if (!sidebarState.selectedVideoId) return;
    const targetVideoId = sidebarState.selectedVideoId;
    const resolved = resolveVideoForAcknowledgeToggle(
      sidebarState.videos,
      targetVideoId,
      pendingSelectedVideo,
    );
    if (!resolved) return;
    const { video, videoFromList } = resolved;

    errorMessage = null;

    const previousVideos = [...sidebarState.videos];
    const previousPendingSelectedVideo = pendingSelectedVideo;
    const previousSelectedVideoId = sidebarState.selectedVideoId;
    const newAcknowledged = !video.acknowledged;

    sidebarState.bumpVideoListMutationEpoch();

    const optimisticVideo = { ...video, acknowledged: newAcknowledged };
    const optimisticList = buildOptimisticAcknowledgeSidebarList(
      videoFromList,
      previousVideos,
      sidebarState.videos,
      targetVideoId,
      newAcknowledged,
      sidebarState.acknowledgedFilter,
    );
    if (videoFromList) {
      sidebarState.setVideos(optimisticList);
    } else {
      pendingSelectedVideo = optimisticVideo;
    }
    videoAcknowledgeSeq += 1;
    videoAcknowledgeSync = {
      seq: videoAcknowledgeSeq,
      video: optimisticVideo,
      confirmed: false,
    };

    const selectionDroppedFromFilter =
      selectionDroppedAfterAcknowledgeOptimistic(
        videoFromList,
        optimisticList,
        previousSelectedVideoId,
        optimisticVideo,
        sidebarState.acknowledgedFilter,
      );
    if (selectionDroppedFromFilter) {
      content.editing = false;
      content.clearFormattingFeedback();
      if (videoFromList) {
        if (optimisticList.length === 0) {
          sidebarState.setSelectedVideoId(null);
          content.contentText = "";
          content.draft = "";
        } else {
          await dataController.selectVideo(optimisticList[0].id);
        }
      } else {
        sidebarState.setSelectedVideoId(null);
        pendingSelectedVideo = null;
        content.contentText = "";
        content.draft = "";
      }
    }

    try {
      const updated = await updateAcknowledged(targetVideoId, newAcknowledged);
      if (videoFromList) {
        sidebarState.setVideos(
          sidebarState.videos
            .map((candidate) =>
              candidate.id === updated.id ? updated : candidate,
            )
            .filter((candidate) =>
              matchesAcknowledgedFilterVideo(
                candidate,
                sidebarState.acknowledgedFilter,
              ),
            ),
        );
      } else if (!selectionDroppedFromFilter) {
        pendingSelectedVideo = updated;
      }
      if (selectedChannelId) {
        track({
          event: "video_acknowledged_changed",
          video_id: targetVideoId,
          channel_id: selectedChannelId,
          acknowledged: newAcknowledged,
        });
      }

      videoAcknowledgeSeq += 1;
      videoAcknowledgeSync = {
        seq: videoAcknowledgeSeq,
        video: updated,
        confirmed: true,
      };

      const stillSelected = isStillSelectedAfterAcknowledgeSuccess(
        sidebarState.selectedVideoId,
        sidebarState.videos,
        pendingSelectedVideo,
      );
      if (!stillSelected) {
        content.editing = false;
        content.clearFormattingFeedback();
        if (sidebarState.videos.length === 0) {
          sidebarState.setSelectedVideoId(null);
          content.contentText = "";
          content.draft = "";
        } else {
          await dataController.selectVideo(sidebarState.videos[0].id);
        }
      }
    } catch (error) {
      sidebarState.setVideos(previousVideos);
      sidebarState.setSelectedVideoId(previousSelectedVideoId);
      pendingSelectedVideo = previousPendingSelectedVideo;
      const reverted = resolveRevertedVideoForAcknowledge(
        previousVideos,
        targetVideoId,
        previousPendingSelectedVideo,
      );
      if (reverted) {
        videoAcknowledgeSeq += 1;
        videoAcknowledgeSync = {
          seq: videoAcknowledgeSeq,
          video: reverted,
          confirmed: true,
        };
      }
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    }
  }

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
      if (!content.contentText.trim()) {
        content.cacheLoadedSummary(summary, targetVideoId);
        content.draft = content.contentText;
        content.videoInfo = null;
        if (
          highlightController.videoHighlightsByVideoId[targetVideoId] ===
          undefined
        ) {
          void highlightController.hydrateVideoHighlights(targetVideoId);
        }
      }
      content.applySummaryQuality(summary);
    } catch {
      // Keep previous quality state if background refresh fails.
    }
  }

  $effect(() =>
    createAiStatusPoller({
      onStatus: (status) => {
        aiAvailable = status.available;
        aiStatus = status.status;
      },
    }),
  );

  $effect(() => {
    if (
      contentMode !== "summary" ||
      !selectedVideoId ||
      editing ||
      loadingContent ||
      content.summaryQualityScore !== null ||
      content.summaryQualityNote !== null
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

  const workspaceContentSelection = $derived({
    mobileVisible: true,
    mobileBackInTopBar: !mobileBrowseOpen && Boolean(selectedVideoId),
    selectedChannel,
    selectedVideo,
    selectedVideoId,
    contentMode,
  });

  const citationScrollText = $derived.by(() => {
    const url = page.url;
    const cite = url.searchParams.get("cite")?.trim();
    if (!cite || loadingContent) {
      return null;
    }
    const videoParam = url.searchParams.get("video")?.trim();
    if (videoParam && selectedVideoId && videoParam !== selectedVideoId) {
      return null;
    }
    return cite;
  });

  function onCitationScrollConsumed() {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- transient URL for one-shot citation navigation cleanup
    const url = new URL(page.url.href);
    if (!url.searchParams.has("cite") && !url.searchParams.has("chunk")) {
      return;
    }
    url.searchParams.delete("cite");
    url.searchParams.delete("chunk");
    persistenceController.replaceWorkspaceUrl(
      `${url.pathname}${url.search}${url.hash}`,
    );
  }

  const workspaceContentState = $derived({
    loadingContent,
    editing,
    aiAvailable: aiAvailable ?? false,
    summaryQualityScore: content.summaryQualityScore,
    summaryQualityNote: content.summaryQualityNote,
    summaryModelUsed: content.summaryModelUsed,
    summaryQualityModelUsed: content.summaryQualityModelUsed,
    videoInfo,
    contentHtml,
    contentText,
    transcriptRenderMode,
    contentHighlights,
    selectedVideoHighlights,
    selectedVideoYoutubeUrl,
    draft: content.draft,
    formattingContent: content.formattingContent,
    formattingVideoId: content.formattingVideoId,
    regeneratingSummaryVideoIds: content.regeneratingSummaryVideoIds,
    revertingContent: content.revertingContent,
    revertingVideoId: content.revertingVideoId,
    resettingVideo: content.resettingVideo,
    resettingVideoId: content.resettingVideoId,
    creatingHighlight: highlightController.creatingHighlight,
    creatingHighlightVideoId: highlightController.creatingHighlightVideoId,
    creatingVocabularyReplacement: vocabulary.creating,
    deletingHighlightId: highlightController.deletingHighlightId,
    canRevertTranscript,
    showRevertTranscriptAction: hasUpdatedTranscript,
    formattingNotice: content.formattingNotice,
    formattingNoticeVideoId: content.formattingNoticeVideoId,
    formattingNoticeTone: content.formattingNoticeTone,
    citationScrollText,
    canPersistHighlights: true,
  });
  const workspaceContentActions = $derived.by(() => ({
    onBack: () => {
      mobileBrowseOpen = true;
    },
    onSetMode: dataController.setMode,
    onStartEdit: content.startEdit,
    onCancelEdit: content.cancelEdit,
    onSaveEdit: content.saveEdit,
    onCleanFormatting: content.cleanFormatting,
    onRegenerateSummary: content.regenerateSummaryContent,
    onRevertTranscript: content.revertToOriginalTranscript,
    onResetVideo: content.resetVideoContent,
    onDraftChange: (value: string) => {
      content.draft = value;
    },
    onToggleAcknowledge: toggleAcknowledge,
    onCreateHighlight: highlightController.saveSelectionHighlight,
    onCreateVocabularyReplacement: vocabulary.open,
    onDeleteHighlight: highlightController.deleteExistingHighlight,
    onShowChannels: () => {
      mobileBrowseOpen = true;
    },
    onShowVideos: () => {
      mobileBrowseOpen = true;
    },
    onCitationScrollConsumed,
  }));
  const workspaceOverlaysState = $derived({
    errorMessage,
    showDeleteConfirmation: sidebarState.showDeleteConfirmation,
    showDeleteAccessPrompt,
    showAddSourceFeedback:
      !!addSourceFeedbackCtrl.feedback && !addSourceFeedbackCtrl.dismissed,
    showResetVideoConfirmation,
  });
  const workspaceOverlaysActions = {
    onDismissError: () => {
      errorMessage = null;
    },
    onConfirmDelete: dataController.confirmDeleteChannel,
    onCancelDelete: () => sidebarState.setShowDeleteConfirmation(false),
    onConfirmAccessPrompt: async () => {
      showDeleteAccessPrompt = false;
      const redirectTo = `${page.url.pathname}${page.url.search}`;
      await goto(`/login?redirectTo=${encodeURIComponent(redirectTo)}`);
    },
    onCancelAccessPrompt: () => {
      showDeleteAccessPrompt = false;
    },
    onConfirmResetVideo: async () => {
      showResetVideoConfirmation = false;
      await content.resetVideoContent();
    },
    onCancelResetVideo: () => {
      showResetVideoConfirmation = false;
    },
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
      return errorMessage;
    },
    set errorMessage(value) {
      errorMessage = value;
    },
    get videoAcknowledgeSync() {
      return videoAcknowledgeSync;
    },
    handleChannelSyncDateSaved: dataController.handleChannelSyncDateSaved,
    handleDeleteChannel,
    get showDeleteAccessPrompt() {
      return showDeleteAccessPrompt;
    },
    set showDeleteAccessPrompt(value) {
      showDeleteAccessPrompt = value;
    },
    get mobileBrowseOpen() {
      return mobileBrowseOpen;
    },
    set mobileBrowseOpen(value) {
      mobileBrowseOpen = value;
    },
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
      return aiAvailable;
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
    content,
    get showResetVideoConfirmation() {
      return showResetVideoConfirmation;
    },
    set showResetVideoConfirmation(value) {
      showResetVideoConfirmation = value;
    },
    toggleAcknowledge,
    get WorkspaceSearchBarComponent() {
      return persistenceController.WorkspaceSearchBarComponent;
    },
    get searchStatus() {
      return searchStatus;
    },
    handleSearchResultSelection: dataController.handleSearchResultSelection,
    loadMoreVideos: dataController.loadMoreVideos,
    get canManageLibrary() {
      return canManageLibrary;
    },
    get workspaceContentSelection() {
      return workspaceContentSelection;
    },
    get workspaceContentState() {
      return workspaceContentState;
    },
    get workspaceContentActions() {
      return workspaceContentActions;
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
    set vocabularyModalValue(value) {
      vocabulary.modalValue = value;
    },
    get creatingVocabularyReplacement() {
      return vocabulary.creating;
    },
    confirmVocabularyReplacement: () => vocabulary.confirm(),
    closeVocabularyModal: () => vocabulary.close(),
  };
}
