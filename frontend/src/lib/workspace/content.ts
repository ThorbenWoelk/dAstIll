import type { Summary, Transcript, TranscriptRenderMode } from "$lib/types";

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
  };
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
