import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
import {
  buildOptimisticAcknowledgeSidebarList,
  isStillSelectedAfterAcknowledgeSuccess,
  matchesAcknowledgedFilterVideo,
  resolveNextVisibleVideoAfterAcknowledgeDrop,
  resolveRevertedVideoForAcknowledge,
  resolveVideoForAcknowledgeToggle,
  selectionDroppedAfterAcknowledgeOptimistic,
} from "$lib/workspace/acknowledge-toggle";
import { updateAcknowledged } from "$lib/api";
import { track } from "$lib/analytics/tracker";
import type { createContentState } from "$lib/workspace/content-state.svelte";
import type { createSidebarState } from "$lib/workspace/sidebar-state.svelte";
import type { VideoAcknowledgeSync } from "$lib/workspace/home-workspace-page-state.svelte";
import type { Video } from "$lib/types";

export function createHomeWorkspaceAcknowledgeController(options: {
  sidebarState: ReturnType<typeof createSidebarState>;
  content: ReturnType<typeof createContentState>;
  getPendingSelectedVideo: () => Video | null;
  setPendingSelectedVideo: (value: Video | null) => void;
  setErrorMessage: (value: string | null) => void;
  getSelectedChannelId: () => string | null;
  selectVideo: (videoId: string) => Promise<void>;
  setVideoAcknowledgeSync: (value: VideoAcknowledgeSync) => void;
  updateAcknowledged?: typeof updateAcknowledged;
}) {
  let videoAcknowledgeSeq = 0;

  function syncAcknowledgedVideo(video: Video, confirmed: boolean) {
    videoAcknowledgeSeq += 1;
    options.setVideoAcknowledgeSync({
      seq: videoAcknowledgeSeq,
      video,
      confirmed,
    });
  }

  async function toggleAcknowledge() {
    if (!options.sidebarState.selectedVideoId) return;
    const targetVideoId = options.sidebarState.selectedVideoId;
    const pendingSelectedVideo = options.getPendingSelectedVideo();
    const resolved = resolveVideoForAcknowledgeToggle(
      options.sidebarState.videos,
      targetVideoId,
      pendingSelectedVideo,
    );
    if (!resolved) return;
    const { video, videoFromList } = resolved;

    options.setErrorMessage(null);

    const previousVideos = [...options.sidebarState.videos];
    const previousPendingSelectedVideo = pendingSelectedVideo;
    const previousSelectedVideoId = options.sidebarState.selectedVideoId;
    const newAcknowledged = !video.acknowledged;

    options.sidebarState.bumpVideoListMutationEpoch();

    const optimisticVideo = { ...video, acknowledged: newAcknowledged };
    const optimisticList = buildOptimisticAcknowledgeSidebarList(
      videoFromList,
      previousVideos,
      options.sidebarState.videos,
      targetVideoId,
      newAcknowledged,
      options.sidebarState.acknowledgedFilter,
    );
    if (videoFromList) {
      options.sidebarState.replaceVideos(optimisticList);
    } else {
      options.setPendingSelectedVideo(optimisticVideo);
    }
    syncAcknowledgedVideo(optimisticVideo, false);

    const selectionDroppedFromFilter =
      selectionDroppedAfterAcknowledgeOptimistic(
        videoFromList,
        optimisticList,
        previousSelectedVideoId,
        optimisticVideo,
        options.sidebarState.acknowledgedFilter,
      );
    if (selectionDroppedFromFilter) {
      if (videoFromList) {
        if (optimisticList.length === 0) {
          options.content.resetInteractionState({
            clearDisplayedContent: true,
          });
          options.sidebarState.selectVideo(null);
        } else {
          const nextVideo = resolveNextVisibleVideoAfterAcknowledgeDrop(
            previousVideos,
            targetVideoId,
            optimisticList,
          );
          options.content.resetInteractionState();
          if (nextVideo) {
            await options.selectVideo(nextVideo.id);
          }
        }
      } else {
        options.content.resetInteractionState({ clearDisplayedContent: true });
        options.sidebarState.selectVideo(null);
        options.setPendingSelectedVideo(null);
      }
    }

    try {
      const updated = await (options.updateAcknowledged ?? updateAcknowledged)(
        targetVideoId,
        newAcknowledged,
      );
      if (videoFromList) {
        options.sidebarState.replaceVideos(
          options.sidebarState.videos
            .map((candidate) =>
              candidate.id === updated.id ? updated : candidate,
            )
            .filter((candidate) =>
              matchesAcknowledgedFilterVideo(
                candidate,
                options.sidebarState.acknowledgedFilter,
              ),
            ),
        );
      } else if (!selectionDroppedFromFilter) {
        options.setPendingSelectedVideo(updated);
      }
      const selectedChannelId = options.getSelectedChannelId();
      if (selectedChannelId) {
        track({
          event: "video_acknowledged_changed",
          video_id: targetVideoId,
          channel_id: selectedChannelId,
          acknowledged: newAcknowledged,
        });
      }

      syncAcknowledgedVideo(updated, true);

      const stillSelected = isStillSelectedAfterAcknowledgeSuccess(
        options.sidebarState.selectedVideoId,
        options.sidebarState.videos,
        options.getPendingSelectedVideo(),
      );
      if (!stillSelected) {
        if (options.sidebarState.videos.length === 0) {
          options.content.resetInteractionState({
            clearDisplayedContent: true,
          });
          options.sidebarState.selectVideo(null);
        } else {
          options.content.resetInteractionState();
          await options.selectVideo(options.sidebarState.videos[0].id);
        }
      }
    } catch (error) {
      options.sidebarState.replaceVideos(previousVideos);
      options.sidebarState.selectVideo(previousSelectedVideoId);
      options.setPendingSelectedVideo(previousPendingSelectedVideo);
      const reverted = resolveRevertedVideoForAcknowledge(
        previousVideos,
        targetVideoId,
        previousPendingSelectedVideo,
      );
      if (reverted) {
        syncAcknowledgedVideo(reverted, true);
      }
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        options.setErrorMessage((error as Error).message);
      }
    }
  }

  return {
    toggleAcknowledge,
  };
}
