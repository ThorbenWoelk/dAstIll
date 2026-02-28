# Tasks: Queue Processing Worker

## Current State
Background queue worker is wired and verified locally. Follow-up fix is complete: refresh now preserves existing statuses, and worker processing now prioritizes transcript work before summary generation.

## Steps
- [x] Add failing tests for selecting pending queue work in DB order.
- [x] Implement DB query helpers used by queue worker.
- [x] Add backend queue worker loop and startup wiring.
- [x] Run format, lint, and tests to verify behavior.
- [x] Validate pending videos move out of pending in local runtime.

## Decisions Made During Implementation
- Scope keeps processing single-threaded in one background loop.
- Worker processes only queue-eligible statuses (`pending/loading` transcript or `pending/loading` summary with ready transcript).
- Summary status is now corrected to `pending` (rate-limit) or `failed` when transcript acquisition fails, preventing stale `loading` states.
- Follow-up fix: `insert_video` now preserves existing transcript/summary statuses on conflict so refreshes update metadata without re-queueing completed videos.
- Follow-up fix: queue worker now selects transcript work before summary work to avoid summary cache-hit loops and to fetch missing transcripts first.
