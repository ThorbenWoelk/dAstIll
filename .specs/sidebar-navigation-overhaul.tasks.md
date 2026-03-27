# Tasks: Sidebar Navigation Overhaul

## Current State

Inspection is complete. The next step is to lock the expected lightweight navigation behavior with tests, then update sidebar preview/session caching and cross-route handoff.

## Steps

- [x] Inspect workspace, channel overview, and shared sidebar navigation flow.
- [ ] Write failing tests for preview-state restoration and lightweight cross-channel video selection.
- [ ] Implement shared sidebar preview session caching and route handoff seeding.
- [ ] Verify the updated flow with targeted tests, lint, build, and staged pre-commit checks.

## Decisions Made During Implementation

- A shared session cache is preferred over localStorage for sidebar preview state because this UI state is transient and should survive route remounts without becoming long-lived persisted state.
