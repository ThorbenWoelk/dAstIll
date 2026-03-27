# Tasks: Sidebar Navigation Overhaul

## Current State

Implementation and verification are complete. Targeted sidebar regression tests, frontend lint, `svelte-check`, production build, and the staged pre-commit hook all passed after introducing shared preview-session caching and lighter selected-video loading rules.

## Steps

- [x] Inspect workspace, channel overview, and shared sidebar navigation flow.
- [x] Write failing tests for preview-state restoration and lightweight cross-channel video selection.
- [x] Implement shared sidebar preview session caching and route handoff seeding.
- [x] Verify the updated flow with targeted tests, lint, build, and staged pre-commit checks.

## Decisions Made During Implementation

- A shared session cache is preferred over localStorage for sidebar preview state because this UI state is transient and should survive route remounts without becoming long-lived persisted state.
- Per-channel preview lists now keep their existing preview payload when the newly selected video is already inside the loaded rows; the sidebar only escalates to a full channel fetch when the selected video is missing from the current preview scope.
