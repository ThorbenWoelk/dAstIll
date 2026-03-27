import type { AddVideoResult, Channel, Video } from "$lib/types";

export type AddSourceFeedbackStatus = "loading" | "ready" | "failed";

export type AddSourceFeedback =
  | {
      kind: "video";
      status: AddSourceFeedbackStatus;
      title: string;
      message: string;
      actionLabel: string | null;
      videoId: string;
      targetChannelId: string;
    }
  | {
      kind: "channel";
      status: "loading" | "ready";
      title: string;
      message: string;
      actionLabel: string | null;
      channelId: string;
    };

export function resolveAddedVideoStatus(
  video: Pick<Video, "transcript_status" | "summary_status">,
): AddSourceFeedbackStatus {
  if (
    video.transcript_status === "failed" ||
    video.summary_status === "failed"
  ) {
    return "failed";
  }

  if (video.summary_status === "ready") {
    return "ready";
  }

  return "loading";
}

export function resolveAddedChannelStatus(
  videos: Array<Pick<Video, "id">>,
): "loading" | "ready" {
  return videos.length > 0 ? "ready" : "loading";
}

export function buildVideoAddFeedback(
  result: AddVideoResult,
  status: AddSourceFeedbackStatus,
): AddSourceFeedback {
  const title = result.video.title.trim() || "Untitled video";
  const isExisting = result.already_exists;

  if (status === "ready") {
    return {
      kind: "video",
      status,
      title: isExisting ? "Video already saved" : "Video ready",
      message: isExisting
        ? `"${title}" is already in your library and ready to open.`
        : `"${title}" is ready to open.`,
      actionLabel: "Open video",
      videoId: result.video.id,
      targetChannelId: result.target_channel_id,
    };
  }

  if (status === "failed") {
    return {
      kind: "video",
      status,
      title: isExisting ? "Video already saved" : "Video needs attention",
      message: isExisting
        ? `"${title}" is already saved, but its transcript or summary is not ready yet.`
        : `"${title}" was added, but its transcript or summary failed to finish.`,
      actionLabel: "Open video",
      videoId: result.video.id,
      targetChannelId: result.target_channel_id,
    };
  }

  return {
    kind: "video",
    status,
    title: isExisting ? "Video already saved" : "Video accepted",
    message: isExisting
      ? `"${title}" is already in your library. We'll let you know when it's ready to open.`
      : `"${title}" looks valid. We're loading it now and will prompt you when it's ready.`,
    actionLabel: null,
    videoId: result.video.id,
    targetChannelId: result.target_channel_id,
  };
}

export function buildChannelAddFeedback(
  channel: Pick<Channel, "id" | "name">,
  status: "loading" | "ready",
): AddSourceFeedback {
  const name = channel.name.trim() || "New channel";

  if (status === "ready") {
    return {
      kind: "channel",
      status,
      title: "Channel ready",
      message: `"${name}" is ready to browse.`,
      actionLabel: "Open channel",
      channelId: channel.id,
    };
  }

  return {
    kind: "channel",
    status,
    title: "Channel added",
    message: `"${name}" was accepted. We're pulling in its videos now and will prompt you when it's ready.`,
    actionLabel: null,
    channelId: channel.id,
  };
}
