import type { Summary as SummaryPayload } from "$lib/types";

export type SummaryQualityState = {
  score: number | null;
  note: string | null;
  modelUsed: string | null;
  qualityModelUsed: string | null;
  tags: string[];
  tagsEvaluated: boolean;
};

export type FormattingFeedbackState = {
  formattingAttemptsMax: number | null;
  formattingAttemptsUsed: number | null;
  formattingAttemptsVideoId: string | null;
  formattingNotice: string | null;
  formattingNoticeVideoId: string | null;
};

export function buildSummaryQualityState(
  presentation: Pick<SummaryQualityState, keyof SummaryQualityState>,
): SummaryQualityState {
  return {
    score: presentation.score,
    note: presentation.note,
    modelUsed: presentation.modelUsed,
    qualityModelUsed: presentation.qualityModelUsed,
    tags: presentation.tags,
    tagsEvaluated: presentation.tagsEvaluated,
  };
}

export function resetSummaryQualityState(): SummaryQualityState {
  return {
    score: null,
    note: null,
    modelUsed: null,
    qualityModelUsed: null,
    tags: [],
    tagsEvaluated: false,
  };
}

export function clearFormattingFeedbackState(): FormattingFeedbackState {
  return {
    formattingAttemptsMax: null,
    formattingAttemptsUsed: null,
    formattingAttemptsVideoId: null,
    formattingNotice: null,
    formattingNoticeVideoId: null,
  };
}

export function buildFormattingAttemptSummary(result: {
  attempts_used: number;
  max_attempts: number;
}) {
  return `Attempts ${result.attempts_used}/${result.max_attempts}.`;
}

export function currentTranscriptRevision(
  revisions: Record<string, number>,
  videoId: string,
): number {
  return revisions[videoId] ?? 0;
}

export function bumpTranscriptRevision(
  revisions: Record<string, number>,
  videoId: string,
): number {
  const next = currentTranscriptRevision(revisions, videoId) + 1;
  revisions[videoId] = next;
  return next;
}

/** Skip a background transcript PUT when the user changed that video after the job started. */
export function shouldPersistBackgroundTranscriptWrite(input: {
  resultDiffersFromSource: boolean;
  capturedRevision: number;
  currentRevision: number;
}): boolean {
  return (
    input.resultDiffersFromSource &&
    input.capturedRevision === input.currentRevision
  );
}

export function hasSummaryContent(summary: SummaryPayload) {
  return Boolean(summary.content?.trim());
}
