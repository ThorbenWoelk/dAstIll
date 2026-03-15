# Tasks: Turso Worker Poll Backoff

## Current State
Adaptive worker backoff is implemented and the verification gates passed across backend, frontend, and docs builds.

## Steps
- [x] Add tests that define adaptive worker interval behavior for queue, summary evaluation, and search indexing loops.
- [x] Implement adaptive idle/backoff timing in backend workers.
- [x] Verify the updated worker behavior with backend formatting, tests, and release build.
- [x] Run the frontend verification gates required by the repo workflow.

## Decisions Made During Implementation
- Background worker polling is the first target because it runs continuously even without user interaction and is likely to dominate idle Turso reads.
- Idle backoff starts quickly but caps at one to two minutes so the app reduces baseline reads substantially without making background processing feel stalled during normal use.
