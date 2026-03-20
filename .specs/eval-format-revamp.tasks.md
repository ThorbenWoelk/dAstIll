# Tasks: Eval Format Revamp

## Current State
Implemented backend prompt updates and frontend markdown rendering. Verified with unit tests.

## Steps
- [x] Research: verify where `quality_note` is displayed in the frontend.
- [x] Research: check how `quality_note` is currently stored in the DB (if at all).
- [x] Backend: Update prompt in `backend/src/services/summary_evaluator.rs`.
- [x] Backend: Update unit tests for `parse_evaluation_response`.
- [x] Frontend: Update `WorkspaceSummaryMeta.svelte` to render markdown.
- [x] Frontend: Add basic styling for markdown lists in the eval note.
- [ ] Verification: Manually trigger evaluation and check result.

## Decisions Made During Implementation
- Added a custom style block to `WorkspaceSummaryMeta.svelte` to handle markdown lists and headers while maintaining a compact, italicized look consistent with the existing UI.
- Used `renderMarkdown` utility in the frontend to safely parse and sanitize the markdown from the evaluator.
