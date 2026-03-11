# Tasks: DRY And Redundancy Audit Fixes

## Current State

Shared frontend and backend helpers are in place, the queue refresh regression is fixed, and all verification gates passed including a local startup smoke check.

## Steps

- [x] Add failing tests for the queue refresh filter regression and backend persistence helpers.
- [x] Extract shared frontend workspace helpers for localStorage state, channel ordering, and queue snapshot request options.
- [x] Refactor the queue and workspace routes to use the shared helpers and preserve the active queue filter on refresh.
- [x] Refactor backend manual content update flows to use shared existence/persistence helpers.
- [x] Remove redundant frontend state and duplicate content editor action markup.
- [x] Run formatting, checks, tests, builds, and a manual smoke verification.

## Decisions Made During Implementation

- Scope is limited to the audit findings and directly related cleanup needed to keep behavior consistent.
- Queue snapshot requests now use a shared helper so the initial queue load and post-refresh reload cannot diverge on `queueTab`.
- Manual transcript and summary edits now go through dedicated DB helpers that also mark the corresponding content status as `ready`.
