import type { OpenAlexSavedSearchQuery } from "$lib/types";

export type OpenAlexInterpretationPhase = "preparing" | "planning" | "failed";

export type OpenAlexInterpretationStatus = {
  phase: OpenAlexInterpretationPhase;
  stateLabel: string;
  message: string;
};

export function buildEmptyOpenAlexPlan(seed = ""): OpenAlexSavedSearchQuery {
  return {
    natural_language_query: seed,
    query_text: seed,
    from_publication_date: null,
    to_publication_date: null,
    work_type: null,
    open_access_only: null,
    search_scope: "title_and_abstract",
    sort: "publication_date_desc",
  };
}

export function syncOpenAlexPlanFromDraft(
  currentPlan: OpenAlexSavedSearchQuery | null,
  previousDraft: string,
  nextDraft: string,
) {
  const nextPlan = currentPlan ?? buildEmptyOpenAlexPlan(nextDraft);

  if (nextDraft === previousDraft) {
    return nextPlan;
  }

  const previousQueryText = nextPlan.query_text.trim();
  const shouldMirrorQueryText =
    previousQueryText.length === 0 ||
    previousQueryText === previousDraft.trim();

  return {
    ...nextPlan,
    natural_language_query: nextDraft,
    query_text: shouldMirrorQueryText ? nextDraft : nextPlan.query_text,
  };
}

export function prepareOpenAlexPlanForSubmit(
  currentPlan: OpenAlexSavedSearchQuery | null,
  draft: string,
) {
  const seed = draft.trim();
  const basePlan = currentPlan ?? buildEmptyOpenAlexPlan(seed);
  const queryText = basePlan.query_text.trim() || seed;
  const naturalLanguageQuery =
    seed || basePlan.natural_language_query.trim() || queryText;

  return {
    ...basePlan,
    natural_language_query: naturalLanguageQuery,
    query_text: queryText,
  };
}

export function buildOpenAlexInterpretationStatus(
  phase: OpenAlexInterpretationPhase,
): OpenAlexInterpretationStatus {
  switch (phase) {
    case "preparing":
      return {
        phase,
        stateLabel: "Preparing request",
        message: "You can keep browsing while we package your OpenAlex search.",
      };
    case "planning":
      return {
        phase,
        stateLabel: "Interpreting request",
        message:
          "The AI planner is translating your topic into reviewable OpenAlex filters.",
      };
    case "failed":
      return {
        phase,
        stateLabel: "Interpretation failed",
        message:
          "We could not prepare the OpenAlex filters. Review the error in the drawer and try again.",
      };
  }
}
