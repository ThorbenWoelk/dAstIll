# Turso Worker Poll Backoff

## Problem

The backend keeps polling Turso aggressively even when there is no useful work to do. Queue processing, summary evaluation, and search indexing all run on short fixed intervals, which creates steady database read pressure while the app is simply running.

## Goal

Reduce background Turso reads during idle periods without making newly queued work feel delayed for normal usage.

## Requirements

- Replace fixed worker polling with adaptive behavior that uses short intervals while work is active and longer intervals while the worker is idle.
- Preserve fast follow-up processing once a worker sees pending work, so active transcript, summary, and search indexing flows do not stall.
- Keep worker behavior deterministic and unit-testable.
- Keep the change local to the backend process and current deployment model.
- Run the existing verification gates after implementation.

## Non-Goals

- Replacing Turso with another database or adding a distributed job queue.
- Reworking the frontend polling behavior in this pass.
- Adding external cache infrastructure.
- Eliminating all background polling entirely.

## Design Considerations

- The current hottest always-on loops are the queue worker, summary evaluation worker, and search indexing worker. These run frequently enough that even empty scans create material read pressure over time.
- Adaptive backoff is the smallest change that reduces idle reads immediately while keeping existing worker semantics.
- Search indexing should back off most aggressively when there is no pending or newly discovered work because its current loop performs multiple read-heavy scans per tick.

## Open Questions

- Whether production should expose the idle and active intervals via environment variables after this pass, or keep them as backend constants until real traffic data is collected.
