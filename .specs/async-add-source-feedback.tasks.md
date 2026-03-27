# Tasks: Async Add Source Feedback

## Current State

Shared add-source feedback helpers, popup UI, and route wiring are implemented and verified with formatting, lint, `svelte-check`, the full frontend test suite, and a production frontend build. Because the repo already contains unrelated staged changes, pre-commit-equivalent checks were run directly against the edited frontend files instead of invoking the staged hook wholesale.

## Steps

- [x] Add failing tests for add-source feedback helper logic and overlay visibility.
- [x] Implement reusable frontend state/helpers for async add-source popup copy and readiness polling.
- [x] Wire the feedback flow into the workspace sidebar state for both video URLs and channel subscriptions.
- [x] Wire the same feedback flow into the channel overview route and support video URL inputs there.
- [x] Run format, lint/check, tests, build, and staged pre-commit verification.

## Decisions Made During Implementation

- Channel readiness will be treated as "ready to open" once the newly added channel has at least one video available to browse.
- Dismissing the loading popup only hides the current loading state; the popup reappears automatically once the tracked source transitions to ready or failed.
