# Tasks: Channel Gap Reconciliation and Continuous Fill

## Current State
Implementation complete and verified: set-diff backfill and periodic gap scan worker are active, with backend fmt/clippy/tests/release build all passing.

## Steps
- [x] Create spec and task tracking files.
- [x] Add failing tests for missing-ID selection and channel video ID lookup support (Red).
- [x] Implement set-diff backfill and periodic gap scan worker (Green).
- [x] Refactor for readability and keep worker bounds explicit.
- [x] Run verification gates (fmt, clippy, tests, release build).

## Decisions Made During Implementation
- Backfill source remains the channel videos page; reconciliation is ID-based over extracted IDs.
- Gap scan interval set to 10 minutes with a per-channel insertion cap of 8 videos per scan.
