import type {
  Summary,
  Transcript,
  TranscriptRenderMode,
  Video,
} from "$lib/types";

export { formatPublishedAt, formatSyncDate } from "$lib/utils/date";

export interface TranscriptPresentation {
  content: string;
  originalText: string;
  renderMode: TranscriptRenderMode;
}

export interface SummaryQualityPresentation {
  score: number | null;
  note: string | null;
  modelUsed: string | null;
  qualityModelUsed: string | null;
  tags: string[];
  tagsEvaluated: boolean;
}

export interface BackgroundSummaryRefreshPresentation {
  contentText: string;
  draft: string;
  shouldClearVideoInfo: boolean;
  quality: SummaryQualityPresentation;
}

export function stripContentPrefix(text: string): string {
  return text.replace(/^(?:Transcript|Summary):\s*/i, "").trimStart();
}

export function resolveTranscriptPresentation(
  transcript: Transcript,
): TranscriptPresentation {
  const rawText = stripContentPrefix(transcript.raw_text || "");
  const formattedMarkdown = stripContentPrefix(
    transcript.formatted_markdown || "",
  );
  const renderMode = transcript.render_mode ?? "plain_text";
  const originalText =
    rawText || formattedMarkdown || "Transcript unavailable.";
  const content =
    renderMode === "markdown"
      ? formattedMarkdown || rawText || "Transcript unavailable."
      : rawText || formattedMarkdown || "Transcript unavailable.";

  return {
    content,
    originalText,
    renderMode,
  };
}

export function resolveSummaryQualityPresentation(
  summary: Summary,
): SummaryQualityPresentation {
  return {
    score:
      typeof summary.quality_score === "number"
        ? Math.max(0, Math.min(10, Math.round(summary.quality_score)))
        : null,
    note: summary.quality_note?.trim() || null,
    modelUsed: summary.model_used ?? null,
    qualityModelUsed: summary.quality_model_used ?? null,
    tags: (summary.summary_tags ?? [])
      .map((tag: string) => tag.trim())
      .filter(Boolean),
    tagsEvaluated: summary.summary_tags_evaluated ?? false,
  };
}

export function hasCompleteSummaryEvaluation(params: {
  score: number | null;
  note: string | null;
  tagsEvaluated: boolean;
}): boolean {
  return (
    Boolean(params.note?.trim()) ||
    (params.score !== null && params.tagsEvaluated)
  );
}

export function resolveBackgroundSummaryRefresh(
  currentContentText: string,
  summary: Summary,
): BackgroundSummaryRefreshPresentation {
  const hasDisplayedContent = currentContentText.trim().length > 0;
  const hydratedContent = stripContentPrefix(
    summary.content || "Summary unavailable.",
  );

  return {
    contentText: hasDisplayedContent ? currentContentText : hydratedContent,
    draft: hasDisplayedContent ? currentContentText : hydratedContent,
    shouldClearVideoInfo: !hasDisplayedContent,
    quality: resolveSummaryQualityPresentation(summary),
  };
}

export function shouldRetryReadySummaryLoad(params: {
  contentMode: "transcript" | "summary" | "highlights" | "info";
  selectedVideo: Pick<Video, "summary_status"> | null | undefined;
  contentText: string;
  loadingContent: boolean;
  editing: boolean;
}): boolean {
  return (
    params.contentMode === "summary" &&
    params.selectedVideo?.summary_status === "ready" &&
    !params.contentText.trim() &&
    !params.loadingContent &&
    !params.editing
  );
}

/** True when an in-flight content mutation should update the visible editor. */
export function shouldApplyCompletedContentMutation(params: {
  selectedVideoId: string | null | undefined;
  targetVideoId: string;
  contentMode: "transcript" | "summary" | "highlights" | "info";
  targetMode: "transcript" | "summary" | "highlights" | "info";
}): boolean {
  return (
    params.selectedVideoId === params.targetVideoId &&
    params.contentMode === params.targetMode
  );
}

export function hasKnownDuration(
  seconds: number | null | undefined,
  iso8601: string | null | undefined,
): boolean {
  return (
    (seconds !== null && seconds !== undefined && seconds >= 0) ||
    Boolean(iso8601?.trim())
  );
}

export function formatDuration(
  seconds: number | null | undefined,
  iso8601: string | null | undefined,
): string {
  if (seconds !== null && seconds !== undefined && seconds >= 0) {
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    if (hrs > 0) {
      return `${hrs}h ${mins}m ${secs}s`;
    }
    return `${mins}m ${secs}s`;
  }

  if (iso8601) {
    return iso8601;
  }

  return "Unknown";
}
