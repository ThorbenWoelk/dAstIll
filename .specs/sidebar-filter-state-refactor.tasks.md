# Tasks: Sidebar Filter State Refactor

## Current State

The shared sidebar state factory has now been split so video-loading/filter flows and channel CRUD live in dedicated modules. Local verification is green again after the extraction, and the next cleanup target is the still-oversized render layer in `WorkspaceSidebar.svelte`.

## Steps

- [x] Inspect the unread filter bug and identify the structural causes.
- [x] Create a refactor spec and tasks file for the sidebar filter cleanup.
- [x] Add the Svelte state-transition rule to `AGENTS.md`.
- [x] Normalize sidebar filter callbacks to use `AcknowledgedFilter` end-to-end.
- [x] Extract per-channel preview state/effects out of `WorkspaceSidebar.svelte`.
- [x] Add regression coverage for the unread filter at logic and UI layers.
- [x] Verify with targeted tests, `svelte-check`, and staged pre-commit checks.
- [x] Extract sidebar video loading/filter operations out of `sidebar-state.svelte.ts`.
- [x] Extract sidebar channel CRUD operations out of `sidebar-state.svelte.ts`.
- [x] Re-run targeted unit tests, lint, `svelte-check`, and staged pre-commit checks.

## Decisions Made During Implementation

- `AcknowledgedFilter` is the canonical UI/domain representation. Conversion to `boolean | undefined` should only happen immediately before API calls.
- Setter/action methods are the only valid write path for state that owns side effects such as URL sync or cache invalidation.
- The unread UI regression is covered by a Playwright test, but the local run can skip when the workspace has no seeded channels/videos. The test remains valuable in seeded environments and CI-like manual runs.
- The next cleanup target is module size and concern separation inside the shared sidebar state factory, not a behavioral change to sidebar UX.
- `sidebar-state.svelte.ts` is now below the repo's 800-line threshold, so the highest-value remaining sidebar refactor is presentational extraction from `WorkspaceSidebar.svelte`.
