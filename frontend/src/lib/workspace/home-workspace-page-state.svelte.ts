import type { VocabularyReplacement } from "$lib/bindings/VocabularyReplacement";
import type { AiStatus, SearchStatus, Video } from "$lib/types";

export type VideoAcknowledgeSync = {
  seq: number;
  video: Video;
  confirmed: boolean;
} | null;

export function createHomeWorkspacePageState() {
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
  let hydratedWorkspaceScopeKey = $state<string | null>(null);
  let videoAcknowledgeSync = $state<VideoAcknowledgeSync>(null);

  return {
    get aiAvailable() {
      return aiAvailable;
    },
    setAiAvailable(value: boolean | null) {
      aiAvailable = value;
    },
    get aiStatus() {
      return aiStatus;
    },
    setAiStatus(value: AiStatus | null) {
      aiStatus = value;
    },
    get searchStatus() {
      return searchStatus;
    },
    setSearchStatus(value: SearchStatus | null) {
      searchStatus = value;
    },
    get vocabularyReplacements() {
      return vocabularyReplacements;
    },
    setVocabularyReplacements(value: VocabularyReplacement[]) {
      vocabularyReplacements = value;
    },
    get errorMessage() {
      return errorMessage;
    },
    setErrorMessage(value: string | null) {
      errorMessage = value;
    },
    clearErrorMessage() {
      errorMessage = null;
    },
    get showDeleteAccessPrompt() {
      return showDeleteAccessPrompt;
    },
    openDeleteAccessPrompt() {
      showDeleteAccessPrompt = true;
    },
    closeDeleteAccessPrompt() {
      showDeleteAccessPrompt = false;
    },
    get showResetVideoConfirmation() {
      return showResetVideoConfirmation;
    },
    openResetVideoConfirmation() {
      showResetVideoConfirmation = true;
    },
    closeResetVideoConfirmation() {
      showResetVideoConfirmation = false;
    },
    get allowLoadedVideoSyncDepthOverride() {
      return allowLoadedVideoSyncDepthOverride;
    },
    setAllowLoadedVideoSyncDepthOverride(value: boolean) {
      allowLoadedVideoSyncDepthOverride = value;
    },
    get mobileViewportMq() {
      return mobileViewportMq;
    },
    setMobileViewportMq(value: boolean) {
      mobileViewportMq = value;
    },
    get mobileBrowseOpen() {
      return mobileBrowseOpen;
    },
    setMobileBrowseOpen(value: boolean) {
      mobileBrowseOpen = value;
    },
    openMobileBrowse() {
      mobileBrowseOpen = true;
    },
    closeMobileBrowse() {
      mobileBrowseOpen = false;
    },
    get pendingSelectedVideo() {
      return pendingSelectedVideo;
    },
    setPendingSelectedVideo(value: Video | null) {
      pendingSelectedVideo = value;
    },
    get hydratedWorkspaceScopeKey() {
      return hydratedWorkspaceScopeKey;
    },
    setHydratedWorkspaceScopeKey(value: string | null) {
      hydratedWorkspaceScopeKey = value;
    },
    get videoAcknowledgeSync() {
      return videoAcknowledgeSync;
    },
    setVideoAcknowledgeSync(value: VideoAcknowledgeSync) {
      videoAcknowledgeSync = value;
    },
  };
}
