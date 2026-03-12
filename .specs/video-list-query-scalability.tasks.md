# Tasks: Video List Query Scalability

## Current State

Performance audit is complete. Backend coverage now asserts the new list-query indexes exist, the migration creates composite and partial `videos` indexes for the workspace and queue filter shapes, and backend verification passed including a release build.

## Steps

- [x] Add backend test coverage that asserts the new indexes are created during DB init.
- [x] Add composite `videos` indexes for combined workspace and queue filters.
- [x] Run backend format, tests, and release build verification.

## Decisions Made During Implementation

- This change is intentionally limited to low-risk index improvements so query behavior stays unchanged.
- Added one composite index for the workspace's combined acknowledged/type filter and partial indexes for transcript-queue, summary-queue, and ready-video channel scans.
