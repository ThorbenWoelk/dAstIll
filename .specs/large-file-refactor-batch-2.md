# Large File Refactor Batch 2

## Problem

After the first refactor batch, three source files still exceed the user's requested 1000-line cap: `frontend/src/lib/workspace/home-workspace.svelte.ts`, `frontend/src/routes/chat/+page.svelte`, and `backend/src/bin/chat_capability_eval.rs`. Each file combines multiple concerns, which makes further changes risky and keeps the repo above the requested threshold.

## Goal

Reduce the remaining over-1000-line files below the threshold without changing user-facing behavior, using module boundaries that match the repo's existing controller/composable patterns.

## Requirements

- Reduce `backend/src/bin/chat_capability_eval.rs` below 1000 lines by splitting evaluation, reporting, CLI, stream parsing, or runner logic into cohesive modules.
- Reduce `frontend/src/routes/chat/+page.svelte` below 1000 lines by moving self-contained chat page logic into one or more route-local controllers or helpers.
- Reduce `frontend/src/lib/workspace/home-workspace.svelte.ts` below 1000 lines by extracting cohesive workspace controllers while preserving the current reactive API exposed to the page.
- Keep all newly introduced source files below 1000 lines as well.
- Preserve existing route behavior, state transitions, and backend CLI behavior.
- Re-run the relevant backend and frontend verification commands after the refactor.

## Non-Goals

- Reworking chat or workspace product behavior.
- Redesigning the frontend UI.
- Changing the evaluator dataset format or backend API contracts.

## Design Considerations

- The chat evaluator is a binary-only tool, so local modules are preferable to moving this logic into the shared backend library unless the refactor clearly benefits reuse.
- The workspace and chat pages already use composable state helpers such as `createSidebarState`, `createContentState`, and `createVocabularyController`; new extractions should follow that closure-based pattern instead of introducing a different state model.
- Reactive values returned from Svelte runes helpers should continue to use getters and setters so the reactive boundary stays intact.

## Open Questions

- None for this batch. The goal is a behavior-preserving structural split with verification.
