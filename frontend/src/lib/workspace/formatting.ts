import type { Summary as SummaryPayload } from "$lib/types";

export type SummaryQualityState = {
  score: number | null;
  note: string | null;
  modelUsed: string | null;
  qualityModelUsed: string | null;
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
  };
}

export function resetSummaryQualityState(): SummaryQualityState {
  return {
    score: null,
    note: null,
    modelUsed: null,
    qualityModelUsed: null,
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

export function hasSummaryContent(summary: SummaryPayload) {
  return Boolean(summary.content?.trim());
}
