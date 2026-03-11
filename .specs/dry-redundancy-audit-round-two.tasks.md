# Tasks: DRY And Redundancy Audit Round Two

## Current State

Shared frontend and backend helpers are in place, duplicate DB write entrypoints are collapsed behind private helpers, and all verification gates passed including a local startup smoke check.

## Steps

- [x] Add failing tests for the new shared frontend workspace helpers and backend query/persistence consolidations.
- [x] Extract shared frontend helpers for workspace snapshot restoration, drag/drop state, and refresh TTL decisions.
- [x] Refactor the main workspace and queue routes to use the shared frontend helpers without changing behavior.
- [x] Extract shared backend query/filter structs and shared channel/video lookup helpers.
- [x] Consolidate backend transcript and summary content write helpers to one public write path per behavior.
- [x] Run format, checks, tests, and release builds for frontend and backend.

## Decisions Made During Implementation

- Frontend red coverage is anchored in `frontend/tests/channel-workspace.test.ts` for shared state restoration, drag/drop transitions, and refresh TTL behavior.
- Backend red coverage is anchored in `backend/src/handlers/videos.rs` for shared query/filter resolution methods on the common video list params type.
- Shared query parsing now lives in `backend/src/handlers/query.rs`, with `WorkspaceBootstrapParams` flattening the same `VideoListParams` type used by the video list and channel snapshot endpoints.
- Shared 404 lookup behavior now lives in `backend/src/handlers/mod.rs` via `require_channel` and `require_video`, which the handlers call before route-specific work.
- Status-neutral transcript and summary writes are now private helpers inside `backend/src/db.rs`; the public manual write entrypoints remain `save_manual_transcript` and `save_manual_summary`.
