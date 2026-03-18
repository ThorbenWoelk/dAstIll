# Spec: Replace `"search progress"` with real semantic search progress

## Assumption

"Semantic search progress" refers to live semantic-index coverage already exposed by `searchStatus` (`embedded_chunk_count`, `total_chunk_count`, `retrieval_mode`), not per-request backend execution phases.

## Current state

- `frontend/src/lib/components/workspace/WorkspaceHeader.svelte` renders a hardcoded badge with `search progress` while a submitted semantic search request is active.
- The same component already subscribes to live search status via SSE using `openSearchStatusStream()`.
- `frontend/src/lib/search-status.ts` already formats keyword and semantic coverage for the idle hint.

## Recommended approach

### 1. Add a semantic-only badge formatter

In `frontend/src/lib/search-status.ts`, add a helper such as `resolveSemanticSearchProgressLabel(status: SearchStatus | null): string | null`.

Behavior:

- return `null` when there is no status or `total_chunk_count === 0`
- return `"70% semantic"` when semantic coverage can be expressed cleanly as a percentage
- return `"7 / 400 semantic"` when rounding would hide visible progress
- return `"keyword only"` when semantic search is unavailable

This should follow the same formatting rules already used by `resolveSearchCoverageHint()`.

### 2. Use the live label in the header

In `frontend/src/lib/components/workspace/WorkspaceHeader.svelte`:

- derive a semantic progress label from the current `searchStatus`
- replace the hardcoded badge text with that derived label
- keep a short fallback like `semantic` only if the formatter returns `null`

### 3. Improve the badge tooltip

Set a descriptive `title` on the badge, for example:

`Semantic index: 7 / 10 chunks embedded. Retrieval mode: hybrid_exact.`

That keeps the badge compact while still exposing detail on hover.

## Why this is the best first step

- no backend API changes required
- uses data already streamed into the component
- matches the existing search-status model
- keeps the change localized to the frontend

## If per-query progress is actually desired

If the intended meaning is progress for the active submitted search request itself, that is a separate feature. It would require:

1. a streamed search endpoint or request-id based progress endpoint
2. backend progress events for phases like embedding, candidate retrieval, and ranking
3. frontend request-scoped progress state

## Files to change

- `frontend/src/lib/search-status.ts`
- `frontend/src/lib/components/workspace/WorkspaceHeader.svelte`
- `frontend/tests/search-status.test.ts`

## Acceptance criteria

- During active submitted search, the badge no longer shows the literal text `"search progress"`.
- It shows semantic progress derived from live `searchStatus`.
- It handles partial progress, tiny progress, semantic-unavailable fallback, and 100% completion.

## Validation

Run in `frontend/`:

```bash
bun test
npm run check
```
