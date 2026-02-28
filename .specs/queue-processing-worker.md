# Queue Processing Worker

## Problem
Refreshing a channel inserts videos with `pending` transcript/summary status, but there is no background consumer to process those rows. Users see many queued videos without any progress unless they manually open each video transcript/summary endpoint.

## Goal
Queued videos transition from `pending` to `loading` to `ready` (or `failed`) automatically, without requiring per-video manual actions.

## Requirements
- Backend runs a background loop that periodically scans for pending work.
- Pending transcripts are downloaded automatically.
- Pending summaries are generated automatically after transcript availability.
- Processing updates status fields so queue UI reflects real progress.
- Failed work is marked `failed` and does not block processing of other videos.

## Non-Goals
- Adding distributed workers or external job queues.
- Changing the queue UI design.
- Reworking transcript/summarizer provider integrations.

## Design Considerations
Reuse existing transcript/summarizer logic to avoid divergence from API behavior. Process one video at a time in-process for simplicity and predictable load.

## Open Questions
- Whether failed jobs should be retried automatically in this scope (initial scope: no retries).
