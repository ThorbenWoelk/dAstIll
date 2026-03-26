import type { Video } from "$lib/types";
import { applyOptimisticAcknowledge } from "$lib/workspace/channel-actions";
import type { AcknowledgedFilter } from "$lib/workspace/types";

/**
 * Pure helpers for read/unread toggle logic. Used by the workspace page and unit
 * tests so pending-only selections cannot regress silently.
 */
export function matchesAcknowledgedFilterVideo(
  video: Video,
  filter: AcknowledgedFilter,
): boolean {
  if (filter === "ack") return video.acknowledged;
  if (filter === "unack") return !video.acknowledged;
  return true;
}

export function resolveVideoForAcknowledgeToggle(
  videos: Video[],
  selectedVideoId: string | null,
  pendingSelectedVideo: Video | null,
): { video: Video; videoFromList: boolean } | null {
  if (!selectedVideoId) return null;
  const fromList = videos.find((v) => v.id === selectedVideoId);
  if (fromList) return { video: fromList, videoFromList: true };
  if (pendingSelectedVideo?.id === selectedVideoId) {
    return { video: pendingSelectedVideo, videoFromList: false };
  }
  return null;
}

export function buildOptimisticAcknowledgeSidebarList(
  videoFromList: boolean,
  previousVideosSnapshot: Video[],
  videos: Video[],
  targetVideoId: string,
  newAcknowledged: boolean,
  filter: AcknowledgedFilter,
): Video[] {
  if (!videoFromList) return previousVideosSnapshot;
  return applyOptimisticAcknowledge(
    videos,
    targetVideoId,
    newAcknowledged,
  ).filter((v) => matchesAcknowledgedFilterVideo(v, filter));
}

export function selectionDroppedAfterAcknowledgeOptimistic(
  videoFromList: boolean,
  optimisticList: Video[],
  previousSelectedVideoId: string | null,
  optimisticVideo: Video,
  filter: AcknowledgedFilter,
): boolean {
  if (!previousSelectedVideoId) return false;
  if (videoFromList) {
    return !optimisticList.some((v) => v.id === previousSelectedVideoId);
  }
  return !matchesAcknowledgedFilterVideo(optimisticVideo, filter);
}

export function isStillSelectedAfterAcknowledgeSuccess(
  selectedVideoId: string | null,
  videos: Video[],
  pendingSelectedVideo: Video | null,
): boolean {
  if (selectedVideoId == null) return false;
  return (
    videos.some((v) => v.id === selectedVideoId) ||
    pendingSelectedVideo?.id === selectedVideoId
  );
}

export function resolveRevertedVideoForAcknowledge(
  previousVideos: Video[],
  targetVideoId: string,
  previousPendingSelectedVideo: Video | null,
): Video | null {
  return (
    previousVideos.find((v) => v.id === targetVideoId) ??
    (previousPendingSelectedVideo?.id === targetVideoId
      ? previousPendingSelectedVideo
      : null) ??
    null
  );
}
