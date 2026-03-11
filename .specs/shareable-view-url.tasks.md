# Tasks: Shareable View URL State

## Current State

Shared URL helpers are in place, both routes restore URL state ahead of bootstrap and keep the query string synced, and all verification gates passed including browser-based share URL smoke checks.

## Steps

- [x] Add failing tests for workspace and queue URL state parsing and serialization.
- [x] Add shared frontend helpers for parsing, sanitizing, and serializing shareable URL view state.
- [x] Refactor the main workspace route to restore URL state before bootstrap and keep the query string synced with selected channel, video, content tab, and filters.
- [x] Refactor the queue route to restore URL state before bootstrap and keep the query string synced with selected channel and queue tab.
- [x] Preserve localStorage fallback behavior and update cross-route navigation helpers to use shareable URLs where appropriate.
- [x] Run frontend format, checks, tests, build, and a local smoke verification.

## Decisions Made During Implementation

- Initial red coverage is isolated to pure URL-state helpers so route refactors can be driven by deterministic parsing and serialization behavior.
- URL state now overrides localStorage by merging the sanitized persisted snapshot with sanitized query params before the routes begin their initial bootstrap fetches.
- Query params are updated with SvelteKit `replaceState` instead of raw `history.replaceState` to avoid router conflicts while keeping the current history entry shareable.
- The queue page now navigates to a fully populated workspace URL when opening a video in the main workspace, instead of relying only on localStorage handoff.
