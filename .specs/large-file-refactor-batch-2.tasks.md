# Tasks: Large File Refactor Batch 2

## Current State
All source files are now under the 1000-line cap, and the refactor path is green through Playwright as well. A follow-up workspace regression from the split (`?guide=0` not restoring the feature guide) was fixed in the persistence controller, the stale E2E assumptions around anonymous chat and the mobile workspace DOM were updated, and frontend verification now includes `bun run test:e2e` passing with `4 passed / 5 skipped` alongside `bun run check`.

## Steps
- [x] Split `backend/src/bin/chat_capability_eval.rs` into smaller binary-local modules.
- [x] Extract route-local chat page logic from `frontend/src/routes/chat/+page.svelte`.
- [x] Extract cohesive workspace controllers from `frontend/src/lib/workspace/home-workspace.svelte.ts`.
- [x] Recount source files to confirm no file remains above 1000 lines.
- [x] Run backend and frontend verification commands for the combined refactor batches.
- [x] Fix the workspace guide restore regression and rebaseline Playwright coverage against the current chat/mobile flows.

## Decisions Made During Implementation
- Keep the evaluator as a binary-local module tree under `backend/src/bin/chat_capability_eval/` instead of moving the logic into the shared backend library.
- Split the chat route by concern: a page controller for thread/session state plus a separate stream controller for SSE lifecycle, scroll state, and reconnect handling.
- Split the home workspace root into dedicated data, persistence, and highlight controllers so the page-level API stayed stable while the implementation moved behind route-local composables.
- Keep Playwright aligned with the current anonymous-first chat experience and the current mobile workspace DOM instead of forcing legacy server-seeding or selector assumptions onto the refactored UI.
