# Channel Gap Reconciliation and Continuous Fill

## Problem
Some channels end up with missing videos because history backfill currently skips by known row count. If rows are missing in the middle, they can be skipped permanently.

## Goal
Continuously detect and fill missing videos for followed channels, including on-demand backfill and periodic background checks.

## Requirements
- Backfill must reconcile by ID difference (visible channel video IDs minus stored IDs), not by count offset.
- Backfill API should keep existing response shape and insert only truly missing videos.
- Add a background worker that periodically scans channels for missing visible IDs and inserts newly found rows.
- Worker behavior must be bounded (limited inserts per channel per scan) and resilient to per-channel fetch failures.
- Add/adjust backend tests for missing-ID selection behavior and DB lookup support.

## Non-Goals
- Full deep-history crawling beyond currently visible channel page IDs.
- New UI controls for scheduling or worker tuning.
- Changing transcript/summary queueing behavior.

## Design Considerations
- ID set-diff is robust against sparse local history and reordering on channel pages.
- Reusing existing `insert_video` keeps status preservation semantics and avoids duplicate logic.
- A dedicated worker keeps gap detection independent from the 30-minute RSS refresh cadence.

## Open Questions
- None for this scoped implementation.
