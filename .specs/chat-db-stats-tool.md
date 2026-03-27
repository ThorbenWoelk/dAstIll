# Chat Tool Loop

**Linear:** none

## Problem

The chat assistant currently relies on a planner-heavy retrieval flow. That makes simple operational questions awkward, and it forces too much decision-making into fixed backend stages instead of exposing a small set of safe tools that the model can use directly.

## Goal

Move chat toward a tool-driven loop where the model can decide whether to:

- answer from conversation history
- search the indexed library through a single search tool
- inspect stored app data through a read-only database tool

The backend must still enforce all safety, validation, and retrieval internals.

## Requirements

- The chat backend must expose a small set of safe tools with clear model-facing descriptions.
- The model must be able to choose tools freely within that allowed set.
- The initial tool set must include:
  - a read-only `db_inspect` tool for approved operational lookups
  - a `search_library` tool that hides the internal keyword/vector retrieval strategy behind one interface
- The assistant must be able to answer the initial target question: how many summaries do we have in the db.
- The tool surface must stay safe and validated server-side: no arbitrary query execution and no write operations.
- Tool usage must stay transparent in the chat UI.
- Existing retrieval-grounded answer quality must be preserved by reusing current source ranking and grounding where possible.
- The new behavior must be covered by automated tests.

## Non-Goals

- Arbitrary SQL, free-form database access, or write operations.
- Exposing internal retrieval modes like keyword vs semantic vs reranking directly to the model.
- A second search implementation separate from the current search stack.
- UI redesign beyond making tool usage visible and avoiding misleading capability claims.

## Design Considerations

- The backend should prefer a single tool loop over separate pre-classification and retrieval-planning stages.
- `search_library` should be the abstraction boundary for retrieval. The model decides that it needs library evidence, but the backend still decides how to run hybrid retrieval, candidate fusion, and ranking.
- Store data is persisted under S3 prefixes, so a read-only inspection tool can be implemented with existing store helpers instead of introducing a second database layer.
- The model should choose whether and when to call tools, but the backend must validate every requested operation, resource, source filter, and limit before execution.
- Final answer generation should continue to use grounded excerpts and visible tool outputs so the UX keeps its source-backed behavior.

## Open Questions

- Whether a future iteration should let `search_library` expose optional channel scoping by name, or keep the first version query-only.
