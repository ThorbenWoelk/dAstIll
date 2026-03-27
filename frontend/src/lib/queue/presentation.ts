import type { Video } from "$lib/types";

export type QueuePipelineStepStatus =
  | "complete"
  | "active"
  | "upcoming"
  | "failed";

export type QueuePipelineStep = {
  key: string;
  label: string;
  status: QueuePipelineStepStatus;
};

/** Primary pipeline label for queue video details. */
export function queueVideoPrimaryState(video: Video): string {
  if (video.transcript_status === "failed") {
    return "Transcript failed";
  }
  if (video.transcript_status === "loading") {
    return "Transcript generating";
  }
  if (video.transcript_status === "pending") {
    return "In queue";
  }
  if (video.summary_status === "failed") {
    return "Summary failed";
  }
  if (video.summary_status === "loading") {
    return "Summary generating";
  }
  if (video.summary_status === "pending") {
    return "In queue";
  }
  return "Complete";
}

/** Minimal 3-step model: queue -> transcript -> summary. */
export function queueVideoPipelineSteps(video: Video): QueuePipelineStep[] {
  const transcript = video.transcript_status;
  const summary = video.summary_status;

  if (transcript === "failed") {
    return [
      { key: "q", label: "Queue", status: "complete" },
      { key: "tr", label: "Transcript", status: "failed" },
      { key: "su", label: "Summary", status: "upcoming" },
    ];
  }
  if (transcript === "pending") {
    return [
      { key: "q", label: "Queue", status: "active" },
      { key: "tr", label: "Transcript", status: "upcoming" },
      { key: "su", label: "Summary", status: "upcoming" },
    ];
  }
  if (transcript === "loading") {
    return [
      { key: "q", label: "Queue", status: "complete" },
      { key: "tr", label: "Transcript", status: "active" },
      { key: "su", label: "Summary", status: "upcoming" },
    ];
  }
  if (summary === "failed") {
    return [
      { key: "q", label: "Queue", status: "complete" },
      { key: "tr", label: "Transcript", status: "complete" },
      { key: "su", label: "Summary", status: "failed" },
    ];
  }
  if (summary === "loading" || summary === "pending") {
    return [
      { key: "q", label: "Queue", status: "complete" },
      { key: "tr", label: "Transcript", status: "complete" },
      { key: "su", label: "Summary", status: "active" },
    ];
  }

  return [
    { key: "q", label: "Queue", status: "complete" },
    { key: "tr", label: "Transcript", status: "complete" },
    { key: "su", label: "Summary", status: "complete" },
  ];
}

export function queueStateAccentClass(video: Video): string {
  if (
    video.transcript_status === "failed" ||
    video.summary_status === "failed"
  ) {
    return "bg-[var(--danger)]";
  }
  if (
    video.transcript_status === "loading" ||
    video.summary_status === "loading"
  ) {
    return "bg-[var(--accent)] motion-safe:animate-pulse";
  }
  if (
    video.transcript_status === "pending" ||
    video.summary_status === "pending"
  ) {
    return "bg-[var(--soft-foreground)]/45";
  }
  if (video.transcript_status === "ready" && video.summary_status === "ready") {
    return "bg-[var(--accent-strong)]/80";
  }
  return "bg-[var(--soft-foreground)]/35";
}
