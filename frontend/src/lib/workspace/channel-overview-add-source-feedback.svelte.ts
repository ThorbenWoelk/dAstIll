import type { AddVideoResult, Channel, Video } from "$lib/types";
import {
  buildChannelAddFeedback,
  buildVideoAddFeedback,
  type AddSourceFeedback,
  resolveAddedChannelStatus,
  resolveAddedVideoStatus,
} from "$lib/workspace/add-source-feedback";

type ChannelOverviewAddSourceFeedbackControllerOptions = {
  refreshVideo: (videoId: string) => Promise<Video>;
  loadChannelVideos: (channelId: string) => Promise<Video[]>;
  openTarget: (feedback: AddSourceFeedback) => Promise<void>;
};

export function createChannelOverviewAddSourceFeedbackController(
  options: ChannelOverviewAddSourceFeedbackControllerOptions,
) {
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
        const refreshedVideo = await options.refreshVideo(nextResult.video.id);
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
        const videos = await options.loadChannelVideos(channel.id);
        const status = resolveAddedChannelStatus(videos);
        present(buildChannelAddFeedback(channel, status));
        if (status === "ready") {
          return;
        }
      } catch {
        // Keep polling quietly; the initial acceptance feedback already surfaced.
      }
    }
  }

  async function openTarget() {
    const current = feedback;
    if (!current) {
      return;
    }

    pollSequence += 1;
    feedback = null;
    dismissed = false;
    await options.openTarget(current);
  }

  function dispose() {
    pollSequence += 1;
  }

  return {
    get feedback() {
      return feedback;
    },
    get dismissed() {
      return dismissed;
    },
    dismiss,
    trackAddedVideo,
    trackAddedChannel,
    openTarget,
    dispose,
  };
}
