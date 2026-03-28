import { getVideo, listVideos } from "$lib/api";
import type { AddVideoResult, Channel } from "$lib/types";
import {
  buildChannelAddFeedback,
  buildVideoAddFeedback,
  type AddSourceFeedback,
  resolveAddedChannelStatus,
  resolveAddedVideoStatus,
} from "$lib/workspace/add-source-feedback";

export type OpenTargetOptions = {
  onOpenVideo: (videoId: string, channelId: string) => void | Promise<void>;
  onOpenChannel: (channelId: string) => void | Promise<void>;
};

export function createAddSourceFeedbackController() {
  let feedback = $state<AddSourceFeedback | null>(null);
  let dismissed = $state(false);
  let pollSequence = 0;

  function present(next: AddSourceFeedback) {
    feedback = next;
    dismissed = false;
  }

  function dismiss() {
    dismissed = true;
    if (feedback?.status !== "loading") {
      feedback = null;
    }
  }

  async function trackAddedVideo(result: AddVideoResult) {
    const sequence = ++pollSequence;
    let nextResult = result;

    present(
      buildVideoAddFeedback(
        nextResult,
        resolveAddedVideoStatus(nextResult.video),
      ),
    );

    while (sequence === pollSequence) {
      const currentStatus = resolveAddedVideoStatus(nextResult.video);
      if (currentStatus !== "loading") {
        return;
      }

      await new Promise((resolve) => window.setTimeout(resolve, 4000));
      if (sequence !== pollSequence) {
        return;
      }

      try {
        const refreshedVideo = await getVideo(nextResult.video.id, true);
        nextResult = { ...nextResult, video: refreshedVideo };
        present(
          buildVideoAddFeedback(
            nextResult,
            resolveAddedVideoStatus(refreshedVideo),
          ),
        );
      } catch {
        // Keep polling quietly; the initial acceptance feedback already surfaced.
      }
    }
  }

  async function trackAddedChannel(channel: Channel) {
    const sequence = ++pollSequence;
    present(buildChannelAddFeedback(channel, "loading"));

    while (sequence === pollSequence) {
      await new Promise((resolve) => window.setTimeout(resolve, 4000));
      if (sequence !== pollSequence) {
        return;
      }

      try {
        const videos = await listVideos(
          channel.id,
          1,
          0,
          "all",
          undefined,
          false,
          undefined,
          true,
        );
        const status = resolveAddedChannelStatus(videos.videos);
        present(buildChannelAddFeedback(channel, status));
        if (status === "ready") {
          return;
        }
      } catch {
        // Keep polling quietly; the initial acceptance feedback already surfaced.
      }
    }
  }

  async function openTarget(options: OpenTargetOptions) {
    const current = feedback;
    if (!current) {
      return;
    }

    pollSequence += 1;
    feedback = null;
    dismissed = false;

    if (current.kind === "video") {
      await options.onOpenVideo(current.videoId, current.targetChannelId);
      return;
    }

    await options.onOpenChannel(current.channelId);
  }

  /** Call from onMount cleanup to stop any in-flight polling when the component unmounts. */
  function cancelPolling() {
    pollSequence += 1;
  }

  return {
    get feedback() {
      return feedback;
    },
    get dismissed() {
      return dismissed;
    },
    present,
    dismiss,
    trackAddedVideo,
    trackAddedChannel,
    openTarget,
    cancelPolling,
  };
}
