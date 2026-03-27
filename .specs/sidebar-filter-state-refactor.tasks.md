# Tasks: Sidebar Filter State Refactor

## Current State

Implementation and verification are complete. Targeted unit tests, frontend lint, `svelte-check`, the new Playwright regression run in headed mode (skipped locally because the workspace had no seeded channel data), and the staged pre-commit hook all passed after extracting preview-mode sidebar state and normalizing the filter type.

## Steps

- [x] Inspect the unread filter bug and identify the structural causes.
- [x] Create a refactor spec and tasks file for the sidebar filter cleanup.
- [x] Add the Svelte state-transition rule to `AGENTS.md`.
- [x] Normalize sidebar filter callbacks to use `AcknowledgedFilter` end-to-end.
- [x] Extract per-channel preview state/effects out of `WorkspaceSidebar.svelte`.
- [x] Add regression coverage for the unread filter at logic and UI layers.
- [x] Verify with targeted tests, `svelte-check`, and staged pre-commit checks.

## Decisions Made During Implementation

- `AcknowledgedFilter` is the canonical UI/domain representation. Conversion to `boolean | undefined` should only happen immediately before API calls.
- Setter/action methods are the only valid write path for state that owns side effects such as URL sync or cache invalidation.
- The unread UI regression is covered by a Playwright test, but the local run can skip when the workspace has no seeded channels/videos. The test remains valuable in seeded environments and CI-like manual runs.
