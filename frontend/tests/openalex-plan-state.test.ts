import { describe, expect, it } from "bun:test";

import {
  buildEmptyOpenAlexPlan,
  buildOpenAlexInterpretationStatus,
  prepareOpenAlexPlanForSubmit,
  syncOpenAlexPlanFromDraft,
} from "../src/lib/openalex-plan-state";
import type { OpenAlexSavedSearchQuery } from "../src/lib/types";

function makePlan(): OpenAlexSavedSearchQuery {
  return {
    natural_language_query: "recent multimodal ai papers",
    query_text: "multimodal ai",
    from_publication_date: "2026-01-01",
    to_publication_date: null,
    work_type: "article",
    open_access_only: true,
    search_scope: "title_and_abstract",
    sort: "publication_date_desc",
  };
}

describe("buildEmptyOpenAlexPlan", () => {
  it("creates a default editable OpenAlex plan", () => {
    expect(buildEmptyOpenAlexPlan("protein folding")).toEqual({
      natural_language_query: "protein folding",
      query_text: "protein folding",
      from_publication_date: null,
      to_publication_date: null,
      work_type: null,
      open_access_only: null,
      search_scope: "title_and_abstract",
      sort: "publication_date_desc",
    });
  });
});

describe("syncOpenAlexPlanFromDraft", () => {
  it("creates a default plan when the user starts with manual entry", () => {
    expect(
      syncOpenAlexPlanFromDraft(null, "", "recent multimodal ai papers"),
    ).toEqual(
      expect.objectContaining({
        natural_language_query: "recent multimodal ai papers",
        query_text: "recent multimodal ai papers",
      }),
    );
  });

  it("keeps a custom query text when the plain-language draft changes", () => {
    expect(
      syncOpenAlexPlanFromDraft(
        makePlan(),
        "recent multimodal ai papers",
        "protein folding diffusion",
      ),
    ).toEqual(
      expect.objectContaining({
        natural_language_query: "protein folding diffusion",
        query_text: "multimodal ai",
      }),
    );
  });

  it("mirrors query text when it still matches the old draft", () => {
    const plan = buildEmptyOpenAlexPlan("recent multimodal ai papers");

    expect(
      syncOpenAlexPlanFromDraft(
        plan,
        "recent multimodal ai papers",
        "protein folding diffusion",
      ),
    ).toEqual(
      expect.objectContaining({
        natural_language_query: "protein folding diffusion",
        query_text: "protein folding diffusion",
      }),
    );
  });
});

describe("prepareOpenAlexPlanForSubmit", () => {
  it("uses structured query text even when the plain-language draft is blank", () => {
    expect(
      prepareOpenAlexPlanForSubmit(
        {
          ...makePlan(),
          natural_language_query: "",
        },
        "",
      ),
    ).toEqual(
      expect.objectContaining({
        natural_language_query: "multimodal ai",
        query_text: "multimodal ai",
      }),
    );
  });

  it("prefers the draft as the final natural-language label", () => {
    expect(
      prepareOpenAlexPlanForSubmit(makePlan(), "recent multimodal ai papers"),
    ).toEqual(
      expect.objectContaining({
        natural_language_query: "recent multimodal ai papers",
        query_text: "multimodal ai",
      }),
    );
  });
});

describe("buildOpenAlexInterpretationStatus", () => {
  it("describes the preparing phase", () => {
    expect(buildOpenAlexInterpretationStatus("preparing")).toEqual(
      expect.objectContaining({
        phase: "preparing",
        stateLabel: "Preparing request",
      }),
    );
  });

  it("describes the planner wait phase", () => {
    expect(buildOpenAlexInterpretationStatus("planning")).toEqual(
      expect.objectContaining({
        phase: "planning",
        stateLabel: "Interpreting request",
      }),
    );
  });

  it("describes the failure phase", () => {
    expect(buildOpenAlexInterpretationStatus("failed")).toEqual(
      expect.objectContaining({
        phase: "failed",
        stateLabel: "Interpretation failed",
      }),
    );
  });
});
